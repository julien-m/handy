//! Actual stored-field coverage compared with an independent access classification.

use handy_app_lib::settings_service::platform_capability::PlatformCapabilities;
use handy_app_lib::settings_service::{descriptor::descriptors, protocol::SettingsSnapshot};
use std::collections::BTreeSet;

#[test]
fn every_actual_stored_field_has_exactly_one_descriptor() -> Result<(), Box<dyn std::error::Error>>
{
    let actual: BTreeSet<_> = SettingsSnapshot::default_field_inventory()?
        .into_iter()
        .collect();
    let described: BTreeSet<_> = descriptors()
        .iter()
        .map(|entry| entry.key.to_owned())
        .collect();
    assert_eq!(actual.len(), 64);
    assert_eq!(descriptors().len(), 64);
    assert_eq!(actual, described);
    Ok(())
}

const READONLY: [&str; 7] = [
    "settings_schema_version",
    "whats_new_last_seen_version",
    "selected_model",
    "onboarding_completed",
    "post_process_provider_id",
    "post_process_providers",
    "post_process_models",
];
const COLLECTIONS: [(&str, &str); 4] = [
    ("bindings", "shortcuts"),
    ("custom_words", "vocabulary"),
    ("custom_filler_words", "vocabulary"),
    ("post_process_prompts", "prompts"),
];

#[test]
fn independent_access_inventory_covers_7_readonly_1_secret_4_collections_52_scalars(
) -> Result<(), Box<dyn std::error::Error>> {
    for key in READONLY {
        let entry = handy_app_lib::settings_service::descriptor::descriptor(key)
            .ok_or_else(|| "readonly descriptor missing".to_owned())?;
        assert!(entry.read_only);
        assert!(!entry.secret);
    }
    let secret = handy_app_lib::settings_service::descriptor::descriptor("post_process_api_keys")
        .ok_or_else(|| "secret descriptor missing".to_owned())?;
    assert!(secret.read_only && secret.secret);
    for (key, owner) in COLLECTIONS {
        let entry = handy_app_lib::settings_service::descriptor::descriptor(key)
            .ok_or_else(|| "collection descriptor missing".to_owned())?;
        assert!(!entry.read_only);
        assert_eq!(entry.owner_domain, owner);
    }
    let scalar_keys: BTreeSet<_> = super::scalars::canonical_cases()
        .into_iter()
        .map(|(key, _, _)| key)
        .collect();
    assert_eq!(scalar_keys.len(), 52);
    assert_eq!(scalar_keys, actual_scalars());
    Ok(())
}
/// Descriptor keys that are neither read-only, secret nor collections.
fn actual_scalars() -> BTreeSet<&'static str> {
    descriptors()
        .iter()
        .filter(|entry| {
            !READONLY.contains(&entry.key)
                && entry.key != "post_process_api_keys"
                && !COLLECTIONS.iter().any(|(key, _)| *key == entry.key)
        })
        .map(|entry| entry.key)
        .collect()
}

pub(super) const ENUM_INVENTORY: &[(&str, &[&str])] = &[
    (
        "shortcut_activation",
        &["toggle", "push_to_talk", "hold_or_toggle"],
    ),
    ("sound_theme", &["marimba", "pop", "custom"]),
    ("overlay_position", &["top", "bottom"]),
    ("log_level", &["trace", "debug", "info", "warn", "error"]),
    (
        "model_unload_timeout",
        &[
            "never",
            "immediately",
            "min2",
            "min5",
            "min10",
            "min15",
            "hour1",
            "sec15",
        ],
    ),
    (
        "recording_retention_period",
        &["never", "preserve_limit", "days3", "weeks2", "months3"],
    ),
    (
        "paste_method",
        &[
            "ctrl_v",
            "direct",
            "none",
            "shift_insert",
            "ctrl_shift_v",
            "external_script",
        ],
    ),
    ("clipboard_handling", &["dont_modify", "copy_to_clipboard"]),
    ("auto_submit_key", &["enter", "ctrl_enter", "cmd_enter"]),
    ("theme", &["system", "light", "dark"]),
    ("keyboard_implementation", &["tauri", "handy_keys"]),
    (
        "typing_tool",
        &["auto", "wtype", "kwtype", "dotool", "ydotool", "xdotool"],
    ),
    (
        "chinese_script",
        &["as_transcribed", "simplified", "traditional"],
    ),
    ("transcribe_accelerator", &["auto", "cpu", "gpu"]),
    (
        "ort_accelerator",
        &["auto", "cpu", "cuda", "directml", "rocm"],
    ),
    ("vad_backend", &["silero", "earshot"]),
    ("overlay_style", &["none", "minimal", "live"]),
];

#[test]
fn independent_enum_inventory_accepts_every_current_canonical_variant(
) -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::{
        descriptor::descriptor,
        protocol::Revision,
        validation::{validate_cli, validate_ui, ValidationContext},
    };
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    assert_eq!(ENUM_INVENTORY.len(), 17);
    for (key, values) in ENUM_INVENTORY {
        assert_eq!(
            descriptor(key).ok_or("enum descriptor")?.allowed_values,
            *values
        );
        for value in *values {
            let expected = serde_json::json!(value);
            assert_eq!(
                validate_cli(key, value, &context)?,
                validate_ui(key, expected.clone(), &context)?
            );
            let mut object = serde_json::Map::new();
            object.insert((*key).to_owned(), expected.clone());
            let snapshot = SettingsSnapshot::from_stored_value(object.into(), Revision(0))?;
            assert_eq!(
                serde_json::to_value(snapshot.public_settings())?[*key],
                expected
            );
        }
        assert!(validate_cli(key, "CANARY_INVALID_ENUM_SECRET", &context).is_err());
    }
    Ok(())
}
