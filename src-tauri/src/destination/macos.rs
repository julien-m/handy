//! macOS workspace adapter over NSWorkspace / NSRunningApplication. Identity is the bundle
//! identifier (the executable path for the rare app without one); Handy is excluded by its
//! own process id. Every call is a brief read or one activation request, made inside an
//! autorelease pool so it can run on a worker thread; no window title is read. `frontmost_now`
//! (activation wait and pre-dispatch guard) reads the focused application live through
//! Accessibility (`macos_focus`), because `NSWorkspace` values only change when this process's
//! main run loop turns, which it does not during the paste delay on the main thread.
//!
//! Activation (N16, every macOS version, same order): `activate_once` is the
//! `NSRunningApplication` request; `activate_by_accessibility` sets `AXFrontmost` on the
//! application element. Since macOS 14, cooperative activation lets the system decline the first
//! one from a background accessory app (`activateWithOptions:` answers NO and nothing comes
//! forward): observed on macOS 26.5.2 after the user switched applications by hand, while the
//! same request from a background process succeeded after programmatic switches. Calling it on
//! the main thread changed nothing in the N16 diagnostic, so both run on the activation worker.
//! `NSApp.yieldActivation(to:)` needs Handy to be the active app, which it is not here.
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
/// Foreground of an application read from the system: none (unreadable), Handy, or another app.
fn classify(front: Option<RunningApp>, own_pid: u32) -> Foreground {
    match front {
        None => Foreground::Unreadable,
        Some(app) if app.pid == own_pid => Foreground::Handy,
        Some(app) => Foreground::Application(app),
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
            classify(front.as_deref().and_then(snapshot), self.own_pid)
        })
    }
    /// Live read through Accessibility; an unreadable focus is `Unreadable`, never the cached
    /// `NSWorkspace` value, so the guard fails closed (`activation_failed`) instead of trusting it.
    fn frontmost_now(&self) -> Foreground {
        let pid = match super::macos_focus::focused_application_pid() {
            Ok(pid) => pid,
            Err(error) => {
                log::debug!("Focused application not readable: {error:?}");
                return Foreground::Unreadable;
            }
        };
        autoreleasepool(|_| {
            let app = i32::try_from(pid)
                .ok()
                .and_then(NSRunningApplication::runningApplicationWithProcessIdentifier);
            classify(app.as_deref().and_then(snapshot), self.own_pid)
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
    // @spec FR-002: Workspace activation request — .specs/features/006-destination-application/spec.md#fr-002
    /// One workspace activation request (no retry): the system brings the app's last active
    /// window, switching Space if needed. Acceptance is not proof that it became frontmost.
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
    /// One `AXFrontmost` request on the application element (no retry); any `AXError`, a
    /// missing Accessibility permission included, is a refusal.
    fn activate_by_accessibility(&self, target: &RunningApp) -> ActivationRequest {
        match super::macos_focus::request_frontmost(target.pid) {
            Ok(()) => ActivationRequest::Sent,
            Err(error) => {
                log::debug!("Accessibility activation request refused: {error:?}");
                ActivationRequest::Refused
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(pid: u32) -> RunningApp {
        RunningApp {
            pid,
            identity: ApplicationIdentity::BundleId("com.apple.Notes".to_owned()),
            display_name: "Notes".to_owned(),
            icon_ref: None,
        }
    }
    #[test]
    fn a_read_is_unreadable_handy_or_another_application() {
        assert_eq!(classify(None, 7), Foreground::Unreadable);
        assert_eq!(classify(Some(notes(7)), 7), Foreground::Handy);
        assert_eq!(
            classify(Some(notes(8)), 7),
            Foreground::Application(notes(8))
        );
    }
}
