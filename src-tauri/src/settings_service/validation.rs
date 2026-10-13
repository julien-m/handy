//! Shared scalar validation ports; existing Tauri writers are wired in Step 4.

use super::errors::{SettingsError, ValidationReason};
use serde::Serialize;

/// Typed scalar patch; collection and secret changes use their owning UI/domain paths.
#[derive(Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ScalarValue {
    Boolean(bool),
    Integer(u64),
    Number(f64),
    String(String),
    Null,
}

/// A validated patch carries a recognized canonical key and a typed value.
// @spec FR-003: Shared validation foundation — .specs/features/008-settings-cli/spec.md#fr-003
#[derive(Clone, PartialEq)]
pub struct SettingsMutation {
    pub(crate) key: &'static str,
    pub(crate) value: ScalarValue,
}
impl SettingsMutation {
    /// Return the canonical key selected by validation.
    pub fn key(&self) -> &'static str {
        self.key
    }
    /// Borrow the typed accepted value; never used in error formatting.
    pub fn value(&self) -> &ScalarValue {
        &self.value
    }
}
// Values may contain private text; Debug intentionally exposes only the recognized key.
impl std::fmt::Debug for SettingsMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsMutation")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

/// Pure context supplies prompt membership and build policy without native effects.
pub struct ValidationContext<'a> {
    pub prompt_ids: &'a [&'a str],
    pub updates_allowed: bool,
}

/// CLI textual values enter the same canonical validator as the UI port.
pub fn validate_cli(
    key: &str,
    candidate: &str,
    context: &ValidationContext<'_>,
) -> Result<SettingsMutation, SettingsError> {
    let entry = editable_descriptor(key)?;
    let value = parse_cli(entry, candidate)?;
    validate_value(entry, value, context)
}

/// UI controls normalize legacy device sentinels before the same typed validation.
pub fn validate_ui(
    key: &str,
    candidate: serde_json::Value,
    context: &ValidationContext<'_>,
) -> Result<SettingsMutation, SettingsError> {
    let entry = editable_descriptor(key)?;
    // Existing microphone/output controls emit default/Default; remove this normalization
    // only after those producers use nullable device selections consistently.
    let value = if matches!(
        entry.key,
        "selected_microphone" | "clamshell_microphone" | "selected_output_device"
    ) && matches!(candidate.as_str(), Some("default" | "Default"))
    {
        serde_json::Value::Null
    } else {
        candidate
    };
    validate_value(entry, value, context)
}

fn editable_descriptor(
    key: &str,
) -> Result<&'static super::descriptor::SettingDescriptor, SettingsError> {
    use super::descriptor::SettingType;
    let entry = super::descriptor::descriptor(key)
        .ok_or_else(|| SettingsError::new(super::errors::SettingsErrorCode::NotFound))?;
    if entry.read_only || entry.secret {
        return Err(SettingsError::validation(
            entry.key,
            ValidationReason::UiOnly,
        ));
    }
    if matches!(entry.value_type, SettingType::Array | SettingType::Object) {
        let reason = match entry.owner_domain {
            "shortcuts" => ValidationReason::ShortcutsDomain,
            "vocabulary" => ValidationReason::VocabularyDomain,
            "prompts" => ValidationReason::PromptsDomain,
            _ => ValidationReason::UiOnly,
        };
        return Err(SettingsError::validation(entry.key, reason));
    }
    Ok(entry)
}

fn parse_cli(
    entry: &super::descriptor::SettingDescriptor,
    candidate: &str,
) -> Result<serde_json::Value, SettingsError> {
    use super::descriptor::SettingType;
    use serde_json::Value;
    if entry.nullable && candidate == "null" {
        return Ok(Value::Null);
    }
    let invalid = || SettingsError::validation(entry.key, ValidationReason::CanonicalType);
    match entry.value_type {
        SettingType::Boolean => match candidate {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(invalid()),
        },
        SettingType::Integer => candidate
            .parse::<u64>()
            .map(|number| Value::Number(number.into()))
            .map_err(|_| invalid()),
        SettingType::Number => candidate
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(invalid),
        SettingType::String => Ok(Value::String(candidate.to_owned())),
        _ => Err(invalid()),
    }
}

