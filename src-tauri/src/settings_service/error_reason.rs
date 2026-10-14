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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
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
    /// A vocabulary term is empty once trimmed (003).
    TermEmpty,
    /// A vocabulary term exceeds 100 characters (003).
    TermTooLong,
    /// A vocabulary term contains a line break (003).
    TermLineBreak,
    /// A vocabulary term contains `<`, `>`, `"` or NUL (003).
    TermForbiddenCharacter,
    /// A vocabulary term repeats another term (003).
    TermDuplicate,
    /// The loaded model declares a recognition-context mechanism whose budget is not documented
    /// for this build (003): a durable internal defect, never a retryable state.
    ContextBudgetUnverified,
}
impl ErrorReason {
    /// Every reason of this build, in wire-schema order.
    pub const ALL: [Self; 13] = [
        Self::UnsupportedPlatform,
        Self::ShortcutSessionBusy,
        Self::InstanceRequired,
        Self::LastShortcut,
        Self::AlternativesUnsupported,
        Self::RegistrationRefused,
        Self::InvalidShortcut,
        Self::TermEmpty,
        Self::TermTooLong,
        Self::TermLineBreak,
        Self::TermForbiddenCharacter,
        Self::TermDuplicate,
        Self::ContextBudgetUnverified,
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
            Self::TermEmpty => "A term cannot be empty.",
            Self::TermTooLong => "A term has at most 100 characters.",
            Self::TermLineBreak => "A term cannot contain a line break.",
            Self::TermForbiddenCharacter => "A term cannot contain <, > or \".",
            Self::TermDuplicate => "Each term may appear only once.",
            Self::ContextBudgetUnverified => {
                "The selected model's recognition context budget is not documented in this build."
            }
        }
    }
}

/// One refused position of a domain input: zero-based `index`, closed `code` and its fixed
/// message; never the candidate value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
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
