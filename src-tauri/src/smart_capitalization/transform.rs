//! Pure first-letter transformation of smart capitalization (feature 015): applies a
//! [`CaseDecision`] to the first letter of the final text only, unless the first word is
//! protected, with Unicode title-case and full lower-case mappings that may change the length
//! of that letter without ever cutting a character.
use super::decision::{is_closer, is_combining_mark, is_line_break, is_terminator, CaseDecision};

/// What the transformation needs besides the text and the decision.
#[derive(Clone, Copy, Debug, Default)]
pub struct CaseInputs<'a> {
    /// The final text is the AI post-processing result (005): its case is already decided.
    pub case_decided: bool,
    /// Spellings configured by the user: 002 replacement values and 003 custom words.
    pub protected: &'a [String],
}

/// Opening marks kept in front of the first letter ("¿qué?", "« Bonjour", "(see").
const OPENERS: [char; 15] = [
    '¿', '¡', '"', '\'', '“', '‘', '«', '‹', '(', '[', '{', '「', '『', '（', '【',
];
/// Apostrophes that join the English pronoun "I" to its contraction ("I'm", "I’ll").
const APOSTROPHES: [char; 2] = ['\'', '’'];

// @spec AC-001: Only the first letter changes, AC-003: Protections, AC-004: Unicode mappings — .specs/features/015-smart-capitalization/spec.md#ac-001
/// `text` with its first letter adapted to `decision`; any other character is unchanged.
pub fn adapt_first_letter(text: &str, decision: CaseDecision, inputs: &CaseInputs<'_>) -> String {
    if decision == CaseDecision::Keep || inputs.case_decided {
        return text.to_owned();
    }
    let Some(start) = first_letter_index(text) else {
        return text.to_owned();
    };
    let word = first_word(&text[start..]);
    if is_protected(text, start, word, decision, inputs.protected) {
        return text.to_owned();
    }
    let mut characters = text[start..].chars();
    let Some(letter) = characters.next() else {
        return text.to_owned();
    };
    let mapped = match decision {
        CaseDecision::Capitalize => title_case(letter),
        CaseDecision::Lower => letter.to_lowercase().collect(),
        CaseDecision::Keep => letter.to_string(),
    };
    let rest = characters.as_str();
    let mut adapted = String::with_capacity(text.len() + mapped.len());
    adapted.push_str(&text[..start]);
    adapted.push_str(&mapped);
    adapted.push_str(rest);
    adapted
}

/// Byte index of the first character after leading whitespace and opening marks, when it is a
/// letter that has a case; `None` for a text that starts with a digit, a symbol or a caseless
/// letter (CJK), which is never changed.
fn first_letter_index(text: &str) -> Option<usize> {
    let (index, character) = text
        .char_indices()
        .find(|(_, character)| !character.is_whitespace() && !OPENERS.contains(character))?;
    has_case(character).then_some(index)
}

/// True when the letter has a distinct upper, lower or title form.
fn has_case(character: char) -> bool {
    let unchanged = |mapped: &mut dyn Iterator<Item = char>| {
        mapped.next() == Some(character) && mapped.next().is_none()
    };
    !(unchanged(&mut character.to_lowercase()) && unchanged(&mut character.to_uppercase()))
}

/// The first word at the start of `text`: letters, digits, combining marks, apostrophes and
/// inner hyphens.
fn first_word(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|(_, character)| !is_word_character(*character))
        .map_or(text.len(), |(index, _)| index);
    text[..end].trim_end_matches(['-', '\'', '’'])
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric()
        || is_combining_mark(character)
        || character == '-'
        || APOSTROPHES.contains(&character)
}

// @spec AC-003: First word protections — .specs/features/015-smart-capitalization/spec.md#ac-003
/// Business rule (spec AC-003): acronyms and words with an inner capital, the pronoun "I",
/// configured spellings, and a capitalized word repeated later in the text are never changed.
fn is_protected(
    text: &str,
    start: usize,
    word: &str,
    decision: CaseDecision,
    protected: &[String],
) -> bool {
    has_inner_capital(word)
        || (decision == CaseDecision::Lower && is_english_i(word))
        || starts_with_configured_spelling(&text[start..], protected)
        || (decision == CaseDecision::Lower && is_repeated_capitalized(text, start, word))
}

fn has_inner_capital(word: &str) -> bool {
    word.chars().skip(1).any(char::is_uppercase)
}

/// "I", or "I" followed by an apostrophe ("I'm", "I'll", "I’d").
fn is_english_i(word: &str) -> bool {
    let mut characters = word.chars();
    characters.next() == Some('I')
        && characters
            .next()
            .is_none_or(|next| APOSTROPHES.contains(&next))
}

/// True when `text` starts with one of `protected`, case-sensitively, on a word boundary.
fn starts_with_configured_spelling(text: &str, protected: &[String]) -> bool {
    protected
        .iter()
        .map(|spelling| spelling.trim())
        .filter(|spelling| !spelling.is_empty())
        .any(|spelling| {
            text.strip_prefix(spelling).is_some_and(|after| {
                after
                    .chars()
                    .next()
                    .is_none_or(|next| !next.is_alphanumeric() && !is_combining_mark(next))
            })
        })
}

/// Independent heuristic for proper nouns (spec decision `abbreviations_and_proper_nouns`): a
/// capitalized first word that appears again, capitalized and not at a sentence start, later in
/// the same text is a name or a brand ("Paris is nice, I love Paris"). It cannot detect a name
/// used once; that limitation is documented.
fn is_repeated_capitalized(text: &str, start: usize, word: &str) -> bool {
    if !word.chars().next().is_some_and(char::is_uppercase) {
        return false;
    }
    let after_first = start + word.len();
    text[after_first..].match_indices(word).any(|(offset, _)| {
        let index = after_first + offset;
        is_word_boundary(text, index, word.len()) && !starts_sentence(&text[..index])
    })
}

/// True when the occurrence at `index` of length `length` is a whole word.
fn is_word_boundary(text: &str, index: usize, length: usize) -> bool {
    let before = text[..index].chars().next_back();
    let after = text[index + length..].chars().next();
    before.is_none_or(|character| !is_word_character(character))
        && after.is_none_or(|character| !is_word_character(character))
}

/// True when `prefix` ends where a sentence starts: nothing, or a terminator or a line break
/// before optional whitespace and closing marks.
fn starts_sentence(prefix: &str) -> bool {
    let trimmed = prefix.trim_end_matches(|character: char| {
        (character.is_whitespace() && !is_line_break(character)) || OPENERS.contains(&character)
    });
    let core = trimmed.trim_end_matches(is_closer);
    core.chars()
        .next_back()
        .is_none_or(|last| is_terminator(last) || is_line_break(last))
}

/// Title case of one letter: the four Latin digraphs have their own title form; otherwise the
/// first character of the full upper-case mapping followed by the rest of that mapping in lower
/// case ("ß" → "Ss", "ﬁ" → "Fi"). Combining marks of the mapping are kept.
fn title_case(letter: char) -> String {
    match letter {
        'Ǆ' | 'ǅ' | 'ǆ' => "ǅ".to_owned(),
        'Ǉ' | 'ǈ' | 'ǉ' => "ǈ".to_owned(),
        'Ǌ' | 'ǋ' | 'ǌ' => "ǋ".to_owned(),
        'Ǳ' | 'ǲ' | 'ǳ' => "ǲ".to_owned(),
        _ => {
            let mut upper = letter.to_uppercase();
            let mut titled: String = upper.next().into_iter().collect();
            titled.extend(upper.flat_map(char::to_lowercase));
            titled
        }
    }
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
