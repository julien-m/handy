//! Feature 001, Step 4: recording-boundary decisions of the audio manager, every combination of
//! recording, mode, open stream and loss. A priority change never touches a recording in
//! progress; it waits for its end. A lost always-on stream reopens once after the recording, and
//! an idle loss recovers once per user action. (The native glue is covered by the macOS harness.)
use handy_app_lib::microphones::boundary::{
    device_change, on_loss, reopen_after_recording, DeviceChange, LossAction, StreamFacts,
};

fn facts(recording: bool, always_on: bool, open: bool, lost: bool) -> StreamFacts {
    StreamFacts {
        recording,
        always_on,
        open,
        lost,
    }
}

// @spec AC-004: Editing during a recording affects the next session — .specs/features/001-microphone-priority/spec.md#ac-004
#[test]
fn a_change_during_a_recording_waits_whatever_the_mode_or_stream() {
    for (always_on, open, lost) in [
        (true, true, false),
        (false, true, false),
        (true, true, true),
    ] {
        let decision = device_change(facts(true, always_on, open, lost), || false);
        assert_eq!(decision, DeviceChange::Defer);
    }
}
#[test]
fn on_demand_closes_a_warm_stream_and_resolves_at_the_next_recording() {
    let mut asked = false;
    let decision = device_change(facts(false, false, true, false), || {
        asked = true;
        false
    });
    assert_eq!(decision, DeviceChange::CloseWarm);
    assert!(!asked, "no catalog check for a stream that closes anyway");
}
// @spec AC-003: Always-on stream follows the saved priority — .specs/features/001-microphone-priority/spec.md#ac-003
#[test]
fn always_on_reopens_only_when_the_preferred_input_changed_or_was_lost() {
    assert_eq!(
        device_change(facts(false, true, true, false), || true),
        DeviceChange::Keep
    );
    assert_eq!(
        device_change(facts(false, true, true, false), || false),
        DeviceChange::Reopen
    );
    assert_eq!(
        device_change(facts(false, true, true, true), || true),
        DeviceChange::Reopen
    );
    assert_eq!(
        device_change(facts(false, true, false, false), || true),
        DeviceChange::Open
    );
}
#[test]
fn the_end_of_a_recording_applies_a_waiting_change_or_a_loss_in_always_on_only() {
    assert!(reopen_after_recording(true, false, true));
    assert!(reopen_after_recording(false, true, true));
    assert!(!reopen_after_recording(false, false, true));
    assert!(!reopen_after_recording(true, true, false));
}
#[test]
fn a_loss_ends_a_recording_and_recovers_an_idle_stream_once_per_user_action() {
    assert_eq!(on_loss(true, true, false), LossAction::EndRecording);
    assert_eq!(on_loss(true, false, true), LossAction::EndRecording);
    assert_eq!(on_loss(false, true, true), LossAction::Recover);
    assert_eq!(on_loss(false, true, false), LossAction::Ignore);
    assert_eq!(on_loss(false, false, true), LossAction::Ignore);
}
