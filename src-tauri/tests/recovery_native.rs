//! Native spike harness of 010 Step 1 (macOS), read-only and ignored by default: it classifies
//! the focused element of the application in front with the production accessibility adapter
//! and records the timing, and checks that 006's live foreground read is unchanged afterwards.
//! It never activates an application, sends a key, touches the clipboard or shows a window.
//! Run by `scripts/recovery-native-proof.ts` with Julien:
//! `cargo test --features destination-return,smart-paste-dispatch --test recovery_native --
//! --include-ignored --nocapture`. The source checks below always run.

/// The adapter reads only the destination's application element: no system-wide element and
/// a zero timeout is refused before it reaches AppKit (spec decision 2026-10-15).
#[test]
fn the_classifier_adapter_never_uses_the_system_wide_element_or_a_zero_timeout() {
    let source = include_str!("../src/recovery/platform/macos_ax.rs");
    assert!(!source.contains("AXUIElementCreateSystemWide"));
    assert!(source.contains("AXUIElementCreateApplication(pid)"));
    assert!(source.contains("if timeout.is_zero()"));
    assert_eq!(
        source
            .matches("AXUIElementSetMessagingTimeout(element.as_ptr()")
            .count(),
        1,
        "one per-element timeout helper"
    );
}

/// Privacy (plan Step 5, QE): the projection goes only to the recovery window (`emit_to`),
/// never in an app-wide event, and no recovery module logs the transcript.
#[test]
fn the_projection_is_emitted_only_to_the_recovery_window_and_never_logged() {
    let surface = include_str!("../src/recovery/surface.rs");
    assert!(surface.contains("app.emit_to(RECOVERY_WINDOW, VIEW_EVENT, view)"));
    assert!(!surface.contains(".emit(VIEW_EVENT"));
    for source in [
        surface,
        include_str!("../src/recovery/service.rs"),
        include_str!("../src/recovery/manager.rs"),
        include_str!("../src/recovery/app.rs"),
        include_str!("../src/recovery/platform/macos_drag.rs"),
    ] {
        for line in source.lines().filter(|line| line.contains("log::")) {
            assert!(!line.contains("transcript"), "{line}");
            assert!(!line.contains("preview"), "{line}");
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use handy_app_lib::destination::native_workspace;
    use handy_app_lib::destination::workspace::{Foreground, SystemClock};
    use handy_app_lib::recovery::classifier::{classify, FocusClassification, CLASSIFY_BUDGET};
    use handy_app_lib::recovery::platform::macos_ax::MacFocusedElementPort;
    use std::time::{Duration, Instant};

    /// Plan Step 1: no sample above 50 ms of AX budget plus 10 ms of overhead.
    const SAMPLE_LIMIT: Duration = Duration::from_millis(60);
    const SAMPLES: usize = 30;

    fn front_pid() -> Option<u32> {
        match native_workspace().frontmost_now() {
            Foreground::Application(app) => Some(app.pid),
            Foreground::Handy | Foreground::Unreadable => None,
        }
    }

    // @spec AC-002: Bounded classification of the real front app — .specs/features/010-recovery-bubble/spec.md#ac-002
    #[test]
    #[ignore = "native: needs a logged-in session and Accessibility for the test runner"]
    fn classify_the_front_application_within_the_budget() -> Result<(), String> {
        let pid = front_pid().ok_or("no application in front")?;
        let mut port = MacFocusedElementPort;
        let mut worst = Duration::ZERO;
        let mut last = FocusClassification::PermissionMissing;
        for _ in 0..SAMPLES {
            let started = Instant::now();
            last = classify(&mut port, pid, &SystemClock);
            worst = worst.max(started.elapsed());
        }
        println!("pid={pid} classification={last:?} worst={worst:?} budget={CLASSIFY_BUDGET:?}");
        assert!(worst <= SAMPLE_LIMIT, "a classification took {worst:?}");
        Ok(())
    }

    #[test]
    #[ignore = "native: needs a logged-in session and Accessibility for the test runner"]
    fn the_006_foreground_read_is_unchanged_after_a_classification() -> Result<(), String> {
        let before = native_workspace().frontmost_now();
        if let Foreground::Application(app) = &before {
            let _ = classify(&mut MacFocusedElementPort, app.pid, &SystemClock);
        }
        let after = native_workspace().frontmost_now();
        println!("front before={before:?} after={after:?}");
        if before == after {
            Ok(())
        } else {
            Err("006's foreground read changed".to_owned())
        }
    }
}
