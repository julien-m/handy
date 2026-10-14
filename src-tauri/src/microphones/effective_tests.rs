//! Current, last-recording and not-used-yet views of the effective input (pure).
use super::{EffectiveInput, EffectiveScope, EffectiveState};
use crate::microphones::types::EffectiveReason;

fn input(name: &str, reason: EffectiveReason) -> EffectiveInput {
    EffectiveInput {
        name: Some(name.to_owned()),
        choice: Some(name.to_owned()),
        reason,
        capture_generation: 0,
    }
}

// @spec AC-002: Effective device reported apart from saved order — .specs/features/001-microphone-priority/spec.md#ac-002
#[test]
fn a_recording_is_remembered_and_idle_openings_never_overwrite_it() {
    let mut state = EffectiveState::default();
    assert_eq!(state.view(false).scope, EffectiveScope::NotUsedYet);
    state.stream_opened(input("USB headset", EffectiveReason::FirstChoice));
    state.recording_started(7);
    let current = state.view(true);
    assert_eq!(current.scope, EffectiveScope::Current);
    assert_eq!(current.input.map(|input| input.capture_generation), Some(7));
    state.stream_closed();
    state.stream_opened(input("MacBook microphone", EffectiveReason::Fallback));
    let idle = state.view(false);
    assert_eq!(idle.scope, EffectiveScope::LastRecording);
    assert_eq!(
        idle.input.and_then(|input| input.name),
        Some("USB headset".into())
    );
}
#[test]
fn the_current_choice_distinguishes_no_stream_from_the_system_default() {
    let mut state = EffectiveState::default();
    assert_eq!(state.current_choice(), None);
    let mut default = input("Built-in", EffectiveReason::SystemDefault);
    default.choice = None;
    state.stream_opened(default);
    assert_eq!(state.current_choice(), Some(None));
}
