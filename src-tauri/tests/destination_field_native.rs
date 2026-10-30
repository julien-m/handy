//! Native spike harness of 011 Step 1 (macOS), read-only and ignored by default: with the
//! production accessibility adapter it reads the focused input of the application in front (or
//! of the process named by `HANDY_FIELD_PID`, a background or pinned application), whether its
//! range is readable, and the verification reads (focused element, identity, range), with their
//! timing; and checks that 006's live foreground read is unchanged afterwards. It never gives
//! focus, restores a range, raises a window, activates an application, sends a key or touches
//! the clipboard: those steps are the manual matrix with Julien (native proofs checklist, 011).
//! It prints stages, kinds and times only, never a value or a range. Run by
//! `scripts/field-return-native-proof.ts`: `cargo test --features
//! destination-return,smart-paste-dispatch --test destination_field_native -- --include-ignored
//! --nocapture`. The source checks below always run.

/// The adapter reads only the destination's application element, never the system-wide one,
/// never the element's value, selected text or title, and writes only focus and range.
#[test]
fn the_field_adapter_reads_no_content_and_never_the_system_wide_element() {
    let source = include_str!("../src/destination/macos_field.rs");
    assert!(!source.contains("AXUIElementCreateSystemWide"));
    assert!(source.contains("AXUIElementCreateApplication(pid)"));
    for attribute in [
        "\"AXValue\"",
        "\"AXSelectedText\"",
        "\"AXTitle\"",
        "\"AXDescription\"",
    ] {
        assert!(!source.contains(attribute), "{attribute} is never read");
    }
    assert!(!source.contains("log::"), "the adapter logs nothing");
    assert!(
        !source.contains("AXUIElementSetMessagingTimeout"),
        "timeouts go through 010's per-element helper, which refuses 0"
    );
}

/// The remembered range can never be printed or serialized (spec AC-008).
#[test]
fn the_selection_range_has_no_debug_display_or_serialize() {
    let source = include_str!("../src/destination/field.rs");
    let at = source.find("pub struct SelectionRange").unwrap_or(0);
    let derive = source[..at].lines().last().unwrap_or_default();
    assert_eq!(derive.trim(), "#[derive(Clone)]");
    assert!(!source.contains("impl std::fmt::Display for SelectionRange"));
    assert!(!source.contains("impl std::fmt::Debug for SelectionRange"));
}

#[cfg(target_os = "macos")]
mod macos {
    use handy_app_lib::destination::field::{FieldAccess, FIELD_VERIFY_BUDGET};
    use handy_app_lib::destination::field_capture::{capture, FieldCapture};
    use handy_app_lib::destination::macos_field::MacFieldAccess;
    use handy_app_lib::destination::native_workspace;
    use handy_app_lib::destination::workspace::{Foreground, SystemClock};
    use std::time::{Duration, Instant};

    /// Plan Step 1: the start read's 50 ms budget plus 10 ms of overhead.
    const CAPTURE_LIMIT: Duration = Duration::from_millis(60);
    /// The verification's 10 ms budget plus 10 ms of overhead.
    const VERIFY_LIMIT: Duration = Duration::from_millis(20);
    const SAMPLES: usize = 30;

    /// The process read: `HANDY_FIELD_PID` (a background application), else the one in front.
    fn target_pid() -> Option<u32> {
        if let Some(pid) = std::env::var("HANDY_FIELD_PID")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            return Some(pid);
        }
        match native_workspace().frontmost_now() {
            Foreground::Application(app) => Some(app.pid),
            Foreground::Handy | Foreground::Unreadable => None,
        }
    }

    // @spec AC-001: Bounded start read of a real application — .specs/features/011-dictation-field-return/spec.md#ac-001
    #[test]
    #[ignore = "native: needs a logged-in session and Accessibility for the test runner"]
    fn capture_the_input_of_the_target_within_the_budget() -> Result<(), String> {
        let pid = target_pid().ok_or("no application to read")?;
        let mut worst = Duration::ZERO;
        let mut last = String::new();
        for _ in 0..SAMPLES {
            let started = Instant::now();
            last = match capture(&MacFieldAccess, pid, &SystemClock) {
                FieldCapture::Remembered(field) => {
                    format!("remembered(range_read: {})", field.range.is_some())
                }
                FieldCapture::NotRemembered(miss) => format!("not_remembered({miss:?})"),
            };
            worst = worst.max(started.elapsed());
        }
        println!("pid={pid} capture={last} worst={worst:?} samples={SAMPLES}");
        assert!(worst <= CAPTURE_LIMIT, "a start read took {worst:?}");
        Ok(())
    }

    // @spec AC-002: Verification reads within 10 ms — .specs/features/011-dictation-field-return/spec.md#ac-002
    #[test]
    #[ignore = "native: needs a logged-in session and Accessibility for the test runner"]
    fn the_verification_reads_of_a_remembered_input_fit_their_budget() -> Result<(), String> {
        let pid = target_pid().ok_or("no application to read")?;
        let FieldCapture::Remembered(mut field) = capture(&MacFieldAccess, pid, &SystemClock)
        else {
            println!("pid={pid}: no editable input focused; nothing to verify");
            return Ok(());
        };
        let range = field
            .range
            .clone()
            .ok_or("range not readable: AC-004 (b) at insertion")?;
        let mut worst = Duration::ZERO;
        let mut same = 0;
        let mut verified = 0;
        for _ in 0..SAMPLES {
            let started = Instant::now();
            let focused = MacFieldAccess.focused_element(pid, FIELD_VERIFY_BUDGET);
            if focused.is_ok_and(|focused| focused.same_as(field.element.as_ref())) {
                same += 1;
                let left = FIELD_VERIFY_BUDGET.saturating_sub(started.elapsed());
                if field.element.range_is(&range, left).unwrap_or(false) {
                    verified += 1;
                }
            }
            worst = worst.max(started.elapsed());
        }
        println!("pid={pid} same_element={same}/{SAMPLES} range_verified={verified}/{SAMPLES} worst={worst:?}");
        assert!(worst <= VERIFY_LIMIT, "a verification took {worst:?}");
        Ok(())
    }

    #[test]
    #[ignore = "native: needs a logged-in session and Accessibility for the test runner"]
    fn the_006_foreground_read_is_unchanged_after_the_reads() -> Result<(), String> {
        let before = native_workspace().frontmost_now();
        if let Foreground::Application(app) = &before {
            let _ = capture(&MacFieldAccess, app.pid, &SystemClock);
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
