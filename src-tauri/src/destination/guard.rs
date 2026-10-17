//! Pre-dispatch guard: called by the insertion method after its configured delay, immediately
//! before the first key, text or script. It refuses a cancelled or superseded dictation and a
//! destination that is no longer frontmost (no retry, no reactivation), builds the
//! `DestinationContext`, decides 011's field return, asks the optional 010 check, and sets the
//! disposition to `Insert`. Later calls by the same method (a fallback path) only recheck
//! foreground and cancellation.
use super::context::{
    DestinationContext, DestinationSource, PreInsertionVerdict, RecoveryDisposition, RecoveryPort,
    RecoveryReason, ResolvedTarget,
};
use super::field_attempt::FieldReturn;
use super::field_check::FieldCheck;
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
    /// The 010 pre-insertion check asked to recover instead, for this reason.
    RecoverInstead(RecoveryReason),
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
    /// 011's decision of the first pass (`Absent` without a remembered input).
    field: FieldCheck,
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
    /// 011's attempt, verified once by the first pass then released.
    field: Mutex<Option<FieldReturn>>,
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
            field: Mutex::new(None),
        }
    }
    #[must_use]
    /// The same guard verifying 011's `field` attempt at its first pass (none: as delivered).
    pub fn with_field(self, field: Option<FieldReturn>) -> Self {
        *self.field.lock().unwrap_or_else(PoisonError::into_inner) = field;
        self
    }
    // @spec FR-002: Re-check the destination before inserting — .specs/features/006-destination-application/spec.md#fr-002
    /// The check run right before the first dispatch; see the module documentation.
    ///
    /// # Errors
    /// The refusal, also recorded for the orchestrator; no input may follow it.
    pub fn check(&self) -> Result<DestinationContext, GuardRefusal> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let checked = match self.recheck() {
            Ok(context) if state.context.is_some() => Ok(context),
            Ok(context) => {
                state.field = self.field_check();
                self.first_pass(context, state.field)
            }
            Err(refusal) => Err(refusal),
        };
        match checked {
            Ok(context) => {
                let context = DestinationContext {
                    field: state.field,
                    ..context
                };
                state.context = Some(context.clone());
                Ok(context)
            }
            Err(refusal) => {
                state.refusal = Some(refusal);
                Err(refusal)
            }
        }
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
            field: FieldCheck::Absent,
        })
    }
    // @spec FR-004: Verify the remembered input before any paste — .specs/features/011-dictation-field-return/spec.md#fr-004
    /// 011's verification of the attempt handed to this guard, once; `Absent` without one. The
    /// attempt's references are released right after the decision (spec AC-008).
    fn field_check(&self) -> FieldCheck {
        let field = self
            .field
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        field.map_or(FieldCheck::Absent, super::field_check::verify)
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
    // @spec FR-001: Verdict, then a fresh recheck before Insert — .specs/features/010-recovery-bubble/spec.md#fr-001
    /// Business rule (plan "DestinationContext and RecoveryDisposition", 010 round 6): the 010
    /// verdict comes first; after an `Insert` verdict cancellation, supersession and the
    /// foreground are read once more, then the disposition becomes `Insert` immediately before
    /// the first dispatch. A `Recover` verdict dispatches nothing and keeps its reason.
    fn first_pass(
        &self,
        context: DestinationContext,
        field: FieldCheck,
    ) -> Result<DestinationContext, GuardRefusal> {
        let (id, generation) = (context.dictation_id, context.destination_generation);
        let context = DestinationContext { field, ..context };
        let (next, context) = match self.verdict(&context) {
            PreInsertionVerdict::Insert => (RecoveryDisposition::Insert, self.recheck()?),
            PreInsertionVerdict::Recover(reason) => {
                let set =
                    self.manager
                        .set_disposition(id, generation, RecoveryDisposition::Recover);
                return Err(set.map_or(GuardRefusal::Cancelled, |_| {
                    GuardRefusal::RecoverInstead(reason)
                }));
            }
        };
        match self.manager.set_disposition(id, generation, next) {
            Ok(_) => Ok(context),
            Err(_) => Err(GuardRefusal::Cancelled),
        }
    }
    // @spec AC-005: A verified input counts as editable for 010 — .specs/features/011-dictation-field-return/spec.md#ac-005
    /// Business rule (spec 011 AC-004, AC-005): a diverted field return never pastes, whatever
    /// the recovery port answers (010's answers the same, without any accessibility query);
    /// otherwise the port decides from the context, where a verified input counts as editable.
    fn verdict(&self, context: &DestinationContext) -> PreInsertionVerdict {
        match context.field {
            FieldCheck::Diverted(_) => {
                PreInsertionVerdict::Recover(RecoveryReason::FieldReturnFailed)
            }
            FieldCheck::Absent | FieldCheck::Verified | FieldCheck::Unchanged(_) => {
                self.recovery.before_insertion(context)
            }
        }
    }
}
