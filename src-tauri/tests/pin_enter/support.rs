//! Test doubles shared by the 009 integration tests: a scripted workspace (foreground, running
//! applications, activation), a manual monotonic clock and pause, a key spy, a pass-through
//! insertion method, and builders for 006's real manager, orchestrator and 009's real Enter
//! transaction. External effects only are faked; the manager, guard, orchestrator, policy and
//! claim are the production code.
#![allow(dead_code)]

use handy_app_lib::automatic_enter::transaction::{
    EnterPorts, EnterSnapshot, EnterTransaction, KeyEmitter, KeyOutcome,
};
use handy_app_lib::automatic_enter::{AutoSubmitKey, AutomaticEnterPolicy};
use handy_app_lib::destination::context::{
    ApplicationIdentity, DestinationContext, NoRecoveryConsumer, RecoveryDisposition,
    ResolvedTarget,
};
use handy_app_lib::destination::guard::DispatchGuard;
use handy_app_lib::destination::insertion::{
    InsertionPort, InsertionReport, InsertionRequest, Orchestrator,
};
use handy_app_lib::destination::manager::DestinationManager;
use handy_app_lib::destination::outcome::MethodResult;
use handy_app_lib::destination::snapshot::DestinationSnapshot;
use handy_app_lib::destination::workspace::{
    ActivationRequest, Clock, Foreground, Pause, RunningApp, WorkspacePort,
};
use handy_app_lib::settings_service::platform_capability::{
    PlatformCapabilities, PlatformCapability,
};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Running application `name` with bundle identifier `bundle` and process `pid`.
pub fn app(pid: u32, bundle: &str, name: &str) -> RunningApp {
    RunningApp {
        pid,
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: name.to_owned(),
        icon_ref: None,
    }
}
pub fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
pub fn comet() -> RunningApp {
    app(22, "ai.perplexity.comet", "Comet")
}
pub fn safari() -> RunningApp {
    app(33, "com.apple.Safari", "Safari")
}
pub fn notes() -> RunningApp {
    app(44, "com.apple.Notes", "Notes")
}
pub fn textedit() -> RunningApp {
    app(55, "com.apple.TextEdit", "TextEdit")
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Scripted workspace: an accepted activation brings its target in front when it works.
pub struct Workspace {
    pub foreground: Mutex<Foreground>,
    pub running: Mutex<Vec<RunningApp>>,
    pub activations: Mutex<Vec<u32>>,
    pub activation_works: Mutex<bool>,
}
impl Workspace {
    pub fn new(front: RunningApp, running: Vec<RunningApp>) -> Arc<Self> {
        Arc::new(Self {
            foreground: Mutex::new(Foreground::Application(front)),
            running: Mutex::new(running),
            activations: Mutex::new(Vec::new()),
            activation_works: Mutex::new(true),
        })
    }
    pub fn bring(&self, front: RunningApp) {
        *lock(&self.foreground) = Foreground::Application(front);
    }
    pub fn set_running(&self, running: Vec<RunningApp>) {
        *lock(&self.running) = running;
    }
    pub fn activation_works(&self, works: bool) {
        *lock(&self.activation_works) = works;
    }
    pub fn activations(&self) -> Vec<u32> {
        lock(&self.activations).clone()
    }
}
impl WorkspacePort for Workspace {
    fn frontmost(&self) -> Foreground {
        lock(&self.foreground).clone()
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        lock(&self.running).clone()
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        lock(&self.running)
            .iter()
            .filter(|app| app.identity == *identity)
            .cloned()
            .collect()
    }
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        lock(&self.activations).push(target.pid);
        if *lock(&self.activation_works) {
            self.bring(target.clone());
        }
        ActivationRequest::Sent
    }
}

/// Manual monotonic clock; a pause advances it instead of sleeping.
pub struct ManualClock {
    now: Mutex<Instant>,
}
impl ManualClock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            now: Mutex::new(Instant::now()),
        })
    }
    pub fn advance(&self, duration: Duration) {
        *lock(&self.now) += duration;
    }
}
impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *lock(&self.now)
    }
}
impl Pause for ManualClock {
    fn pause(&self, duration: Duration) {
        self.advance(duration);
    }
}

