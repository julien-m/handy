//! macOS workspace adapter over NSWorkspace / NSRunningApplication. Identity is the bundle
//! identifier (the executable path for the rare app without one); Handy is excluded by its
//! own process id. Every call is a brief read or one activation request, made inside an
//! autorelease pool so it can run on a worker thread; no window title is read.
use super::context::ApplicationIdentity;
use super::workspace::{ActivationRequest, Foreground, RunningApp, WorkspacePort};
use objc2::rc::{autoreleasepool, Retained};
use objc2_app_kit::{
    NSApplicationActivationOptions, NSApplicationActivationPolicy, NSRunningApplication,
    NSWorkspace,
};
use objc2_foundation::NSString;

/// The macOS workspace of this process.
pub struct MacWorkspace {
    own_pid: u32,
}
impl MacWorkspace {
    /// Adapter excluding the current process (Handy and its panels).
    pub fn new() -> Self {
        Self {
            own_pid: std::process::id(),
        }
    }
}
impl Default for MacWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

/// Typed snapshot of a running application; `None` for one without any identity.
fn snapshot(app: &NSRunningApplication) -> Option<RunningApp> {
    let pid = u32::try_from(app.processIdentifier()).ok()?;
    let identity = match app.bundleIdentifier() {
        Some(bundle) => ApplicationIdentity::BundleId(bundle.to_string()),
        None => {
            ApplicationIdentity::ExecutablePath(app.executableURL()?.path()?.to_string().into())
        }
    };
    let display_name = app
        .localizedName()
        .map(|name| name.to_string())
        .unwrap_or_else(|| fallback_name(&identity));
    Some(RunningApp {
        pid,
        identity,
        display_name,
        icon_ref: None,
    })
}
/// Name shown when the system has no localized name: the last identity component.
fn fallback_name(identity: &ApplicationIdentity) -> String {
    match identity {
        ApplicationIdentity::BundleId(text) | ApplicationIdentity::Aumid(text) => {
            text.rsplit('.').next().unwrap_or(text).to_owned()
        }
        ApplicationIdentity::ExecutablePath(path) => path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}
/// An application the user can switch to: a regular (Dock) app that is still running.
fn is_windowed(app: &NSRunningApplication) -> bool {
    app.activationPolicy() == NSApplicationActivationPolicy::Regular && !app.isTerminated()
}

impl WorkspacePort for MacWorkspace {
    // @spec FR-001: Read the frontmost application identity — .specs/features/006-destination-application/spec.md#fr-001
    fn frontmost(&self) -> Foreground {
        autoreleasepool(|_| {
            let front = NSWorkspace::sharedWorkspace().frontmostApplication();
            match front.as_deref().and_then(snapshot) {
                None => Foreground::Unreadable,
                Some(app) if app.pid == self.own_pid => Foreground::Handy,
                Some(app) => Foreground::Application(app),
            }
        })
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        autoreleasepool(|_| {
            let running = NSWorkspace::sharedWorkspace().runningApplications();
            let mut apps: Vec<RunningApp> = Vec::new();
            for app in running.iter().filter(|app| is_windowed(app)) {
                let Some(app) = snapshot(&app) else { continue };
                // Skip Handy itself and a second instance of an identity already listed.
                if app.pid == self.own_pid
                    || apps.iter().any(|known| known.identity == app.identity)
                {
                    continue;
                }
                apps.push(app);
            }
            apps
        })
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        autoreleasepool(|_| {
            let running: Vec<Retained<NSRunningApplication>> = match identity {
                ApplicationIdentity::BundleId(bundle) => {
                    let bundle = NSString::from_str(bundle);
                    NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle).to_vec()
                }
                _ => NSWorkspace::sharedWorkspace()
                    .runningApplications()
                    .to_vec(),
            };
            running
                .iter()
                .filter(|app| !app.isTerminated())
                .filter_map(|app| snapshot(app))
                .filter(|app| app.identity == *identity && app.pid != self.own_pid)
                .collect()
        })
    }
    // @spec FR-002: One activation request per insertion — .specs/features/006-destination-application/spec.md#fr-002
    /// One activation request (no retry): the system brings the app's last active window,
    /// switching Space if needed. Acceptance is not proof that it became frontmost.
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        autoreleasepool(|_| {
            let Ok(pid) = i32::try_from(target.pid) else {
                return ActivationRequest::Refused;
            };
            let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid);
            match app {
                Some(app) if app.activateWithOptions(NSApplicationActivationOptions::empty()) => {
                    ActivationRequest::Sent
                }
                _ => ActivationRequest::Refused,
            }
        })
    }
}
