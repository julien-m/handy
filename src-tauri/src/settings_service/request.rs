//! Closed request DTOs; peer identity, framing and deadlines belong to Step 5.

use super::errors::{SettingsError, SettingsErrorCode};
use super::protocol::Revision;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The only supported wire version; construction cannot produce another version.
#[derive(Clone, Copy, Default)]
pub struct ProtocolVersion;
impl Serialize for ProtocolVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(1)
    }
}
impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match u16::deserialize(deserializer) {
            Ok(1) => Ok(Self),
            _ => Err(serde::de::Error::custom(
                "Unsupported settings protocol version.",
            )),
        }
    }
}

/// Typed current-build operations; unknown and cross-variant fields are rejected.
#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SettingsOperation {
    Get {
        key: Option<String>,
    },
    Keys,
    Set {
        key: String,
        value: String,
        expected_revision: Option<Revision>,
    },
    Domain {
        domain: String,
        action: String,
        input: Option<String>,
        dry_run: bool,
        expected_revision: Option<Revision>,
    },
}

/// Exact versioned wire request; diagnostics never contain candidate contents.
// @spec FR-006: Closed settings request DTO — .specs/features/008-settings-cli/spec.md#fr-006
#[derive(Serialize)]
pub struct OperationRequest {
    pub protocol_version: ProtocolVersion,
    pub request_id: String,
    pub store_identity: String,
    #[serde(flatten)]
    pub operation: SettingsOperation,
}
impl OperationRequest {
    /// Decode a closed request with fixed redacted errors for every invalid shape.
    pub fn from_value(value: serde_json::Value) -> Result<Self, SettingsError> {
        let mut object = value.as_object().cloned().ok_or_else(invalid_request)?;
        let protocol_version = serde_json::from_value(
            object
                .remove("protocol_version")
                .ok_or_else(invalid_request)?,
        )
        .map_err(|_| invalid_request())?;
        let request_id = take_string(&mut object, "request_id")?;
        let store_identity = take_string(&mut object, "store_identity")?;
        let operation = object
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(invalid_request)?;
        let fields: &[&str] = match operation {
            "get" => &["operation", "key"],
            "keys" => &["operation"],
            "set" => &["operation", "key", "value", "expected_revision"],
            "domain" => &[
                "operation",
                "domain",
                "action",
                "input",
                "dry_run",
                "expected_revision",
            ],
            _ => return Err(invalid_request()),
        };
        if object.keys().any(|key| !fields.contains(&key.as_str())) {
            return Err(invalid_request());
        }
        let operation = serde_json::from_value(object.into()).map_err(|_| invalid_request())?;
        Ok(Self {
            protocol_version,
            request_id,
            store_identity,
            operation,
        })
    }
}
impl<'de> Deserialize<'de> for OperationRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)
            .map_err(|_| serde::de::Error::custom("Invalid settings request."))?;
        Self::from_value(value).map_err(|_| serde::de::Error::custom("Invalid settings request."))
    }
}
fn invalid_request() -> SettingsError {
    SettingsError::new(SettingsErrorCode::UsageError)
}
fn take_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, SettingsError> {
    let value = object.remove(key).ok_or_else(invalid_request)?;
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(invalid_request)
}
