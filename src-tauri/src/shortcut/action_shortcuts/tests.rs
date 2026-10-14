//! Tests of the pure shortcut list edits.

use super::*;
use crate::shortcut::conflicts::{inventory_of, ConflictKind};
use std::collections::HashMap;

type TestResult = Result<(), ShortcutEditError>;

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

fn standard() -> HashMap<String, ShortcutBinding> {
    bindings(&[
        binding("transcribe", &["option+space"], "option+space"),
        binding(
            "transcribe_with_post_process",
            &["option+shift+space"],
            "option+shift+space",
        ),
        binding("cancel", &["escape"], "escape"),
    ])
}

fn edit(
    all: &HashMap<String, ShortcutBinding>,
    action_id: &str,
    change: ShortcutEdit,
) -> Result<ShortcutBinding, ShortcutEditError> {
    let inventory = inventory_of(all);
    let context = EditContext {
        inventory: &inventory,
        implementation: KeyboardImplementation::Tauri,
    };
    let current = all
        .get(action_id)
        .cloned()
        .unwrap_or_else(|| binding(action_id, &[], ""));
    plan_edit(&current, &change, &context)
}

fn add(shortcut: &str) -> ShortcutEdit {
    ShortcutEdit::Add {
        shortcut: shortcut.to_string(),
    }
}

fn remove(shortcut: &str) -> ShortcutEdit {
    ShortcutEdit::Remove {
        shortcut: shortcut.to_string(),
    }
}

#[test]
fn add_appends_an_alternative_after_the_first_shortcut() -> TestResult {
    let planned = edit(&standard(), "transcribe", add("ctrl+option+d"))?;
    assert_eq!(planned.current_binding, "option+space");
    assert_eq!(planned.shortcuts(), vec!["option+space", "ctrl+option+d"]);
    Ok(())
}

#[test]
fn fifth_shortcut_is_refused_with_the_limit() {
    let mut all = standard();
    all.insert(
        "transcribe".into(),
        binding("transcribe", &["f13", "f14", "f15", "f16"], "f13"),
    );
    let result = edit(&all, "transcribe", add("f17"));
    assert_eq!(
        result.err(),
        Some(ShortcutEditError::LimitExceeded { limit: 4 })
    );
}

#[test]
fn fourth_shortcut_is_accepted() -> TestResult {
    let mut all = standard();
    all.insert(
        "transcribe".into(),
        binding("transcribe", &["f13", "f14", "f15"], "f13"),
    );
    let planned = edit(&all, "transcribe", add("f16"))?;
    assert_eq!(planned.shortcuts().len(), MAX_SHORTCUTS_PER_ACTION);
    Ok(())
}

#[test]
fn cancel_does_not_accept_alternatives() {
    let result = edit(&standard(), "cancel", add("f13"));
    assert_eq!(
        result.err(),
        Some(ShortcutEditError::AlternativesUnsupported)
    );
}

#[test]
fn conflicting_addition_names_the_owning_action() {
    let owner = match edit(&standard(), "transcribe", add("shift+option+space")) {
        Err(ShortcutEditError::ShortcutConflict { conflict }) => {
            Some((conflict.owning_action, conflict.kind))
        }
        _ => None,
    };
    assert_eq!(
        owner,
        Some((
            "transcribe_with_post_process".to_string(),
            ConflictKind::Duplicate
        ))
    );
}

#[test]
fn duplicate_within_the_action_is_refused() {
    let result = edit(&standard(), "transcribe", add("option+space"));
    assert!(matches!(
        result,
        Err(ShortcutEditError::ShortcutConflict { .. })
    ));
}

#[test]
fn cancel_shortcut_cannot_be_reused() {
    let result = edit(&standard(), "transcribe", add("escape"));
    assert!(matches!(
        result,
        Err(ShortcutEditError::ShortcutConflict { .. })
    ));
}

#[test]
fn invalid_syntax_is_refused_before_the_conflict_check() {
    let result = edit(&standard(), "transcribe", add("option"));
    assert!(matches!(
        result,
        Err(ShortcutEditError::InvalidShortcut { .. })
    ));
}

#[test]
fn last_shortcut_cannot_be_removed() {
    let result = edit(&standard(), "transcribe", remove("option+space"));
    assert_eq!(result.err(), Some(ShortcutEditError::LastShortcut));
}

