//! Lone press against combination (014 AC-005, AC-006, SC-002): only a press then a release of
//! the same modifier, with nothing in between, is a tap.
use super::*;

fn down(key: Modifiers) -> KeyInput {
    KeyInput::Modifier { key, down: true }
}
fn up(key: Modifiers) -> KeyInput {
    KeyInput::Modifier { key, down: false }
}
fn key_down(key: Key) -> KeyInput {
    KeyInput::Other { key, down: true }
}
fn key_up(key: Key) -> KeyInput {
    KeyInput::Other { key, down: false }
}

/// The tapped keys `inputs` produce with no Handy shortcut firing.
fn taps(inputs: &[KeyInput]) -> Vec<Modifiers> {
    let mut detector = TapDetector::default();
    inputs
        .iter()
        .filter_map(|input| detector.feed(*input, 0))
        .map(|tap| tap.key)
        .collect()
}

// @spec SC-002: Lone press against combination — .specs/features/014-in-dictation-keys/spec.md#sc-002
#[test]
fn a_lone_press_is_a_tap_and_combinations_are_not() {
    let option = Modifiers::OPT_LEFT;
    let control = Modifiers::CTRL_LEFT;
    let shift = Modifiers::SHIFT_LEFT;
    let cases: [(&str, Vec<KeyInput>, Vec<Modifiers>); 7] = [
        ("option alone", vec![down(option), up(option)], vec![option]),
        (
            "option+space",
            vec![
                down(option),
                key_down(Key::Space),
                key_up(Key::Space),
                up(option),
            ],
            vec![],
        ),
        (
            "ctrl+c",
            vec![down(control), key_down(Key::C), up(control)],
            vec![],
        ),
        (
            "shift for a capital",
            vec![down(shift), key_down(Key::A), key_up(Key::A), up(shift)],
            vec![],
        ),
        (
            "two modifiers",
            vec![down(option), down(control), up(control), up(option)],
            vec![],
        ),
        (
            "a key released during the press",
            vec![key_down(Key::A), down(option), key_up(Key::A), up(option)],
            vec![],
        ),
        (
            "two lone presses in a row",
            vec![down(option), up(option), down(control), up(control)],
            vec![option, control],
        ),
    ];
    for (name, inputs, expected) in cases {
        assert_eq!(taps(&inputs), expected, "{name}");
    }
}

// @spec AC-006: Push-to-talk Option+Space keeps the keys usable — .specs/features/014-in-dictation-keys/spec.md#ac-006
#[test]
fn the_auto_repeat_of_a_held_key_does_not_cancel_a_tap() {
    // Push-to-talk Option+Space: Space stays down and repeats while Control left is tapped.
    let control = Modifiers::CTRL_LEFT;
    let space = key_down(Key::Space);
    let inputs = [space, space, down(control), space, space, up(control)];
    assert_eq!(taps(&inputs), vec![control]);
}

#[test]
fn a_modifier_held_before_the_press_does_not_matter() {
    // Push-to-talk on Command right: held since before the observer started, released at the end.
    let option = Modifiers::OPT_LEFT;
    let held = Modifiers::CMD_RIGHT;
    assert_eq!(taps(&[down(option), up(option), up(held)]), vec![option]);
}

#[test]
fn a_handy_shortcut_firing_during_the_press_cancels_the_tap() {
    // The backend hid the Space of Option+Space from the observer, but the shortcut fired.
    let mut detector = TapDetector::default();
    let option = Modifiers::OPT_LEFT;
    assert_eq!(detector.feed(down(option), 4), None);
    assert_eq!(detector.feed(up(option), 5), None);
    assert_eq!(detector.feed(down(option), 5), None);
    let tap = detector.feed(up(option), 5);
    assert_eq!(
        tap,
        Some(Tap {
            key: option,
            shortcuts_at_press: 5
        })
    );
}

#[test]
fn a_release_of_another_modifier_during_the_press_cancels_the_tap() {
    let option = Modifiers::OPT_LEFT;
    let held = Modifiers::CMD_RIGHT;
    assert!(taps(&[down(option), up(held), up(option)]).is_empty());
}

#[test]
fn a_lost_release_does_not_block_the_next_press_of_the_same_key() {
    let option = Modifiers::OPT_LEFT;
    assert_eq!(
        taps(&[down(option), down(option), up(option)]),
        vec![option]
    );
}

#[test]
fn the_direction_comes_from_the_real_state_when_the_listener_toggles() {
    // macOS: Command right is held when the observer starts; the listener reports its release
    // as a press (its own state started empty).
    let held = Modifiers::CMD_RIGHT;
    let mut derived = ModifierState::new(held, true);
    assert!(
        !derived.is_press(held, true),
        "a release despite the report"
    );
    assert!(derived.is_press(Modifiers::OPT_LEFT, true));
    assert!(!derived.is_press(Modifiers::OPT_LEFT, false));
    let mut reported = ModifierState::new(Modifiers::empty(), false);
    assert!(!reported.is_press(held, false), "the report is trusted");
}

#[test]
fn a_side_or_both_sides_match_the_tapped_key() {
    assert!(is_configured_key(Modifiers::OPT_LEFT, Modifiers::OPT_LEFT));
    assert!(!is_configured_key(
        Modifiers::OPT_LEFT,
        Modifiers::OPT_RIGHT
    ));
    assert!(is_configured_key(Modifiers::OPT, Modifiers::OPT_RIGHT));
    assert!(!is_configured_key(
        Modifiers::CTRL_LEFT,
        Modifiers::OPT_LEFT
    ));
}
