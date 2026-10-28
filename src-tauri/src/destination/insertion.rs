//! Insertion orchestrator (worker thread): freeze the destination, skip activation for paste
//! method "none", resolve the chosen application, run the activation chain (`activation`: the
//! workspace request, then the accessibility request when the first is declined or has not
//! brought it forward) while waiting for it in front for at most 1 s in all by bounded sampling
//! (never on the main thread), then hand the existing insertion method a `DispatchGuard` it
//! calls after its configured delay. A failed destination never falls back to another
//! application and its guard is never retried.
use super::activation::{ActivationChain, ActivationStrategy, ChainStep};
use super::context::{
    ApplicationIdentity, DestinationContext, DestinationSource, RecoveryDisposition, RecoveryPort,
};
use super::field::RememberedField;
use super::guard::{DispatchGuard, GuardRefusal};
use super::manager::{DestinationManager, FrozenDestination};
use super::outcome::{map_method_result, DestinationFailure, InsertionStatus, MethodResult};
use super::session::{Destination, DestinationSupport, Selection};
use super::workspace::{
    resolve_instance, ActivationRequest, Clock, Foreground, Pause, Resolution, RunningApp,
};
use std::sync::Arc;
use std::time::Duration;

/// Bounded wait for the activated destination to be in front (AC-002).
pub const ACTIVATION_DEADLINE: Duration = Duration::from_secs(1);
// Foreground sampling period: 20 ms keeps the observed latency well under the 1 s budget for
// about 50 cheap reads, with no busy loop.
const SAMPLE_PERIOD: Duration = Duration::from_millis(20);

/// The existing insertion method, run with the guard (on the main thread in production).
pub trait InsertionPort: Send + Sync {
    /// Insert the final text; the method calls `guard.check()` before its first dispatch.
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult;
}

/// Ports of the orchestrator.
pub struct Orchestrator {
    pub manager: Arc<DestinationManager>,
    pub clock: Arc<dyn Clock>,
    pub pause: Arc<dyn Pause>,
    pub recovery: Arc<dyn RecoveryPort>,
}
/// One insertion request.
pub struct InsertionRequest {
    pub dictation_id: super::context::DictationId,
    pub destination_generation: u64,
    /// Paste method "none": no activation, `skipped` outcome.
    pub paste_method_none: bool,
    pub cancelled: Box<dyn Fn() -> bool + Send + Sync>,
}
/// What happened, before history and conservation are added.
#[derive(Clone, Debug, PartialEq)]
pub struct InsertionReport {
    pub status: InsertionStatus,
    pub reason: Option<DestinationFailure>,
    pub method_result: Option<MethodResult>,
    pub target: Option<ApplicationIdentity>,
    pub source: DestinationSource,
    /// The context of the guard's last accepted check (the route actually dispatched, with the
    /// activated instance), read by 009's Enter authority; `None` when nothing was dispatched.
    /// Boxed so the report (also an early-return error value) stays small.
    pub context: Option<Box<DestinationContext>>,
}
impl InsertionReport {
    fn ended(
        status: InsertionStatus,
        reason: Option<DestinationFailure>,
        source: DestinationSource,
    ) -> Self {
        let (method_result, target, context) = (None, None, None);
        Self {
            status,
            reason,
            method_result,
            target,
            source,
            context,
        }
    }
}

