//! 009 Step 1, automatic part on the real operating system: the Enter authority's final focus
//! check (claim, then the live frontmost read, then the key) runs on the real macOS workspace
//! within the AC-005 budget. The key press itself is a spy here: no key, text or activation is
//! sent. The real key count, the target text when Enter lands (ChatGPT, Comet) and the reliable
//! paste's read attribution are native proofs run with the maintainer through
//! `scripts/pin-enter-native-proof.ts` (see `.specs/native-proofs-checklist.md`, section 009).
//! Ignored by default because it depends on the desktop session.
#![cfg(target_os = "macos")]

use handy_app_lib::automatic_enter::key_press::{
    GuardedKeyPress, MainThreadRunner, MAIN_THREAD_BOUND,
};
use handy_app_lib::automatic_enter::policy::Completion;
use handy_app_lib::automatic_enter::transaction::{
    EnterPorts, EnterResult, EnterSnapshot, EnterTransaction,
};
use handy_app_lib::automatic_enter::{AutoSubmitKey, AutomaticEnterPolicy};
use handy_app_lib::destination::context::{
    DestinationContext, RecoveryDisposition, ResolvedTarget,
};
use handy_app_lib::destination::macos::MacWorkspace;
use handy_app_lib::destination::manager::DestinationManager;
use handy_app_lib::destination::workspace::{Foreground, SystemClock, WorkspacePort};
use handy_app_lib::settings_service::platform_capability::{
    PlatformCapabilities, PlatformCapability,
};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Runs the press task inline: the real guarded press checks the real foreground right before
/// its (spied) key.
struct Inline;
impl MainThreadRunner for Inline {
    fn run(&self, task: Box<dyn FnOnce() + Send>) -> Result<(), String> {
        task();
        Ok(())
    }
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    /// See `destination_platform.rs`: Accessibility needs a window-server connection.
    fn NSApplicationLoad() -> u8;
}

/// A dictation of the frontmost application, frozen and accepted by the guard (`Insert`), and
/// the context its method would be dispatched with.
fn inserting_dictation(
    workspace: &Arc<MacWorkspace>,
) -> Result<(Arc<DestinationManager>, DestinationContext), Box<dyn std::error::Error>> {
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace.clone(),
        Arc::new(SystemClock),
        platform,
    ));
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.mark_processing();
    manager.freeze(id, generation)?;
    manager.set_disposition(id, generation, RecoveryDisposition::Insert)?;
    let Foreground::Application(front) = workspace.frontmost_now() else {
        return Err("live foreground not readable".into());
    };
    let target = ResolvedTarget {
        pid: front.pid,
        identity: front.identity,
    };
    let context = DestinationContext {
        dictation_id: id,
        destination_generation: generation,
        source: started.source,
        target: Some(target),
        field: Default::default(),
    };
    Ok((manager, context))
}

// @spec AC-005: Focus check then key within the QE budget — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
#[test]
#[ignore = "native: needs a logged-in macOS desktop session and Accessibility for the runner"]
fn the_final_focus_check_on_the_real_workspace_fits_the_100_ms_budget() -> TestResult {
    // SAFETY: no arguments; idempotent AppKit initialisation of this test process.
    unsafe { NSApplicationLoad() };
    let workspace = Arc::new(MacWorkspace::new());
    let (manager, context) = inserting_dictation(&workspace)?;
    let pressed = Arc::new(Mutex::new(Vec::new()));
    let spy = Arc::clone(&pressed);
    let keys = GuardedKeyPress {
        runner: Arc::new(Inline),
        workspace: workspace.clone(),
        press: Arc::new(move |_| {
            spy.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(Instant::now());
            Ok(())
        }),
        bound: MAIN_THREAD_BOUND,
    };
    let ports = EnterPorts {
        claims: manager.clone(),
        workspace,
        keys: Arc::new(keys),
        clock: Arc::new(SystemClock),
        pause: Arc::new(SystemClock),
    };
    let frozen = EnterSnapshot {
        dictation_id: context.dictation_id,
        destination_generation: context.destination_generation,
        policy: AutomaticEnterPolicy::Always,
        key: AutoSubmitKey::Enter,
    };
    let enter = EnterTransaction::new(frozen, ports);
    let t0 = Instant::now();
    let result = enter.complete(Completion::Classic { succeeded: true }, Some(&context), t0);
    assert_eq!(result, EnterResult::Sent, "the spy, never a real key");
    let pressed = pressed
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let delay = pressed
        .first()
        .ok_or("no attempt")?
        .saturating_duration_since(t0);
    println!("t0_to_key_attempt_ms={:.3}", delay.as_secs_f64() * 1000.0);
    assert!(delay >= Duration::from_millis(50) && delay <= Duration::from_millis(100));
    Ok(())
}
