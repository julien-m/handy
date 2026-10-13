//! Typed settings error returned by Tauri commands, so the UI never falls back silently.
use super::errors::{Outcome, SettingsError, SettingsErrorCode, ValidationReason};
use serde::Serialize;

/// Error of a settings UI command: the stable code and English message shared with the CLI,
/// plus the field and typed reason the UI maps to a translation (never free-text parsing).
#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct SettingsCommandError {
    /// Stable category, identical to the CLI error code.
    pub code: SettingsErrorCode,
    /// Stable English message, identical to the CLI message.
    pub message: String,
    /// Recognized setting key the refusal concerns, if any.
    pub field: Option<String>,
    /// Typed validation reason, if any.
    pub reason: Option<ValidationReason>,
    /// `committed` when the value is stored but a live effect failed; `unknown` when unconfirmed.
    pub outcome: Option<Outcome>,
    /// Revision the outcome or conflict refers to.
    pub current_revision: Option<u64>,
}
impl From<SettingsError> for SettingsCommandError {
    fn from(error: SettingsError) -> Self {
        Self {
            code: error.code(),
            message: error.message().to_owned(),
            field: error.field().map(str::to_owned),
            reason: error.reason(),
            outcome: error.outcome(),
            current_revision: error.current_revision().map(|revision| revision.0),
        }
    }
}
impl SettingsCommandError {
    /// A native preparation was refused by live state (for example a recording in progress);
    /// nothing was committed. The detail is logged by the caller, never returned.
    pub fn unavailable(key: &str) -> Self {
        let mut error = Self::from(SettingsError::new(SettingsErrorCode::Unavailable));
        error.field = super::descriptor::descriptor(key).map(|entry| entry.key.to_owned());
        error
    }
    /// A UI-owned edit refused its input for `key` with a fixed reason.
    pub fn invalid(key: &str, reason: ValidationReason) -> Self {
        Self::from(SettingsError::validation(key, reason))
    }
}
impl std::fmt::Display for SettingsCommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for SettingsCommandError {}
