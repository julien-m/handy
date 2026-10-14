//! Input-source switching around the paste chord (AC-011).

use std::time::Duration;

use handy_app_lib::smart_paste::clock::{Clock, ManualClock};
use handy_app_lib::smart_paste::keyboard::{
    restore_after_chord, switch_for_paste, InputSourceKind, KeyboardError, LayoutSwitch,
    ABC_LAYOUT_ID, US_LAYOUT_ID,
};

use super::fakes::FakeInputSources;

// @spec FR-007: Layout switch and restoration — .specs/features/007-smart-paste/spec.md#fr-007

const FRENCH: &str = "com.apple.keylayout.French";

fn french(clock: &ManualClock, selectable: &[&str]) -> FakeInputSources {
    FakeInputSources::new(clock, FRENCH, InputSourceKind::KeyboardLayout, selectable)
}

#[test]
fn azerty_switches_to_abc_and_waits_50_ms() {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[ABC_LAYOUT_ID, US_LAYOUT_ID, FRENCH]);
    let start = clock.now();
    let switch = switch_for_paste(&mut sources, &clock);
    assert_eq!(
        switch,
        LayoutSwitch::Switched {
            previous: FRENCH.to_string(),
            selected: ABC_LAYOUT_ID
        }
    );
    assert_eq!(clock.now() - start, Duration::from_millis(50));
}

#[test]
fn us_is_used_only_when_abc_cannot_be_selected() {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[US_LAYOUT_ID, FRENCH]);
    let switch = switch_for_paste(&mut sources, &clock);
    assert!(matches!(
        switch,
        LayoutSwitch::Switched {
            selected: US_LAYOUT_ID,
            ..
        }
    ));
}

#[test]
fn unselectable_layouts_and_input_methods_stay_unchanged() {
    let clock = ManualClock::new();
    let mut no_candidate = french(&clock, &[FRENCH]);
    let kana_id = "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese";
    let mut kana = FakeInputSources::new(
        &clock,
        kana_id,
        InputSourceKind::InputMethod,
        &[ABC_LAYOUT_ID],
    );
    let start = clock.now();
    assert_eq!(
        switch_for_paste(&mut no_candidate, &clock),
        LayoutSwitch::Unchanged
    );
    assert_eq!(switch_for_paste(&mut kana, &clock), LayoutSwitch::Unchanged);
    assert!(kana.state().selections.is_empty());
    assert_eq!(clock.now(), start);
}

#[test]
fn unreadable_source_sends_the_chord_unchanged() {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[ABC_LAYOUT_ID]);
    sources.state().current = None;
    assert_eq!(
        switch_for_paste(&mut sources, &clock),
        LayoutSwitch::Unchanged
    );
}

#[test]
fn previous_source_is_restored_100_ms_after_the_last_key_up() -> Result<(), KeyboardError> {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[ABC_LAYOUT_ID, FRENCH]);
    let switch = switch_for_paste(&mut sources, &clock);
    clock.advance(Duration::from_millis(100));
    let last_key_up = clock.now();
    restore_after_chord(&mut sources, &clock, &switch, last_key_up)?;
    let selections = sources.state().selections.clone();
    assert_eq!(selections.last().map(|(id, _)| id.as_str()), Some(FRENCH));
    assert_eq!(
        selections.last().map(|(_, at)| *at - last_key_up),
        Some(Duration::from_millis(100))
    );
    Ok(())
}

#[test]
fn restoration_failure_is_reported_not_swallowed() {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[ABC_LAYOUT_ID, FRENCH]);
    let switch = switch_for_paste(&mut sources, &clock);
    sources.state().selectable.retain(|id| id != FRENCH);
    let result = restore_after_chord(&mut sources, &clock, &switch, clock.now());
    assert_eq!(result, Err(KeyboardError::SelectFailed));
}

#[test]
fn unchanged_layout_needs_no_restoration() -> Result<(), KeyboardError> {
    let clock = ManualClock::new();
    let mut sources = french(&clock, &[]);
    let start = clock.now();
    restore_after_chord(&mut sources, &clock, &LayoutSwitch::Unchanged, start)?;
    assert!(sources.state().selections.is_empty());
    assert_eq!(clock.now(), start);
    Ok(())
}
