//! Native workspace adapter on the real operating system (006 Step 1, read-only part): the
//! frontmost application is read with its identity within the 50 ms capture budget, running
//! applications with windows are listed without duplicates, and the frontmost identity resolves
//! to its own running instance. Ignored by default because it depends on the desktop session;
//! run with `--ignored` on a logged-in macOS session. Nothing is activated and nothing is typed.
#![cfg(target_os = "macos")]

use handy_app_lib::destination::{
    macos::MacWorkspace,
    workspace::{resolve_instance, Foreground, Resolution, WorkspacePort},
};
use std::time::{Duration, Instant};
type TestResult = Result<(), Box<dyn std::error::Error>>;

// AC-001 / QE: capture at start under 50 ms, native read included.
const CAPTURE_BUDGET: Duration = Duration::from_millis(50);

#[test]
#[ignore = "native: needs a logged-in macOS desktop session"]
fn the_frontmost_application_is_read_with_its_identity_within_the_capture_budget() -> TestResult {
    let workspace = MacWorkspace::new();
    let started = Instant::now();
    let foreground = workspace.frontmost();
    let elapsed = started.elapsed();
    let Foreground::Application(front) = foreground else {
        return Err(format!("foreground not readable as another app: {foreground:?}").into());
    };
    println!(
        "capture_ms={:.3} identity_kind={}",
        elapsed.as_secs_f64() * 1000.0,
        kind(&front)
    );
    assert!(elapsed < CAPTURE_BUDGET, "capture took {elapsed:?}");
    assert_ne!(front.pid, std::process::id());
    let resolved = resolve_instance(
        &front.identity,
        front.pid,
        &workspace.instances(&front.identity),
    );
    assert_eq!(resolved, Resolution::Running(front));
    Ok(())
}
#[test]
#[ignore = "native: needs a logged-in macOS desktop session"]
fn running_applications_with_windows_are_listed_once_each_without_this_process() {
    let workspace = MacWorkspace::new();
    let started = Instant::now();
    let apps = workspace.running_windowed_apps();
    println!(
        "menu_enumeration_ms={:.3} apps={}",
        started.elapsed().as_secs_f64() * 1000.0,
        apps.len()
    );
    assert!(!apps.is_empty());
    assert!(apps.iter().all(|app| app.pid != std::process::id()));
    for (index, app) in apps.iter().enumerate() {
        assert!(apps[index + 1..]
            .iter()
            .all(|other| other.identity != app.identity));
    }
}
/// Identity variant only (the identity itself is not logged).
fn kind(app: &handy_app_lib::destination::workspace::RunningApp) -> &'static str {
    use handy_app_lib::destination::context::ApplicationIdentity;
    match app.identity {
        ApplicationIdentity::BundleId(_) => "bundle_id",
        ApplicationIdentity::Aumid(_) => "aumid",
        ApplicationIdentity::ExecutablePath(_) => "executable_path",
    }
}
