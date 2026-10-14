//! Workspace boundary of the destination feature: typed snapshots of running applications read
//! from the operating system (never framework objects), one-shot activation requests, and the
//! monotonic clock and pause used for the bounded activation wait. Native adapters implement
//! `WorkspacePort`; tests inject fakes.
use super::context::ApplicationIdentity;
use serde::Serialize;
use std::time::{Duration, Instant};

/// One running application as the destination feature sees it; no window title is kept.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunningApp {
    pub pid: u32,
    pub identity: ApplicationIdentity,
    pub display_name: String,
    /// Opaque reference of a cached icon asset, filled asynchronously; `None` until loaded.
    pub icon_ref: Option<String>,
}

/// What is in front right now.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Foreground {
    /// Another application is frontmost.
    Application(RunningApp),
    /// Handy itself (settings window, panels or helpers) is frontmost.
    Handy,
    /// The foreground cannot be read on this system.
    Unreadable,
}

/// Which running instance an identity resolves to at activation time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    Running(RunningApp),
    /// No running instance has this identity (never auto-launched).
    Closed,
    /// Several instances match and none is the captured one: fail closed.
    Ambiguous,
}

/// Whether the system accepted an activation request; never proof of foreground success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationRequest {
    Sent,
    Refused,
}

/// Operating-system workspace: foreground reads, enumeration and activation.
pub trait WorkspacePort: Send + Sync {
    /// The application in front now; brief, callable from any thread.
    fn frontmost(&self) -> Foreground;
    /// The application in front now, read from a source that does not depend on this process's
    /// main run loop having turned. The activation wait and the pre-dispatch guard use it: the
    /// guard runs on the main thread right after the configured delay, where a value cached by
    /// the run loop (macOS `NSWorkspace`) would still show the application of before the delay.
    /// Adapters whose `frontmost` is already a live system read keep this default.
    fn frontmost_now(&self) -> Foreground {
        self.frontmost()
    }
    /// Other running applications that have user windows, deduplicated by identity, Handy
    /// and windowless helpers excluded.
    fn running_windowed_apps(&self) -> Vec<RunningApp>;
    /// Every running instance of `identity`.
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp>;
    /// Ask the system once to bring `target` forward (its last active window).
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest;
}

/// Monotonic time of the activation deadline.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}
/// Wait between two foreground observations (on a worker thread, never on the main thread).
pub trait Pause: Send + Sync {
    fn pause(&self, duration: Duration);
}
/// The system monotonic clock and thread sleep.
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
impl Pause for SystemClock {
    fn pause(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Resolve the instance to activate. Business rule: the captured process wins while it is
/// still running with the same identity (a reused PID has another identity and does not
/// match); otherwise exactly one running instance of the identity is accepted (a restarted
/// application); several are ambiguous and none is closed. A display name never matches.
pub fn resolve_instance(
    identity: &ApplicationIdentity,
    captured_pid: u32,
    instances: &[RunningApp],
) -> Resolution {
    let matching = instances
        .iter()
        .filter(|app| app.identity == *identity)
        .collect::<Vec<_>>();
    if let Some(captured) = matching.iter().find(|app| app.pid == captured_pid) {
        return Resolution::Running((*captured).clone());
    }
    match matching.as_slice() {
        [] => Resolution::Closed,
        [only] => Resolution::Running((*only).clone()),
        _ => Resolution::Ambiguous,
    }
}

/// Workspace of a platform without destination support: nothing can be read or activated.
pub struct UnsupportedWorkspace;
impl WorkspacePort for UnsupportedWorkspace {
    fn frontmost(&self) -> Foreground {
        Foreground::Unreadable
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        Vec::new()
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        // `identity` is unused: this platform cannot enumerate applications.
        let _ = identity;
        Vec::new()
    }
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        // `target` is unused: this platform cannot activate applications.
        let _ = target;
        ActivationRequest::Refused
    }
}
