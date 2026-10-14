//! Unit tests of the overlay surface state (extent growth and notice holds).
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
