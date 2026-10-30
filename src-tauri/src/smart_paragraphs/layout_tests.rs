//! Paragraph rules (016 AC-001 to AC-004, AC-006) over UAX #29 sentences.

use super::super::test_text::{paragraph_words, prose, sentence};
use super::super::unicode_segmenter::sentences;
use super::super::{arrange_sentences, Arrangement, Preserved};
use super::{SENTENCE_LIMIT, SUBSTANTIAL_MIN_WORDS, WORD_LIMIT};

fn arranged(text: &str) -> Arrangement {
    arrange_sentences(text, &sentences(text))
}
type TestResult = Result<(), String>;

/// The arranged text, or an error naming why the text was kept.
fn arranged_text(text: &str) -> Result<String, String> {
    match arranged(text) {
        Arrangement::Arranged(text) => Ok(text),
        Arrangement::Preserved(reason) => Err(format!("expected paragraphs, kept: {reason:?}")),
    }
}

#[test]
fn limits_are_those_of_the_public_reference() {
    assert_eq!(
        (WORD_LIMIT, SENTENCE_LIMIT, SUBSTANTIAL_MIN_WORDS),
        (45, 4, 4)
    );
}

#[test]
fn below_the_word_limit_nothing_breaks() {
    // 20 + 24 = 44 words in two substantial sentences, then the text ends.
    let text = prose(&[20, 24]);
    assert_eq!(arranged(&text), Arrangement::Preserved(Preserved::NoBreak));
}

#[test]
fn exactly_the_word_limit_breaks_after_that_sentence() -> TestResult {
    let text = prose(&[20, 25, 6]);
    assert_eq!(paragraph_words(&arranged_text(&text)?), [45, 6]);
    Ok(())
}

#[test]
fn the_break_falls_at_the_next_sentence_boundary_past_the_limit() -> TestResult {
    // 44 words after two sentences: the paragraph closes after the third, at 49 words.
    let text = prose(&[20, 24, 5, 7]);
    assert_eq!(paragraph_words(&arranged_text(&text)?), [49, 7]);
    Ok(())
}

#[test]
fn four_substantial_sentences_close_a_paragraph() -> TestResult {
    let text = prose(&[5, 5, 5, 5, 5, 5]);
    assert_eq!(paragraph_words(&arranged_text(&text)?), [20, 10]);
    Ok(())
}

#[test]
fn short_interjections_add_words_but_do_not_count_as_sentences() -> TestResult {
    let text = format!("Yes. OK. Right. {}", prose(&[5, 5, 5, 5, 6]));
    let paragraphs = arranged_text(&text)?;
    // 3 interjection words + 4 substantial sentences of 5 words.
    assert_eq!(paragraph_words(&paragraphs), [23, 6]);
    assert!(paragraphs.starts_with("Yes. OK. Right. "));
    Ok(())
}

#[test]
fn a_three_word_sentence_is_not_substantial_but_a_four_word_one_is() -> TestResult {
    // Three words: four more substantial sentences are needed.
    let three = prose(&[3, 5, 5, 5, 5, 6]);
    assert_eq!(paragraph_words(&arranged_text(&three)?), [3 + 5 * 4, 6]);
    // Four words: it is the first of the four.
    let four = prose(&[4, 5, 5, 5, 6]);
    assert_eq!(paragraph_words(&arranged_text(&four)?), [4 + 5 * 3, 6]);
    Ok(())
}

#[test]
fn a_lone_long_sentence_stays_whole() -> TestResult {
    let text = sentence(80);
    assert_eq!(arranged(&text), Arrangement::Preserved(Preserved::NoBreak));
    let text = prose(&[80, 6]);
    // The long sentence closes its paragraph; it is never cut.
    assert_eq!(paragraph_words(&arranged_text(&text)?), [80, 6]);
    Ok(())
}

#[test]
fn a_short_dictation_is_unchanged() {
    let text = "Call me back tomorrow. Thanks.";
    assert_eq!(arranged(text), Arrangement::Preserved(Preserved::NoBreak));
}

#[test]
fn the_last_sentence_never_opens_an_empty_paragraph() {
    let text = prose(&[5, 5, 5, 5]);
    assert_eq!(arranged(&text), Arrangement::Preserved(Preserved::NoBreak));
}

#[test]
fn a_break_is_one_blank_line_and_every_other_character_is_kept() -> TestResult {
    let first = prose(&[5, 5, 5, 5]);
    let rest = sentence(6);
    let text = format!("  {first}   {rest}  ");
    assert_eq!(arranged_text(&text)?, format!("  {first}\n\n{rest}  "));
    Ok(())
}

#[test]
fn spacing_inside_a_paragraph_is_kept() -> TestResult {
    let text = format!("{}  {}", prose(&[5, 5, 5, 5]), sentence(6)).replacen(". ", ".  ", 1);
    let paragraphs = arranged_text(&text)?;
    assert_eq!(paragraphs.replace("\n\n", "  "), text);
    Ok(())
}

#[test]
fn a_quotation_is_never_split() -> TestResult {
    let quoted = format!("\u{201c}{}\u{201d}", prose(&[5, 5, 5]));
    let text = format!(
        "{} {} {} {}",
        prose(&[5, 5]),
        quoted,
        sentence(5),
        sentence(6)
    );
    let paragraphs = arranged_text(&text)?;
    let opening = paragraphs.find('\u{201c}').unwrap_or(0);
    let closing = paragraphs.find('\u{201d}').unwrap_or(0);
    assert!(
        !paragraphs[opening..closing].contains("\n\n"),
        "{paragraphs}"
    );
    // The limit is reached inside the quotation; the paragraph closes right after it.
    assert_eq!(paragraph_words(&paragraphs), [25, 5 + 6]);
    Ok(())
}

#[test]
fn french_guillemets_are_quotations_too() -> TestResult {
    // The fourth substantial sentence ends inside the quotation: the break waits for « » to close.
    let text = "Il a dit : « Nous signons le contrat demain matin. Les équipes sont prêtes depuis \
                lundi. Le client attend notre appel ce soir. Nous devons tout relire avant. Rien \
                ne doit manquer au dossier. » Puis il est parti très vite.";
    let paragraphs = arranged_text(text)?;
    let opening = paragraphs.find('«').unwrap_or(0);
    let closing = paragraphs.find('»').unwrap_or(0);
    assert!(
        !paragraphs[opening..closing].contains("\n\n"),
        "{paragraphs}"
    );
    assert!(
        paragraphs.ends_with("dossier. »\n\nPuis il est parti très vite."),
        "{paragraphs}"
    );
    Ok(())
}

#[test]
fn an_unclosed_quotation_keeps_the_rest_in_one_paragraph() {
    let text = format!("\"{}", prose(&[5, 5, 5, 5, 5, 5]));
    assert_eq!(arranged(&text), Arrangement::Preserved(Preserved::NoBreak));
}

#[test]
fn several_paragraphs_each_close_at_their_own_limit() -> TestResult {
    let text = prose(&[5; 12]);
    assert_eq!(paragraph_words(&arranged_text(&text)?), [20, 20, 20]);
    Ok(())
}
