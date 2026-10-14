//! Pre-dispatch guard: called by the insertion method after its configured delay, immediately
//! before the first key, text or script. It refuses a cancelled or superseded dictation and a
//! destination that is no longer frontmost (no retry, no reactivation), builds the
//! `DestinationContext`, asks the optional 010 check, and sets the disposition to `Insert`.
//! Later calls by the same method (a fallback path) only recheck foreground and cancellation.
use super::context::{
    DestinationContext, DestinationSource, PreInsertionVerdict, RecoveryDisposition, RecoveryPort,
    ResolvedTarget,
};
use super::manager::{DestinationManager, FrozenDestination};
use super::workspace::{Foreground, RunningApp};
use std::sync::{Arc, Mutex, PoisonError};

/// Why the guard refused; no input was dispatched.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardRefusal {
    /// Cancelled, superseded or already finished.
    Cancelled,
    /// The activated destination is no longer frontmost (counts as an activation failure).
    TargetLost,
    /// The 010 pre-insertion check asked to recover instead.
    RecoverInstead,
}
impl std::fmt::Display for GuardRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "insertion stopped before dispatch: {self:?}")
    }
}
impl std::error::Error for GuardRefusal {}

#[derive(Default)]
struct GuardState {
    context: Option<DestinationContext>,
    refusal: Option<GuardRefusal>,
}

/// One dictation's guard, shared with the insertion method (it may run on the main thread).
pub struct DispatchGuard {
    manager: Arc<DestinationManager>,
    recovery: Arc<dyn RecoveryPort>,
    frozen: FrozenDestination,
    /// The activated instance; `None` for the frontmost-at-the-end route.
    target: Option<RunningApp>,
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
    state: Mutex<GuardState>,
}

impl DispatchGuard {
    /// Guard of `frozen`, expecting `target` in front (or any app for frontmost-at-the-end).
    pub fn new(
        manager: Arc<DestinationManager>,
        recovery: Arc<dyn RecoveryPort>,
        frozen: FrozenDestination,
        target: Option<RunningApp>,
        cancelled: Box<dyn Fn() -> bool + Send + Sync>,
    ) -> Self {
        Self {
            manager,
            recovery,
            frozen,
            target,
            cancelled,
            state: Mutex::new(GuardState::default()),
        }
    }
    // @spec FR-002: Re-check the destination before inserting — .specs/features/006-destination-application/spec.md#fr-002
    /// The check run right before the first dispatch; see the module documentation.
    ///
    /// # Errors
    /// The refusal, also recorded for the orchestrator; no input may follow it.
    pub fn check(&self) -> Result<DestinationContext, GuardRefusal> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let checked = self.recheck().and_then(|context| match &state.context {
            Some(_) => Ok(context),
            None => self.first_pass(context),
        });
        match &checked {
            Ok(context) => state.context = Some(context.clone()),
            Err(refusal) => state.refusal = Some(*refusal),
        }
        checked
    }
    /// The first refusal, if any check refused.
    pub fn refusal(&self) -> Option<GuardRefusal> {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .refusal
    }
    /// The context of the last accepted check, if the method reached its first dispatch.
    pub fn context(&self) -> Option<DestinationContext> {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .context
            .clone()
    }
    /// Cancellation, then the foreground: the activated instance, or whatever is in front.
    fn recheck(&self) -> Result<DestinationContext, GuardRefusal> {
        let (id, generation) = (self.frozen.dictation_id, self.frozen.destination_generation);
        if (self.cancelled)() || !self.manager.is_current(id, generation) {
            return Err(GuardRefusal::Cancelled);
        }
        let target = self.target_in_front()?;
        let source = match self.target {
            Some(_) => self.frozen.selection.source(),
            None => DestinationSource::FrontmostAtEnd,
        };
        Ok(DestinationContext {
            dictation_id: id,
            destination_generation: generation,
            source,
            target: target.map(|app| ResolvedTarget {
                pid: app.pid,
                identity: app.identity,
            }),
        })
    }
    /// The activated instance must still be in front; without one, whatever app is in front.
    fn target_in_front(&self) -> Result<Option<RunningApp>, GuardRefusal> {
        // Live read: this runs on the main thread right after the configured delay.
        let front = match self.manager.workspace.frontmost_now() {
            Foreground::Application(app) => Some(app),
            Foreground::Handy | Foreground::Unreadable => None,
        };
        match (&self.target, front) {
            (Some(expected), Some(app))
                if app.pid == expected.pid && app.identity == expected.identity =>
            {
                Ok(Some(app))
            }
            (Some(_), _) => Err(GuardRefusal::TargetLost),
            (None, front) => Ok(front),
        }
    }
    /// Business rule (plan "DestinationContext and RecoveryDisposition"): the 010 verdict comes
    /// first, then the disposition becomes `Insert` immediately before the first dispatch.
    fn first_pass(&self, context: DestinationContext) -> Result<DestinationContext, GuardRefusal> {
        let (id, generation) = (context.dictation_id, context.destination_generation);
        let next = match self.recovery.before_insertion(&context) {
            PreInsertionVerdict::Insert => RecoveryDisposition::Insert,
            PreInsertionVerdict::Recover(_) => RecoveryDisposition::Recover,
        };
        let set = self.manager.set_disposition(id, generation, next);
        match (next, set) {
            (RecoveryDisposition::Insert, Ok(_)) => Ok(context),
            (_, Ok(_)) => Err(GuardRefusal::RecoverInstead),
            (_, Err(_)) => Err(GuardRefusal::Cancelled),
        }
    }
}
