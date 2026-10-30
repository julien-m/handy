//! Final period of a dictation inserted inside a sentence (feature 017, decision N30): some
//! recognition engines end every dictation with a period, even one inserted in the middle of
//! a sentence ("Bonjour|demain" + "à tous." would give "Bonjour à tous. demain"). When the
//! text after the selection was read and shows the sentence goes on, the dictation's single
//! final period is dropped. Pure, no I/O; only that one character may be removed.
use crate::smart_capitalization::decision::is_horizontal_space;

/// The only character this rule may remove: the ASCII full stop. `…`, `?`, `!` and the CJK
/// full stop `。` are never touched (spec decision `n30_trailing_period`).
const FULL_STOP: char = '.';
/// Characters that continue or end a sentence when they start the text after the selection.
const CONTINUATION_MARKS: [char; 13] = [
    ',', '.', ';', ':', '!', '?', ')', ']', '}', '”', '’', '»', '›',
];
/// The straight double quote: a closing quote when no word follows it.
const STRAIGHT_QUOTE: char = '"';
/// Common abbreviations written with a final period (compared in lower case, without it).
const ABBREVIATIONS: [&str; 24] = [
    "etc", "cf", "vs", "env", "ex", "mme", "mmes", "mlle", "mlles", "mm", "mr", "mrs", "ms", "dr",
    "pr", "prof", "st", "jr", "sr", "inc", "ltd", "fig", "vol", "approx",
];

// @spec AC-017: Final period inside a sentence — .specs/features/017-smart-spacing/spec.md#ac-017
/// `body` (the dictation without trailing whitespace) without its final period when the text
/// `after` the selection (non-empty, read from the field) continues the sentence; otherwise
/// `body` unchanged.
///
/// Strategy: three independent tests must all pass: the text after continues the sentence
/// (`continues_sentence`), the dictation ends with one sentence period (`ends_with_sentence_period`),
/// and that period is not an abbreviation's (`is_abbreviation`).
pub fn drop_final_period<'a>(body: &'a str, after: &str) -> &'a str {
    match body.strip_suffix(FULL_STOP) {
        Some(stripped) if continues_sentence(after) && ends_with_sentence_period(stripped) => {
            stripped
        }
        _ => body,
    }
}

/// Business rule: the text after the selection, past horizontal spaces, starts with a mark
/// that continues or ends the sentence, or with a lower-case letter. A line break, an
/// upper-case or uncased letter, a digit, or nothing at all (field end or window of spaces)
/// keeps the period.
fn continues_sentence(after: &str) -> bool {
    let mut characters = after.trim_start_matches(is_horizontal_space).chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if first == STRAIGHT_QUOTE {
        return !characters.next().is_some_and(char::is_alphanumeric);
    }
    CONTINUATION_MARKS.contains(&first) || first.is_lowercase()
}

/// Business rule: `stripped` (the dictation without its final period) ends with a word whose
/// period closes a sentence: its last character is a letter or digit (so "..." and "?." keep
/// their period) and that word is not an abbreviation.
fn ends_with_sentence_period(stripped: &str) -> bool {
    let word = stripped
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default();
    word.chars().next_back().is_some_and(char::is_alphanumeric) && !is_abbreviation(word)
}

/// Business rule (prudent): a word with an inner period ("U.S", "e.g", "3.5"), a single
/// letter ("M", initials) or a listed abbreviation ("etc", "Mme") keeps its period, since it
/// may belong to the word rather than end the sentence.
fn is_abbreviation(word: &str) -> bool {
    let letters = word.trim_start_matches(|character: char| !character.is_alphanumeric());
    let mut characters = letters.chars();
    let single_letter =
        characters.next().is_some_and(char::is_alphabetic) && characters.next().is_none();
    single_letter
        || letters.contains(FULL_STOP)
        || ABBREVIATIONS.contains(&letters.to_lowercase().as_str())
}

#[cfg(test)]
#[path = "final_period_tests.rs"]
mod tests;
