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
