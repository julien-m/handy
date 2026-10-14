//! Closed reasons of `details.reason` and `details.items[].code` of a settings error, and the
//! refused positions of a domain input. Shared by the settings UI, the offline CLI and the
//! running instance; each reason has a fixed English message and never carries a candidate.
use serde::Serialize;

/// Most refused positions one error reports; the first ones by index are kept. 100 bounds the
/// envelope well under one IPC frame while naming every line of a typical import.
pub const MAX_ERROR_ITEMS: usize = 100;

/// Closed snake_case reasons of `details.reason` and `details.items[].code`; later features add
/// their own variants. Each has a fixed English message and never carries a candidate value.
// @spec FR-006: Shared error details for the settings CLI — .specs/features/006-destination-application/spec.md#fr-006
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorReason {
    /// The operation or setting needs a platform capability this build does not have.
    UnsupportedPlatform,
    /// A dictation session owns the shortcuts until it ends (004).
    ShortcutSessionBusy,
    /// A live-only change was asked without a running instance (004).
    InstanceRequired,
    /// The last shortcut of a dictation action cannot be removed (004).
    LastShortcut,
    /// The action keeps a single shortcut (004).
    AlternativesUnsupported,
    /// The system refused to register the shortcut (004).
    RegistrationRefused,
    /// The shortcut text is not valid for the active keyboard implementation (004).
    InvalidShortcut,
    /// The prompt name is empty after trimming (005).
    PromptNameEmpty,
    /// The prompt name is longer than 60 characters (005).
    PromptNameTooLong,
    /// Another prompt has the same name, ignoring case (005).
    PromptNameDuplicate,
    /// The prompt text is empty or only whitespace (005).
    PromptTextEmpty,
    /// The prompt text is longer than 8000 characters (005).
    PromptTextTooLong,
    /// A provider override needs both a provider and a model (005).
    OverrideIncomplete,
    /// The override names a provider this build does not have (005).
    UnknownProvider,
    /// The prompt has issues and cannot be used until edited (005).
    PromptInvalid,
}
impl ErrorReason {
    /// Every reason of this build, in wire-schema order.
    pub const ALL: [Self; 15] = [
        Self::UnsupportedPlatform,
        Self::ShortcutSessionBusy,
        Self::InstanceRequired,
        Self::LastShortcut,
        Self::AlternativesUnsupported,
        Self::RegistrationRefused,
        Self::InvalidShortcut,
        Self::PromptNameEmpty,
        Self::PromptNameTooLong,
        Self::PromptNameDuplicate,
        Self::PromptTextEmpty,
        Self::PromptTextTooLong,
        Self::OverrideIncomplete,
        Self::UnknownProvider,
        Self::PromptInvalid,
    ];
    /// Fixed English message of this reason.
    pub fn message(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "Not supported on this platform.",
            Self::ShortcutSessionBusy => "A dictation is in progress.",
            Self::InstanceRequired => "Needs a running Handy.",
            Self::LastShortcut => "The last shortcut of an action cannot be removed.",
            Self::AlternativesUnsupported => "This action keeps a single shortcut.",
            Self::RegistrationRefused => "The system refused the shortcut.",
            Self::InvalidShortcut => "Not a valid shortcut.",
            Self::PromptNameEmpty => "The prompt name is empty.",
            Self::PromptNameTooLong => "The prompt name is longer than 60 characters.",
            Self::PromptNameDuplicate => "Another prompt already has this name.",
            Self::PromptTextEmpty => "The prompt text is empty.",
            Self::PromptTextTooLong => "The prompt text is longer than 8000 characters.",
            Self::OverrideIncomplete => "A provider override needs a provider and a model.",
            Self::UnknownProvider => "The provider does not exist in this build.",
            Self::PromptInvalid => "The prompt is invalid and must be edited first.",
        }
    }
}

/// One refused position of a domain input: zero-based `index`, closed `code` and its fixed
/// message; never the candidate value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorItem {
    index: u64,
    code: ErrorReason,
    message: &'static str,
}
impl ErrorItem {
    /// Refusal of the input at zero-based position `index` for `code`.
    pub fn new(index: u64, code: ErrorReason) -> Self {
        Self {
            index,
            code,
            message: code.message(),
        }
    }
    /// Zero-based position in the domain input.
    pub fn index(&self) -> u64 {
        self.index
    }
    /// Closed reason of this position.
    pub fn code(&self) -> ErrorReason {
        self.code
    }
}