impl Orchestrator {
    // @spec FR-002: Activate then delegate to the configured method — .specs/features/006-destination-application/spec.md#fr-002
    /// Deliver one dictation's text to its destination; see the module documentation.
    pub fn deliver(
        &self,
        request: InsertionRequest,
        method: &dyn InsertionPort,
    ) -> InsertionReport {
        let (frozen, field) = match self.freeze(&request) {
            Ok(frozen) => frozen,
            Err(report) => return report,
        };
        let target = match activation_route(&frozen, request.paste_method_none) {
            Some(destination) => match self.activate(&frozen, &destination, &*request.cancelled) {
                Ok(app) => Some(app),
                Err(report) => return self.fail(&frozen, report),
            },
            None => None,
        };
        // 011: return to the input remembered at the start, only in its activated instance.
        let access = self.manager.field_access();
        let clock = Arc::clone(&self.clock);
        let cancelled: &dyn Fn() -> bool = &*request.cancelled;
        let route = (target.as_ref(), cancelled);
        let field = super::field_attempt::begin(access, clock, field, route);
        let manager = Arc::clone(&self.manager);
        let recovery = Arc::clone(&self.recovery);
        let guard =
            DispatchGuard::new(manager, recovery, frozen.clone(), target, request.cancelled);
        let guard = Arc::new(guard.with_field(field));
        let result = method.dispatch(Arc::clone(&guard));
        self.conclude(&frozen, &guard, result, request.paste_method_none)
    }
    /// Insertion entry: a cancelled or superseded dictation ends here without activation; the
    /// input 011 remembered comes out with the frozen selection when it still serves it.
    fn freeze(
        &self,
        request: &InsertionRequest,
    ) -> Result<(FrozenDestination, Option<RememberedField>), InsertionReport> {
        let (id, generation) = (request.dictation_id, request.destination_generation);
        let cancelled = InsertionReport::ended(
            InsertionStatus::Cancelled,
            None,
            DestinationSource::FrontmostAtEnd,
        );
        if (request.cancelled)() {
            let _ = self
                .manager
                .set_disposition(id, generation, RecoveryDisposition::Cancelled);
            return Err(cancelled);
        }
        self.manager
            .freeze_with_field(id, generation)
            .map_err(|_| cancelled)
    }
    // @spec FR-004: No fallback when the destination is unavailable — .specs/features/006-destination-application/spec.md#fr-004
    /// Resolve, run the activation chain, and wait for the destination in front within the
    /// deadline.
    fn activate(
        &self,
        frozen: &FrozenDestination,
        destination: &Destination,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<RunningApp, InsertionReport> {
        let app = self.resolve(destination)?;
        let started = self.clock.now();
        let mut chain = ActivationChain::default();
        let waited = self.wait_in_front(&app, started, cancelled, &mut chain);
        if !chain.attempts().is_empty() {
            // No identity or title: the strategies, the system's answers and their times only.
            log::debug!(
                "Destination activation {waited:?} after {:?}: {:?}",
                self.clock.now().saturating_duration_since(started),
                chain.attempts()
            );
        }
        match waited {
            WaitResult::InFront => Ok(app),
            WaitResult::Cancelled => Err(InsertionReport::ended(
                InsertionStatus::Cancelled,
                None,
                frozen.selection.source(),
            )),
            WaitResult::TimedOut => Err(failed(destination, DestinationFailure::ActivationFailed)),
        }
    }
    /// The running instance of `destination` (never another application).
    fn resolve(&self, destination: &Destination) -> Result<RunningApp, InsertionReport> {
        let instances = self.manager.workspace.instances(&destination.identity);
        match resolve_instance(&destination.identity, destination.captured_pid, &instances) {
            Resolution::Running(app) => Ok(app),
            Resolution::Closed => Err(failed(destination, closed_reason(destination))),
            Resolution::Ambiguous => Err(failed(destination, DestinationFailure::ActivationFailed)),
        }
    }
    /// Same live foreground source as the guard, so the wait and the guard never disagree.
    fn in_front(&self, app: &RunningApp) -> bool {
        matches!(self.manager.workspace.frontmost_now(), Foreground::Application(front) if front.pid == app.pid && front.identity == app.identity)
    }
    /// Sample the foreground until `app` is in front, the dictation is cancelled, or more than
    /// `ACTIVATION_DEADLINE` has elapsed since `started` (an observation at exactly 1 s counts).
    /// Between samples the chain sends its due requests; an application already in front gets
    /// none, and a chain whose every request was declined ends the wait at once.
    fn wait_in_front(
        &self,
        app: &RunningApp,
        started: std::time::Instant,
        cancelled: &dyn Fn() -> bool,
        chain: &mut ActivationChain,
    ) -> WaitResult {
        loop {
            if cancelled() {
                return WaitResult::Cancelled;
            }
            let elapsed = self.clock.now().saturating_duration_since(started);
            if self.in_front(app) && elapsed <= ACTIVATION_DEADLINE {
                return WaitResult::InFront;
            }
            // Exit once the deadline has passed without the destination in front.
            if elapsed >= ACTIVATION_DEADLINE {
                return WaitResult::TimedOut;
            }
            if let Some(ended) = self.advance(app, chain, elapsed) {
                return ended;
            }
        }
    }
    /// One step of the chain between two foreground samples: send the due request, or pause
    /// until the next sample; `Some` ends the wait.
    fn advance(
        &self,
        app: &RunningApp,
        chain: &mut ActivationChain,
        elapsed: Duration,
    ) -> Option<WaitResult> {
        match chain.step(elapsed) {
            ChainStep::Request(strategy) => {
                chain.record(strategy, self.request(app, strategy), elapsed);
            }
            // Nothing was accepted: no request is pending that could bring `app` forward.
            ChainStep::Exhausted => return Some(WaitResult::TimedOut),
            ChainStep::Wait => self
                .pause
                .pause(SAMPLE_PERIOD.min(ACTIVATION_DEADLINE - elapsed)),
        }
        None
    }
    /// One request of `strategy` for `app` (never another application).
    fn request(&self, app: &RunningApp, strategy: ActivationStrategy) -> ActivationRequest {
        let workspace = &self.manager.workspace;
        match strategy {
            ActivationStrategy::Workspace => workspace.activate_once(app),
            ActivationStrategy::Accessibility => workspace.activate_by_accessibility(app),
        }
    }
    /// A destination or method failure: the disposition becomes `Recover` (or `Cancelled`).
    fn fail(&self, frozen: &FrozenDestination, report: InsertionReport) -> InsertionReport {
        let next = match report.status {
            InsertionStatus::Cancelled => RecoveryDisposition::Cancelled,
            _ => RecoveryDisposition::Recover,
        };
        let _ =
            self.manager
                .set_disposition(frozen.dictation_id, frozen.destination_generation, next);
        report
    }
    /// Map the guard refusal or the method result to the report and the disposition.
    fn conclude(
        &self,
        frozen: &FrozenDestination,
        guard: &DispatchGuard,
        result: MethodResult,
        skipped: bool,
    ) -> InsertionReport {
        let context = guard.context();
        let source = context
            .as_ref()
            .map_or(frozen.selection.source(), |context| context.source);
        let report = match (guard.refusal(), skipped) {
            (Some(refusal), _) => refusal_report(refusal, source),
            (None, true) => InsertionReport::ended(InsertionStatus::Skipped, None, source),
            (None, false) => method_report(result, context.clone(), source),
        };
        let report = InsertionReport {
            context: context.map(Box::new),
            ..report
        };
        if guard.refusal().is_some() || report.reason.is_some() {
            return self.fail(frozen, report);
        }
        report
    }
}

/// The method ran past the guard: its exact result, mapped (plan "Outcome" table).
fn method_report(
    result: MethodResult,
    context: Option<super::context::DestinationContext>,
    source: DestinationSource,
) -> InsertionReport {
    let (status, reason, _) = map_method_result(&result);
    let target = context
        .as_ref()
        .and_then(|context| context.target.as_ref())
        .map(|target| target.identity.clone());
    InsertionReport {
        status,
        reason,
        method_result: Some(result),
        target,
        source,
        context: context.map(Box::new),
    }
}

// @spec FR-002: Unavailable pin, typed reason, pin kept — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-002
/// Reason of a destination with no running instance: a pinned one is `pinned_app_not_running`
/// (009 AC-003), any other 006's `app_closed`.
fn closed_reason(destination: &Destination) -> DestinationFailure {
    match destination.source {
        DestinationSource::Pinned => DestinationFailure::PinnedAppNotRunning,
        _ => DestinationFailure::AppClosed,
    }
}

/// A destination failure that names the destination.
fn failed(destination: &Destination, reason: DestinationFailure) -> InsertionReport {
    InsertionReport {
        target: Some(destination.identity.clone()),
        ..InsertionReport::ended(InsertionStatus::Failed, Some(reason), destination.source)
    }
}
/// A guard refusal: no input was dispatched.
fn refusal_report(refusal: GuardRefusal, source: DestinationSource) -> InsertionReport {
    let failed = |reason| InsertionReport::ended(InsertionStatus::Failed, Some(reason), source);
    match refusal {
        GuardRefusal::Cancelled => InsertionReport::ended(InsertionStatus::Cancelled, None, source),
        GuardRefusal::TargetLost => failed(DestinationFailure::ActivationFailed),
        // 010: a pre-insertion verdict keeps its typed reason (never a generic failure).
        GuardRefusal::RecoverInstead(super::context::RecoveryReason::NoTextField) => {
            failed(DestinationFailure::NoTextField)
        }
        // 011 AC-004 (b): nothing was pasted; 010 shows "Insertion failed", nothing sent.
        GuardRefusal::RecoverInstead(super::context::RecoveryReason::FieldReturnFailed) => {
            failed(DestinationFailure::FieldReturnFailed)
        }
    }
}

#[derive(Debug)]
enum WaitResult {
    InFront,
    Cancelled,
    TimedOut,
}

// @spec AC-002: Pinned and badge-chosen apps are activated with return off — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-002
/// The application to activate, or `None` for the frontmost-at-the-end route: destination
/// return off or unsupported for a captured destination, no destination, or paste method "none".
/// Business rule (009, orchestrator decision of the delivery review): a pin and an application
/// chosen in the badge during the dictation are explicit choices, so both are activated even with
/// return off (the badge is shown then only because an app is pinned); a pinned route with an
/// unreadable foreground fails closed with `activation_failed`. Only the capture at start
/// depends on the setting and the runtime support.
fn activation_route(
    frozen: &FrozenDestination,
    paste_method_none: bool,
) -> Option<super::session::Destination> {
    let Selection::Application(destination) = &frozen.selection else {
        return None;
    };
    let explicit = matches!(
        destination.source,
        DestinationSource::Pinned | DestinationSource::Badge
    );
    let captured_route = frozen.return_enabled && frozen.support == DestinationSupport::Supported;
    (!paste_method_none && (explicit || captured_route)).then(|| destination.clone())
}

#[cfg(test)]
#[path = "insertion_tests.rs"]
mod tests;
