//! Snapshot and wire DTO contracts; transport/rendering are wired in later steps.

use super::errors::{SettingsError, SettingsErrorCode};
use super::public::PublicSettings;
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};

/// Persisted monotonic compare-and-set revision, separate from schema version.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(pub u64);

/// Internal typed store snapshot; it cannot serialize as a public response.
#[derive(Clone)]
pub struct SettingsSnapshot {
    pub(crate) settings: AppSettings,
    pub revision: Revision,
}
impl SettingsSnapshot {
    /// Load an isolated stored object without printing its raw contents or parser error.
    pub fn from_stored_value(
        value: serde_json::Value,
        revision: Revision,
    ) -> Result<Self, SettingsError> {
        // Unknown stored fields are intentionally ignored by this typed read projection.
        // This is not a writer; Step 3 preserves the original raw document on persistence.
        let settings = serde_json::from_value(value)
            .map_err(|_| SettingsError::new(SettingsErrorCode::PersistenceFailed))?;
        Ok(Self { settings, revision })
    }
    /// Project only the explicit public allowlist.
    pub fn public_settings(&self) -> PublicSettings {
        PublicSettings::from_settings(&self.settings)
    }
    /// Inventory actual default serialized field names only, never their values.
    pub fn default_field_inventory() -> Result<Vec<String>, SettingsError> {
        let object = serde_json::to_value(AppSettings::default())
            .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
        object
            .as_object()
            .map(|fields| fields.keys().cloned().collect())
            .ok_or_else(|| SettingsError::new(SettingsErrorCode::InternalError))
    }
}

/// Serializable envelope shape; formatting stdout/stderr belongs to Step 5.
// @spec FR-006: Exact success/error envelope DTO — .specs/features/008-settings-cli/spec.md#fr-006
#[derive(Serialize)]
#[serde(untagged)]
enum EnvelopeBody<T: Serialize> {
    Success {
        ok: bool,
        revision: Revision,
        data: T,
    },
    Failure {
        ok: bool,
        error: SettingsError,
    },
}
/// Closed envelope construction guarantees the status bit matches its body.
#[derive(Serialize)]
#[serde(transparent)]
pub struct SettingsEnvelope<T: Serialize>(EnvelopeBody<T>);

impl<T: Serialize> SettingsEnvelope<T> {
    /// Wrap typed public response data with its durable revision.
    pub fn success(revision: Revision, data: T) -> Self {
        Self(EnvelopeBody::Success {
            ok: true,
            revision,
            data,
        })
    }
    /// Wrap a fixed redacted failure with object details.
    pub fn failure(error: SettingsError) -> Self {
        Self(EnvelopeBody::Failure { ok: false, error })
    }
}

pub use super::request::{OperationRequest, ProtocolVersion, SettingsOperation};

/// Generate the checked contract from actual descriptor types and closed wire DTO shapes.
// @spec FR-006: Deterministic JSON wire schema — .specs/features/008-settings-cli/spec.md#fr-006
pub fn contract_schema() -> Result<serde_json::Value, SettingsError> {
    use serde_json::json;
    let properties = super::descriptor::descriptors()
        .iter()
        .map(|entry| (entry.key.to_owned(), public_field_schema(entry)))
        .collect::<serde_json::Map<_, _>>();
    let keys = super::descriptor::descriptors()
        .iter()
        .map(|entry| entry.key)
        .collect::<Vec<_>>();
    let codes = [
        SettingsErrorCode::UsageError,
        SettingsErrorCode::ValidationFailed,
        SettingsErrorCode::ShortcutConflict,
        SettingsErrorCode::LimitExceeded,
        SettingsErrorCode::RevisionConflict,
        SettingsErrorCode::NotFound,
        SettingsErrorCode::Unavailable,
        SettingsErrorCode::PersistenceFailed,
        SettingsErrorCode::InternalError,
    ];
    let code_values = serde_json::to_value(codes)
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    let metadata = serde_json::to_value(super::descriptor::descriptors())
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    Ok(
        json!({"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Handy settings CLI envelope","oneOf":[{"$ref":"#/$defs/success"},{"$ref":"#/$defs/failure"}],"$defs":{
      "success":{"type":"object","additionalProperties":false,"required":["ok","revision","data"],"properties":{"ok":{"const":true},"revision":{"$ref":"#/$defs/revision"},"data":{"type":"object"}}},
      "failure":{"type":"object","additionalProperties":false,"required":["ok","error"],"properties":{"ok":{"const":false},"error":{"$ref":"#/$defs/error"}}},
      "revision":{"type":"integer","minimum":0,"maximum":u64::MAX},
      "error":{"type":"object","additionalProperties":false,"required":["code","message","details"],"properties":{"code":{"enum":code_values},"message":{"type":"string"},"details":{"$ref":"#/$defs/error_details"}}},
      "error_details":{"type":"object","additionalProperties":false,"properties":{"current_revision":{"$ref":"#/$defs/revision"},"outcome":{"enum":["unknown","committed"]},"field":{"enum":keys},"offset":{"type":"integer","minimum":0},"issues":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["field","reason"],"properties":{"field":{"enum":keys},"reason":{"type":"string"}}}}}},
      "public_settings":{"type":"object","additionalProperties":false,"required":keys,"properties":properties},
      "provider":public_provider_schema(),"binding":public_binding_schema(),"prompt":public_prompt_schema(),"replacement_rule":public_replacement_rule_schema(),
      "operation_request":operation_request_schema()
    },"x-setting-descriptors":metadata}),
    )
}

