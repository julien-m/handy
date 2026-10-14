//! Unit tests of the overlay surface state (extent growth, monitor bound and notice holds).
use super::*;

#[test]
fn extent_grows_the_base_and_never_shrinks_it() {
    let state = SurfaceState::new();
    assert_eq!(state.grow((256.0, 50.0)), (256.0, 50.0));
    assert!(state.store_extent(320.0, 40.0));
    assert_eq!(state.grow((256.0, 50.0)), (320.0, 50.0));
    assert!(!state.store_extent(320.0, 40.0), "same extent is no change");
    state.clear_extent();
    assert_eq!(state.grow((400.0, 120.0)), (400.0, 120.0));
}

#[test]
fn extent_is_clamped_and_non_finite_requests_are_ignored() {
    assert_eq!(sanitize_extent(-4.0, 10.0), Some((0.0, 10.0)));
    assert_eq!(sanitize_extent(5000.0, 5000.0), Some((960.0, 720.0)));
    assert_eq!(sanitize_extent(f64::NAN, 10.0), None);
    let state = SurfaceState::new();
    assert!(!state.store_extent(f64::INFINITY, 10.0));
    assert_eq!(state.grow((256.0, 50.0)), (256.0, 50.0));
}

#[test]
fn a_newer_hold_supersedes_the_release_of_an_older_one() {
    let state = SurfaceState::new();
    let first = state.start_hold();
    let second = state.start_hold();
    assert!(!state.release_hold(Some(first)), "stale release is ignored");
    assert!(state.is_held());
    assert!(state.release_hold(Some(second)));
    assert!(!state.is_held());
}

#[test]
fn an_explicit_release_ends_any_hold() {
    let state = SurfaceState::new();
    state.start_hold();
    assert!(state.release_hold(None));
    assert!(!state.is_held());
}

#[test]
fn hold_duration_is_capped_and_keeps_the_fade_grace() {
    assert_eq!(hold_duration(4000), Duration::from_millis(4300));
    assert_eq!(hold_duration(u32::MAX), Duration::from_millis(15_300));
}

#[test]
fn session_marks_keep_the_last_form() {
    let state = SurfaceState::new();
    state.mark_session(true, true);
    assert!(state.session_shown() && state.streaming());
    state.mark_session(false, false);
    assert!(!state.session_shown());
    assert!(
        state.streaming(),
        "hiding keeps the form of the last dictation"
    );
}

const COMPACT: (f64, f64) = (256.0, 50.0);
const LIVE: (f64, f64) = (400.0, 120.0);

#[test]
fn a_small_windows_monitor_with_the_largest_text_scale_bounds_the_extent() {
    // 1366 x 768 at 100 % DPI with the accessibility text size at 225 %, bottom offset 40.
    let room = monitor_room((1366, 768), 1.0, 2.25, 40.0);
    let (width, height) = fit_to_monitor((960.0, 720.0), COMPACT, room);
    // Once the placement multiplies by scale x text scale, the window fits the screen.
    assert!(width * 2.25 <= 1366.0 + 1e-9, "{width}");
    assert!(height * 2.25 + 40.0 <= 768.0 + 1e-9, "{height}");
    assert!((width - 1366.0 / 2.25).abs() < 1e-9);
}

#[test]
fn the_monitor_scale_and_text_scale_both_shrink_the_room() {
    // 2560 x 1440 at 150 % DPI and 110 % text: 2560 / 1.65 wide, (1440 - 40 x 1.5) / 1.65 high.
    let (width, height) = monitor_room((2560, 1440), 1.5, 1.1, 40.0);
    assert!((width - 2560.0 / 1.65).abs() < 1e-9);
    assert!((height - 1380.0 / 1.65).abs() < 1e-9);
    // A 4K Retina screen (scale 2) leaves the largest request untouched.
    let room = monitor_room((3840, 2160), 2.0, 1.0, 46.0);
    assert_eq!(
        fit_to_monitor((960.0, 720.0), COMPACT, room),
        (960.0, 720.0)
    );
}

#[test]
fn the_base_size_of_each_form_is_kept_on_any_monitor() {
    let tiny = monitor_room((320, 100), 1.0, 2.25, 40.0);
    assert_eq!(fit_to_monitor((960.0, 720.0), COMPACT, tiny), COMPACT);
    assert_eq!(fit_to_monitor((960.0, 720.0), LIVE, tiny), LIVE);
    // Without any growth request the base is unchanged everywhere.
    let room = monitor_room((1366, 768), 1.0, 2.25, 40.0);
    assert_eq!(fit_to_monitor(LIVE, LIVE, room), LIVE);
}

#[test]
fn a_degenerate_monitor_scale_leaves_only_the_base() {
    assert_eq!(monitor_room((1920, 1080), 0.0, 1.0, 40.0), (0.0, 0.0));
    assert_eq!(monitor_room((1920, 1080), f64::NAN, 1.0, 40.0), (0.0, 0.0));
    let none = monitor_room((1920, 1080), 0.0, 1.0, 40.0);
    assert_eq!(fit_to_monitor((960.0, 720.0), COMPACT, none), COMPACT);
}
