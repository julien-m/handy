//! Stable English errors with allowlisted details and no candidate-value diagnostics.

use super::protocol::Revision;
use serde::{Deserialize, Serialize};

/// Stable wire categories; their fixed exit meanings never depend on input text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SettingsErrorCode {
    UsageError,
    ValidationFailed,
    ShortcutConflict,
    LimitExceeded,
    RevisionConflict,
    NotFound,
    Unavailable,
    PersistenceFailed,
    InternalError,
}

impl SettingsErrorCode {
    /// Stable exit mapping shared by every renderer.
    pub fn exit_code(self) -> i32 {
        match self {
            Self::UsageError
            | Self::ValidationFailed
            | Self::ShortcutConflict
            | Self::LimitExceeded => 2,
            Self::RevisionConflict => 3,
            Self::NotFound => 4,
            Self::Unavailable => 5,
            Self::PersistenceFailed | Self::InternalError => 1,
        }
    }
}

/// Durable outcome of a failed operation; clients must reread before retrying either one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// A timed-out or unconfirmed operation may or may not have committed.
    Unknown,
    /// The write is durable, but a live effect failed afterwards and is reconciled later.
    Committed,
}

/// A recognized field and a fixed English reason, never the rejected candidate.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FieldIssue {
    field: &'static str,
    reason: &'static str,
}

/// Closed canonical reasons prevent an arbitrary candidate/error string entering details. The
/// snake_case name is the stable key the settings UI maps to a translation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ValidationReason {
    CanonicalType,
    CanonicalValue,
    OutOfRange,
    UiOnly,
    ShortcutsDomain,
    VocabularyDomain,
    PromptsDomain,
    UnknownPrompt,
    UpdaterDisabled,
    GpuDeviceRequired,
    /// Domain input (file or stdin) is not UTF-8 text; `details.offset` names the first bad byte.
    InvalidEncoding,
    /// Domain input does not follow the domain's exchange format.
    InvalidInput,
}
impl ValidationReason {
    fn message(self) -> &'static str {
        match self {
            Self::CanonicalType => "Expected the descriptor's canonical scalar type.",
            Self::CanonicalValue => "Expected a canonical allowed value.",
            Self::OutOfRange => "Value is outside the allowed range.",
            Self::UiOnly => "This setting is changed in the settings UI.",
            Self::ShortcutsDomain => {
                "Use handy settings shortcuts; this domain is not available in this build."
            }
            Self::VocabularyDomain => {
                "Use handy settings vocabulary; this domain is not available in this build."
            }
            Self::PromptsDomain => {
                "Use handy settings prompts; this domain is not available in this build."
            }
            Self::UnknownPrompt => "Selected prompt does not exist.",
            Self::UpdaterDisabled => "Update checks are disabled by system configuration.",
            Self::GpuDeviceRequired => "The GPU accelerator requires a selected GPU device.",
            Self::InvalidEncoding => "Input is not valid UTF-8 text.",
            Self::InvalidInput => "Input does not follow the exchange format.",
        }
    }
}

/// Details always serialize as an object; only fixed typed evidence is admitted.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ErrorDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    current_revision: Option<Revision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<Outcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<&'static str>,
    /// Byte offset of the first invalid byte of a domain input.
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    issues: Vec<FieldIssue>,
}

/// No input value, internal serde error or arbitrary key enters the public error.
// @spec FR-006: Stable redacted errors — .specs/features/008-settings-cli/spec.md#fr-006
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsError {
    code: SettingsErrorCode,
    message: &'static str,
    details: ErrorDetails,
    /// Typed reason for the settings UI; the wire form keeps only its English message in `issues`.
    #[serde(skip)]
    reason: Option<ValidationReason>,
}

impl SettingsError {
    /// Return the machine category without exposing internal error context.
    pub fn code(&self) -> SettingsErrorCode {
        self.code
    }
    /// Return the category's stable process exit status.
    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
    /// Construct only fixed category messages.
    pub fn new(code: SettingsErrorCode) -> Self {
        let message = match code {
            SettingsErrorCode::UsageError => "Invalid settings command usage.",
            SettingsErrorCode::ValidationFailed => "Setting value failed validation.",
            SettingsErrorCode::ShortcutConflict => "Shortcut conflicts with another binding.",
            SettingsErrorCode::LimitExceeded => "Setting limit exceeded.",
            SettingsErrorCode::RevisionConflict => "Settings revision changed.",
            SettingsErrorCode::NotFound => "Unknown setting key.",
            SettingsErrorCode::Unavailable => "Settings operation is unavailable.",
            SettingsErrorCode::PersistenceFailed => "Settings persistence failed.",
            SettingsErrorCode::InternalError => "Settings operation failed.",
        };
        Self {
            code,
            message,
            details: ErrorDetails::default(),
            reason: None,
        }
    }
    /// Attach a fixed reason only to a key present in the descriptor allowlist.
    pub fn validation(key: &str, reason: ValidationReason) -> Self {
        let mut error = Self::new(SettingsErrorCode::ValidationFailed);
        error.reason = Some(reason);
        if let Some(entry) = super::descriptor::descriptor(key) {
            error.details.field = Some(entry.key);
            error.details.issues.push(FieldIssue {
                field: entry.key,
                reason: reason.message(),
            });
        }
        error
    }
    /// Domain input refused for `reason`, without echoing it; `offset` locates invalid UTF-8.
    pub fn invalid_input(reason: ValidationReason, offset: Option<u64>) -> Self {
        let mut error = Self::new(SettingsErrorCode::ValidationFailed);
        error.reason = Some(reason);
        error.details.offset = offset;
        error
    }
    /// Report a stale compare-and-set with the observed revision.
    pub fn revision_conflict(revision: Revision) -> Self {
        let mut error = Self::new(SettingsErrorCode::RevisionConflict);
        error.details.current_revision = Some(revision);
        error
    }
    /// A missing reply never authorizes a fallback or a replay.
    pub fn unknown_outcome() -> Self {
        let mut error = Self::new(SettingsErrorCode::Unavailable);
        error.details.outcome = Some(Outcome::Unknown);
        error
    }
    /// Live state refused the native preparation of `key` (for example a recording in progress);
    /// nothing was committed.
    pub fn live_state_refused(key: &str) -> Self {
        let mut error = Self::new(SettingsErrorCode::Unavailable);
        error.details.field = super::descriptor::descriptor(key).map(|entry| entry.key);
        error
    }
    /// The write at `revision` is durable but the live effect of `key` failed afterwards; the
    /// committed value stays authoritative (never reported as a validation refusal).
    pub fn committed_effect_failed(revision: Revision, key: &str) -> Self {
        let mut error = Self::new(SettingsErrorCode::InternalError);
        error.details.outcome = Some(Outcome::Committed);
        error.details.current_revision = Some(revision);
        error.details.field = super::descriptor::descriptor(key).map(|entry| entry.key);
        error
    }
    /// Fixed English message of this error.
    pub fn message(&self) -> &'static str {
        self.message
    }
    /// Recognized field named by the error, if any.
    pub fn field(&self) -> Option<&'static str> {
        self.details.field
    }
    /// Typed validation reason, if this is a validation refusal.
    pub fn reason(&self) -> Option<ValidationReason> {
        self.reason
    }
    /// Durable outcome reported with an unavailable or internal error.
    pub fn outcome(&self) -> Option<Outcome> {
        self.details.outcome
    }
    /// Current revision reported by a conflict or a committed outcome.
    pub fn current_revision(&self) -> Option<Revision> {
        self.details.current_revision
    }
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}
impl std::error::Error for SettingsError {}
