//! Limits, validation issues and refusals of the prompt library (005), shared by the settings
//! UI commands and the `handy settings prompts` domain so both surfaces refuse identically.
use crate::settings_service::errors::{
    ErrorReason, SettingsError, SettingsErrorCode, ShortcutDetails, ValidationReason,
};
use serde::Serialize;
use specta::Type;

/// Most prompts a library holds (AC-001); a legacy library above it is kept, never trimmed.
pub const MAX_PROMPTS: usize = 50;
/// Longest prompt name, in Unicode scalar values after trimming (AC-001).
pub const MAX_NAME_CHARS: usize = 60;
/// Longest prompt text, in Unicode scalar values (AC-001).
pub const MAX_TEXT_CHARS: usize = 8000;
/// Most shortcuts one prompt may have (AC-006).
pub const MAX_PROMPT_SHORTCUTS: usize = 2;
/// Settings key that owns the library; every prompt refusal names it.
pub const PROMPTS_KEY: &str = "post_process_prompts";

/// Why a prompt is not valid (and cannot be used until edited). Legacy prompts that break a rule
/// stay in the library with their issues listed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PromptIssue {
    /// The trimmed name is empty.
    NameEmpty,
    /// The trimmed name is longer than `MAX_NAME_CHARS`.
    NameTooLong,
    /// Another prompt has the same trimmed name, ignoring case.
    NameDuplicate,
    /// The text is empty or only whitespace (precision of AC-001, decision 2026-10-14).
    TextEmpty,
    /// The text is longer than `MAX_TEXT_CHARS`.
    TextTooLong,
    /// The override names a provider but no model (or the reverse).
    OverrideIncomplete,
    /// The override names a provider this build does not have.
    UnknownProvider,
}

/// A refused library change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptError {
    /// The prompt would be invalid for this reason.
    Invalid(PromptIssue),
    /// No prompt has this id.
    NotFound,
    /// The library already holds `MAX_PROMPTS` prompts.
    LimitExceeded,
    /// The prompt exists but is invalid, so it cannot be the default.
    NotUsable,
    /// The override needs a platform capability this build lacks (Apple Intelligence).
    UnsupportedPlatform,
}

impl PromptIssue {
    /// Typed reason of the settings UI (translated) for this issue.
    pub fn validation_reason(self) -> ValidationReason {
        match self {
            Self::NameEmpty => ValidationReason::PromptNameEmpty,
            Self::NameTooLong => ValidationReason::PromptNameTooLong,
            Self::NameDuplicate => ValidationReason::PromptNameDuplicate,
            Self::TextEmpty => ValidationReason::PromptTextEmpty,
            Self::TextTooLong => ValidationReason::PromptTextTooLong,
            Self::OverrideIncomplete => ValidationReason::PromptOverrideIncomplete,
            Self::UnknownProvider => ValidationReason::PromptUnknownProvider,
        }
    }
    /// Closed `details.reason` of the CLI envelope for this issue.
    pub fn error_reason(self) -> ErrorReason {
        match self {
            Self::NameEmpty => ErrorReason::PromptNameEmpty,
            Self::NameTooLong => ErrorReason::PromptNameTooLong,
            Self::NameDuplicate => ErrorReason::PromptNameDuplicate,
            Self::TextEmpty => ErrorReason::PromptTextEmpty,
            Self::TextTooLong => ErrorReason::PromptTextTooLong,
            Self::OverrideIncomplete => ErrorReason::OverrideIncomplete,
            Self::UnknownProvider => ErrorReason::UnknownProvider,
        }
    }
}

// @spec FR-006: Shared prompt mutation validation — .specs/features/005-custom-ai-prompts/spec.md#fr-006
impl From<PromptError> for SettingsError {
    /// The 008 envelope of a refusal: `validation_failed` (exit 2) with a fixed reason,
    /// `not_found` (4), `limit_exceeded` (2) or `unsupported_platform`; never the candidate.
    fn from(error: PromptError) -> Self {
        match error {
            PromptError::Invalid(issue) => {
                SettingsError::validation(PROMPTS_KEY, issue.validation_reason())
                    .with_reason(issue.error_reason())
            }
            PromptError::NotFound => SettingsError::new(SettingsErrorCode::NotFound),
            PromptError::LimitExceeded => {
                let details = ShortcutDetails {
                    limit: Some(MAX_PROMPTS as u32),
                    ..ShortcutDetails::default()
                };
                SettingsError::shortcut(SettingsErrorCode::LimitExceeded, None, details)
            }
            PromptError::NotUsable => {
                SettingsError::validation(PROMPTS_KEY, ValidationReason::PromptNotUsable)
                    .with_reason(ErrorReason::PromptInvalid)
            }
            PromptError::UnsupportedPlatform => SettingsError::unsupported_platform(None),
        }
    }
}
