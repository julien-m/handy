//! Pure decision of smart capitalization (feature 015): from the text before the cursor in the
//! destination field, whether the first letter of a dictation should be a capital, a lower-case
//! letter, or left as received. Independent design (spec "Independent design"): typographic rules
//! for sentence starts, line breaks, list markers and continuation punctuation. No I/O.

/// What precedes the cursor in the destination field.
#[derive(Clone, PartialEq, Eq)]
pub enum CursorContext {
    /// The field could not be read (option off, no permission, unreadable or non-editable
    /// field, password field, other destination, spent budget): nothing is changed.
    Absent,
    /// At most the last `CONTEXT_WINDOW_UNITS` UTF-16 units before the caret, and whether that
    /// window starts at position 0 of the field (an empty `text` with `reaches_start` is an
    /// empty field).
    Before { text: String, reaches_start: bool },
}

impl std::fmt::Debug for CursorContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The field content never reaches a log (spec AC-006): only its kind is printed.
        match self {
            Self::Absent => formatter.write_str("Absent"),
            Self::Before { reaches_start, .. } => formatter
                .debug_struct("Before")
                .field("reaches_start", reaches_start)
                .finish_non_exhaustive(),
        }
    }
}

/// How the first letter of the dictation is adapted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseDecision {
    /// A sentence, a line, a list item or the field starts: title-case the first letter.
    Capitalize,
    /// The dictation continues a sentence: lower the first letter.
    Lower,
    /// Not enough context to decide reliably: keep the case received.
    Keep,
}

/// Sentence terminators (Latin, CJK full-width, Arabic, Devanagari).
const TERMINATORS: [char; 13] = [
    '.', '!', '?', '…', '‽', '。', '！', '？', '．', '｡', '؟', '।', '॥',
];
/// Punctuation after which a sentence goes on.
const CONTINUATIONS: [char; 8] = [',', ';', ':', '、', '，', '；', '：', '،'];
/// Closing quotes and brackets that may follow a terminator or a word ("Done." ", (see above)).
/// Straight quotes are ambiguous; at the end of the context they are treated as closing, and an
/// opening one is recognized afterwards by the whitespace before it.
const CLOSERS: [char; 16] = [
    '"', '\'', '”', '’', '»', '›', ')', ']', '}', '」', '』', '）', '】', '〉', '》', '〕',
];
/// Bullet markers of an unordered list item.
const BULLETS: [char; 8] = ['-', '*', '•', '‣', '◦', '▪', '–', '—'];
/// Longest number of an ordered list marker ("1." to "999.").
const MAX_ORDINAL_DIGITS: usize = 3;

/// True for a character that ends a line.
pub(crate) fn is_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r' | '\u{0B}' | '\u{0C}' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

/// True for whitespace that does not end a line (spaces, tabs, no-break spaces).
pub(crate) fn is_horizontal_space(character: char) -> bool {
    character.is_whitespace() && !is_line_break(character)
}

/// True for a combining mark of the common blocks (accents typed after their letter).
pub(crate) fn is_combining_mark(character: char) -> bool {
    matches!(
        character,
        '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}'
    )
}

/// True for a sentence terminator.
pub(crate) fn is_terminator(character: char) -> bool {
    TERMINATORS.contains(&character)
}

/// True for a closing quote or bracket.
pub(crate) fn is_closer(character: char) -> bool {
    CLOSERS.contains(&character)
}

// @spec AC-002: Context decision table — .specs/features/015-smart-capitalization/spec.md#ac-002
/// Decide the case of the first letter from `context` (spec AC-002 and AC-005).
pub fn decide(context: &CursorContext) -> CaseDecision {
    let CursorContext::Before {
        text,
        reaches_start,
    } = context
    else {
        return CaseDecision::Keep;
    };
    let trimmed = text.trim_end_matches(is_horizontal_space);
    if trimmed.is_empty() {
        // Only whitespace: an empty field when the window reaches position 0, otherwise the
        // start of the field is not proven (older text may sit before the window).
        return if *reaches_start {
            CaseDecision::Capitalize
        } else {
            CaseDecision::Keep
        };
    }
    if trimmed.ends_with(is_line_break) {
        return CaseDecision::Capitalize;
    }
    let spaced = trimmed.len() < text.len();
    // A marker is only known to start its line when the line start is inside the window.
    let line_start_seen = *reaches_start || trimmed.contains(is_line_break);
    if spaced && line_start_seen && is_list_marker(last_line(trimmed)) {
        return CaseDecision::Capitalize;
    }
    decide_from_last_character(trimmed)
}

/// The text after the last line break of `text`.
fn last_line(text: &str) -> &str {
    // `rsplit` cuts after the whole break character (U+2028 and U+0085 take several bytes).
    text.rsplit(is_line_break).next().unwrap_or(text)
}

/// True when `line` is a list marker alone, after optional indentation: a bullet, or one to
/// three digits followed by `.` or `)`.
fn is_list_marker(line: &str) -> bool {
    let marker = line.trim_start_matches(is_horizontal_space);
    let mut characters = marker.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if BULLETS.contains(&first) && characters.next().is_none() {
        return true;
    }
    let Some(digits) = marker
        .strip_suffix('.')
        .or_else(|| marker.strip_suffix(')'))
    else {
        return false;
    };
    (1..=MAX_ORDINAL_DIGITS).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Decision from the last meaningful character of `trimmed` (no trailing whitespace), after
/// stepping over closing quotes and brackets.
fn decide_from_last_character(trimmed: &str) -> CaseDecision {
    let core = trimmed.trim_end_matches(is_closer);
    let Some(last) = core.chars().next_back() else {
        return CaseDecision::Keep;
    };
    if is_terminator(last) {
        return CaseDecision::Capitalize;
    }
    if CONTINUATIONS.contains(&last) || last.is_alphanumeric() || is_combining_mark(last) {
        return CaseDecision::Lower;
    }
    // Whitespace before a quote (an opening quote), an opening bracket, a symbol or an emoji:
    // the context does not say whether a sentence starts, so the case received is kept.
    CaseDecision::Keep
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod tests;
