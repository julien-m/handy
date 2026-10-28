//! Smart paste as the insertion method of a dictation (feature 007, step 7).
//!
//! With a 006 destination session, [`SmartPasteMethod`] is 006's
//! `InsertionPort`: its first guard check builds the destination context,
//! then the serialized actor runs the AX-first chain while 006's guard is
//! registered for every later destination check, and the terminal
//! [`PasteOutcome`] becomes 006's typed method result. Without a session
//! (destination-return gate closed), [`ForegroundInsertion`] reads the live
//! foreground itself, like 006's frontmost-at-the-end route. The caller runs
//! both on a worker thread, never on the main thread the pasteboard needs.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use super::actor::{shutdown_allowance, ActorStopped, SmartPasteActor};
use super::destination_bridge::{DestinationChecks, ForegroundCheck, RequestCheck, SessionCheck};
use super::ids::{DestinationGeneration, DictationId, Transcript};
use super::outcome::{InsertionPath, PasteOutcome, PasteOutcomeKind};
use super::pasteboard_port::PasteboardError;
use super::request::{DestinationContext, InsertionRequest};
use super::setting_policy::Rollout;
use super::settlement::ClipboardPolicy;
use crate::destination::context::DestinationContext as SessionContext;
use crate::destination::guard::DispatchGuard;
use crate::destination::insertion::InsertionPort;
use crate::destination::outcome::MethodResult;
use crate::destination::workspace::WorkspacePort;
use crate::settings::{AppSettings, ClipboardHandling, PasteMethod};

/// Queues one insertion on the smart-paste actor.
pub trait InsertionSubmitter: Send + Sync {
    /// Queues `request`; its terminal outcome arrives on the receiver.
    ///
    /// # Errors
    /// [`ActorStopped`] once the actor has shut down.
    fn submit(&self, request: InsertionRequest) -> Result<Receiver<PasteOutcome>, ActorStopped>;
}

/// The app's actor, present until the normal exit settles it.
#[derive(Clone, Default)]
pub struct ActorSlot(Arc<Mutex<Option<SmartPasteActor>>>);

impl ActorSlot {
    /// Slot owning `actor`.
    pub fn new(actor: SmartPasteActor) -> Self {
        Self(Arc::new(Mutex::new(Some(actor))))
    }
    /// Takes the actor out (normal exit); later submissions are refused.
    pub fn take(&self) -> Option<SmartPasteActor> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
    /// Queues 010's deliberate Copy on the actor (see [`SmartPasteActor::deliberate_copy`]).
    ///
    /// # Errors
    /// [`ActorStopped`] once the actor is gone.
    pub fn deliberate_copy(
        &self,
        transcript: Transcript,
    ) -> Result<Receiver<Result<(), PasteboardError>>, ActorStopped> {
        let actor = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        actor
            .as_ref()
            .ok_or(ActorStopped)?
            .deliberate_copy(transcript)
    }
}

impl InsertionSubmitter for ActorSlot {
    fn submit(&self, request: InsertionRequest) -> Result<Receiver<PasteOutcome>, ActorStopped> {
        let actor = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        actor.as_ref().ok_or(ActorStopped)?.insert(request)
    }
}

/// What one insertion needs besides its destination, read once from the
/// settings snapshot of the insertion.
#[derive(Clone, Debug)]
pub struct InsertionInputs {
    /// Final text after trailing-space handling (never logged).
    pub transcript: Transcript,
    /// Clipboard handling setting.
    pub clipboard_policy: ClipboardPolicy,
    /// `paste_delay_ms`, applied before the keystroke.
    pub paste_delay: Duration,
}

impl InsertionInputs {
    /// Inputs for `final_text` under `settings`.
    pub fn new(final_text: String, settings: &AppSettings) -> Self {
        Self {
            transcript: Transcript::new(final_text),
            clipboard_policy: match settings.clipboard_handling {
                ClipboardHandling::DontModify => ClipboardPolicy::DontModify,
                ClipboardHandling::CopyToClipboard => ClipboardPolicy::CopyToClipboard,
            },
            paste_delay: Duration::from_millis(settings.paste_delay_ms),
        }
    }

