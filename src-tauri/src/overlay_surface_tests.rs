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

/// A 1440 × 875 work area (MacBook Air below the menu bar) and a 400 × 180 bubble.
const SCREEN: Frame = Frame {
    x: 0.0,
    y: 25.0,
    width: 1440.0,
    height: 875.0,
};
const BUBBLE: (f64, f64) = (400.0, 180.0);

fn frame(x: f64, y: f64, width: f64, height: f64) -> Frame {
    Frame {
        x,
        y,
        width,
        height,
    }
}
fn overlap(a: Frame, b: Frame) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}
fn bubble_at((x, y): (f64, f64)) -> Frame {
    frame(x, y, BUBBLE.0, BUBBLE.1)
}
fn on_screen(bubble: Frame) -> bool {
    bubble.x >= SCREEN.x
        && bubble.y >= SCREEN.y
        && bubble.x + bubble.width <= SCREEN.x + SCREEN.width
        && bubble.y + bubble.height <= SCREEN.y + SCREEN.height
}

// @spec FR-002: The bubble never covers a held indicator notice — .specs/features/010-recovery-bubble/spec.md#fr-002
#[test]
fn the_bubble_stacks_above_a_held_bottom_notice_and_below_a_held_top_one() {
    // Bottom: a 440 × 120 notice window flush to the bottom edge; the bubble goes above it.
    let bottom = frame(500.0, 765.0, 440.0, 120.0);
    let placed = stack_beside(bottom, BUBBLE, SCREEN, true);
    assert_eq!(placed, (520.0, 765.0 - STACK_GAP - 180.0));
    assert!(!overlap(bubble_at(placed), bottom) && on_screen(bubble_at(placed)));
    // Top: the notice under the menu bar; the bubble goes below it.
    let top = frame(500.0, 71.0, 440.0, 120.0);
    let placed = stack_beside(top, BUBBLE, SCREEN, false);
    assert_eq!(placed, (520.0, 71.0 + 120.0 + STACK_GAP));
    assert!(!overlap(bubble_at(placed), top) && on_screen(bubble_at(placed)));
}

#[test]
fn without_room_on_its_side_the_bubble_takes_the_other_and_never_leaves_the_screen() {
    // A tall notice near the top of a bottom placement: no room above, so below.
    let tall = frame(500.0, 60.0, 440.0, 300.0);
    let placed = stack_beside(tall, BUBBLE, SCREEN, true);
    assert_eq!(placed.1, 60.0 + 300.0 + STACK_GAP);
    assert!(!overlap(bubble_at(placed), tall) && on_screen(bubble_at(placed)));
    // A notice window at the left edge: the bubble is pulled back inside the monitor.
    let left = frame(-30.0, 765.0, 256.0, 50.0);
    let placed = stack_beside(left, BUBBLE, SCREEN, true);
    assert_eq!(placed.0, SCREEN.x);
    assert!(!overlap(bubble_at(placed), left) && on_screen(bubble_at(placed)));
    // No room on either side: kept inside the monitor all the same.
    let tiny = frame(0.0, 0.0, 800.0, 300.0);
    let huge = frame(100.0, 50.0, 400.0, 200.0);
    let placed = stack_beside(huge, (400.0, 250.0), tiny, true);
    assert!(placed.0 >= 0.0 && placed.1 >= 0.0 && placed.1 + 250.0 <= 300.0);
}
