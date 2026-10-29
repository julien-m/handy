//! Smart paragraphs (feature 016): split a long dictation into paragraphs at sentence
//! boundaries, without any generative model and without leaving the computer.
//!
//! The rules (`layout`) are pure and run over the sentences of a [`Segmenter`]: Apple
//! NaturalLanguage on macOS, Unicode UAX #29 elsewhere. Structured or already broken text is
//! left unchanged (`structure`), and no break is placed inside an open quotation (`quotes`).
//! The dictation output stages call [`for_dictation`] after AI post-processing and before the
//! insertion's final preparation (see `dictation_output`).

#[cfg(target_os = "macos")]
mod apple_segmenter;
mod layout;
mod quotes;
mod segmenter;
mod structure;
#[cfg(test)]
mod test_text;
mod unicode_segmenter;

pub(crate) use segmenter::{platform_segmenter, Segmenter, Sentence};
pub(crate) use structure::Structure;

/// Why a text is returned unchanged; logged without any text content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Preserved {
    /// Empty or whitespace only.
    Empty,
    /// The user's own layout (line breaks, list, code) must not be reorganized.
    Structured(Structure),
    /// The segmenter does not support this text (Chinese or Japanese outside macOS).
    UnsupportedText,
    /// No limit was reached at a sentence boundary: short text or one long sentence.
    NoBreak,
}

/// Result of [`arrange`]: the text with its paragraph breaks, or why it is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Arrangement {
    Arranged(String),
    Preserved(Preserved),
}

// @spec FR-001: Paragraphs at sentence boundaries, FR-003: Keep structure, AC-004: Short text unchanged — .specs/features/016-smart-paragraphs/spec.md#fr-001
/// Arrange `text` into paragraphs with the sentences of `segmenter`.
///
/// Order matters: the structure check runs before segmentation, so a deliberate layout is never
/// handed to a segmenter; then the layout rules decide the breaks.
pub(crate) fn arrange(text: &str, segmenter: &dyn Segmenter) -> Arrangement {
    if text.trim().is_empty() {
        return Arrangement::Preserved(Preserved::Empty);
    }
    if let Some(structure) = structure::detect(text) {
        return Arrangement::Preserved(Preserved::Structured(structure));
    }
    let Some(sentences) = segmenter.segment(text) else {
        return Arrangement::Preserved(Preserved::UnsupportedText);
    };
    arrange_sentences(text, &sentences)
}

/// Apply the paragraph rules to `sentences`, already segmented from `text`.
pub(crate) fn arrange_sentences(text: &str, sentences: &[Sentence]) -> Arrangement {
    let breaks = layout::plan_breaks(text, sentences);
    if breaks.is_empty() {
        return Arrangement::Preserved(Preserved::NoBreak);
    }
    Arrangement::Arranged(layout::apply_breaks(text, &breaks))
}

// @spec FR-005: Step of the dictation output stages — .specs/features/016-smart-paragraphs/spec.md#fr-005
/// The text a dictation hands to insertion: `text` arranged when `enabled` (the dictation's
/// `smart_paragraphs_enabled`), otherwise `text` unchanged.
pub(crate) fn for_dictation(enabled: bool, text: String) -> String {
    if !enabled {
        return text;
    }
    match arrange(&text, platform_segmenter().as_ref()) {
        Arrangement::Arranged(arranged) => arranged,
        Arrangement::Preserved(reason) => {
            log::debug!("Smart paragraphs left the text unchanged: {reason:?}");
            text
        }
    }
}

#[cfg(test)]
#[path = "arrange_tests.rs"]
mod arrange_tests;
