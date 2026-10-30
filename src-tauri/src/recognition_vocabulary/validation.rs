//! Shared term validation and uniqueness key of the vocabulary, the single authority for the
//! settings UI, the CLI and the recognition context (the UI never validates with UTF-16 lengths
//! or `toLowerCase`).
use crate::settings_service::errors::ErrorReason;
use caseless::Caseless;
use serde::Serialize;

/// Most terms a vocabulary holds. Spec AC-001 says 500; Julien raised it to 5,000 on 2026-10-28
/// (native session finding N7, `spec-refine` 003 pending). Whisper still receives only the terms
/// that fit its prompt budget (`budget.rs`); the approximate correction of the other models
/// stays under 50 ms per dictation at this size (`audio_toolkit/custom_words_perf_tests.rs`).
pub const MAX_TERMS: usize = 5_000;
/// Most Unicode scalar values of a trimmed term (spec AC-001).
pub const MAX_TERM_CHARS: usize = 100;

/// Line separators refused inside a term, checked before any whitespace processing: CR, LF,
/// NEL, LINE SEPARATOR and PARAGRAPH SEPARATOR.
const LINE_BREAKS: [char; 5] = ['\r', '\n', '\u{85}', '\u{2028}', '\u{2029}'];
/// Characters refused in a term: the markup characters of the historical input sanitizing, and
/// NUL, which cannot cross the engine's C-string boundary.
const FORBIDDEN: [char; 4] = ['<', '>', '"', '\0'];

/// Why a term (new or stored) is refused or excluded from the recognition context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum TermIssue {
    /// Nothing left after trimming.
    Empty,
    /// More than `MAX_TERM_CHARS` Unicode scalar values.
    TooLong,
    /// A line separator inside the term.
    LineBreak,
    /// `<`, `>`, `"` or NUL.
    ForbiddenCharacter,
    /// Same key as an earlier term.
    Duplicate,
    /// Stored beyond the first `MAX_TERMS` positions (legacy list only).
    OverCapacity,
}

impl TermIssue {
    /// Closed CLI reason of a refused input term (`details.items[].code`); a capacity problem is
    /// `limit_exceeded`, never an item.
    pub fn error_reason(self) -> Option<ErrorReason> {
        match self {
            Self::Empty => Some(ErrorReason::TermEmpty),
            Self::TooLong => Some(ErrorReason::TermTooLong),
            Self::LineBreak => Some(ErrorReason::TermLineBreak),
            Self::ForbiddenCharacter => Some(ErrorReason::TermForbiddenCharacter),
            Self::Duplicate => Some(ErrorReason::TermDuplicate),
            Self::OverCapacity => None,
        }
    }
}

/// A valid term: its display text and its uniqueness key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidTerm {
    /// Trimmed text with inner whitespace runs collapsed to one ASCII space; case, accents and
    /// compatibility forms are kept as typed.
    pub text: String,
    /// `text` with full default (non-Turkic) Unicode case folding and no normalization.
    pub key: String,
}

/// Validate one term as typed or pasted.
///
/// # Errors
/// The first issue found, line breaks first (they are refused before trimming).
// @spec FR-001: Shared term validation and keys — .specs/features/003-recognition-vocabulary/spec.md#fr-001
pub fn validate_term(raw: &str) -> Result<ValidTerm, TermIssue> {
    if raw.contains(LINE_BREAKS) {
        return Err(TermIssue::LineBreak);
    }
    let trimmed = raw.trim();
    // The limit counts the trimmed text before whitespace collapsing (plan decision).
    let length = trimmed.chars().count();
    if length == 0 {
        return Err(TermIssue::Empty);
    }
    if length > MAX_TERM_CHARS {
        return Err(TermIssue::TooLong);
    }
    if trimmed.contains(FORBIDDEN) {
        return Err(TermIssue::ForbiddenCharacter);
    }
    let text = collapse_whitespace(trimmed);
    let key = fold(&text);
    Ok(ValidTerm { text, key })
}

/// Uniqueness key of any text, valid or not: trimmed, whitespace-collapsed and case-folded.
pub fn term_key(raw: &str) -> String {
    fold(&collapse_whitespace(raw.trim()))
}

fn fold(text: &str) -> String {
    text.chars().default_case_fold().collect()
}

/// Collapse every run of Unicode White_Space into one ASCII space.
fn collapse_whitespace(trimmed: &str) -> String {
    trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_counts_unicode_scalars_of_the_trimmed_text() {
        assert_eq!(
            validate_term(&"a".repeat(100)).map(|t| t.text.len()),
            Ok(100)
        );
        assert_eq!(validate_term(&"a".repeat(101)), Err(TermIssue::TooLong));
        // 100 emoji are 100 scalars (200 UTF-16 units): still valid.
        assert!(validate_term(&"😀".repeat(100)).is_ok());
        assert_eq!(validate_term(&"😀".repeat(101)), Err(TermIssue::TooLong));
        // A combining mark is its own scalar.
        assert_eq!(
            validate_term(&"e\u{301}".repeat(51)),
            Err(TermIssue::TooLong)
        );
        assert!(validate_term(&format!("  {}  ", "a".repeat(100))).is_ok());
        assert_eq!(validate_term("   "), Err(TermIssue::Empty));
        assert_eq!(validate_term(""), Err(TermIssue::Empty));
    }

    #[test]
    fn separators_and_forbidden_characters_are_refused() {
        for raw in [
            "a\nb",
            "a\rb",
            "a\u{85}b",
            "a\u{2028}b",
            "a\u{2029}b",
            "trailing\n",
        ] {
            assert_eq!(validate_term(raw), Err(TermIssue::LineBreak), "{raw:?}");
        }
        for raw in ["<tag>", "a>b", "say \"hi\"", "nul\0"] {
            assert_eq!(
                validate_term(raw),
                Err(TermIssue::ForbiddenCharacter),
                "{raw:?}"
            );
        }
        assert_eq!(
            validate_term("O'Neil").map(|t| t.text),
            Ok("O'Neil".to_owned())
        );
    }

    #[test]
    fn whitespace_is_trimmed_and_collapsed_without_changing_case() {
        let term = validate_term("\u{a0} Claude \t  Code\u{3000}");
        assert_eq!(term.map(|t| t.text), Ok("Claude Code".to_owned()));
    }

    #[test]
    fn keys_use_full_case_folding_without_normalization() {
        assert_eq!(term_key("Straße"), term_key("STRASSE"));
        assert_eq!(term_key("Claude  Code"), term_key("claude code"));
        // Composed and decomposed accents stay distinct (no NFC/NFKC).
        assert_ne!(term_key("caf\u{e9}"), term_key("cafe\u{301}"));
        // Non-Turkic folding: dotted capital I folds to "i" + combining dot.
        assert_eq!(term_key("\u{130}"), "i\u{307}");
    }

    #[test]
    fn unicode_tables_are_pinned_together() {
        // Case folding comes from `caseless` (Unicode 16.0.0), whitespace from the standard
        // library (Unicode 17.0.0 with Rust 1.94): a toolchain or crate update must be reviewed
        // here, since it can change which terms collide.
        assert_eq!(caseless::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(char::UNICODE_VERSION, (17, 0, 0));
    }

    #[test]
    fn every_input_issue_has_a_closed_reason() {
        assert_eq!(
            TermIssue::Duplicate.error_reason(),
            Some(ErrorReason::TermDuplicate)
        );
        assert_eq!(TermIssue::OverCapacity.error_reason(), None);
    }
}
