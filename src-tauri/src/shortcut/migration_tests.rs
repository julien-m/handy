//! Upgrade and downgrade compatibility of the stored shortcut lists.
//!
//! The first shortcut of an action stays in `current_binding`, where Handy
//! versions before multiple shortcuts read it; alternatives live in an
//! additive `additional_bindings` field those versions ignore.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::settings::{AppSettings, ShortcutBinding};
use crate::shortcut::conflicts::{find_stored_conflicts, inventory_of};

type TestResult = Result<(), serde_json::Error>;

/// Binding shape read by Handy versions before multiple shortcuts, frozen
/// from the pre-004 source (no `additional_bindings`, no unknown-field
/// rejection).
#[derive(Deserialize)]
struct LegacyShortcutBinding {
    id: String,
    current_binding: String,
}

fn legacy_bindings() -> Value {
    json!({
        "transcribe": {
            "id": "transcribe",
            "name": "Transcribe",
            "description": "Converts your speech into text.",
            "default_binding": "option+space",
            "current_binding": "f13"
        },
        "transcribe_with_post_process": {
            "id": "transcribe_with_post_process",
            "name": "Transcribe with Post-Processing",
            "description": "Converts your speech into text and applies AI post-processing.",
            "default_binding": "option+shift+space",
            "current_binding": "option+shift+space"
        },
        "cancel": {
            "id": "cancel",
            "name": "Cancel",
            "description": "Cancels the current recording.",
            "default_binding": "escape",
            "current_binding": "escape"
        }
    })
}

#[test]
fn legacy_store_loads_each_shortcut_as_the_first_of_its_action() -> TestResult {
    let settings: AppSettings = serde_json::from_value(json!({ "bindings": legacy_bindings() }))?;
    let transcribe = &settings.bindings["transcribe"];
    assert_eq!(transcribe.shortcuts(), vec!["f13"]);
    assert!(transcribe.additional_bindings.is_empty());
    Ok(())
}

#[test]
fn single_shortcut_bindings_serialize_byte_identical_to_the_legacy_shape() -> TestResult {
    let legacy = legacy_bindings();
    let loaded: HashMap<String, ShortcutBinding> = serde_json::from_value(legacy.clone())?;
    assert_eq!(serde_json::to_value(&loaded)?, legacy);
    Ok(())
}

#[test]
fn loading_twice_is_idempotent() -> TestResult {
    let mut bindings: HashMap<String, ShortcutBinding> = serde_json::from_value(legacy_bindings())?;
    if let Some(transcribe) = bindings.get_mut("transcribe") {
        transcribe.additional_bindings = vec!["ctrl+option+d".into()];
    }
    let once = serde_json::to_value(&bindings)?;
    let reloaded: HashMap<String, ShortcutBinding> = serde_json::from_value(once.clone())?;
    assert_eq!(serde_json::to_value(&reloaded)?, once);
    Ok(())
}

#[test]
fn older_reader_keeps_the_first_shortcut_and_ignores_alternatives() -> TestResult {
    let mut bindings: HashMap<String, ShortcutBinding> = serde_json::from_value(legacy_bindings())?;
    if let Some(transcribe) = bindings.get_mut("transcribe") {
        transcribe.additional_bindings = vec!["ctrl+option+d".into(), "f14".into()];
    }
    let stored = serde_json::to_value(&bindings)?;
    let downgraded: HashMap<String, LegacyShortcutBinding> = serde_json::from_value(stored)?;
    let transcribe = &downgraded["transcribe"];
    assert_eq!(transcribe.id, "transcribe");
    assert_eq!(transcribe.current_binding, "f13");
    Ok(())
}

#[test]
fn other_actions_keep_their_stored_values() -> TestResult {
    let legacy = legacy_bindings();
    let bindings: HashMap<String, ShortcutBinding> = serde_json::from_value(legacy.clone())?;
    let stored = serde_json::to_value(&bindings)?;
    assert_eq!(stored["cancel"], legacy["cancel"]);
    Ok(())
}

#[test]
fn legacy_conflicts_are_loaded_unchanged_and_reported() -> TestResult {
    let mut legacy = legacy_bindings();
    legacy["transcribe_with_post_process"]["current_binding"] = json!("alt+f13");
    legacy["transcribe"]["current_binding"] = json!("option+f13");
    let bindings: HashMap<String, ShortcutBinding> = serde_json::from_value(legacy)?;
    assert_eq!(bindings["transcribe"].current_binding, "option+f13");
    assert_eq!(
        bindings["transcribe_with_post_process"].current_binding,
        "alt+f13"
    );

    let conflicts = find_stored_conflicts(
        &inventory_of(&bindings),
        crate::settings::KeyboardImplementation::Tauri,
    );
    let involved: Vec<&str> = conflicts.iter().map(|c| c.action_id.as_str()).collect();
    assert_eq!(involved, vec!["transcribe", "transcribe_with_post_process"]);
    Ok(())
}
