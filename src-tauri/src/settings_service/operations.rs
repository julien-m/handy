//! Generic settings operations (get, keys, set) shared by the offline CLI and the running
//! instance; each produces the one envelope that every surface renders.
use super::{
    descriptor,
    errors::{SettingsError, SettingsErrorCode},
    protocol::{Revision, SettingsEnvelope, SettingsOperation},
    store::SettingsRead,
    validation::{validate_cli, SettingsMutation, ValidationContext},
};
use serde_json::{json, Value};

/// Settings access of one writer: the offline repository or the running instance.
pub trait SettingsBackend {
    /// Fresh settings and their revision; `warning` when the store was recovered.
    ///
    /// # Errors
    /// Any refusal of the underlying read (`unavailable`, `persistence_failed`).
    fn read(&self) -> Result<SettingsRead, SettingsError>;
    /// Commit one validated scalar on the latest settings, where it is revalidated under the lock.
    ///
    /// # Errors
    /// Every refusal of the shared commit path (conflict, validation, ownership, persistence).
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError>;
}

/// Result of one operation: its envelope, and whether the settings read were recovered.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    envelope: Value,
    warning: bool,
}
impl Reply {
    /// Successful envelope carrying the revision the data was read or committed at.
    pub fn success(revision: Revision, data: Value, warning: bool) -> Self {
        Self::from_envelope(SettingsEnvelope::success(revision, data), warning)
    }
    /// Failure envelope with the error's fixed message and allowlisted details.
    pub fn failure(error: SettingsError) -> Self {
        Self::from_envelope(SettingsEnvelope::failure(error), false)
    }
    fn from_envelope(envelope: SettingsEnvelope<Value>, warning: bool) -> Self {
        let envelope = serde_json::to_value(envelope).unwrap_or_else(|_| {
            // Serializing these closed DTOs cannot fail; keep the contract shape if it ever did.
            json!({"ok": false, "error": {"code": "internal_error",
                "message": "Settings operation failed.", "details": {}}})
        });
        Self { envelope, warning }
    }
    /// The JSON envelope printed with `--json`.
    pub fn envelope(&self) -> &Value {
        &self.envelope
    }
    /// True when the settings read or written were salvaged from a damaged store.
    pub fn warning(&self) -> bool {
        self.warning
    }
    /// Stable process exit status of this reply.
    pub fn exit_code(&self) -> i32 {
        match self.envelope.get("ok").and_then(Value::as_bool) {
            Some(true) => 0,
            _ => error_code(&self.envelope).map_or(1, SettingsErrorCode::exit_code),
        }
    }
}
fn error_code(envelope: &Value) -> Option<SettingsErrorCode> {
    serde_json::from_value(envelope.get("error")?.get("code")?.clone()).ok()
}

/// Run one operation against `backend`; every refusal becomes a failure envelope.
// @spec FR-002: Generic get, keys and set over public settings — .specs/features/008-settings-cli/spec.md#fr-002
pub fn execute(operation: &SettingsOperation, backend: &dyn SettingsBackend) -> Reply {
    let result = match operation {
        SettingsOperation::Get { key } => get(backend, key.as_deref()),
        SettingsOperation::Keys => keys(backend),
        SettingsOperation::Set {
            key,
            value,
            expected_revision,
        } => set(backend, key, value, *expected_revision),
        // No domain is registered in this build; domains are added by their owning features.
        SettingsOperation::Domain { .. } => Err(SettingsError::new(SettingsErrorCode::UsageError)),
    };
    match result {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    }
}
type Outcome = Result<(Revision, Value, bool), SettingsError>;
fn public(read: &SettingsRead) -> Result<Value, SettingsError> {
    serde_json::to_value(read.snapshot.public_settings())
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))
}
/// Unknown keys are refused before any read; the key text never enters the error.
fn known_key(key: &str) -> Result<&'static str, SettingsError> {
    descriptor::descriptor(key)
        .map(|entry| entry.key)
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))
}
fn get(backend: &dyn SettingsBackend, key: Option<&str>) -> Outcome {
    let key = key.map(known_key).transpose()?;
    let read = backend.read()?;
    let settings = public(&read)?;
    let data = match key {
        Some(key) => json!({ key: settings[key] }),
        None => settings,
    };
    Ok((read.snapshot.revision, data, read.warning))
}
fn keys(backend: &dyn SettingsBackend) -> Outcome {
    let read = backend.read()?;
    let keys = serde_json::to_value(descriptor::descriptors())
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    Ok((
        read.snapshot.revision,
        json!({ "keys": keys }),
        read.warning,
    ))
}
/// Validation context comes from a fresh read; the commit revalidates under the lock anyway.
fn set(
    backend: &dyn SettingsBackend,
    key: &str,
    value: &str,
    expected: Option<Revision>,
) -> Outcome {
    known_key(key)?;
    let read = backend.read()?;
    let prompt_ids = read
        .snapshot
        .settings
        .post_process_prompts
        .iter()
        .map(|prompt| prompt.id.as_str())
        .collect::<Vec<_>>();
    let context = ValidationContext {
        prompt_ids: &prompt_ids,
        updates_allowed: !crate::settings::update_checks_forced_disabled(),
    };
    let mutation = validate_cli(key, value, &context)?;
    let key = mutation.key();
    let committed = backend.commit(mutation, expected)?;
    let settings = public(&committed)?;
    let data = json!({ key: settings[key] });
    Ok((committed.snapshot.revision, data, committed.warning))
}
