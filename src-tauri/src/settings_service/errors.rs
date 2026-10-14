//! Stable English errors with allowlisted details and no candidate-value diagnostics.

pub use super::error_reason::{ErrorItem, ErrorReason, MAX_ERROR_ITEMS};
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
    /// `custom_filler_words` belongs to the vocabulary domain in the 008 inventory, but this
    /// build's vocabulary commands only manage `custom_words`.
    FillerWordsUnavailable,
    PromptsDomain,
    UnknownPrompt,
    UpdaterDisabled,
    GpuDeviceRequired,
    /// Domain input (file or stdin) is not UTF-8 text; `details.offset` names the first bad byte.
    InvalidEncoding,
    /// Domain input does not follow the domain's exchange format.
    InvalidInput,
    /// The setting or operation needs a platform capability this build does not have.
    UnsupportedPlatform,
    /// A dictation session (recording, processing, a held or remembered shortcut) is running;
    /// the shortcut registrations cannot change until it ends.
    ShortcutSessionBusy,
}
impl ValidationReason {
    fn message(self) -> &'static str {
        match self {
            Self::CanonicalType => "Expected the descriptor's canonical scalar type.",
            Self::CanonicalValue => "Expected a canonical allowed value.",
            Self::OutOfRange => "Value is outside the allowed range.",
            Self::UiOnly => "This setting is changed in the settings UI.",
            Self::ShortcutsDomain => "Use handy settings shortcuts to change shortcuts.",
            Self::VocabularyDomain => "Use handy settings vocabulary to change this list.",
            Self::FillerWordsUnavailable => {
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
            Self::UnsupportedPlatform => "This setting is not supported on this platform.",
            Self::ShortcutSessionBusy => "A dictation is in progress; retry when it ends.",
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
    /// Closed machine reason of the refusal (absent unless a constructor sets it).
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<ErrorReason>,
    /// Refused positions of a domain input, at most `MAX_ERROR_ITEMS`, sorted by index.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<ErrorItem>,
    /// Shortcut details of a refused shortcut change (all absent otherwise).
    #[serde(flatten)]
    shortcut: ShortcutDetails,
}

/// Allowlisted details of a refused shortcut change (see `ErrorDetails`): the action concerned,
/// the owner of a conflicting stored shortcut and the kind of conflict, or the per-action limit.
/// Only known action ids and stored shortcut texts enter them.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ShortcutDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owning_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owning_shortcut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// No input value, internal serde error or arbitrary key enters the public error.
// @spec FR-006: Stable redacted errors — .specs/features/008-settings-cli/spec.md#fr-006
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsError {
    code: SettingsErrorCode,
    message: &'static str,
    /// Boxed so a `Result<_, SettingsError>` stays small on every settings path.
    details: Box<ErrorDetails>,
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
            details: Box::default(),
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
    // @spec FR-006: Unsupported-platform refusal — .specs/features/006-destination-application/spec.md#fr-006
    /// Setting `key` (or the operation, without a key) needs a platform capability this build
    /// lacks: `validation_failed` (exit 2) with `details.reason = "unsupported_platform"`, decided
    /// by the pure validator so that offline and routed calls refuse identically.
    pub fn unsupported_platform(key: Option<&str>) -> Self {
        let error = match key {
            Some(key) => Self::validation(key, ValidationReason::UnsupportedPlatform),
            None => {
                let mut error = Self::new(SettingsErrorCode::ValidationFailed);
                error.reason = Some(ValidationReason::UnsupportedPlatform);
                error
            }
        };
        error.with_reason(ErrorReason::UnsupportedPlatform)
    }
    /// Attach the closed machine reason `details.reason`.
    pub fn with_reason(mut self, reason: ErrorReason) -> Self {
        self.details.reason = Some(reason);
        self
    }
    /// Attach refused input positions as `details.items`: sorted by index, the first
    /// `MAX_ERROR_ITEMS` kept (lossy on purpose: the envelope stays bounded).
    pub fn with_items(mut self, items: impl IntoIterator<Item = ErrorItem>) -> Self {
        let mut items = items.into_iter().collect::<Vec<_>>();
        items.sort_by_key(ErrorItem::index);
        items.truncate(MAX_ERROR_ITEMS);
        self.details.items = items;
        self
    }
    /// Closed machine reason of `details.reason`, if any.
    pub fn detail_reason(&self) -> Option<ErrorReason> {
        self.details.reason
    }
    /// Refused input positions of `details.items`.
    pub fn items(&self) -> &[ErrorItem] {
        &self.details.items
    }
    /// A dictation session owns the shortcuts: the change of `key` was refused before anything
    /// was registered or written.
    pub fn session_busy(key: &str) -> Self {
        let mut error = Self::live_state_refused(key);
        error.reason = Some(ValidationReason::ShortcutSessionBusy);
        error.with_reason(ErrorReason::ShortcutSessionBusy)
    }
    /// A live-only change was asked without a running instance: nothing was written.
    pub fn instance_required() -> Self {
        Self::shortcut(
            SettingsErrorCode::Unavailable,
            Some(ErrorReason::InstanceRequired),
            ShortcutDetails::default(),
        )
    }
    /// A refused shortcut change: `code` with the machine `reason` (if any) and the allowlisted
    /// shortcut `details`.
    pub fn shortcut(
        code: SettingsErrorCode,
        reason: Option<ErrorReason>,
        details: ShortcutDetails,
    ) -> Self {
        let mut error = Self::new(code);
        error.details.reason = reason;
        error.details.shortcut = details;
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
