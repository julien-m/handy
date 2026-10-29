//! Pure junction rule of smart spacing (feature 017): whether one space belongs between the
//! text on the left of a junction and the text on its right (field text and dictation, in
//! either order). Independent design (spec "Independent design" and decisions): typographic
//! rules for whitespace already present, punctuation that attaches to a word, opening marks,
//! elision, attached hyphens, decimal numbers and scripts written without spaces. No I/O.

/// Opening marks after which a word follows directly ("(see", "[1", "¿Qué", "« " is typed by
/// the user). Straight quotes are decided by what precedes them (`opens_at_end`).
const OPENERS: [char; 17] = [
    '(', '[', '{', '¿', '¡', '“', '‘', '„', '‚', '«', '‹', '「', '『', '（', '【', '〈', '《',
];
/// Characters written right after a word, without a space before them (spec AC-004 and the
/// French typography decision: `;` `:` `!` `?` included, no space is ever invented before them).
const ATTACH_TO_PREVIOUS: [char; 16] = [
    ',', '.', ';', ':', '!', '?', '…', '‽', ')', ']', '}', '%', '»', '›', '”', '’',
];
/// Straight and typographic apostrophes (elision "l'", contraction "'s").
const APOSTROPHES: [char; 2] = ['\'', '’'];
/// Joiners written inside a compound ("anti-", "and/or").
const WORD_JOINERS: [char; 3] = ['-', '/', '‐'];
/// Decimal and thousands separators between two digits ("3.5", "1,000").
const NUMBER_SEPARATORS: [char; 2] = ['.', ','];

// @spec FR-001: Pure junction rule, AC-003: Junction rule — .specs/features/017-smart-spacing/spec.md#fr-001
/// Whether one space is needed between `left` and `right` (both non-empty to need one).
///
/// Strategy: whitespace on either side already separates; a script written without spaces on
/// either side joins; then the last character of `left` may open or attach, the first one of
/// `right` may attach to `left`; a decimal number joins. Every other pair is two words.
pub fn needs_space(left: &str, right: &str) -> bool {
    let mut backwards = left.chars().rev();
    let (Some(last), Some(first)) = (backwards.next(), right.chars().next()) else {
        return false;
    };
    let before_last = backwards.next();
    let after_first = right.chars().nth(1);
    if last.is_whitespace() || first.is_whitespace() {
        return false;
    }
    if is_unspaced_script(last) || is_unspaced_script(first) {
        return false;
    }
    !opens_at_end(last, before_last)
        && !attaches_at_start(first, after_first)
        && !is_decimal_junction(before_last, last, first)
}

/// True for a character of a script written without spaces between words: Thai, Lao,
/// Myanmar, Khmer, Tibetan, CJK ideographs, kana, bopomofo, CJK punctuation and full-width
/// forms. Hangul is written with spaces and is not one (spec decision `unspaced_scripts`).
pub fn is_unspaced_script(character: char) -> bool {
    matches!(
        character,
        '\u{0E00}'..='\u{0EFF}'
            | '\u{0F00}'..='\u{0FFF}'
            | '\u{1000}'..='\u{109F}'
            | '\u{1780}'..='\u{17FF}'
            | '\u{19E0}'..='\u{19FF}'
            | '\u{2E80}'..='\u{2FDF}'
            | '\u{3000}'..='\u{30FF}'
            | '\u{3100}'..='\u{312F}'
            | '\u{31A0}'..='\u{31FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FE30}'..='\u{FE4F}'
            | '\u{FF00}'..='\u{FFEF}'
            | '\u{20000}'..='\u{3FFFF}'
    )
}

/// True when `text` may be followed by one fallback or end-of-field space: it ends with a
/// character of a script that uses spaces and not with an opening mark (spec AC-006).
pub fn suits_trailing_space(text: &str) -> bool {
    let mut backwards = text.chars().rev();
    let Some(last) = backwards.next() else {
        return false;
    };
    !last.is_whitespace() && !is_unspaced_script(last) && !opens_at_end(last, backwards.next())
}

/// Business rule: `last` (preceded by `previous`) leaves no room for a space after it: an
/// opening mark; a straight quote after whitespace or at the start (an opening quote); an
/// apostrophe after a letter (French elision "l'", "qu'") or after whitespace (an opening
/// quote); a hyphen or slash attached to a word ("anti-", "and/").
fn opens_at_end(last: char, previous: Option<char>) -> bool {
    let after_space = previous.is_none_or(char::is_whitespace);
    let after_word = previous.is_some_and(char::is_alphanumeric);
    if OPENERS.contains(&last) {
        return true;
    }
    if last == '"' {
        return after_space;
    }
    if APOSTROPHES.contains(&last) {
        return after_space || previous.is_some_and(char::is_alphabetic);
    }
    WORD_JOINERS.contains(&last) && after_word
}

/// Business rule: `first` (followed by `next`) is written against the word before it:
/// closing punctuation and marks, an apostrophe ("'s"), a straight quote not followed by a
/// word (a closing quote), a hyphen or slash followed by a letter ("-ci", "/or"; "-5" is a
/// signed number and gets its space).
fn attaches_at_start(first: char, next: Option<char>) -> bool {
    if ATTACH_TO_PREVIOUS.contains(&first) || APOSTROPHES.contains(&first) {
        return true;
    }
    if first == '"' {
        return !next.is_some_and(char::is_alphanumeric);
    }
    WORD_JOINERS.contains(&first) && next.is_some_and(char::is_alphabetic)
}

/// A separator between two digits ("3." then "5"): one number, no space.
fn is_decimal_junction(before_last: Option<char>, last: char, first: char) -> bool {
    NUMBER_SEPARATORS.contains(&last)
        && before_last.is_some_and(|digit| digit.is_ascii_digit())
        && first.is_ascii_digit()
}

#[cfg(test)]
#[path = "junction_tests.rs"]
mod tests;
