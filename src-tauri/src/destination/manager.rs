//! `DestinationManager`: owner of the current dictation's destination session. It captures the
//! frontmost application at an accepted recording start, applies versioned badge choices,
//! freezes the destination at insertion entry, and serializes the disposition and the Enter
//! claim under one lock. Native reads happen outside the lock; the session is never persisted.
use super::context::{DictationId, RecoveryDisposition};
use super::session::{
    Destination, DestinationSession, DestinationSupport, DispositionRefusal, EnterClaim,
    EnterRefusal, Selection, SelectionRefusal, SessionPhase, SupportReason,
};
use super::snapshot::{DestinationMenu, DestinationSnapshot, MenuChoice};
use super::workspace::{Clock, Foreground, WorkspacePort};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Destination read at a recording start, committed only if the recording really starts.
#[derive(Clone, Debug, PartialEq)]
pub struct TentativeCapture {
    pub return_enabled: bool,
    pub support: DestinationSupport,
    pub selection: Selection,
    pub captured_at: Instant,
    /// Time spent reading the foreground (bounded at 50 ms by the spec).
    pub elapsed: Duration,
}
/// The destination frozen at insertion entry.
#[derive(Clone, Debug, PartialEq)]
pub struct FrozenDestination {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub return_enabled: bool,
    pub support: DestinationSupport,
    pub selection: Selection,
}

#[derive(Default)]
pub(super) struct ManagerState {
    pub(super) last_dictation: u64,
    pub(super) generation: u64,
    pub(super) session: Option<DestinationSession>,
    /// Rows of the open badge menu by token (012: a favorite not running has no process).
    pub(super) menu: Vec<(u64, super::menu_order::MenuTarget)>,
    pub(super) next_token: u64,
    pub(super) support: Option<DestinationSupport>,
    /// Session pin of 009 (RAM only, ends with the process).
    pub(super) pin: Option<super::pin::PinState>,
    pub(super) pin_version: u64,
    /// When the disposition of a dictation became `Recover` (010 latency start).
    pub(super) recovery_decided: Option<(DictationId, Instant)>,
    /// 011's input remembered for the current dictation (never content; see `field_lifetime`).
    pub(super) field: Option<super::field::RememberedField>,
}

/// Session owner shared by the recording lifecycle, the session commands and 009/010.
pub struct DestinationManager {
    pub(super) state: Mutex<ManagerState>,
    pub(super) workspace: Arc<dyn WorkspacePort>,
    pub(super) clock: Arc<dyn Clock>,
    platform: PlatformCapabilities,
    /// 011's field port (unsupported until `with_field_access`).
    pub(super) field_access: Arc<dyn super::field::FieldAccess>,
    /// 013's ports (nothing remembered until `with_default_inputs`).
    pub(super) default_inputs: Arc<super::default_input::services::DefaultInputServices>,
}

