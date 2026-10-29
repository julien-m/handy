//! Per-dictation Enter transaction of 009: the policy and key frozen at insertion entry, the
//! single decision token, and the ordered attempt after an eligible completion: the 50 ms
//! stabilization pause, 006's atomic `claim_enter`, the frontmost check of a route with a
//! destination, then one key press. A claim consumed before a failed check is never retried, and
//! a key whose delivery fails or is unknown is never sent again.
use super::policy::{evaluate_enter, route_has_destination, Completion, EnterFacts, Suppression};
use super::PRE_ENTER_SETTLE;
use crate::destination::context::{DestinationContext, DictationId, ResolvedTarget};
use crate::destination::insertion::InsertionReport;
use crate::destination::manager::DestinationManager;
use crate::destination::outcome::{InsertionStatus, MethodResult};
use crate::destination::session::{EnterClaim, EnterRefusal};
use crate::destination::workspace::{Clock, Foreground, Pause, WorkspacePort};
use crate::settings::{AutoSubmitKey, AutomaticEnterPolicy};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

/// 006's Enter claim (the disposition veto, the stale-dictation check and the single claim).
pub trait EnterClaims: Send + Sync {
    /// See `DestinationManager::claim_enter`.
    ///
    /// # Errors
    /// The refusal; no Enter may follow it.
    fn claim_enter(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<EnterClaim, EnterRefusal>;
}
impl EnterClaims for DestinationManager {
    fn claim_enter(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<EnterClaim, EnterRefusal> {
        DestinationManager::claim_enter(self, dictation_id, generation)
    }
}

/// What became of one key press request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyOutcome {
    /// The key was pressed once.
    Sent,
    /// The destination was no longer in front on the input thread: not pressed.
    FocusLost,
    /// The input thread could not run the press within its bound: not pressed.
    Late,
    /// The press failed or could not be scheduled; never retried.
    Failed(String),
}

/// The native key press (the only new seam of 009).
pub trait KeyEmitter: Send + Sync {
    /// Press `key` once, after checking that `expected` (the destination of a route that has
    /// one) is still in front immediately before the key.
    fn send(&self, key: AutoSubmitKey, expected: Option<&ResolvedTarget>) -> KeyOutcome;
}

/// The ports of a transaction.
#[derive(Clone)]
pub struct EnterPorts {
    pub claims: Arc<dyn EnterClaims>,
    pub workspace: Arc<dyn WorkspacePort>,
    pub keys: Arc<dyn KeyEmitter>,
    pub clock: Arc<dyn Clock>,
    pub pause: Arc<dyn Pause>,
}

/// What is frozen at insertion entry: a later settings change applies to the next dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnterSnapshot {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub policy: AutomaticEnterPolicy,
    pub key: AutoSubmitKey,
}

/// Final result of a dictation's Enter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterResult {
    /// One key press was sent.
    Sent,
    /// No key press was attempted.
    Suppressed(Suppression),
    /// The key press failed or its delivery is unknown; never retried.
    EmissionFailed(String),
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    /// Nothing decided yet.
    Frozen,
    /// A reliable paste transfer started: only its rendezvous decides.
    AwaitingReceipt,
    /// One completion is being decided (pause, claim, focus, key).
    Deciding,
    Decided(EnterResult),
}

/// One dictation's Enter, keyed by `(dictation_id, destination_generation)`.
pub struct EnterTransaction {
    frozen: EnterSnapshot,
    ports: EnterPorts,
    token: Mutex<Token>,
}

impl EnterTransaction {
    /// Transaction of `frozen` over `ports`.
    pub fn new(frozen: EnterSnapshot, ports: EnterPorts) -> Self {
        Self {
            frozen,
            ports,
            token: Mutex::new(Token::Frozen),
        }
    }
    /// The frozen policy and key.
    pub fn snapshot(&self) -> EnterSnapshot {
        self.frozen
    }
    /// The decided result, if any.
    pub fn result(&self) -> Option<EnterResult> {
        match &*self.lock() {
            Token::Decided(result) => Some(result.clone()),
            _ => None,
        }
    }
    /// A reliable paste transferred the transcript: its rendezvous alone decides from now on,
    /// so the method's return is not a classic success for Enter. False when already decided.
    pub fn await_receipt(&self) -> bool {
        let mut token = self.lock();
        if *token != Token::Frozen {
            return false;
        }
        *token = Token::AwaitingReceipt;
        true
    }

