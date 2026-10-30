//! Unit tests of the indicator's placement anchor (006 AC-009): the monitor is resolved the way
//! `placement_monitor` resolves it, with the cursor's monitor found by the production hit-test.
use super::*;

/// Three screens stacked vertically (central, top, bottom of the native report), each laid out
/// from the previous one's edge.
fn stacked_screens() -> Vec<MonitorKey> {
    let sizes = [(3024u32, 1964u32), (2560, 1440), (1920, 1080)];
    let mut y = 0i32;
    sizes
        .iter()
        .map(|&(width, height)| {
            let key = MonitorKey {
                x: 0,
                y,
                width,
                height,
            };
            y -= i32::try_from(height).unwrap_or(i32::MAX);
            key
        })
        .collect()
}

/// A cursor point at the centre of `monitor`.
fn centre_of(monitor: MonitorKey) -> (i32, i32) {
    let half = |extent: u32| i32::try_from(extent / 2).unwrap_or(i32::MAX);
    (
        monitor.x + half(monitor.width),
        monitor.y + half(monitor.height),
    )
}

/// The monitor under `cursor`, by the hit-test `get_monitor_with_cursor` uses.
fn under(monitors: &[MonitorKey], cursor: (i32, i32)) -> Option<MonitorKey> {
    monitors.iter().copied().find(|monitor| {
        is_mouse_within_monitor(
            cursor,
            &PhysicalPosition::new(monitor.x, monitor.y),
            &PhysicalSize::new(monitor.width, monitor.height),
        )
    })
}

/// One placement with the cursor at `cursor`.
fn place(state: &AnchorState, monitors: &[MonitorKey], cursor: (i32, i32)) -> Option<MonitorKey> {
    state.resolve(monitors, || under(monitors, cursor))
}

#[test]
fn without_destination_every_placement_follows_the_cursor() {
    let screens = stacked_screens();
    let state = AnchorState::new();
    state.begin();
    for screen in &screens {
        assert_eq!(place(&state, &screens, centre_of(*screen)), Some(*screen));
    }
}

#[test]
fn with_a_destination_the_start_monitor_is_kept_when_the_cursor_moves() {
    let screens = stacked_screens();
    let (start, elsewhere) = (screens[0], screens[1]);
    let state = AnchorState::new();
    state.begin();
    assert_eq!(place(&state, &screens, centre_of(start)), Some(start));
    state.hold();
    // Live growth, transcribing, processing and the recovery bubble all place again.
    for _ in 0..4 {
        assert_eq!(place(&state, &screens, centre_of(elsewhere)), Some(start));
    }
}

#[test]
fn a_hold_before_the_first_placement_anchors_on_the_cursor_monitor_then() {
    let screens = stacked_screens();
    let state = AnchorState::new();
    state.begin();
    state.hold();
    assert_eq!(
        place(&state, &screens, centre_of(screens[2])),
        Some(screens[2])
    );
    assert_eq!(
        place(&state, &screens, centre_of(screens[0])),
        Some(screens[2])
    );
}

#[test]
fn a_destination_chosen_mid_dictation_keeps_the_current_monitor() {
    let screens = stacked_screens();
    let state = AnchorState::new();
    state.begin();
    place(&state, &screens, centre_of(screens[0]));
    assert_eq!(
        place(&state, &screens, centre_of(screens[1])),
        Some(screens[1])
    );
    state.hold();
    assert_eq!(
        place(&state, &screens, centre_of(screens[0])),
        Some(screens[1])
    );
}

#[test]
fn the_next_dictation_starts_on_the_cursor_monitor_again() {
    let screens = stacked_screens();
    let state = AnchorState::new();
    state.begin();
    place(&state, &screens, centre_of(screens[0]));
    state.hold();
    state.begin();
    assert_eq!(
        place(&state, &screens, centre_of(screens[1])),
        Some(screens[1])
    );
    assert_eq!(
        place(&state, &screens, centre_of(screens[2])),
        Some(screens[2])
    );
}

#[test]
fn a_disconnected_held_monitor_falls_back_to_the_cursor_and_is_replaced() {
    let screens = stacked_screens();
    let state = AnchorState::new();
    state.begin();
    place(&state, &screens, centre_of(screens[1]));
    state.hold();
    let remaining: Vec<MonitorKey> = screens
        .iter()
        .copied()
        .filter(|screen| *screen != screens[1])
        .collect();
    assert_eq!(
        place(&state, &remaining, centre_of(remaining[0])),
        Some(remaining[0])
    );
    assert_eq!(
        place(&state, &remaining, centre_of(remaining[1])),
        Some(remaining[0])
    );
}

#[test]
fn no_monitor_under_the_cursor_places_nothing_and_keeps_no_anchor() {
    let state = AnchorState::new();
    state.begin();
    state.hold();
    assert_eq!(state.resolve(&[], || None), None);
    let screens = stacked_screens();
    assert_eq!(
        place(&state, &screens, centre_of(screens[1])),
        Some(screens[1])
    );
}
