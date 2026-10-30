//! Apple NaturalLanguage segmentation on macOS (016 AC-007). These tests call the real system
//! framework, so they assert properties that hold for any reasonable segmentation rather than
//! Apple's exact token boundaries.

use super::super::{arrange, Arrangement};
use super::{dominant_language, natural_language_sentences, AppleSegmenter, Utf16Offsets};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn english_sentences_and_words() -> TestResult {
    let text = "We met the client today. They agreed on the budget. Great!";
    let sentences = natural_language_sentences(text).ok_or("NaturalLanguage unavailable")?;
    let words: Vec<usize> = sentences.iter().map(|sentence| sentence.words).collect();
    assert_eq!(words, [5, 5, 1]);
    assert_eq!(sentences[0].range.start, 0);
    assert_eq!(sentences[2].range.end, text.len());
    Ok(())
}

#[test]
fn the_dominant_language_is_recognized() {
    assert_eq!(
        dominant_language("Nous avons revu le budget avec l'équipe financière ce matin.")
            .as_deref(),
        Some("fr")
    );
    assert_eq!(
        dominant_language("今日は予算会議を行いました。来週また話し合います。").as_deref(),
        Some("ja")
    );
}

#[test]
fn chinese_words_are_dictionary_words_not_characters() -> TestResult {
    let text = "今天我们开会讨论了预算。下周继续讨论。";
    let sentences = natural_language_sentences(text).ok_or("NaturalLanguage unavailable")?;
    assert_eq!(sentences.len(), 2);
    let characters = text.chars().filter(|c| c.is_alphabetic()).count();
    let words: usize = sentences.iter().map(|sentence| sentence.words).sum();
    assert!(
        words > 0 && words < characters,
        "{words} words, {characters} characters"
    );
    Ok(())
}

#[test]
fn japanese_paragraphs_add_no_space() -> TestResult {
    let sentence = "本日は四半期の予算について財務チームと詳しく話し合いました。";
    let text = sentence.repeat(10);
    let Arrangement::Arranged(arranged) = arrange(&text, &AppleSegmenter) else {
        return Err("expected paragraphs".into());
    };
    assert!(!arranged.contains(' '), "{arranged}");
    assert_eq!(arranged.replace("\n\n", ""), text);
    Ok(())
}

#[test]
fn a_french_dictation_keeps_every_character() -> TestResult {
    let text = "Ce matin, nous avons revu le budget trimestriel avec l'équipe financière. Les \
        dépenses en services cloud ont augmenté plus vite que prévu. Nous avons décidé de geler \
        les nouveaux projets jusqu'en mars. Le marketing présentera un plan révisé la semaine \
        prochaine. Deux postes d'ingénieur restent ouverts. Le déménagement est repoussé.";
    let Arrangement::Arranged(arranged) = arrange(text, &AppleSegmenter) else {
        return Err("expected paragraphs".into());
    };
    assert_eq!(arranged.replace("\n\n", " "), text);
    assert!(arranged.contains("prochaine.\n\nDeux"), "{arranged}");
    Ok(())
}

#[test]
fn utf16_offsets_map_to_byte_boundaries() {
    let offsets = Utf16Offsets::of("é😀a");
    assert_eq!(offsets.byte(0), Some(0));
    assert_eq!(offsets.byte(1), Some(2));
    assert_eq!(offsets.byte(2), None);
    assert_eq!(offsets.byte(3), Some(6));
    assert_eq!(offsets.byte(4), Some(7));
    assert_eq!(offsets.byte(5), None);
}