fn public_field_schema(entry: &super::descriptor::SettingDescriptor) -> serde_json::Value {
    use super::descriptor::SettingType;
    use serde_json::json;
    let wire_type = match entry.value_type {
        SettingType::Boolean => "boolean",
        SettingType::Integer => "integer",
        SettingType::Number => "number",
        SettingType::String => "string",
        SettingType::Array => "array",
        SettingType::Object => "object",
    };
    let mut schema = if entry.nullable {
        json!({"type":[wire_type,"null"]})
    } else {
        json!({"type":wire_type})
    };
    apply_intrinsic_constraints(entry, &mut schema);
    // Public reads use intrinsic wire bounds, never UI mutation policy bounds.
    match entry.key {
        "post_process_api_keys" => {
            schema["additionalProperties"] = json!({"type":"object","additionalProperties":false,"required":["present"],"properties":{"present":{"type":"boolean"}}});
        }
        "post_process_providers" => {
            schema["items"] = json!({"$ref":"#/$defs/provider"});
        }
        "bindings" => {
            schema["additionalProperties"] = json!({"$ref":"#/$defs/binding"});
        }
        "post_process_prompts" => {
            schema["items"] = json!({"$ref":"#/$defs/prompt"});
        }
        "word_replacements" => {
            schema["items"] = json!({"$ref":"#/$defs/replacement_rule"});
        }
        "post_process_models" => {
            schema["additionalProperties"] = json!({"type":"string"});
        }
        _ if entry.value_type == SettingType::Array => {
            schema["items"] = json!({"type":"string"});
        }
        _ => {}
    }
    schema
}

fn apply_intrinsic_constraints(
    entry: &super::descriptor::SettingDescriptor,
    schema: &mut serde_json::Value,
) {
    use super::descriptor::SettingType;
    use serde_json::json;
    if !entry.allowed_values.is_empty() {
        let mut values: Vec<_> = entry
            .allowed_values
            .iter()
            .map(|value| json!(value))
            .collect();
        if entry.nullable {
            values.push(serde_json::Value::Null);
        }
        schema["enum"] = json!(values);
    }
    if entry.value_type == SettingType::Integer {
        schema["minimum"] = json!(0);
        schema["maximum"] = json!(match entry.key {
            "settings_schema_version" => u32::MAX as u64,
            "selected_channel" => u16::MAX as u64,
            "history_limit" => usize::MAX as u64,
            _ => u64::MAX,
        });
    }
}

fn public_provider_schema() -> serde_json::Value {
    serde_json::json!({"type":"object","additionalProperties":false,"required":["id","label","base_url","allow_base_url_edit","models_endpoint","supports_structured_output"],"properties":{"id":{"type":"string"},"label":{"type":"string"},"base_url":{"type":"string"},"allow_base_url_edit":{"type":"boolean"},"models_endpoint":{"type":["string","null"]},"supports_structured_output":{"type":"boolean"}}})
}
fn public_binding_schema() -> serde_json::Value {
    serde_json::json!({"type":"object","additionalProperties":false,"required":["id","name","description","default_binding","current_binding"],"properties":{"id":{"type":"string"},"name":{"type":"string"},"description":{"type":"string"},"default_binding":{"type":"string"},"current_binding":{"type":"string"}}})
}
fn public_prompt_schema() -> serde_json::Value {
    serde_json::json!({"type":"object","additionalProperties":false,"required":["id","name","prompt"],"properties":{"id":{"type":"string"},"name":{"type":"string"},"prompt":{"type":"string"}}})
}

fn public_replacement_rule_schema() -> serde_json::Value {
    serde_json::json!({"type":"object","additionalProperties":false,"required":["id","recognized","replacement"],"properties":{"id":{"type":"string"},"recognized":{"type":"string"},"replacement":{"type":"string"}}})
}

fn operation_request_schema() -> serde_json::Value {
    use serde_json::json;
    let branches = ["get", "keys", "set", "domain"].map(|operation| {
        let mut properties = json!({"protocol_version":{"const":1},"request_id":{"type":"string"},"store_identity":{"type":"string"},"operation":{"const":operation}});
        let mut required = vec!["protocol_version", "request_id", "store_identity", "operation"];
        match operation {
            "get" => properties["key"] = json!({"type":["string","null"]}),
            "set" => {
                properties["key"] = json!({"type":"string"});
                properties["value"] = json!({"type":"string"});
                required.extend(["key", "value"]);
            }
            "domain" => {
                properties["domain"] = json!({"type":"string"});
                properties["action"] = json!({"type":"string"});
                properties["arguments"] = json!({"type":"array","items":{"type":"string"}});
                properties["dry_run"] = json!({"type":"boolean"});
                properties["input"] = json!({"type":["string","null"]});
                required.extend(["domain", "action", "arguments", "dry_run"]);
            }
            _ => {}
        }
        if matches!(operation, "set" | "domain") {
            properties["expected_revision"] = json!({"anyOf":[{"$ref":"#/$defs/revision"},{"type":"null"}]});
        }
        json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
    });
    json!({"oneOf":branches})
}
