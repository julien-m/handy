//! Native session finding N13 (2026-10-28): in hold-or-toggle mode a start that takes longer
//! than the hold threshold (355 ms observed) queued the key-up behind it, so a short tap was
//! measured at processing time, read as a hold, and stopped the dictation at once. Hold
//! durations are now measured between the instants the backend received the edges. The clock
//! is injected: `received` is when the edge happened, `processed` when the coordinator saw it.
use super::{CoordinatorState, Effect, InputEvent, Stage};
use crate::settings::ShortcutActivation;
use std::time::{Duration, Instant};

const HOLD_THRESHOLD: Duration = Duration::from_millis(300);
/// The start observed in the native session took 355 ms; 400 ms is slower still.
const SLOW_START: Duration = Duration::from_millis(400);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn auto(is_pressed: bool, received_at: Instant) -> InputEvent {
    InputEvent {
        binding_id: "transcribe".into(),
        shortcut_id: "option+space".into(),
        hotkey_string: "option+space".into(),
        is_pressed,
        mode: ShortcutActivation::HoldOrToggle,
        hold_threshold: HOLD_THRESHOLD,
        external: false,
        received_at,
    }
}

/// A press received and processed at `t0`, whose start then keeps the coordinator busy.
fn started(t0: Instant) -> CoordinatorState {
    let mut state = CoordinatorState::new();
    let effect = state.on_input(auto(true, t0), t0);
    assert!(matches!(effect, Some(Effect::Start { .. })));
    state
}

#[test]
fn a_tap_queued_behind_a_slow_start_locks_the_session() {
    let t0 = Instant::now();
    let mut state = started(t0);
    // Key up 100 ms after the press, processed once the 400 ms start returned.
    assert_eq!(
        state.on_input(auto(false, t0 + ms(100)), t0 + SLOW_START),
        None
    );
    assert_eq!(state.on_grace_expired(), None, "a tap never stops");
    assert!(state.is_locked());
    assert_eq!(state.stage, Stage::Recording("transcribe".into()));
    let next = state.on_input(auto(true, t0 + ms(3000)), t0 + ms(3000));
    assert!(
        matches!(next, Some(Effect::Stop { .. })),
        "the next press stops"
    );
}

#[test]
fn a_real_hold_processed_late_still_stops() {
    let t0 = Instant::now();
    let mut state = started(t0);
    let released = t0 + ms(500);
    assert_eq!(
        state.on_input(auto(false, released), released + SLOW_START),
        None
    );
    assert!(matches!(
        state.on_grace_expired(),
        Some(Effect::Stop { .. })
    ));
}

#[test]
fn a_queued_auto_repeat_press_still_cancels_the_deferred_release() {
    let t0 = Instant::now();
    let mut state = started(t0);
    // X11 auto-repeat: release and press 5 ms apart, both queued behind the start.
    assert_eq!(
        state.on_input(auto(false, t0 + ms(500)), t0 + SLOW_START),
        None
    );
    let repeat = state.on_input(auto(true, t0 + ms(505)), t0 + SLOW_START + ms(1));
    assert_eq!(repeat, None);
    assert!(
        state.grace_deadline().is_none(),
        "the release was cancelled"
    );
    assert!(!state.is_locked(), "the key is still held");
}

#[test]
fn a_tap_remembered_while_busy_is_measured_from_its_own_edges() {
    let t0 = Instant::now();
    let mut state = started(t0);
    let stopped = state.on_input(auto(true, t0 + ms(1000)), t0 + ms(1000));
    let stopped = stopped.or_else(|| {
        state.on_input(auto(false, t0 + ms(1600)), t0 + ms(1600));
        state.on_grace_expired()
    });
    assert!(matches!(stopped, Some(Effect::Stop { .. })));
    assert_eq!(state.stage, Stage::Processing);
    // A tap while the pipeline is busy, its key-up processed 500 ms late.
    let press = t0 + ms(2000);
    assert_eq!(state.on_input(auto(true, press), press), None);
    assert_eq!(
        state.on_input(auto(false, press + ms(80)), press + ms(580)),
        None
    );
    assert_eq!(state.on_grace_expired(), None);
    assert!(
        state.is_locked(),
        "the remembered tap starts a locked session"
    );
}