    // @spec AC-005: Enter after the insertion step only — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
    /// The insertion worker finished (`Orchestrator::deliver` returned, before `publish`): decide
    /// the Enter of a legacy method, or suppress a dictation that dispatched nothing. `None` when
    /// smart paste or a reliable rendezvous owns the decision.
    pub fn after_insertion(&self, report: &InsertionReport) -> Option<EnterResult> {
        if *self.lock() != Token::Frozen {
            return None;
        }
        let completion = match (&report.method_result, report.status) {
            (_, InsertionStatus::Skipped) => Completion::NoInsertion,
            _ if report.addressed => Completion::AddressedTerminal,
            (Some(MethodResult::Legacy(result)), status) => Completion::Classic {
                succeeded: result.is_ok() && status == InsertionStatus::InsertionAttempted,
            },
            (Some(MethodResult::Detailed { outcome, .. }), _) => Completion::SmartPaste(*outcome),
            (None, _) => Completion::Classic { succeeded: false },
        };
        let t0 = self.ports.clock.now();
        Some(self.complete(completion, report.context.as_deref(), t0))
    }

    /// Decide this dictation's Enter for `completion`, dispatched with `context` (`None` when no
    /// method passed the guard), ready at `t0`. A second completion is `AlreadyDecided`.
    pub fn complete(
        &self,
        completion: Completion,
        context: Option<&DestinationContext>,
        t0: Instant,
    ) -> EnterResult {
        {
            let mut token = self.lock();
            if matches!(*token, Token::Deciding | Token::Decided(_)) {
                return EnterResult::Suppressed(Suppression::AlreadyDecided);
            }
            *token = Token::Deciding;
        }
        let result = self.attempt(completion, context, t0);
        log_result(self.frozen.dictation_id, &result);
        *self.lock() = Token::Decided(result.clone());
        result
    }

    /// Order matters (decision 2026-10-14, AC-005 "immediately before the Enter press"): the
    /// pause, then the claim (which also refuses a dictation superseded, cancelled or recovered),
    /// then the focus check, then the key.
    fn attempt(
        &self,
        completion: Completion,
        context: Option<&DestinationContext>,
        t0: Instant,
    ) -> EnterResult {
        let facts = EnterFacts {
            policy: self.frozen.policy,
            source: context.map(|context| context.source),
            completion,
        };
        if let Err(reason) = evaluate_enter(facts) {
            return EnterResult::Suppressed(reason);
        }
        self.settle(t0);
        let (id, generation) = (self.frozen.dictation_id, self.frozen.destination_generation);
        if let Err(refusal) = self.ports.claims.claim_enter(id, generation) {
            return EnterResult::Suppressed(suppression_of(refusal));
        }
        if context.is_some_and(|context| !self.still_in_front(context)) {
            return EnterResult::Suppressed(Suppression::FocusLost);
        }
        // Checked again by the emitter on the input thread, immediately before the key.
        let expected = context
            .filter(|context| route_has_destination(context.source))
            .and_then(|context| context.target.as_ref());
        match self.ports.keys.send(self.frozen.key, expected) {
            KeyOutcome::Sent => EnterResult::Sent,
            KeyOutcome::FocusLost => EnterResult::Suppressed(Suppression::FocusLost),
            KeyOutcome::Late => EnterResult::Suppressed(Suppression::KeyLate),
            KeyOutcome::Failed(error) => EnterResult::EmissionFailed(error),
        }
    }
    /// Wait until `PRE_ENTER_SETTLE` has elapsed since `t0` (through the clock, on this worker).
    fn settle(&self, t0: Instant) {
        let elapsed = self.ports.clock.now().saturating_duration_since(t0);
        if let Some(remaining) = PRE_ENTER_SETTLE.checked_sub(elapsed) {
            if !remaining.is_zero() {
                self.ports.pause.pause(remaining);
            }
        }
    }
    /// A route with a destination needs that instance frontmost now; the frontmost-at-the-end
    /// route targets whatever is in front, as before 006.
    fn still_in_front(&self, context: &DestinationContext) -> bool {
        if !route_has_destination(context.source) {
            return true;
        }
        let Some(target) = &context.target else {
            return false;
        };
        matches!(self.ports.workspace.frontmost_now(), Foreground::Application(front)
            if front.pid == target.pid && front.identity == target.identity)
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Token> {
        self.token.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

// @spec FR-004: Suppressed(UserSkipped) — .specs/features/014-in-dictation-keys/spec.md#fr-004
/// Why a refused claim suppresses the Enter: « Insert without Enter » (014) has its own reason.
fn suppression_of(refusal: EnterRefusal) -> Suppression {
    match refusal {
        EnterRefusal::UserSkipped => Suppression::UserSkipped,
        other => Suppression::Claim(other),
    }
}

/// Metadata only (no transcript, no window title) at debug level, except a failed key press.
fn log_result(dictation_id: DictationId, result: &EnterResult) {
    match result {
        EnterResult::EmissionFailed(error) => {
            log::warn!(
                "Automatic Enter of dictation {} failed: {error}",
                dictation_id.0
            );
        }
        other => log::debug!("Automatic Enter of dictation {}: {other:?}", dictation_id.0),
    }
}

#[cfg(test)]
#[path = "transaction_fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "transaction_flow_tests.rs"]
mod flow_tests;
#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
