//! Whole transformation of feature 016: preservation reasons, languages and the dictation switch.

use super::test_text::{paragraph_words, prose};
use super::unicode_segmenter::{sentences, UnicodeSegmenter};
use super::{arrange, arrange_sentences, for_dictation, Arrangement, Preserved, Structure};

const ENGLISH: &str = "This morning we reviewed the quarterly budget with the finance team. \
    Spending on cloud services grew faster than planned. We agreed to cap new projects until \
    March. Marketing will present a revised plan next week. Hiring stays open for two \
    engineering roles. The office move is delayed by a month. Everyone should update their \
    calendars accordingly.";

const FRENCH: &str = "Ce matin, nous avons revu le budget trimestriel avec l'équipe financière. \
    Les dépenses en services cloud ont augmenté beaucoup plus vite que prévu pendant tout le \
    trimestre dernier. Nous avons donc décidé de geler tous les nouveaux projets jusqu'en mars \
    prochain au plus tard, sans exception. Le marketing présentera un plan révisé la semaine \
    prochaine. Deux postes d'ingénieur restent ouverts. Le déménagement est repoussé d'un mois.";

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn empty_and_blank_texts_are_unchanged() {
    for text in ["", "   ", "\t"] {
        assert_eq!(
            arrange(text, &UnicodeSegmenter),
            Arrangement::Preserved(Preserved::Empty)
        );
    }
}

#[test]
fn a_structured_text_is_never_handed_to_the_rules() {
    let list = format!("- {}", prose(&[5; 8]));
    assert_eq!(
        arrange(&list, &UnicodeSegmenter),
        Arrangement::Preserved(Preserved::Structured(Structure::ListItem))
    );
    let broken = format!("{}\n{}", prose(&[5; 4]), prose(&[5; 4]));
    assert_eq!(
        arrange(&broken, &UnicodeSegmenter),
        Arrangement::Preserved(Preserved::Structured(Structure::LineBreak))
    );
}

#[test]
fn an_english_dictation_gets_paragraphs_of_four_substantial_sentences() -> TestResult {
    let Arrangement::Arranged(text) = arrange(ENGLISH, &UnicodeSegmenter) else {
        return Err("expected paragraphs".into());
    };
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    assert_eq!(paragraphs.len(), 2);
    assert!(paragraphs[0].ends_with("next week."));
    assert!(paragraphs[1].starts_with("Hiring stays open"));
    assert_eq!(text.replace("\n\n", " "), ENGLISH);
    Ok(())
}

#[test]
fn a_french_dictation_breaks_at_the_word_limit() -> TestResult {
    let Arrangement::Arranged(text) = arrange(FRENCH, &UnicodeSegmenter) else {
        return Err("expected paragraphs".into());
    };
    let words = paragraph_words(&text);
    assert!(words[0] >= 45, "{words:?}");
    assert_eq!(words.len(), 2);
    assert_eq!(text.replace("\n\n", " "), FRENCH);
    Ok(())
}

#[test]
fn chinese_and_japanese_outside_macos_are_unchanged() {
    let japanese = "今日は会議でした。".repeat(12);
    assert_eq!(
        arrange(&japanese, &UnicodeSegmenter),
        Arrangement::Preserved(Preserved::UnsupportedText)
    );
}

#[test]
fn japanese_sentences_are_joined_without_any_added_space() -> TestResult {
    // The rules on UAX #29 sentences (each ideograph a word): only line breaks are added.
    let japanese = "今日は予算会議を行いました。".repeat(6);
    let Arrangement::Arranged(text) = arrange_sentences(&japanese, &sentences(&japanese)) else {
        return Err("expected paragraphs".into());
    };
    assert!(!text.contains(' '), "{text}");
    assert_eq!(text.replace("\n\n", ""), japanese);
    assert!(text.contains("。\n\n今日"));
    Ok(())
}

#[test]
fn the_switch_off_returns_the_text_unchanged() {
    assert_eq!(for_dictation(false, ENGLISH.to_owned()), ENGLISH);
}

#[test]
fn the_switch_on_arranges_with_the_platform_segmenter() {
    let arranged = for_dictation(true, ENGLISH.to_owned());
    assert!(arranged.contains("\n\n"), "{arranged}");
    assert_eq!(arranged.replace("\n\n", " "), ENGLISH);
    let short = "Call me back tomorrow.".to_owned();
    assert_eq!(for_dictation(true, short.clone()), short);
}
