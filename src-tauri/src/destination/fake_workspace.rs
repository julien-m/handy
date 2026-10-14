//! Test doubles of the workspace boundary: a scripted foreground, running applications,
//! recorded activation requests and a manual monotonic clock.
use super::context::ApplicationIdentity;
use super::workspace::{ActivationRequest, Clock, Foreground, Pause, RunningApp, WorkspacePort};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Running application `name` with bundle identifier `bundle` and process `pid`.
pub(crate) fn app(pid: u32, bundle: &str, name: &str) -> RunningApp {
    RunningApp {
        pid,
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: name.to_owned(),
        icon_ref: None,
    }
}

/// Scripted workspace: `foreground` is what `frontmost` returns; an accepted activation makes
/// its target frontmost when `activation_works` is true.
pub(crate) struct FakeWorkspace {
    pub foreground: Mutex<Foreground>,
    pub running: Mutex<Vec<RunningApp>>,
    pub activations: Mutex<Vec<u32>>,
    pub activation_works: Mutex<bool>,
    pub frontmost_reads: Mutex<u32>,
}
impl FakeWorkspace {
    /// Workspace showing `foreground` with `running` applications.
    pub(crate) fn new(foreground: Foreground, running: Vec<RunningApp>) -> Self {
        Self {
            foreground: Mutex::new(foreground),
            running: Mutex::new(running),
            activations: Mutex::new(Vec::new()),
            activation_works: Mutex::new(true),
            frontmost_reads: Mutex::new(0),
        }
    }
    /// Put `foreground` in front.
    pub(crate) fn set_foreground(&self, foreground: Foreground) {
        *self
            .foreground
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = foreground;
    }
    /// Replace the running applications.
    pub(crate) fn set_running(&self, running: Vec<RunningApp>) {
        *self.running.lock().unwrap_or_else(PoisonError::into_inner) = running;
    }
    /// Number of activation requests received.
    pub(crate) fn activation_count(&self) -> usize {
        self.activations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
impl WorkspacePort for FakeWorkspace {
    fn frontmost(&self) -> Foreground {
        *self
            .frontmost_reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner) += 1;
        self.foreground
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running
            .iter()
            .filter(|app| app.identity == *identity)
            .cloned()
            .collect()
    }
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        self.activations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(target.pid);
        if *self
            .activation_works
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
        {
            self.set_foreground(Foreground::Application(target.clone()));
        }
        ActivationRequest::Sent
    }
}

/// Manual monotonic clock; `pause` advances it instead of sleeping.
pub(crate) struct FakeClock {
    pub now: Mutex<Instant>,
}
impl FakeClock {
    /// Clock at the current instant.
    pub(crate) fn new() -> Self {
        Self {
            now: Mutex::new(Instant::now()),
        }
    }
    /// Move the clock forward by `duration`.
    pub(crate) fn advance(&self, duration: Duration) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) += duration;
    }
}
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
impl Pause for FakeClock {
    fn pause(&self, duration: Duration) {
        self.advance(duration);
    }
}