impl DestinationManager {
    /// Manager over `workspace`, timed by `clock`, for a build with `platform` capabilities.
    pub fn new(
        workspace: Arc<dyn WorkspacePort>,
        clock: Arc<dyn Clock>,
        platform: PlatformCapabilities,
    ) -> Self {
        Self {
            state: Mutex::new(ManagerState::default()),
            workspace,
            clock,
            platform,
            field_access: Arc::new(super::field::UnsupportedFieldAccess),
            default_inputs: Arc::new(super::default_input::services::DefaultInputServices::none()),
        }
    }
    /// Session state, recovered after a poisoning panic (the state stays consistent per call).
    pub(super) fn lock(&self) -> MutexGuard<'_, ManagerState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
    /// The workspace this manager reads (for the insertion orchestrator).
    pub fn workspace(&self) -> Arc<dyn WorkspacePort> {
        Arc::clone(&self.workspace)
    }
    // @spec FR-001: Capture the destination at dictation start — .specs/features/006-destination-application/spec.md#fr-001
    /// Read the destination of a starting recording. The frontmost application is read only
    /// when destination return is on and the platform supports it; nothing is committed.
    pub fn capture(&self, return_enabled: bool) -> TentativeCapture {
        let captured_at = self.clock.now();
        let (support, selection) = if !self.platform.supports(PlatformCapability::Destination) {
            let unsupported = DestinationSupport::Unsupported(SupportReason::PlatformUnsupported);
            (unsupported, Selection::FrontmostAtEnd)
        } else if !return_enabled {
            (self.support(), Selection::FrontmostAtEnd)
        } else {
            selection_of(self.workspace.frontmost())
        };
        TentativeCapture {
            return_enabled,
            support,
            selection,
            captured_at,
            elapsed: self.clock.now().saturating_duration_since(captured_at),
        }
    }
    // @spec FR-001: The pin wins over the capture, kept for unpin — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-001
    /// Commit the session of a recording that actually started; a new dictation and
    /// generation invalidate every callback of the previous one. A session pin (009) becomes
    /// its route; 006's capture is kept as the start route an Unpin restores.
    pub fn commit_start(&self, capture: TentativeCapture) -> DestinationSnapshot {
        let mut state = self.lock();
        state.last_dictation += 1;
        state.generation += 1;
        state.menu.clear();
        state.field = None;
        state.support = Some(capture.support);
        let (selection, pin_view) = self.start_route(state.pin.as_ref(), &capture.selection);
        let session = DestinationSession {
            dictation_id: DictationId(state.last_dictation),
            destination_generation: state.generation,
            selection_version: 0,
            phase: SessionPhase::Recording,
            return_enabled: capture.return_enabled,
            support: capture.support,
            selection,
            start_selection: capture.selection,
            pin: pin_view,
            captured_at: capture.captured_at,
            disposition: RecoveryDisposition::Pending,
            enter_claimed: false,
        };
        let snapshot = DestinationSnapshot::from(&session);
        state.session = Some(session);
        snapshot
    }
    /// The route of a starting dictation and the pin it shows: the pin when one exists on a
    /// build with destination support, otherwise 006's capture.
    fn start_route(
        &self,
        pin: Option<&super::pin::PinState>,
        captured: &Selection,
    ) -> (Selection, Option<super::pin::PinView>) {
        match pin.filter(|_| self.platform.supports(PlatformCapability::Destination)) {
            Some(pin) => (Selection::Application(pin.destination()), Some(pin.view())),
            None => (captured.clone(), None),
        }
    }
    /// Last runtime support observed (supported until a read fails, on a capable build).
    pub fn support(&self) -> DestinationSupport {
        if !self.platform.supports(PlatformCapability::Destination) {
            return DestinationSupport::Unsupported(SupportReason::PlatformUnsupported);
        }
        self.lock().support.unwrap_or(DestinationSupport::Supported)
    }
    /// Read the foreground once (startup) and record whether it is readable.
    pub fn refresh_support(&self) -> DestinationSupport {
        if !self.platform.supports(PlatformCapability::Destination) {
            return self.support();
        }
        let (support, _) = selection_of(self.workspace.frontmost());
        self.lock().support = Some(support);
        // Warm the live read off the main thread: its first macOS call took about 40 ms on the
        // native run (later calls under 1 ms), and the guard makes it on the main thread.
        let _ = self.workspace.frontmost_now();
        support
    }
    /// Snapshot of the current session, if any.
    pub fn snapshot(&self) -> Option<DestinationSnapshot> {
        self.lock().session.as_ref().map(DestinationSnapshot::from)
    }
    /// True while `dictation_id`/`generation` is the live (not cancelled, not finished) session.
    pub fn is_current(&self, dictation_id: DictationId, generation: u64) -> bool {
        let state = self.lock();
        state.generation == generation
            && state.session.as_ref().is_some_and(|session| {
                session.is(dictation_id, generation)
                    && !matches!(
                        session.phase,
                        SessionPhase::Cancelled | SessionPhase::Finished
                    )
            })
    }
    /// The recording stopped; the destination stays as chosen through processing and AI.
    pub fn mark_processing(&self) {
        if let Some(session) = self.lock().session.as_mut() {
            if session.phase == SessionPhase::Recording {
                session.phase = SessionPhase::Processing;
            }
        }
    }
    /// Cancel the current dictation: a new generation invalidates every late callback, choice
    /// or activation of it, and its disposition becomes `Cancelled` when still allowed.
    pub fn cancel(&self) -> Option<DestinationSnapshot> {
        let mut state = self.lock();
        state.generation += 1;
        state.menu.clear();
        state.field = None;
        let session = state.session.as_mut()?;
        if matches!(
            session.phase,
            SessionPhase::Finished | SessionPhase::Cancelled
        ) {
            return None;
        }
        session.phase = SessionPhase::Cancelled;
        if session
            .disposition
            .may_become(RecoveryDisposition::Cancelled)
        {
            session.disposition = RecoveryDisposition::Cancelled;
        }
        Some(DestinationSnapshot::from(&*session))
    }
    /// Insertion entry: freeze the selection, close the menu and refuse later choices.
    ///
    /// # Errors
    /// `Stale` when the dictation is no longer current (cancelled, superseded, already frozen).
    pub fn freeze(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<FrozenDestination, DispositionRefusal> {
        self.freeze_with_field(dictation_id, generation)
            .map(|(frozen, _released)| frozen)
    }
    /// 006's freeze itself; 011's remembered input is handled by `freeze_with_field`.
    pub(super) fn freeze_session(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<FrozenDestination, DispositionRefusal> {
        let mut state = self.lock();
        if state.generation != generation {
            return Err(DispositionRefusal::Stale);
        }
        state.menu.clear();
        let session = state.session.as_mut().ok_or(DispositionRefusal::Stale)?;
        let selection = session.freeze(dictation_id, generation)?;
        Ok(FrozenDestination {
            dictation_id,
            destination_generation: generation,
            return_enabled: session.return_enabled,
            support: session.support,
            selection,
        })
    }
    /// The insertion worker of this dictation failed before reporting: freeze its selection
    /// (no later choice) and move its disposition to `Recover`. Returns the source of its
    /// selection, or `None` when the dictation is no longer current (cancelled or superseded).
    pub fn fail_insertion(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Option<super::context::DestinationSource> {
        let mut state = self.lock();
        if state.generation != generation {
            return None;
        }
        state.menu.clear();
        state.field = None;
        let session = state.session.as_mut()?;
        if !session.is(dictation_id, generation) || session.phase == SessionPhase::Finished {
            return None;
        }
        // Already frozen when the worker passed the insertion entry; the refusal is expected.
        let _ = session.freeze(dictation_id, generation);
        // Refused only when the disposition is already final (`Recover` or `Cancelled`).
        let _ = session.set_disposition(dictation_id, generation, RecoveryDisposition::Recover);
        Some(session.selection.source())
    }
    /// The terminal outcome of this dictation was published.
    pub fn finish(&self, dictation_id: DictationId, generation: u64) {
        let mut state = self.lock();
        if state
            .field
            .as_ref()
            .is_some_and(|field| field.dictation_id == dictation_id)
        {
            state.field = None;
        }
        if let Some(session) = state.session.as_mut() {
            if session.is(dictation_id, generation) && session.phase == SessionPhase::Inserting {
                session.phase = SessionPhase::Finished;
            }
        }
    }
    // @spec FR-005: Single insert-or-recover disposition — .specs/features/006-destination-application/spec.md#fr-005
    /// Move the disposition of `dictation_id` forward (monotonic; see `RecoveryDisposition`).
    ///
    /// # Errors
    /// `Stale` for another dictation or generation; `Final` for a backward change.
    pub fn set_disposition(
        &self,
        dictation_id: DictationId,
        generation: u64,
        next: RecoveryDisposition,
    ) -> Result<RecoveryDisposition, DispositionRefusal> {
        let now = self.clock.now();
        let mut state = self.lock();
        let session = state.session.as_mut().ok_or(DispositionRefusal::Stale)?;
        let previous = session.disposition;
        let set = session.set_disposition(dictation_id, generation, next)?;
        if set == RecoveryDisposition::Recover && previous != RecoveryDisposition::Recover {
            // 010's QE latency starts at this decision (text-free timing only).
            state.recovery_decided = Some((dictation_id, now));
        }
        Ok(set)
    }
    /// When `dictation_id`'s disposition became `Recover`, if it did (010 latency start).
    pub fn recovery_decided_at(&self, dictation_id: DictationId) -> Option<Instant> {
        self.lock()
            .recovery_decided
            .filter(|(decided, _)| *decided == dictation_id)
            .map(|(_, at)| at)
    }
    /// 009's atomic Enter claim, under the same lock as `set_disposition`. A cancellation after
    /// the outcome was published moves the manager's generation on, so a late claim (Windows
    /// reliable paste) is stale too.
    ///
    /// # Errors
    /// `Stale`, `Disposition(current)` unless `Insert`, or `AlreadyClaimed`.
    pub fn claim_enter(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<EnterClaim, EnterRefusal> {
        let mut state = self.lock();
        let moved_on = state.generation != generation;
        let session = state.session.as_mut().ok_or(EnterRefusal::Stale)?;
        // A published dictation keeps `Insert`; only the manager's generation records a later
        // cancellation (a live session's cancellation already reports `Cancelled`).
        if moved_on && session.disposition == RecoveryDisposition::Insert {
            return Err(EnterRefusal::Stale);
        }
        session.claim_enter(dictation_id, generation)
    }
    /// Current menu of the exact dictation/version without favorites; see `menu::open`.
    ///
    /// # Errors
    /// `Stale` or `Frozen`.
    pub fn menu(
        &self,
        dictation_id: DictationId,
        version: u64,
    ) -> Result<DestinationMenu, SelectionRefusal> {
        super::menu::open(self, dictation_id, version, &[])
    }
    /// Apply a badge choice; see `menu::choose`.
    ///
    /// # Errors
    /// `Stale`, `Frozen` or `Unavailable`; the session is unchanged.
    pub fn choose(
        &self,
        dictation_id: DictationId,
        version: u64,
        choice: MenuChoice,
    ) -> Result<DestinationSnapshot, SelectionRefusal> {
        super::menu::choose(self, dictation_id, version, choice)
    }
}

/// Support and selection of one foreground read: another application becomes the captured
/// destination; Handy in front means no destination (but support); unreadable is unsupported.
fn selection_of(foreground: Foreground) -> (DestinationSupport, Selection) {
    match foreground {
        Foreground::Application(app) => (
            DestinationSupport::Supported,
            Selection::Application(Destination {
                identity: app.identity,
                display_name: app.display_name,
                icon_ref: app.icon_ref,
                captured_pid: app.pid,
                source: super::context::DestinationSource::Captured,
            }),
        ),
        Foreground::Handy => (DestinationSupport::Supported, Selection::FrontmostAtEnd),
        Foreground::Unreadable => (
            DestinationSupport::Unsupported(SupportReason::FrontmostUnreadable),
            Selection::FrontmostAtEnd,
        ),
    }
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