    fn request(
        &self,
        dictation_id: DictationId,
        destination: DestinationContext,
    ) -> InsertionRequest {
        InsertionRequest {
            dictation_id,
            transcript: self.transcript.clone(),
            destination,
            clipboard_policy: self.clipboard_policy,
            paste_delay: self.paste_delay,
        }
    }
}

// @spec FR-008: Pipeline outcome, Enter and recovery — .specs/features/007-smart-paste/spec.md#fr-008
/// Runs `request` on the actor with `check` registered for its generation
/// and returns the single terminal outcome. A refused submission inserted
/// nothing (`insertion_failed`); a lost or late reply cannot tell whether
/// anything was inserted (`uncertain`), so both stay recoverable failures.
pub fn run_insertion(
    submitter: &dyn InsertionSubmitter,
    checks: &Arc<DestinationChecks>,
    request: InsertionRequest,
    check: Arc<dyn RequestCheck>,
) -> PasteOutcome {
    let _registration = checks.register(request.destination.generation, check);
    let identity = RequestIdentity::of(&request);
    let wait = shutdown_allowance(request.paste_delay);
    let Ok(receiver) = submitter.submit(request) else {
        return identity.missing_reply(MissingReply::Refused);
    };
    match receiver.recv_timeout(wait) {
        Ok(outcome) => outcome,
        Err(RecvTimeoutError::Timeout) => identity.missing_reply(MissingReply::TimedOut(wait)),
        Err(RecvTimeoutError::Disconnected) => identity.missing_reply(MissingReply::Dropped),
    }
}

/// Why `run_insertion` has no outcome from the actor.
#[derive(Clone, Copy, Debug)]
enum MissingReply {
    /// The actor had stopped: the request was never queued.
    Refused,
    /// No reply within the allowance.
    TimedOut(Duration),
    /// The actor dropped the reply channel.
    Dropped,
}

/// What a synthetic outcome keeps of a request handed to the actor.
#[derive(Clone, Copy)]
struct RequestIdentity {
    dictation_id: DictationId,
    generation: DestinationGeneration,
}

impl RequestIdentity {
    fn of(request: &InsertionRequest) -> Self {
        Self {
            dictation_id: request.dictation_id,
            generation: request.destination.generation,
        }
    }

    /// Logs the reason and builds the synthetic outcome: a refused request
    /// inserted nothing, a missing reply cannot tell (see `run_insertion`).
    /// Logged here only, so a reply that does arrive is never reported as
    /// missing (N12: the old log ran before the request was even submitted).
    fn missing_reply(self, reason: MissingReply) -> PasteOutcome {
        let id = self.dictation_id.0;
        let kind = match reason {
            MissingReply::Refused => {
                log::debug!("[smart-paste] dictation {id}: actor stopped, insertion refused");
                PasteOutcomeKind::InsertionFailed
            }
            MissingReply::TimedOut(wait) => {
                log::debug!("[smart-paste] dictation {id}: no actor reply within {wait:?}");
                PasteOutcomeKind::Uncertain
            }
            MissingReply::Dropped => {
                log::debug!("[smart-paste] dictation {id}: no actor reply (reply channel dropped)");
                PasteOutcomeKind::Uncertain
            }
        };
        PasteOutcome {
            dictation_id: self.dictation_id,
            transaction_id: None,
            destination_generation: self.generation,
            path: InsertionPath::NotAttempted,
            kind,
        }
    }
}

/// Called once with the terminal outcome and 006's context (Enter hook,
/// hand-over message); runs on the insertion worker.
pub type SessionFollowUp = Box<dyn Fn(&PasteOutcome, &SessionContext) + Send + Sync>;

/// Smart paste as 006's insertion method for one dictation.
pub struct SmartPasteMethod {
    /// The app's actor.
    pub submitter: Arc<dyn InsertionSubmitter>,
    /// Checks consulted by the actor's destination port.
    pub checks: Arc<DestinationChecks>,
    /// Text and settings of this insertion.
    pub inputs: InsertionInputs,
    /// Follow-ups of the outcome.
    pub follow_up: SessionFollowUp,
}

