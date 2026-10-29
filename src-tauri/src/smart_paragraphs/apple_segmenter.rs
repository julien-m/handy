//! Apple NaturalLanguage segmentation for smart paragraphs on macOS (feature 016 AC-007), through
//! the public `NLLanguageRecognizer` and `NLTokenizer` APIs (spec source S3).
//!
//! The classes are looked up at run time and messaged through `objc2`; when a class or a call
//! is unavailable the segmenter falls back to UAX #29, so the step never fails because of the
//! framework. Everything runs locally.

use objc2::msg_send;
use objc2::rc::{autoreleasepool, Allocated, Retained};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::{NSArray, NSRange, NSString, NSValue};

use super::unicode_segmenter::UnicodeSegmenter;
use super::{Segmenter, Sentence};

// System contract: links the public NaturalLanguage framework (macOS 10.14+), whose
// `NLTokenizer` and `NLLanguageRecognizer` classes are looked up by name below.
#[link(name = "NaturalLanguage", kind = "framework")]
extern "C" {}

/// `NLTokenUnitWord` of `NLTokenUnit` (S3).
const UNIT_WORD: isize = 0;
/// `NLTokenUnitSentence` of `NLTokenUnit` (S3).
const UNIT_SENTENCE: isize = 1;

/// Sentences and words from NaturalLanguage in the dominant language of the text.
pub(crate) struct AppleSegmenter;

impl Segmenter for AppleSegmenter {
    fn segment(&self, text: &str) -> Option<Vec<Sentence>> {
        match natural_language_sentences(text) {
            Some(sentences) => Some(sentences),
            None => {
                log::warn!("NaturalLanguage unavailable for smart paragraphs; using UAX #29");
                UnicodeSegmenter.segment(text)
            }
        }
    }
}

#[cfg(test)]
/// The dominant language of `text` by `NLLanguageRecognizer` (a BCP 47 code such as "fr" or
/// "ja"), or `None` when it is undetermined or the class is unavailable. Test-only view of the
/// language that [`natural_language_sentences`] hands to its tokenizers.
pub(crate) fn dominant_language(text: &str) -> Option<String> {
    autoreleasepool(|_| dominant_language_code(&NSString::from_str(text)).map(|c| c.to_string()))
}

/// Sentences of `text` with their word counts, or `None` when NaturalLanguage cannot be used.
///
/// Strategy: one sentence tokenizer and one word tokenizer over the whole text, both set to the
/// dominant language; each word token is attributed to the sentence containing its start. Token
/// ranges are UTF-16 offsets, converted to byte offsets of `text`.
pub(crate) fn natural_language_sentences(text: &str) -> Option<Vec<Sentence>> {
    autoreleasepool(|_| {
        let string = NSString::from_str(text);
        let language = dominant_language_code(&string);
        let whole = NSRange::new(0, string.length());
        let sentence_ranges = tokens(UNIT_SENTENCE, &string, language.as_deref(), whole)?;
        let word_ranges = tokens(UNIT_WORD, &string, language.as_deref(), whole)?;
        let offsets = Utf16Offsets::of(text);
        let mut sentences = Vec::with_capacity(sentence_ranges.len());
        for range in &sentence_ranges {
            let end = range.location + range.length;
            let words = word_ranges
                .iter()
                .filter(|word| (range.location..end).contains(&word.location))
                .count();
            let range = offsets.byte(range.location)?..offsets.byte(end)?;
            if !text[range.clone()].trim().is_empty() {
                sentences.push(Sentence { range, words });
            }
        }
        Some(sentences)
    })
}

fn dominant_language_code(string: &NSString) -> Option<Retained<NSString>> {
    let class = AnyClass::get(c"NLLanguageRecognizer")?;
    // SAFETY: `+dominantLanguageForString:` takes an NSString and returns a nullable NSString
    // (`NLLanguage`); the class was found at run time.
    unsafe { msg_send![class, dominantLanguageForString: string] }
}

/// Token ranges of `unit` over `range` of `string`, with `language` as the tokenizer's language.
fn tokens(
    unit: isize,
    string: &NSString,
    language: Option<&NSString>,
    range: NSRange,
) -> Option<Vec<NSRange>> {
    let class = AnyClass::get(c"NLTokenizer")?;
    // SAFETY: `+alloc` then `-initWithUnit:` (an NSInteger `NLTokenUnit`) is the documented
    // initializer; `-setString:`, `-setLanguage:` take NSStrings and `-tokensForRange:` takes an
    // NSRange of the string and returns an NSArray of NSValue-wrapped NSRanges.
    unsafe {
        let allocated: Allocated<AnyObject> = msg_send![class, alloc];
        let tokenizer: Option<Retained<AnyObject>> = msg_send![allocated, initWithUnit: unit];
        let tokenizer = tokenizer?;
        let _: () = msg_send![&*tokenizer, setString: string];
        if let Some(language) = language {
            let _: () = msg_send![&*tokenizer, setLanguage: language];
        }
        let values: Retained<NSArray<NSValue>> = msg_send![&*tokenizer, tokensForRange: range];
        values.iter().map(|value| value.get_range()).collect()
    }
}

/// Byte offset of every UTF-16 offset of a text that falls on a character boundary.
struct Utf16Offsets(Vec<Option<usize>>);

impl Utf16Offsets {
    fn of(text: &str) -> Self {
        let mut offsets = Vec::with_capacity(text.len() + 1);
        for (byte, character) in text.char_indices() {
            offsets.push(Some(byte));
            // The second unit of a surrogate pair is not a character boundary.
            offsets.extend(std::iter::repeat_n(None, character.len_utf16() - 1));
        }
        offsets.push(Some(text.len()));
        Self(offsets)
    }
    /// The byte offset of UTF-16 offset `unit`, or `None` outside the text or inside a pair.
    fn byte(&self, unit: usize) -> Option<usize> {
        self.0.get(unit).copied().flatten()
    }
}

#[cfg(test)]
#[path = "apple_segmenter_tests.rs"]
mod tests;
