//! Sentence and word segmentation port of smart paragraphs (feature 016 AC-007), and the
//! segmenter of this platform.

use std::ops::Range;

/// One sentence of a text: its byte range (the next sentence starts where it ends, spaces
/// included or not, depending on the segmenter) and its word count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Sentence {
    pub range: Range<usize>,
    pub words: usize,
}

/// Splits a text into sentences with their word counts.
pub(crate) trait Segmenter {
    /// The sentences of `text` in order, or `None` when this segmenter does not support the
    /// text (the text is then left unchanged).
    fn segment(&self, text: &str) -> Option<Vec<Sentence>>;
}

// @spec FR-004: NaturalLanguage on macOS, UAX #29 elsewhere — .specs/features/016-smart-paragraphs/spec.md#fr-004
/// The segmenter of this platform: Apple NaturalLanguage on macOS (falling back to UAX #29 if
/// the framework cannot be used), UAX #29 elsewhere. Both run locally.
pub(crate) fn platform_segmenter() -> Box<dyn Segmenter> {
    #[cfg(target_os = "macos")]
    return Box::new(super::apple_segmenter::AppleSegmenter);
    #[cfg(not(target_os = "macos"))]
    return Box::new(super::unicode_segmenter::UnicodeSegmenter);
}
