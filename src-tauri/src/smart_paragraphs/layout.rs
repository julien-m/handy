//! Paragraph rules of feature 016 over segmented sentences: where to break, and how a break is
//! written into the text.

use super::quotes::QuoteBalance;
use super::Sentence;

/// Running word count that closes a paragraph at the next sentence boundary (public behaviour,
/// spec source S2).
pub(crate) const WORD_LIMIT: usize = 45;
/// Number of substantial sentences that closes a paragraph (S2).
pub(crate) const SENTENCE_LIMIT: usize = 4;
/// Minimum word count of a substantial sentence; shorter ones (interjections such as "Yes." or
/// "OK.") add their words but are not counted as sentences (S2).
pub(crate) const SUBSTANTIAL_MIN_WORDS: usize = 4;
/// Paragraph separator: one blank line (S2).
pub(crate) const PARAGRAPH_SEPARATOR: &str = "\n\n";

// @spec AC-001: 45 words at a sentence boundary, AC-002: 4 substantial sentences, AC-006: Not in a quotation — .specs/features/016-smart-paragraphs/spec.md#ac-001
/// Byte offsets of `text` where a paragraph closes (each one a sentence boundary).
///
/// Strategy: accumulate the words and the substantial sentences of the current paragraph; after
/// each sentence but the last, close the paragraph when a limit is reached and every quotation
/// opened so far is closed. A limit reached inside a quotation keeps the paragraph open until
/// the first later boundary outside it, so a quotation is never split. Closing marks that a
/// segmenter left at the start of the next sentence (French « … » with a space before », where
/// UAX #29 breaks before the ») are taken into the boundary first.
pub(crate) fn plan_breaks(text: &str, sentences: &[Sentence]) -> Vec<usize> {
    let mut breaks = Vec::new();
    let mut quotes = QuoteBalance::default();
    let (mut words, mut substantial) = (0usize, 0usize);
    let mut fed = 0;
    for (index, sentence) in sentences.iter().enumerate() {
        let Some(next) = sentences.get(index + 1) else {
            // The last sentence closes the text, never a paragraph followed by nothing.
            break;
        };
        let boundary = next.range.start.max(fed);
        quotes.feed(&text[fed..boundary]);
        let boundary = boundary + quotes.feed_leading_closers(&text[boundary..]);
        fed = boundary;
        words += sentence.words;
        substantial += usize::from(sentence.words >= SUBSTANTIAL_MIN_WORDS);
        let limit_reached = words >= WORD_LIMIT || substantial >= SENTENCE_LIMIT;
        if limit_reached && quotes.is_closed() {
            breaks.push(boundary);
            (words, substantial) = (0, 0);
        }
    }
    breaks
}

// @spec FR-002: Blank line replaces only the gap, AC-003: Spacing kept — .specs/features/016-smart-paragraphs/spec.md#fr-002
/// `text` with a blank line at each boundary of `breaks` (increasing byte offsets).
///
/// Only the whitespace around the boundary is replaced (none in Chinese or Japanese, so nothing
/// is added there but the line break); every other character is kept.
pub(crate) fn apply_breaks(text: &str, breaks: &[usize]) -> String {
    let mut arranged = String::with_capacity(text.len() + breaks.len() * 2);
    let mut copied = 0;
    for &boundary in breaks {
        let content_end = text[..boundary].trim_end().len().max(copied);
        let gap = &text[boundary..];
        let next_start = boundary + (gap.len() - gap.trim_start().len());
        arranged.push_str(&text[copied..content_end]);
        arranged.push_str(PARAGRAPH_SEPARATOR);
        copied = next_start;
    }
    arranged.push_str(&text[copied..]);
    arranged
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
