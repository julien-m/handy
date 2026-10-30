//! The 007 destination context from 006 (or from the live foreground), the
//! registry of per-dictation checks, and the checks backed by 006's real
//! pre-dispatch guard and by the live foreground.
use super::*;
use crate::destination::context::{
    ApplicationIdentity, DestinationSource, DictationId, NoRecoveryConsumer, ResolvedTarget,
};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::manager::DestinationManager;
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::atomic::{AtomicBool, Ordering};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn session_context(target: Option<ResolvedTarget>) -> SessionContext {
    SessionContext {
        dictation_id: DictationId(4),
        destination_generation: 9,
        source: DestinationSource::FrontmostAtEnd,
        target,
        field: Default::default(),
    }
}

fn textedit_target() -> ResolvedTarget {
    ResolvedTarget {
        pid: 11,
        identity: ApplicationIdentity::BundleId("com.apple.TextEdit".into()),
    }
}

/// Answers a fixed verdict and counts its calls.
struct Fixed(bool, Mutex<u32>);
impl RequestCheck for Fixed {
    fn is_current(&self, _destination: &DestinationContext) -> bool {
        *self.1.lock().unwrap_or_else(PoisonError::into_inner) += 1;
        self.0
    }
}

fn destination(generation: u64, pid: Option<i32>) -> DestinationContext {
    DestinationContext {
        pid,
        bundle_id: None,
        generation: DestinationGeneration(generation),
    }
}

#[test]
fn a_named_006_target_gives_its_process_bundle_and_generation() {
    let context = DestinationContext::from_session(&session_context(Some(textedit_target())));
    assert_eq!(context.pid, Some(11));
    assert_eq!(context.bundle_id.as_deref(), Some("com.apple.TextEdit"));
    assert_eq!(context.generation, DestinationGeneration(9));
}

#[test]
fn no_006_target_or_a_windows_identity_names_no_process_or_bundle() {
    let none = DestinationContext::from_session(&session_context(None));
    assert_eq!((none.pid, none.bundle_id), (None, None));
    let aumid = ResolvedTarget {
        pid: 5,
        identity: ApplicationIdentity::Aumid("App!Main".into()),
    };
    let windows = DestinationContext::from_session(&session_context(Some(aumid)));
    assert_eq!((windows.pid, windows.bundle_id), (Some(5), None));
}

#[test]
fn the_foreground_names_an_application_but_never_handy_or_an_unreadable_front() {
    let generation = DestinationGeneration(3);
    let front = Foreground::Application(app(22, "com.apple.Safari", "Safari"));
    let named = DestinationContext::from_foreground(&front, generation);
    assert_eq!(named.pid, Some(22));
    assert_eq!(named.bundle_id.as_deref(), Some("com.apple.Safari"));
    for unnamed in [Foreground::Handy, Foreground::Unreadable] {
        let context = DestinationContext::from_foreground(&unnamed, generation);
        assert_eq!((context.pid, context.bundle_id), (None, None));
    }
}

#[test]
fn an_unregistered_generation_fails_closed() {
    let mut port = RegisteredDestinations(Arc::new(DestinationChecks::default()));
    assert!(!port.is_current(&destination(1, Some(7))));
}

#[test]
fn each_generation_consults_its_own_check_until_unregistered() {
    let checks = Arc::new(DestinationChecks::default());
    let first = Arc::new(Fixed(true, Mutex::new(0)));
    let second = Arc::new(Fixed(false, Mutex::new(0)));
    let mut port = RegisteredDestinations(Arc::clone(&checks));
    let registration = checks.register(DestinationGeneration(1), first.clone());
    let _other = checks.register(DestinationGeneration(2), second.clone());
    assert!(port.is_current(&destination(1, Some(7))));
    assert!(!port.is_current(&destination(2, Some(7))));
    drop(registration);
    assert!(!port.is_current(&destination(1, Some(7))));
    assert_eq!(*first.1.lock().unwrap_or_else(PoisonError::into_inner), 1);
    assert_eq!(*second.1.lock().unwrap_or_else(PoisonError::into_inner), 1);
}

