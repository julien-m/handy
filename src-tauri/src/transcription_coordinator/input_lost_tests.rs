//! Feature 001: a lost input ends the recording once through the normal stop, on the real state
//! machine; the held key's release is then a no-op and a later press starts a new session.
use super::{CoordinatorState, Effect, InputEvent, Stage};
use crate::settings::ShortcutActivation;
use std::time::{Duration, Instant};

fn push_to_talk(is_pressed: bool) -> InputEvent {
    InputEvent {
        binding_id: "transcribe".into(),
        shortcut_id: "transcribe".into(),
        hotkey_string: "transcribe".into(),
        is_pressed,
        mode: ShortcutActivation::PushToTalk,
        hold_threshold: Duration::ZERO,
        external: false,
    }
}

// @spec AC-004: Loss ends the session without losing its audio — .specs/features/001-microphone-priority/spec.md#ac-004
#[test]
fn a_lost_input_stops_once_and_the_later_release_is_a_no_op() {
    let mut state = CoordinatorState::new();
    let now = Instant::now();
    let started = state.on_input(push_to_talk(true), now);
    assert!(matches!(started, Some(Effect::Start { .. })));
    assert!(matches!(state.on_input_lost(), Some(Effect::Stop { .. })));
    assert_eq!(state.stage, Stage::Processing);
    assert_eq!(state.on_input_lost(), None, "exactly one stop");
    let later = now + Duration::from_millis(200);
    assert_eq!(state.on_input(push_to_talk(false), later), None);
    assert_eq!(state.on_grace_expired(), None);
    assert_eq!(state.on_processing_finished(), None);
    assert_eq!(state.stage, Stage::Idle);
    let next = state.on_input(push_to_talk(true), later + Duration::from_secs(1));
    assert!(matches!(next, Some(Effect::Start { .. })));
}
#[test]
fn a_loss_without_a_recording_changes_nothing() {
    let mut state = CoordinatorState::new();
    assert_eq!(state.on_input_lost(), None);
    assert_eq!(state.stage, Stage::Idle);
}
