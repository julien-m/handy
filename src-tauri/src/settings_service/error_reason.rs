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
}
impl ErrorReason {
    /// Every reason of this build, in wire-schema order.
    pub const ALL: [Self; 27] = [
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
            rule => rule.rule_message(),
        }
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