/// Revalidate one canonical value against fresh membership/build context without I/O.
pub(super) fn validate_value(
    entry: &'static super::descriptor::SettingDescriptor,
    candidate: serde_json::Value,
    context: &ValidationContext<'_>,
) -> Result<SettingsMutation, SettingsError> {
    let value = validate_type(entry, &candidate)?;
    if !entry.allowed_values.is_empty()
        && !candidate
            .as_str()
            .is_some_and(|text| entry.allowed_values.contains(&text))
    {
        return Err(SettingsError::validation(
            entry.key,
            ValidationReason::CanonicalValue,
        ));
    }
    if let Some(range) = entry.range {
        if let Some(number) = candidate.as_f64() {
            if !number.is_finite() || number < range.min || number > range.max {
                return Err(SettingsError::validation(
                    entry.key,
                    ValidationReason::OutOfRange,
                ));
            }
        }
    }
    validate_context(entry, &candidate, context)?;
    Ok(SettingsMutation {
        key: entry.key,
        value,
    })
}

fn validate_context(
    entry: &super::descriptor::SettingDescriptor,
    candidate: &serde_json::Value,
    context: &ValidationContext<'_>,
) -> Result<(), SettingsError> {
    if entry.key == "post_process_selected_prompt_id"
        && candidate
            .as_str()
            .is_some_and(|id| !context.prompt_ids.contains(&id))
    {
        return Err(SettingsError::validation(
            entry.key,
            ValidationReason::UnknownPrompt,
        ));
    }
    if entry.key == "update_checks_enabled" && !context.updates_allowed {
        return Err(SettingsError::validation(
            entry.key,
            ValidationReason::UpdaterDisabled,
        ));
    }
    Ok(())
}

fn validate_type(
    entry: &super::descriptor::SettingDescriptor,
    candidate: &serde_json::Value,
) -> Result<ScalarValue, SettingsError> {
    use super::descriptor::SettingType;
    if candidate.is_null() && entry.nullable {
        return Ok(ScalarValue::Null);
    }
    let value = match entry.value_type {
        SettingType::Boolean => candidate.as_bool().map(ScalarValue::Boolean),
        SettingType::Integer => candidate.as_u64().map(ScalarValue::Integer),
        SettingType::Number => candidate
            .as_f64()
            .filter(|number| number.is_finite())
            .map(ScalarValue::Number),
        SettingType::String => candidate
            .as_str()
            .map(|text| ScalarValue::String(text.to_owned())),
        _ => None,
    };
    value.ok_or_else(|| SettingsError::validation(entry.key, ValidationReason::CanonicalType))
}

/// Cross-field rules checked on the complete candidate under the transaction lock, so every
/// surface (UI command, running instance, CLI) gets the same decision.
///
/// `before` is the locked current settings; the reported field is the dependent key the change
/// touched.
///
/// # Errors
/// `validation_failed` when `after` violates a dependency between fields.
pub(crate) fn validate_dependencies(
    before: &crate::settings::AppSettings,
    after: &crate::settings::AppSettings,
) -> Result<(), SettingsError> {
    use crate::settings::TranscribeAcceleratorSetting;
    // Business rule: the GPU accelerator targets one exact device. Without one, read normalization
    // would silently turn it back into Auto, and AC-003 forbids replacing a value by a default.
    if after.transcribe_accelerator == TranscribeAcceleratorSetting::Gpu
        && after.transcribe_gpu_device.is_none()
    {
        let field = if before.transcribe_accelerator == after.transcribe_accelerator {
            "transcribe_gpu_device"
        } else {
            "transcribe_accelerator"
        };
        return Err(SettingsError::validation(
            field,
            ValidationReason::GpuDeviceRequired,
        ));
    }
    Ok(())
}
