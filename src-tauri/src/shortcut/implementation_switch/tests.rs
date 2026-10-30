//! Tests of the keyboard implementation switch repair, with the real
//! parsers of both implementations.

use super::*;
use crate::shortcut::conflicts::ConflictKind;

const TAURI: KeyboardImplementation = KeyboardImplementation::Tauri;
const HANDY_KEYS: KeyboardImplementation = KeyboardImplementation::HandyKeys;

fn binding(id: &str, shortcuts: &[&str], default: &str) -> ShortcutBinding {
    let mut entries = shortcuts.iter().map(|s| s.to_string());
    ShortcutBinding {
        id: id.to_string(),
        name: id.to_string(),
        description: String::new(),
        default_binding: default.to_string(),
        current_binding: entries.next().unwrap_or_default(),
        additional_bindings: entries.collect(),
    }
}

fn bindings(list: &[ShortcutBinding]) -> HashMap<String, ShortcutBinding> {
    list.iter().map(|b| (b.id.clone(), b.clone())).collect()
}

fn repaired_for(shortcuts: &[&str], to: KeyboardImplementation) -> Vec<String> {
    let original = binding("transcribe", shortcuts, "option+space");
    repair_for_implementation(&original, "option+space", |raw| {
        destination_form(raw, to).ok()
    })
    .shortcuts()
}

#[test]
fn an_alternative_equivalent_to_the_reset_default_is_merged() {
    // `option_left+space` is unreadable for Tauri and resets to the
    // default `option+space`, which `alt+space` duplicates once parsed.
    let repaired = repaired_for(&["option_left+space", "alt+space", "f13"], TAURI);
    assert_eq!(repaired, vec!["option+space", "f13"]);
}

#[test]
fn unreadable_entries_are_reset_or_removed() {
    let repaired = repaired_for(&["option_left+space", "f13", "fn", "ctrl_right+a"], TAURI);
    assert_eq!(repaired, vec!["option+space", "f13"]);
}

#[test]
fn unchanged_equivalent_entries_are_kept_as_inherited_conflicts() {
    for to in [TAURI, HANDY_KEYS] {
        let repaired = repaired_for(&["alt+space", "option+space"], to);
        assert_eq!(repaired, vec!["alt+space", "option+space"]);
    }
}

#[test]
fn a_valid_list_is_unchanged() {
    let repaired = repaired_for(&["option+space", "f13"], HANDY_KEYS);
    assert_eq!(repaired, vec!["option+space", "f13"]);
}

#[test]
fn plan_reports_repaired_actions_and_skips_inactive_ones() {
    let defaults = bindings(&[
        binding("transcribe", &["option+space"], "option+space"),
        binding("cancel", &["escape"], "escape"),
        binding("test", &["f12"], "f12"),
    ]);
    let stored = bindings(&[
        binding("transcribe", &["option_left+space", "f13"], "option+space"),
        binding("cancel", &["fn+escape"], "escape"),
    ]);
    let repair = plan_switch_repair(
        &stored,
        &defaults,
        |id| id != "cancel",
        |raw| destination_form(raw, TAURI).ok(),
    );
    assert_eq!(repair.repaired, vec!["transcribe".to_string()]);
    assert_eq!(
        repair.bindings["transcribe"].shortcuts(),
        ["option+space", "f13"]
    );
    assert_eq!(repair.bindings["cancel"].shortcuts(), ["fn+escape"]);
    let active: Vec<&str> = repair.active.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(active, vec!["test", "transcribe"]);
}

/// HandyKeys store whose `fn+space` resets, under Tauri, to the default
/// `option+space` that `alt+space` of post-processing already means.
fn store_with_a_colliding_reset() -> (HashMap<String, ShortcutBinding>, SwitchRepair) {
    let post = "transcribe_with_post_process";
    let stored = bindings(&[
        binding("transcribe", &["fn+space"], "option+space"),
        binding(post, &["alt+space"], "option+shift+space"),
    ]);
    let defaults = bindings(&[
        binding("transcribe", &["option+space"], "option+space"),
        binding(post, &["option+shift+space"], "option+shift+space"),
    ]);
    let repair = plan_switch_repair(
        &stored,
        &defaults,
        |_| true,
        |raw| destination_form(raw, TAURI).ok(),
    );
    (stored, repair)
}

#[test]
fn a_reset_default_owned_by_another_action_is_kept() {
    let (_, repair) = store_with_a_colliding_reset();
    assert_eq!(repair.bindings["transcribe"].shortcuts(), ["option+space"]);
    assert_eq!(
        repair.bindings["transcribe_with_post_process"].shortcuts(),
        ["alt+space"]
    );
}

#[test]
fn a_conflict_introduced_by_the_switch_is_reported_on_both_sides() {
    let (stored, repair) = store_with_a_colliding_reset();
    let conflicts = introduced_conflicts(&stored, HANDY_KEYS, &repair.bindings, TAURI);
    let reported: Vec<(&str, &str)> = conflicts
        .iter()
        .map(|c| (c.action_id.as_str(), c.shortcut.as_str()))
        .collect();
    let expected = [
        ("transcribe", "option+space"),
        ("transcribe_with_post_process", "alt+space"),
    ];
    assert_eq!(reported, expected);
    assert!(conflicts
        .iter()
        .all(|c| c.conflict.kind == ConflictKind::Duplicate));
}

#[test]
fn conflicts_present_before_the_switch_are_not_reported_again() {
    let stored = bindings(&[
        binding("transcribe", &["alt+space"], "option+space"),
        binding(
            "transcribe_with_post_process",
            &["option+space"],
            "option+shift+space",
        ),
    ]);
    assert!(introduced_conflicts(&stored, HANDY_KEYS, &stored, TAURI).is_empty());
    assert!(!find_stored_conflicts(&inventory_of(&stored), TAURI).is_empty());
}