/// Key spy recording the clock time of every press.
pub struct Keys {
    clock: Arc<ManualClock>,
    pub pressed: Mutex<Vec<Instant>>,
}
impl Keys {
    pub fn new(clock: &Arc<ManualClock>) -> Arc<Self> {
        Arc::new(Self {
            clock: Arc::clone(clock),
            pressed: Mutex::new(Vec::new()),
        })
    }
    pub fn count(&self) -> usize {
        lock(&self.pressed).len()
    }
}
impl KeyEmitter for Keys {
    fn send(&self, _key: AutoSubmitKey, _expected: Option<&ResolvedTarget>) -> KeyOutcome {
        lock(&self.pressed).push(self.clock.now());
        KeyOutcome::Sent
    }
}

/// Insertion method double: passes the guard and returns `result`.
pub struct Method {
    pub result: MethodResult,
    pub contexts: Mutex<Vec<DestinationContext>>,
}
impl Method {
    pub fn succeeding() -> Self {
        Self::returning(MethodResult::Legacy(Ok(())))
    }
    pub fn returning(result: MethodResult) -> Self {
        Self {
            result,
            contexts: Mutex::new(Vec::new()),
        }
    }
    pub fn dispatched(&self) -> Vec<DestinationContext> {
        lock(&self.contexts).clone()
    }
}
impl InsertionPort for Method {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        match guard.check() {
            Ok(context) => {
                lock(&self.contexts).push(context);
                self.result.clone()
            }
            Err(refusal) => MethodResult::Legacy(Err(refusal.to_string())),
        }
    }
}

/// 006's real manager and orchestrator over `workspace` and `clock`.
pub struct Rig {
    pub workspace: Arc<Workspace>,
    pub clock: Arc<ManualClock>,
    pub manager: Arc<DestinationManager>,
    pub keys: Arc<Keys>,
}
impl Rig {
    pub fn new(front: RunningApp, running: Vec<RunningApp>) -> Self {
        let workspace = Workspace::new(front, running);
        let clock = ManualClock::new();
        let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
        let manager = Arc::new(DestinationManager::new(
            workspace.clone(),
            clock.clone(),
            platform,
        ));
        let keys = Keys::new(&clock);
        Self {
            workspace,
            clock,
            manager,
            keys,
        }
    }
    /// A recording that actually started, with destination return `return_on`.
    pub fn start(&self, return_on: bool) -> DestinationSnapshot {
        self.manager.commit_start(self.manager.capture(return_on))
    }
    pub fn orchestrator(&self) -> Orchestrator {
        Orchestrator {
            manager: Arc::clone(&self.manager),
            clock: self.clock.clone(),
            pause: self.clock.clone(),
            recovery: Arc::new(NoRecoveryConsumer),
        }
    }
    /// Insert the dictation `started` with `method` (paste method none when `none`).
    pub fn deliver(
        &self,
        started: &DestinationSnapshot,
        method: &dyn InsertionPort,
        none: bool,
    ) -> InsertionReport {
        self.manager.mark_processing();
        let request = InsertionRequest {
            dictation_id: started.dictation_id,
            destination_generation: started.destination_generation,
            paste_method_none: none,
            cancelled: Box::new(|| false),
        };
        self.orchestrator().deliver(request, method)
    }
    /// 009's transaction of `started`, with `policy` frozen and Enter as key.
    pub fn enter(
        &self,
        started: &DestinationSnapshot,
        policy: AutomaticEnterPolicy,
    ) -> Arc<EnterTransaction> {
        let frozen = EnterSnapshot {
            dictation_id: started.dictation_id,
            destination_generation: started.destination_generation,
            policy,
            key: AutoSubmitKey::Enter,
        };
        let ports = EnterPorts {
            claims: self.manager.clone(),
            workspace: self.workspace.clone(),
            keys: self.keys.clone(),
            clock: self.clock.clone(),
            pause: self.clock.clone(),
        };
        Arc::new(EnterTransaction::new(frozen, ports))
    }
    pub fn disposition(&self) -> Option<RecoveryDisposition> {
        self.manager.snapshot().map(|snapshot| snapshot.disposition)
    }
    pub fn clock_now(&self) -> Instant {
        self.clock.now()
    }
}

/// One started dictation and the context its method was dispatched with.
pub struct DictationOf {
    pub started: DestinationSnapshot,
    pub context: DestinationContext,
}
