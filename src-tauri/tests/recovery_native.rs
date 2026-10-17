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

/// Every `.ts` and `.tsx` source under `directory`, recursively.
fn frontend_sources(directory: &std::path::Path, found: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            frontend_sources(&path, found);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "ts" || ext == "tsx")
        {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            found.push((path.display().to_string(), text));
        }
    }
}

/// The projection is emitted to the recovery window only, and Tauri also delivers a targeted
/// event to `Any` listeners: no other frontend source may listen to the bubble's events.
#[test]
fn only_the_recovery_window_listens_to_the_bubble_events() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let mut sources = Vec::new();
    frontend_sources(&root, &mut sources);
    assert!(!sources.is_empty());
    for (path, text) in sources {
        let listens = text.contains("\"recovery-bubble-changed\"");
        let allowed = path.ends_with("use-recovery-bubble.ts");
        assert!(
            !listens || allowed,
            "{path} listens to the bubble projection"
        );
    }
}

/// The text of `source` from `name` to the first `end` after it (the end of that function).
fn body<'a>(source: &'a str, name: &str, end: &str) -> &'a str {
    let start = source.find(name).unwrap_or(source.len());
    let rest = &source[start..];
    &rest[..rest.find(end).unwrap_or(rest.len())]
}

/// 005 AC-005 and 007: presenting the bubble hides the indicator as at the end of a dictation
/// and never ends a notice hold, so an AI-failure or hand-over notice keeps its duration.
#[test]
fn presenting_the_bubble_never_cuts_an_indicator_notice() {
    let overlay = include_str!("../src/overlay.rs");
    let yielding = body(overlay, "pub fn yield_to_recovery", "\n}");
    assert!(yielding.contains("hide_recording_overlay(app_handle)"));
    assert!(!yielding.contains("release_hold"), "{yielding}");
    assert!(
        overlay.contains("if SURFACE.is_held()"),
        "the delayed hide keeps a held notice"
    );
}

/// AC-005 with no text loss: the bubble closes only after the recording really started.
#[test]
fn the_bubble_closes_only_after_the_recording_started() {
    let actions = include_str!("../src/actions.rs");
    let started = actions
        .find("rm.try_start_recording(")
        .unwrap_or(usize::MAX);
    let accepted = actions[started..]
        .find("Ok(readiness) =>")
        .map(|at| started + at);
    let closed = actions.find("recovery::app::dictation_started(app)");
    assert!(
        accepted.is_some() && closed > accepted,
        "closed inside the accepted start"
    );
    let starting = body(
        include_str!("../src/recovery/service.rs"),
        "pub fn dictation_starting",
        "\n    }",
    );
    assert!(!starting.contains("retire"), "{starting}");
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

/// FR-002: the bubble is placed again whenever the held indicator changes its extent (a notice
/// measured after its hold started), and before a refused drag shows it again; the indicator
/// is placed first, so the bubble reads its grown frame.
#[test]
fn the_bubble_is_placed_again_after_a_held_extent_change_and_a_refused_drag() {
    let indicator = include_str!("../src/overlay_indicator.rs");
    let extent = body(indicator, "fn set_overlay_extent(", "\n}");
    let placed = extent.find("place_overlay(&handle, &window, width, height)");
    let followed = extent.find("if SURFACE.bubble_follows_extent() {");
    assert!(placed.is_some() && followed > placed, "{extent}");
    assert!(extent.contains("crate::recovery::surface::follow_indicator(&handle)"));
    let surface = include_str!("../src/recovery/surface.rs");
    let again = body(surface, "fn show_again", "\n    }\n");
    let place = again.find("place(app, &window)");
    assert!(
        place.is_some() && again.find("platform::restore_after_drag(&window)") > place,
        "{again}"
    );
}
