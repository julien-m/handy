//! UAX #29 segmentation (Unicode Text Segmentation, through `unicode-segmentation`) for smart
//! paragraphs outside macOS (feature 016 AC-007).

use unicode_segmentation::UnicodeSegmentation;

use super::{Segmenter, Sentence};

/// Sentences and words by the Unicode default boundaries. Chinese and Japanese are not
/// supported: UAX #29 has no dictionary word segmentation for them (each ideograph would count
/// as one word), so the word and sentence limits would not mean what they mean elsewhere.
pub(crate) struct UnicodeSegmenter;

impl Segmenter for UnicodeSegmenter {
    fn segment(&self, text: &str) -> Option<Vec<Sentence>> {
        (!is_mostly_chinese_or_japanese(text)).then(|| sentences(text))
    }
}

/// The UAX #29 sentences of `text` with their UAX #29 word counts (punctuation and spaces are
/// not words). Whitespace-only segments are dropped: they hold no sentence.
pub(crate) fn sentences(text: &str) -> Vec<Sentence> {
    text.split_sentence_bound_indices()
        .filter(|(_, sentence)| !sentence.trim().is_empty())
        .map(|(start, sentence)| Sentence {
            range: start..start + sentence.len(),
            words: sentence.unicode_words().count(),
        })
        .collect()
}

// @spec AC-007: CJK unchanged outside macOS — .specs/features/016-smart-paragraphs/spec.md#ac-007
/// True when more than half of the letters of `text` are Han, Hiragana or Katakana. Korean
/// (Hangul) separates words with spaces and is segmented normally.
pub(crate) fn is_mostly_chinese_or_japanese(text: &str) -> bool {
    let (mut letters, mut cjk) = (0usize, 0usize);
    for character in text.chars().filter(|c| c.is_alphabetic()) {
        letters += 1;
        cjk += usize::from(is_han_or_kana(character));
    }
    cjk * 2 > letters
}

/// Han ideographs (unified, extension A, compatibility, supplementary planes), Hiragana and
/// Katakana (with phonetic extensions), by their Unicode blocks.
fn is_han_or_kana(character: char) -> bool {
    matches!(
        u32::from(character),
        0x3040..=0x30FF | 0x31F0..=0x31FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF | 0xFF66..=0xFF9F | 0x20000..=0x3FFFF
    )
}

#[cfg(test)]
#[path = "unicode_segmenter_tests.rs"]
mod tests;
