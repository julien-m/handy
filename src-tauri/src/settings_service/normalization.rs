//! Pure legacy salvage and normalization; callers decide explicitly whether to persist.
use super::errors::{SettingsError, SettingsErrorCode};
use crate::settings::{self, AppSettings};
use serde_json::Value;

/// Normalize an existing payload without mutating its raw document or logging values.
// @spec FR-002: In-memory salvage and migrations — .specs/features/008-settings-cli/spec.md#fr-002
pub(super) fn normalize(payload: &Value) -> Result<(AppSettings, bool), SettingsError> {
    let (mut settings, recovered) = match serde_json::from_value(payload.clone()) {
        Ok(settings) => (settings, false),
        Err(_) => (salvage(payload)?, true),
    };
    settings::apply_settings_migrations(&mut settings, payload);
    for (key, binding) in settings::get_default_settings().bindings {
        settings.bindings.entry(key).or_insert(binding);
    }
    settings::ensure_post_process_defaults(&mut settings);
    Ok((settings, recovered))
}
/// Fresh-install settings for a store that has no settings payload yet; no migration applies.
pub(super) fn fresh_install() -> AppSettings {
    let mut settings = settings::get_default_settings();
    settings::ensure_post_process_defaults(&mut settings);
    settings
}
fn salvage(payload: &Value) -> Result<AppSettings, SettingsError> {
    let mut merged =
        serde_json::to_value(settings::get_default_settings()).map_err(|_| failed())?;
    // Probe one recognized field at a time against defaults; unknown values stay only in raw storage.
    if let Some(fields) = payload.as_object() {
        for entry in super::descriptor::descriptors() {
            if let Some(value) = fields.get(entry.key) {
                let mut candidate = merged.clone();
                candidate[entry.key] = value.clone();
                if serde_json::from_value::<AppSettings>(candidate.clone()).is_ok() {
                    merged = candidate;
                }
            }
        }
    }
    serde_json::from_value(merged).map_err(|_| failed())
}
fn failed() -> SettingsError {
    SettingsError::new(SettingsErrorCode::PersistenceFailed)
}
#[cfg(test)]
mod tests {
    //! Salvage keeps every individually valid stored field (#1619); ported from the retired
    //! plugin-store reader so the single salvage implementation stays covered.
    use super::salvage;
    use crate::settings::{get_default_settings, AppSettings};
    use serde_json::{json, Value};
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn default_settings_json() -> Result<Value, serde_json::Error> {
        serde_json::to_value(get_default_settings())
    }
    #[test]
    fn salvage_preserves_valid_fields_when_one_value_is_invalid() -> TestResult {
        let mut stored = default_settings_json()?;
        stored["selected_model"] = json!("parakeet-tdt-0.6b-v3");
        stored["onboarding_completed"] = json!(true);
        // An enum variant this build does not know, e.g. written by a newer version.
        stored["sound_theme"] = json!("theremin");
        stored["bindings"]["transcribe"]["current_binding"] = json!("f13");
        assert!(serde_json::from_value::<AppSettings>(stored.clone()).is_err());
        let salvaged = salvage(&stored)?;
        assert_eq!(salvaged.selected_model, "parakeet-tdt-0.6b-v3");
        assert!(salvaged.onboarding_completed);
        assert_eq!(salvaged.bindings["transcribe"].current_binding, "f13");
        assert_eq!(salvaged.sound_theme, get_default_settings().sound_theme);
        Ok(())
    }
    #[test]
    fn salvage_drops_only_wrong_typed_fields() -> TestResult {
        let mut stored = default_settings_json()?;
        stored["paste_delay_ms"] = json!("sixty");
        stored["sound_theme"] = json!(42);
        stored["custom_words"] = json!(["handy"]);
        assert!(serde_json::from_value::<AppSettings>(stored.clone()).is_err());
        let salvaged = salvage(&stored)?;
        let defaults = get_default_settings();
        assert_eq!(salvaged.paste_delay_ms, defaults.paste_delay_ms);
        assert_eq!(salvaged.sound_theme, defaults.sound_theme);
        assert_eq!(salvaged.custom_words, vec!["handy".to_string()]);
        Ok(())
    }
    #[test]
    fn salvage_of_poisoned_bindings_keeps_other_fields() -> TestResult {
        let mut stored = default_settings_json()?;
        // One malformed entry poisons the whole bindings map, not the rest of the settings.
        stored["bindings"] = json!({ "transcribe": { "id": 42 } });
        stored["selected_model"] = json!("whisper-small");
        assert!(serde_json::from_value::<AppSettings>(stored.clone()).is_err());
        let salvaged = salvage(&stored)?;
        assert_eq!(salvaged.selected_model, "whisper-small");
        assert_eq!(
            salvaged
                .bindings
                .get("transcribe")
                .map(|b| &b.current_binding),
            get_default_settings()
                .bindings
                .get("transcribe")
                .map(|b| &b.current_binding)
        );
        Ok(())
    }
    #[test]
    fn salvage_tolerates_unknown_keys() -> TestResult {
        let mut stored = default_settings_json()?;
        stored["field_from_the_future"] = json!({ "nested": true });
        stored["selected_model"] = json!("kept");
        stored["sound_theme"] = json!("theremin");
        let salvaged = salvage(&stored)?;
        assert_eq!(salvaged.selected_model, "kept");
        assert_eq!(salvaged.sound_theme, get_default_settings().sound_theme);
        Ok(())
    }
    #[test]
    fn salvage_of_non_object_store_falls_back_to_defaults() -> TestResult {
        for stored in [json!("corrupt"), json!(null), json!([1, 2, 3])] {
            let salvaged = salvage(&stored)?;
            assert_eq!(serde_json::to_value(&salvaged)?, default_settings_json()?);
        }
        Ok(())
    }
}