#[test]
fn removing_the_first_shortcut_promotes_the_next() -> TestResult {
    let mut all = standard();
    all.insert(
        "transcribe".into(),
        binding(
            "transcribe",
            &["option+space", "f13", "f14"],
            "option+space",
        ),
    );
    let planned = edit(&all, "transcribe", remove("option+space"))?;
    assert_eq!(planned.current_binding, "f13");
    assert_eq!(planned.additional_bindings, vec!["f14"]);
    Ok(())
}

#[test]
fn removing_an_unknown_shortcut_is_not_found() {
    let result = edit(&standard(), "transcribe", remove("f20"));
    assert_eq!(
        result.err(),
        Some(ShortcutEditError::NotFound {
            shortcut: "f20".into()
        })
    );
}

#[test]
fn replace_excludes_only_the_replaced_entry() -> TestResult {
    let mut all = standard();
    all.insert(
        "transcribe".into(),
        binding("transcribe", &["option+space", "f13"], "option+space"),
    );
    let change = ShortcutEdit::Replace {
        previous: "f13".into(),
        shortcut: "f14".into(),
    };
    let planned = edit(&all, "transcribe", change)?;
    assert_eq!(planned.shortcuts(), vec!["option+space", "f14"]);

    let onto_sibling = ShortcutEdit::Replace {
        previous: "f13".into(),
        shortcut: "option+space".into(),
    };
    assert!(edit(&all, "transcribe", onto_sibling).is_err());
    Ok(())
}

#[test]
fn legacy_conflict_survives_a_no_op_replace() -> TestResult {
    let all = bindings(&[
        binding("transcribe", &["alt+space"], "option+space"),
        binding(
            "transcribe_with_post_process",
            &["option+space"],
            "option+shift+space",
        ),
        binding("cancel", &["escape"], "escape"),
    ]);
    let change = ShortcutEdit::Replace {
        previous: "alt+space".into(),
        shortcut: "alt+space".into(),
    };
    let planned = edit(&all, "transcribe", change)?;
    assert_eq!(planned.current_binding, "alt+space");
    Ok(())
}

#[test]
fn unrelated_legacy_conflict_does_not_block_an_addition() -> TestResult {
    let all = bindings(&[
        binding("transcribe", &["alt+space"], "option+space"),
        binding(
            "transcribe_with_post_process",
            &["option+space"],
            "option+shift+space",
        ),
        binding("cancel", &["escape"], "escape"),
    ]);
    let planned = edit(&all, "transcribe", add("f13"))?;
    assert_eq!(planned.shortcuts(), vec!["alt+space", "f13"]);
    Ok(())
}

#[test]
fn reset_restores_the_single_default() -> TestResult {
    let mut all = standard();
    all.insert(
        "transcribe".into(),
        binding(
            "transcribe",
            &["f13", "option+space", "f14"],
            "option+space",
        ),
    );
    let planned = edit(&all, "transcribe", ShortcutEdit::Reset)?;
    assert_eq!(planned.shortcuts(), vec!["option+space"]);
    Ok(())
}

#[test]
fn reset_refuses_a_default_owned_by_another_action() {
    let all = bindings(&[
        binding("transcribe", &["f13"], "option+space"),
        binding(
            "transcribe_with_post_process",
            &["option+space"],
            "option+shift+space",
        ),
        binding("cancel", &["escape"], "escape"),
    ]);
    let result = edit(&all, "transcribe", ShortcutEdit::Reset);
    assert!(matches!(
        result,
        Err(ShortcutEditError::ShortcutConflict { .. })
    ));
}

#[test]
fn reset_of_a_default_list_is_a_no_op() -> TestResult {
    let all = standard();
    let planned = edit(&all, "transcribe", ShortcutEdit::Reset)?;
    assert_eq!(planned.shortcuts(), all["transcribe"].shortcuts());
    Ok(())
}

#[test]
fn edit_kinds_deserialize_from_tagged_json() -> Result<(), serde_json::Error> {
    let parsed: ShortcutEdit =
        serde_json::from_str(r#"{"kind":"add","shortcut":"ctrl+option+d"}"#)?;
    assert_eq!(parsed, add("ctrl+option+d"));
    let parsed: ShortcutEdit = serde_json::from_str(r#"{"kind":"reset"}"#)?;
    assert_eq!(parsed, ShortcutEdit::Reset);
    Ok(())
}