impl InsertionPort for SmartPasteMethod {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        // The first check builds the context and sets the disposition to
        // `Insert`; a refusal means no dispatch at all (006 contract).
        let context = match guard.check() {
            Ok(context) => context,
            Err(refusal) => return MethodResult::Legacy(Err(refusal.to_string())),
        };
        let destination = DestinationContext::from_session(&context);
        let request = self
            .inputs
            .request(DictationId(context.dictation_id.0), destination);
        let check = Arc::new(SessionCheck(guard));
        let outcome = run_insertion(self.submitter.as_ref(), &self.checks, request, check);
        (self.follow_up)(&outcome, &context);
        outcome.method_result()
    }
}

/// 006 generations count up from 0 in one process and never reach 2^63, so
/// identifiers of insertions without a session never collide with them.
const LOCAL_GENERATION_BASE: u64 = 1 << 63;
static LAST_LOCAL_INSERTION: AtomicU64 = AtomicU64::new(0);

/// Smart paste for a dictation without a 006 session (destination-return
/// gate closed): the destination is the application in front now.
pub struct ForegroundInsertion {
    /// The app's actor.
    pub submitter: Arc<dyn InsertionSubmitter>,
    /// Checks consulted by the actor's destination port.
    pub checks: Arc<DestinationChecks>,
    /// Live foreground source (006's workspace adapter).
    pub workspace: Arc<dyn WorkspacePort>,
    /// Text and settings of this insertion.
    pub inputs: InsertionInputs,
}

impl ForegroundInsertion {
    /// Inserts into the application in front; `None` when the dictation was
    /// cancelled first (nothing dispatched, like the classic path).
    pub fn insert(&self, cancelled: Box<dyn Fn() -> bool + Send + Sync>) -> Option<PasteOutcome> {
        if cancelled() {
            return None;
        }
        let local = LAST_LOCAL_INSERTION.fetch_add(1, Ordering::Relaxed) + 1;
        let generation = DestinationGeneration(LOCAL_GENERATION_BASE + local);
        let front = self.workspace.frontmost_now();
        let destination = DestinationContext::from_foreground(&front, generation);
        let request = self.inputs.request(DictationId(local), destination);
        let workspace = Arc::clone(&self.workspace);
        let check = Arc::new(ForegroundCheck {
            workspace,
            cancelled,
        });
        Some(run_insertion(
            self.submitter.as_ref(),
            &self.checks,
            request,
            check,
        ))
    }
}

/// What follows a terminal outcome outside the clipboard lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FollowUps {
    /// Show "Text placed on the clipboard for the remote screen" (AC-012).
    pub hand_over_message: bool,
    /// Send the automatic Enter once (until 009 owns it).
    pub auto_submit: bool,
    /// Report an insertion failure (010 recovery, interim toast).
    pub failure_notice: bool,
}

/// Business rule (AC-012): only "inserted, verified", "read observed" and
/// "typed" may submit; "clipboard handed over" is neither a success for
/// Enter nor a failure for recovery and shows its own message; the four
/// failures are reported for recovery.
pub fn follow_ups(kind: PasteOutcomeKind, auto_submit_enabled: bool) -> FollowUps {
    FollowUps {
        hand_over_message: kind == PasteOutcomeKind::ClipboardHandedOver,
        auto_submit: auto_submit_enabled && kind.is_auto_submit_eligible(),
        failure_notice: kind.requires_recovery(),
    }
}

/// How a dictation's text reaches its application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsertionRoute {
    /// The method of `dev`, unchanged (the classic paste methods, or smart
    /// paste refused by the paste pipeline while its gate is closed).
    Classic,
    /// Smart paste as 006's insertion method (both gates open).
    SmartPasteInSession,
    /// Smart paste into the application in front (destination-return gate
    /// closed, smart-paste-dispatch open).
    SmartPasteAtForeground,
}

/// Business rule (gate combinations, implementation.md "Rollout gates"):
/// smart paste runs only where its gate makes it active; it uses 006's
/// session when one exists and the live foreground otherwise.
pub fn insertion_route(rollout: Rollout, has_session: bool, method: PasteMethod) -> InsertionRoute {
    match (
        method == PasteMethod::SmartPaste && rollout.is_active(),
        has_session,
    ) {
        (false, _) => InsertionRoute::Classic,
        (true, true) => InsertionRoute::SmartPasteInSession,
        (true, false) => InsertionRoute::SmartPasteAtForeground,
    }
}

#[cfg(test)]
#[path = "dispatch_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "dispatch_log_tests.rs"]
mod log_tests;
