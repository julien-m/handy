//! Transformation table of spec AC-001, AC-003, AC-004 and AC-011.
use super::*;

const NONE: CaseInputs<'static> = CaseInputs {
    case_decided: false,
    protected: &[],
};

fn lower(text: &str) -> String {
    adapt_first_letter(text, CaseDecision::Lower, &NONE)
}

fn capital(text: &str) -> String {
    adapt_first_letter(text, CaseDecision::Capitalize, &NONE)
}

#[test]
fn only_first_letter_changes() {
    assert_eq!(lower("Pears And Plums."), "pears And Plums.");
    assert_eq!(capital("hello there, World"), "Hello there, World");
}

#[test]
fn keep_changes_nothing() {
    assert_eq!(
        adapt_first_letter("Hello", CaseDecision::Keep, &NONE),
        "Hello"
    );
}

#[test]
fn keeps_acronym() {
    assert_eq!(lower("API keys rotate"), "API keys rotate");
    assert_eq!(lower("NASA launched"), "NASA launched");
}

#[test]
fn keeps_inner_capital() {
    assert_eq!(lower("iPhone"), "iPhone");
    assert_eq!(capital("iPhone sales"), "iPhone sales");
    assert_eq!(lower("LiveSpec works"), "LiveSpec works");
    assert_eq!(lower("McDonald's"), "McDonald's");
}

#[test]
fn keeps_english_i() {
    for text in ["I think so", "I'm here", "I’ll go", "I'd say"] {
        assert_eq!(lower(text), text);
    }
    assert_eq!(lower("It works"), "it works");
    assert_eq!(lower("Ice cream"), "ice cream");
}

#[test]
fn keeps_protected_replacement() {
    let protected = ["Handy Computer".to_owned()];
    let inputs = CaseInputs {
        case_decided: false,
        protected: &protected,
    };
    assert_eq!(
        adapt_first_letter("Handy Computer rocks", CaseDecision::Lower, &inputs),
        "Handy Computer rocks"
    );
    assert_eq!(
        adapt_first_letter("Handy Computers", CaseDecision::Lower, &inputs),
        "handy Computers",
        "a prefix of a longer word is not the configured spelling"
    );
}

#[test]
fn keeps_custom_word() {
    let protected = ["Notion".to_owned(), "cjpais".to_owned()];
    let inputs = CaseInputs {
        case_decided: false,
        protected: &protected,
    };
    assert_eq!(
        adapt_first_letter("Notion is open", CaseDecision::Lower, &inputs),
        "Notion is open"
    );
    assert_eq!(
        adapt_first_letter("cjpais wrote it", CaseDecision::Capitalize, &inputs),
        "cjpais wrote it"
    );
}

#[test]
fn keeps_repeated_proper_noun() {
    assert_eq!(
        lower("Paris is nice, I love Paris"),
        "Paris is nice, I love Paris"
    );
}

#[test]
fn lowers_unknown_single_proper_noun() {
    // Documented limitation (spec decision `abbreviations_and_proper_nouns`).
    assert_eq!(lower("Paris is nice"), "paris is nice");
    assert_eq!(
        lower("Hello there. Hello again"),
        "hello there. Hello again",
        "a repetition at a sentence start is not a sign of a name"
    );
}

#[test]
fn ai_decided_case_is_kept() {
    let inputs = CaseInputs {
        case_decided: true,
        protected: &[],
    };
    assert_eq!(
        adapt_first_letter("Polished text", CaseDecision::Lower, &inputs),
        "Polished text"
    );
}

#[test]
fn sharp_s_title_case() {
    assert_eq!(capital("ßtraße"), "Sstraße");
}

#[test]
fn ligature_title_case() {
    assert_eq!(capital("ﬁne tuning"), "Fine tuning");
}

#[test]
fn digraph_title_case() {
    assert_eq!(capital("ǆungla"), "ǅungla");
    assert_eq!(capital("ǉubav"), "ǈubav");
    assert_eq!(lower("ǅungla"), "ǆungla");
}

#[test]
fn dotted_capital_i_lowers_fully() {
    let lowered = lower("İstanbul");
    assert_eq!(lowered, "i\u{307}stanbul");
    assert_eq!(lowered.chars().count(), "İstanbul".chars().count() + 1);
}

#[test]
fn greek_with_expansion_is_whole() {
    assert_eq!(capital("\u{390}"), "\u{399}\u{308}\u{301}");
}

#[test]
fn combining_mark_kept() {
    assert_eq!(capital("e\u{301}te\u{301}"), "E\u{301}te\u{301}");
    assert_eq!(lower("E\u{301}cole"), "e\u{301}cole");
}

#[test]
fn cjk_unchanged() {
    assert_eq!(capital("東京に行く"), "東京に行く");
    assert_eq!(lower("42 apples"), "42 apples");
    assert_eq!(lower("…"), "…");
    assert_eq!(lower(""), "");
}

#[test]
fn leading_punctuation_kept() {
    assert_eq!(capital("¿qué pasa?"), "¿Qué pasa?");
    assert_eq!(lower("« Bonjour »"), "« bonjour »");
    assert_eq!(capital("  \"quoted\""), "  \"Quoted\"");
}

#[test]
fn reference_example_je_souhaite() {
    use super::super::decision::{decide, CursorContext};
    let context = CursorContext::Before {
        text: "Je souhaite".to_owned(),
        reaches_start: true,
    };
    assert_eq!(
        adapt_first_letter("Vérifier ce point.", decide(&context), &NONE),
        "vérifier ce point."
    );
}
