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
    /// A replacement rule has no recognized text (002).
    RecognizedEmpty,
    /// A recognized text is longer than 100 characters (002).
    RecognizedTooLong,
    /// A recognized text holds a line break, a tab or `;` (002).
    RecognizedForbiddenCharacter,
    /// A recognized text holds a control or bidirectional formatting character (002).
    RecognizedControlCharacter,
    /// A recognized text starts with `#`, like an exchange comment (002).
    RecognizedLeadingComment,
    /// Two rules have the same normalized recognized text (002).
    DuplicateRecognized,
    /// A replacement rule has no replacement text (002).
    ReplacementEmpty,
    /// A replacement text is longer than 500 characters (002).
    ReplacementTooLong,
    /// A replacement text holds a line break or a tab (002).
    ReplacementForbiddenCharacter,
    /// A replacement text holds a control or bidirectional formatting character (002).
    ReplacementControlCharacter,
    /// A replacement rule has an empty id (002).
    EmptyRuleId,
    /// Two replacement rules share an id (002).
    DuplicateRuleId,
    /// An exchange line has neither a tab nor a `;` separator (002).
    MissingSeparator,
    /// The rule set would hold more than 1000 rules (002).
    TooManyRules,
    /// The saved rules are invalid, so no import can be previewed on them (002).
    InvalidSavedRules,
    /// The Unicode data needed to validate rules is unavailable (002).
    UnicodeDataUnavailable,
    /// The running instance's answer does not fit one channel message (002).
    ReplyTooLarge,
    /// The export file could not be written; the destination is unchanged (002).
    ExportNotWritten,
    /// The export destination is a symbolic link, refused; nothing is written (002).
    ExportSymbolicLink,
    /// The export destination is a file the user cannot write; it is unchanged (002).
    ExportReadOnly,
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
    /// The microphone choices change only through `handy settings microphones` (001).
    MicrophonesDomain,
    /// A microphone choice is empty or only whitespace (001).
    MicrophoneNameEmpty,
    /// A microphone choice appears more than once (001).
    MicrophoneNameDuplicate,
}
impl ErrorReason {
    /// Every reason of this build, in wire-schema order.
    pub const ALL: [Self; 44] = [
        Self::UnsupportedPlatform,
        Self::ShortcutSessionBusy,
        Self::InstanceRequired,
        Self::LastShortcut,
        Self::AlternativesUnsupported,
        Self::RegistrationRefused,
        Self::InvalidShortcut,
        Self::RecognizedEmpty,
        Self::RecognizedTooLong,
        Self::RecognizedForbiddenCharacter,
        Self::RecognizedControlCharacter,
        Self::RecognizedLeadingComment,
        Self::DuplicateRecognized,
        Self::ReplacementEmpty,
        Self::ReplacementTooLong,
        Self::ReplacementForbiddenCharacter,
        Self::ReplacementControlCharacter,
        Self::EmptyRuleId,
        Self::DuplicateRuleId,
        Self::MissingSeparator,
        Self::TooManyRules,
        Self::InvalidSavedRules,
        Self::UnicodeDataUnavailable,
        Self::ReplyTooLarge,
        Self::ExportNotWritten,
        Self::ExportSymbolicLink,
        Self::ExportReadOnly,
        Self::TermEmpty,
        Self::TermTooLong,
        Self::TermLineBreak,
        Self::TermForbiddenCharacter,
        Self::TermDuplicate,
        Self::ContextBudgetUnverified,
        Self::PromptNameEmpty,
        Self::PromptNameTooLong,
        Self::PromptNameDuplicate,
        Self::PromptTextEmpty,
        Self::PromptTextTooLong,
        Self::OverrideIncomplete,
        Self::UnknownProvider,
        Self::PromptInvalid,
        Self::MicrophonesDomain,
        Self::MicrophoneNameEmpty,
        Self::MicrophoneNameDuplicate,
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
            Self::ReplyTooLarge => {
                "The answer is larger than one message; read large lists with their domain command."
            }
            Self::ExportNotWritten => "The export file could not be written; it is unchanged.",
            Self::ExportSymbolicLink => {
                "The export destination is a symbolic link; nothing was written."
            }
            Self::ExportReadOnly => "The export destination is read-only; it is unchanged.",
            other => other
                .term_message()
                .or_else(|| other.prompt_message())
                .or_else(|| other.microphone_message())
                .unwrap_or_else(|| other.rule_message()),
        }
    }
    /// Message of a vocabulary term or recognition context reason (003); `None` for every other
    /// reason.
    fn term_message(self) -> Option<&'static str> {
        let message = match self {
            Self::TermEmpty => "A term cannot be empty.",
            Self::TermTooLong => "A term has at most 100 characters.",
            Self::TermLineBreak => "A term cannot contain a line break.",
            Self::TermForbiddenCharacter => "A term cannot contain <, > or \".",
            Self::TermDuplicate => "Each term may appear only once.",
            Self::ContextBudgetUnverified => {
                "The selected model's recognition context budget is not documented in this build."
            }
            _ => return None,
        };
        Some(message)
    }
    /// Message of a custom prompt reason (005); `None` for every other reason.
    fn prompt_message(self) -> Option<&'static str> {
        let message = match self {
            Self::PromptNameEmpty => "The prompt name is empty.",
            Self::PromptNameTooLong => "The prompt name is longer than 60 characters.",
            Self::PromptNameDuplicate => "Another prompt already has this name.",
            Self::PromptTextEmpty => "The prompt text is empty.",
            Self::PromptTextTooLong => "The prompt text is longer than 8000 characters.",
            Self::OverrideIncomplete => "A provider override needs a provider and a model.",
            Self::UnknownProvider => "The provider does not exist in this build.",
            Self::PromptInvalid => "The prompt is invalid and must be edited first.",
            _ => return None,
        };
        Some(message)
    }
    /// Message of a microphone choice reason (001); `None` for every other reason.
    fn microphone_message(self) -> Option<&'static str> {
        let message = match self {
            Self::MicrophonesDomain => "Use handy settings microphones set-priority.",
            Self::MicrophoneNameEmpty => "A microphone name cannot be empty.",
            Self::MicrophoneNameDuplicate => "Each microphone may appear only once.",
            _ => return None,
        };
        Some(message)
    }
    /// Message of a replacement rule or rule set reason (002); `message` answers every other
    /// reason first, and a test checks that no reason reaches the fallback.
    fn rule_message(self) -> &'static str {
        match self {
            Self::RecognizedEmpty => "The recognized text is empty.",
            Self::RecognizedTooLong => "The recognized text is longer than 100 characters.",
            Self::RecognizedForbiddenCharacter => {
                "The recognized text contains a line break, a tab or a semicolon."
            }
            Self::RecognizedControlCharacter => {
                "The recognized text contains a control or bidirectional formatting character."
            }
            Self::RecognizedLeadingComment => "The recognized text cannot start with #.",
            Self::DuplicateRecognized => "Another rule has the same recognized text.",
            Self::ReplacementEmpty => "The replacement is empty.",
            Self::ReplacementTooLong => "The replacement is longer than 500 characters.",
            Self::ReplacementForbiddenCharacter => {
                "The replacement contains a line break or a tab."
            }
            Self::ReplacementControlCharacter => {
                "The replacement contains a control or bidirectional formatting character."
            }
            Self::EmptyRuleId => "The rule id is empty.",
            Self::DuplicateRuleId => "Another rule has the same id.",
            Self::MissingSeparator => "The line has no tab or semicolon separator.",
            Self::TooManyRules => "More than 1000 rules.",
            Self::InvalidSavedRules => "The saved rules are invalid; repair them first.",
            Self::UnicodeDataUnavailable => "Unicode data is unavailable.",
            _ => UNMATCHED_MESSAGE,
        }
    }
}

/// Fallback of `rule_message` for a reason `message` should have answered; never reached.
const UNMATCHED_MESSAGE: &str = "The settings change was refused.";

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

#[cfg(test)]
mod tests {
    use super::{ErrorReason, UNMATCHED_MESSAGE};

    #[test]
    fn every_reason_has_its_own_message() {
        let messages: Vec<&str> = ErrorReason::ALL
            .iter()
            .map(|reason| reason.message())
            .collect();
        assert!(!messages.contains(&UNMATCHED_MESSAGE));
        let unique: std::collections::HashSet<_> = messages.iter().collect();
        assert_eq!(unique.len(), ErrorReason::ALL.len());
    }
}
