//! Bench of the 011 integration tests: 006's real manager, orchestrator and guard (009's rig)
//! with the scripted accessibility world as field port, a method double that records which
//! element would receive the paste, and the scenario of Julien's request: Comet (pid 22) with
//! its ChatGPT input (element 1, window 100, caret inside a draft) and another element (2).
#![allow(dead_code)]

use super::destination::{comet, Keys, ManualClock, Rig, Workspace};
use super::fake::{Access, Element, World};
use handy_app_lib::destination::context::{DestinationContext, RecoveryPort};
use handy_app_lib::destination::field_capture::read_and_attach;
use handy_app_lib::destination::guard::DispatchGuard;
use handy_app_lib::destination::insertion::{
    InsertionPort, InsertionReport, InsertionRequest, Orchestrator,
};
use handy_app_lib::destination::manager::DestinationManager;
use handy_app_lib::destination::outcome::MethodResult;
use handy_app_lib::destination::snapshot::DestinationSnapshot;
use handy_app_lib::destination::workspace::RunningApp;
use handy_app_lib::settings_service::platform_capability::{
    PlatformCapabilities, PlatformCapability,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

/// The remembered input of the scenario.
pub const INPUT: u32 = 1;
/// Another element of Comet (the page after a reload, another tab's field…).
pub const OTHER: u32 = 2;
/// The input's window, focused.
pub const WINDOW: u32 = 100;
/// Caret inside the draft at the start.
pub const CARET: (usize, usize) = (12, 0);

/// 006's rig with the field world.
pub struct FieldRig {
    pub rig: Rig,
    pub world: Arc<World>,
}
impl FieldRig {
    /// `front` in front, `running` applications, and Comet's input focused with its caret.
    pub fn new(front: RunningApp, running: Vec<RunningApp>) -> Self {
        let workspace = Workspace::new(front, running);
        let clock = ManualClock::new();
        let world = World::new(&clock);
        let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
        let manager = DestinationManager::new(workspace.clone(), clock.clone(), platform)
            .with_field_access(Arc::new(Access(Arc::clone(&world))));
        let keys = Keys::new(&clock);
        let rig = Rig {
            workspace,
            clock,
            manager: Arc::new(manager),
            keys,
        };
        let comet = comet().pid;
        world.element(INPUT, Element::input(comet, WINDOW, CARET));
        world.element(OTHER, Element::input(comet, WINDOW, (0, 0)));
        world.element(WINDOW, Element::control(comet, "AXWindow"));
        world.focus(comet, Ok(INPUT));
        world.focus_window(comet, Ok(WINDOW));
        Self { rig, world }
    }
    /// Comet in front with its input focused; Notes running.
    pub fn comet() -> Self {
        Self::new(comet(), vec![comet(), super::destination::notes()])
    }
    /// A started dictation whose start read ran to completion (as the worker does).
    pub fn start(&self, return_on: bool) -> DestinationSnapshot {
        let started = self.rig.start(return_on);
        self.read(&started);
        started
    }
    /// Run the start read of `started` synchronously.
    pub fn read(&self, started: &DestinationSnapshot) {
        let target = self
            .rig
            .manager
            .field_target(started.dictation_id, started.destination_generation);
        if let Some(target) = target {
            read_and_attach(&self.rig.manager, target, self.rig.clock_now());
        }
    }
    /// Deliver `started` with `method`, 010's `recovery` port and a cancellation flag.
    pub fn deliver(
        &self,
        started: &DestinationSnapshot,
        method: &dyn InsertionPort,
        recovery: Arc<dyn RecoveryPort>,
        cancelled: &Arc<AtomicBool>,
    ) -> InsertionReport {
        self.rig.manager.mark_processing();
        let flag = Arc::clone(cancelled);
        let request = InsertionRequest {
            dictation_id: started.dictation_id,
            destination_generation: started.destination_generation,
            paste_method_none: false,
            cancelled: Box::new(move || flag.load(Ordering::SeqCst)),
        };
        let orchestrator = Orchestrator {
            recovery,
            ..self.rig.orchestrator()
        };
        orchestrator.deliver(request, method)
    }
    /// Deliver with 006's port that always inserts (a build without 010).
    pub fn deliver_plain(
        &self,
        started: &DestinationSnapshot,
        method: &dyn InsertionPort,
    ) -> InsertionReport {
        let recovery = Arc::new(handy_app_lib::destination::context::NoRecoveryConsumer);
        self.deliver(started, method, recovery, &Arc::new(AtomicBool::new(false)))
    }
    /// Field-port calls named `op` so far.
    pub fn calls(&self, op: &str) -> usize {
        self.world.count(op)
    }
}

/// Method double: runs `before_check` (the paste delay), then the guard; on success it records
/// the element of the target that would receive the paste and the context.
pub struct PasteSpy {
    world: Arc<World>,
    pub before_check: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub pasted: Mutex<Vec<Option<u32>>>,
    pub contexts: Mutex<Vec<DestinationContext>>,
}
impl PasteSpy {
    /// A spy reading which element of `world` is focused at dispatch.
    pub fn new(world: &Arc<World>) -> Self {
        Self {
            world: Arc::clone(world),
            before_check: Mutex::new(None),
            pasted: Mutex::new(Vec::new()),
            contexts: Mutex::new(Vec::new()),
        }
    }
    /// Run `effect` during the configured paste delay, before the guard.
    pub fn during_delay(self, effect: impl FnOnce() + Send + 'static) -> Self {
        *self
            .before_check
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Box::new(effect));
        self
    }
    /// The element each paste went to (`None`: no element focused in the target).
    pub fn pasted(&self) -> Vec<Option<u32>> {
        self.pasted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// The contexts the guard accepted.
    pub fn contexts(&self) -> Vec<DestinationContext> {
        self.contexts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
impl InsertionPort for PasteSpy {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        let effect = self
            .before_check
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(effect) = effect {
            effect();
        }
        match guard.check() {
            Ok(context) => {
                let pid = context.target.as_ref().map(|target| target.pid);
                let element = pid.and_then(|pid| self.world.focused(pid));
                self.pasted
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(element);
                self.contexts
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(context);
                MethodResult::Legacy(Ok(()))
            }
            Err(refusal) => MethodResult::Legacy(Err(refusal.to_string())),
        }
    }
}
