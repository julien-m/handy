//! Real orchestrator, manager and guard over fake workspace, clock and insertion method: the
//! captured app is activated before dispatch, closed/late/lost destinations insert nothing and
//! are never retried, the 1 s deadline boundaries, the preserved insertion delay, paste method
//! "none", destination return off, cancellation and the 010 recovery verdict.
use super::*;
use crate::destination::context::{DestinationContext, PreInsertionVerdict, RecoveryReason};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::session::EnterRefusal;
use crate::destination::workspace::WorkspacePort;
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Workspace whose activation brings the target in front `delay` after the request; `live`,
/// when set, is what the live read sees while the cached `frontmost` still shows the old app.
struct ScheduledWorkspace {
    inner: FakeWorkspace,
    clock: Arc<FakeClock>,
    delay: Option<Duration>,
    front_at: Mutex<Option<(std::time::Instant, RunningApp)>>,
    live: Mutex<Option<Foreground>>,
}
impl WorkspacePort for ScheduledWorkspace {
    fn frontmost_now(&self) -> Foreground {
        let live = self.live.lock().ok().and_then(|live| live.clone());
        live.unwrap_or_else(|| self.frontmost())
    }
    fn frontmost(&self) -> Foreground {
        let scheduled = self.front_at.lock().ok().and_then(|at| at.clone());
        match scheduled {
            Some((at, app)) if self.clock.now() >= at => Foreground::Application(app),
            _ => self.inner.frontmost(),
        }
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        self.inner.running_windowed_apps()
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        self.inner.instances(identity)
    }
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        self.inner
            .activations
            .lock()
            .map(|mut list| list.push(target.pid))
            .ok();
        if let (Some(delay), Ok(mut at)) = (self.delay, self.front_at.lock()) {
            *at = Some((self.clock.now() + delay, target.clone()));
        }
        ActivationRequest::Sent
    }
}
/// Insertion method double: runs `during_delay`, then the guard, and records each dispatch.
struct Method {
    during_delay: Box<dyn Fn() + Send + Sync>,
    result: MethodResult,
    calls_guard: bool,
    inputs: Mutex<Vec<DestinationContext>>,
}
impl InsertionPort for Method {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        (self.during_delay)();
        if !self.calls_guard {
            return self.result.clone();
        }
        match guard.check() {
            Ok(context) => {
                self.inputs
                    .lock()
                    .map(|mut inputs| inputs.push(context))
                    .ok();
                self.result.clone()
            }
            Err(refusal) => MethodResult::Legacy(Err(refusal.to_string())),
        }
    }
}
fn method() -> Method {
    Method {
        during_delay: Box::new(|| {}),
        result: MethodResult::Legacy(Ok(())),
        calls_guard: true,
        inputs: Mutex::new(Vec::new()),
    }
}
fn inputs(method: &Method) -> Vec<DestinationContext> {
    method
        .inputs
        .lock()
        .map(|inputs| inputs.clone())
        .unwrap_or_default()
}
struct Recover;
impl RecoveryPort for Recover {
    fn before_insertion(&self, _context: &DestinationContext) -> PreInsertionVerdict {
        PreInsertionVerdict::Recover(RecoveryReason::NoTextField)
    }
}
struct Fixture {
    workspace: Arc<ScheduledWorkspace>,
    clock: Arc<FakeClock>,
    orchestrator: Orchestrator,
}
fn textedit() -> RunningApp {
    app(11, "com.apple.TextEdit", "TextEdit")
}
fn safari() -> RunningApp {
    app(22, "com.apple.Safari", "Safari")
}
/// TextEdit in front, TextEdit and Safari running.
fn scheduled_workspace(clock: Arc<FakeClock>, delay: Option<Duration>) -> ScheduledWorkspace {
    let inner = FakeWorkspace::new(
        Foreground::Application(textedit()),
        vec![textedit(), safari()],
    );
    ScheduledWorkspace {
        inner,
        clock,
        delay,
        front_at: Mutex::new(None),
        live: Mutex::new(None),
    }
}
fn fixture(delay: Option<Duration>, recovery: Arc<dyn RecoveryPort>) -> Fixture {
    let clock = Arc::new(FakeClock::new());
    let workspace = Arc::new(scheduled_workspace(clock.clone(), delay));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace.clone(),
        clock.clone(),
        platform,
    ));
    let orchestrator = Orchestrator {
        manager,
        clock: clock.clone(),
        pause: clock.clone(),
        recovery,
    };
    Fixture {
        workspace,
        clock,
        orchestrator,
    }
}
fn default_fixture() -> Fixture {
    fixture(
        Some(Duration::from_millis(100)),
        Arc::new(crate::destination::context::NoRecoveryConsumer),
    )
}
/// Start a dictation in TextEdit (or with `return_enabled` off), then switch to Safari.
fn start(fixture: &Fixture, return_enabled: bool) -> InsertionRequest {
    let manager = &fixture.orchestrator.manager;
    let started = manager.commit_start(manager.capture(return_enabled));
    fixture
        .workspace
        .inner
        .set_foreground(Foreground::Application(safari()));
    manager.mark_processing();
    request(
        started.dictation_id,
        started.destination_generation,
        false,
        Arc::new(AtomicBool::new(false)),
    )
}
fn request(
    dictation_id: super::super::context::DictationId,
    destination_generation: u64,
    paste_method_none: bool,
    cancelled: Arc<AtomicBool>,
) -> InsertionRequest {
    InsertionRequest {
        dictation_id,
        destination_generation,
        paste_method_none,
        cancelled: Box::new(move || cancelled.load(Ordering::SeqCst)),
    }
}
fn activations(fixture: &Fixture) -> usize {
    fixture.workspace.inner.activation_count()
}
fn disposition(fixture: &Fixture) -> Option<RecoveryDisposition> {
    fixture
        .orchestrator
        .manager
        .snapshot()
        .map(|snapshot| snapshot.disposition)
}

#[path = "insertion_cases.rs"]
mod cases;
