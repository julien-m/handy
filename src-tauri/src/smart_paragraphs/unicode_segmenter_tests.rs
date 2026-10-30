//! UAX #29 segmentation outside macOS (016 AC-007).

use super::super::Segmenter;
use super::{is_mostly_chinese_or_japanese, sentences, UnicodeSegmenter};

#[test]
fn sentences_cover_the_text_and_count_words_without_punctuation() {
    let text = "We met today. Did it go well? Yes!";
    let found = sentences(text);
    let words: Vec<usize> = found.iter().map(|sentence| sentence.words).collect();
    assert_eq!(words, [3, 4, 1]);
    assert_eq!(found[0].range.start, 0);
    assert_eq!(found[2].range.end, text.len());
    assert_eq!(&text[found[1].range.clone()], "Did it go well? ");
}

#[test]
fn french_elisions_and_accents_are_words() {
    let found = sentences("L'équipe a validé le budget. C'est réglé.");
    let words: Vec<usize> = found.iter().map(|sentence| sentence.words).collect();
    assert_eq!(words, [5, 2]);
}

#[test]
fn chinese_and_japanese_are_not_supported_outside_macos() {
    assert!(is_mostly_chinese_or_japanese(
        "今天我们开会了。预算已经确认。"
    ));
    assert!(is_mostly_chinese_or_japanese(
        "今日は会議でした。予算は承認されました。"
    ));
    assert_eq!(
        UnicodeSegmenter.segment("今天我们开会了。预算已经确认。"),
        None
    );
}

#[test]
fn latin_text_with_a_few_ideographs_and_korean_are_supported() {
    assert!(!is_mostly_chinese_or_japanese(
        "We ordered the 茶 set for the meeting room."
    ));
    assert!(!is_mostly_chinese_or_japanese(
        "오늘 회의를 했습니다. 예산이 확정되었습니다."
    ));
    assert!(UnicodeSegmenter.segment("오늘 회의를 했습니다.").is_some());
}