#[test]
fn dropping_a_replaced_registration_keeps_the_newer_check() {
    let checks = Arc::new(DestinationChecks::default());
    let older = checks.register(
        DestinationGeneration(1),
        Arc::new(Fixed(false, Mutex::new(0))),
    );
    let _newer = checks.register(
        DestinationGeneration(1),
        Arc::new(Fixed(true, Mutex::new(0))),
    );
    drop(older);
    let mut port = RegisteredDestinations(checks);
    assert!(port.is_current(&destination(1, Some(7))));
}

fn foreground_check(workspace: Arc<FakeWorkspace>, cancelled: Arc<AtomicBool>) -> ForegroundCheck {
    ForegroundCheck {
        workspace,
        cancelled: Box::new(move || cancelled.load(Ordering::SeqCst)),
    }
}

#[test]
fn the_foreground_check_needs_the_same_process_in_front_and_no_cancellation() {
    let textedit = app(11, "com.apple.TextEdit", "TextEdit");
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(textedit.clone()),
        vec![textedit],
    ));
    let cancelled = Arc::new(AtomicBool::new(false));
    let check = foreground_check(Arc::clone(&workspace), Arc::clone(&cancelled));
    assert!(check.is_current(&destination(1, Some(11))));
    workspace.set_foreground(Foreground::Application(app(
        22,
        "com.apple.Safari",
        "Safari",
    )));
    assert!(!check.is_current(&destination(1, Some(11))));
    workspace.set_foreground(Foreground::Unreadable);
    assert!(!check.is_current(&destination(1, Some(11))));
    assert!(check.is_current(&destination(1, None)));
    cancelled.store(true, Ordering::SeqCst);
    assert!(!check.is_current(&destination(1, None)));
}

/// 006's real guard for a frontmost-at-the-end dictation (return off), with TextEdit in front.
fn session_guard(
    cancelled: Arc<AtomicBool>,
) -> Result<(Arc<FakeWorkspace>, Arc<DispatchGuard>), String> {
    let textedit = app(11, "com.apple.TextEdit", "TextEdit");
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(textedit.clone()),
        vec![textedit],
    ));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let clock = Arc::new(FakeClock::new());
    let manager = Arc::new(DestinationManager::new(workspace.clone(), clock, platform));
    let started = manager.commit_start(manager.capture(false));
    manager.mark_processing();
    let frozen = manager
        .freeze(started.dictation_id, started.destination_generation)
        .map_err(|refusal| format!("{refusal:?}"))?;
    let cancelled = Box::new(move || cancelled.load(Ordering::SeqCst));
    let recovery = Arc::new(NoRecoveryConsumer);
    let guard = DispatchGuard::new(manager, recovery, frozen, None, cancelled);
    Ok((workspace, Arc::new(guard)))
}

#[test]
fn the_session_check_follows_006s_guard_and_refuses_another_application() -> TestResult {
    let cancelled = Arc::new(AtomicBool::new(false));
    let (workspace, guard) = session_guard(Arc::clone(&cancelled))?;
    let context = guard.check().map_err(|refusal| refusal.to_string())?;
    let resolved = DestinationContext::from_session(&context);
    assert_eq!(resolved.pid, Some(11));
    let check = SessionCheck(Arc::clone(&guard));
    assert!(check.is_current(&resolved));
    workspace.set_foreground(Foreground::Application(app(
        22,
        "com.apple.Safari",
        "Safari",
    )));
    assert!(!check.is_current(&resolved));
    workspace.set_foreground(Foreground::Application(app(
        11,
        "com.apple.TextEdit",
        "TextEdit",
    )));
    cancelled.store(true, Ordering::SeqCst);
    assert!(!check.is_current(&resolved));
    Ok(())
}
