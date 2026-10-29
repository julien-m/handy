//! Tests of the shared shortcut conflict checker.
use super::*;

const TAURI: KeyboardImplementation = KeyboardImplementation::Tauri;
const HANDY_KEYS: KeyboardImplementation = KeyboardImplementation::HandyKeys;

fn entry(action_id: &str, shortcut: &str) -> InventoryEntry {
    InventoryEntry {
        action_id: action_id.to_string(),
        shortcut: shortcut.to_string(),
    }
}

fn sample_inventory() -> Vec<InventoryEntry> {
    vec![
        entry("cancel", "escape"),
        entry("transcribe", "option+space"),
        entry("transcribe_with_post_process", "option+shift+space"),
    ]
}

#[test]
fn reordered_modifiers_conflict_with_the_owning_action() -> Result<(), String> {
    for implementation in [TAURI, HANDY_KEYS] {
        let conflict = find_conflict(
            "shift+option+space",
            &sample_inventory(),
            None,
            implementation,
        )?;
        let expected = ShortcutConflict {
            kind: ConflictKind::Duplicate,
            owning_action: "transcribe_with_post_process".into(),
            owning_shortcut: "option+shift+space".into(),
        };
        assert_eq!(conflict, Some(expected), "{implementation:?}");
    }
    Ok(())
}

#[test]
fn modifier_aliases_normalize_to_the_same_family() -> Result<(), String> {
    let cases = [("alt+space", "option+space"), ("cmd+k", "command+k")];
    for (candidate, stored) in cases {
        for implementation in [TAURI, HANDY_KEYS] {
            let inventory = [entry("transcribe", stored)];
            let conflict = find_conflict(candidate, &inventory, None, implementation)?;
            assert!(
                conflict.is_some(),
                "{candidate} vs {stored} on {implementation:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn side_specific_modifiers_collapse_for_comparison() -> Result<(), String> {
    let inventory = [entry("transcribe", "option_left+space")];
    let conflict = find_conflict("option_right+space", &inventory, None, HANDY_KEYS)?;
    assert_eq!(
        conflict.map(|c| c.kind),
        Some(ConflictKind::Duplicate),
        "left and right option must compare equal"
    );
    Ok(())
}

#[test]
fn duplicate_within_the_same_action_is_rejected() -> Result<(), String> {
    let conflict = find_conflict("option+space", &sample_inventory(), None, TAURI)?;
    assert_eq!(
        conflict.map(|c| c.owning_action),
        Some("transcribe".to_string())
    );
    Ok(())
}

#[test]
fn cancel_and_disabled_actions_take_part_in_the_check() -> Result<(), String> {
    let conflict = find_conflict("escape", &sample_inventory(), None, TAURI)?;
    assert_eq!(
        conflict.map(|c| c.owning_action),
        Some("cancel".to_string())
    );
    Ok(())
}

#[test]
fn distinct_combination_has_no_conflict() -> Result<(), String> {
    for implementation in [TAURI, HANDY_KEYS] {
        let conflict = find_conflict("ctrl+option+d", &sample_inventory(), None, implementation)?;
        assert_eq!(conflict, None, "{implementation:?}");
    }
    Ok(())
}

#[test]
fn modifier_only_subset_overlaps_in_both_directions() -> Result<(), String> {
    let inventory = [entry("transcribe", "option")];
    let conflict = find_conflict("option+space", &inventory, None, HANDY_KEYS)?;
    assert_eq!(
        conflict.map(|c| c.kind),
        Some(ConflictKind::ModifierOverlap)
    );

    let inventory = [entry("transcribe", "option+shift+space")];
    let conflict = find_conflict("shift", &inventory, None, HANDY_KEYS)?;
    assert_eq!(
        conflict.map(|c| c.kind),
        Some(ConflictKind::ModifierOverlap)
    );
    Ok(())
}

#[test]
fn side_specific_modifier_only_overlaps_only_on_its_side() -> Result<(), String> {
    // (modifier-only, combination, overlaps)
    let cases = [
        ("command_right", "cmd+j", true),
        ("command_right", "command_right+j", true),
        ("command_right", "command_left+j", false),
        ("cmd", "command_left+j", true),
        ("cmd", "command_right+j", true),
    ];
    for (modifier_only, combination, overlaps) in cases {
        let expected = overlaps.then_some(ConflictKind::ModifierOverlap);
        // Candidate is the modifier-only shortcut.
        let inventory = [entry("transcribe", combination)];
        let conflict = find_conflict(modifier_only, &inventory, None, HANDY_KEYS)?;
        assert_eq!(
            conflict.map(|c| c.kind),
            expected,
            "candidate {modifier_only} vs stored {combination}"
        );
        // Stored shortcut is the modifier-only one.
        let inventory = [entry("transcribe", modifier_only)];
        let conflict = find_conflict(combination, &inventory, None, HANDY_KEYS)?;
        assert_eq!(
            conflict.map(|c| c.kind),
            expected,
            "candidate {combination} vs stored {modifier_only}"
        );
    }
    Ok(())
}

#[test]
fn tauri_modifiers_register_both_sides() -> Result<(), String> {
    let a = normalize_shortcut("super+j", TAURI)?;
    let b = normalize_shortcut("command+k", TAURI)?;
    assert!(a.sides.iter().all(|s| *s == 0 || *s == SIDE_BOTH));
    assert!(b.sides.iter().all(|s| *s == 0 || *s == SIDE_BOTH));
    Ok(())
}

#[test]
fn modifier_only_without_shared_family_is_accepted() -> Result<(), String> {
    let inventory = [entry("transcribe", "option")];
    let conflict = find_conflict("ctrl+space", &inventory, None, HANDY_KEYS)?;
    assert_eq!(conflict, None);
    Ok(())
}

#[test]
fn excluded_entry_is_ignored_for_a_replacement() -> Result<(), String> {
    let inventory = sample_inventory();
    let replaced = entry("transcribe", "option+space");
    let conflict = find_conflict("option+space", &inventory, Some(&replaced), TAURI)?;
    assert_eq!(conflict, None);
    Ok(())
}

#[test]
fn invalid_candidate_reports_the_parser_error() {
    let result = find_conflict("option+space+k", &sample_inventory(), None, TAURI);
    assert!(result.is_err());
}

#[test]
fn stored_conflicts_report_every_involved_entry() {
    let inventory = [
        entry("cancel", "escape"),
        entry("transcribe", "option+space"),
        entry("transcribe_with_post_process", "alt+space"),
    ];
    let conflicts = find_stored_conflicts(&inventory, TAURI);
    let involved: Vec<(&str, &str)> = conflicts
        .iter()
        .map(|c| (c.action_id.as_str(), c.conflict.owning_action.as_str()))
        .collect();
    assert_eq!(
        involved,
        vec![
            ("transcribe", "transcribe_with_post_process"),
            ("transcribe_with_post_process", "transcribe"),
        ]
    );
}

#[test]
fn inventory_lists_alternatives_in_order() {
    let mut bindings = HashMap::new();
    let mut transcribe = crate::settings::get_default_settings().bindings["transcribe"].clone();
    transcribe.current_binding = "f13".into();
    transcribe.additional_bindings = vec!["f14".into(), "f15".into()];
    bindings.insert("transcribe".to_string(), transcribe);
    let shortcuts: Vec<String> = inventory_of(&bindings)
        .into_iter()
        .map(|e| e.shortcut)
        .collect();
    assert_eq!(shortcuts, vec!["f13", "f14", "f15"]);
}
