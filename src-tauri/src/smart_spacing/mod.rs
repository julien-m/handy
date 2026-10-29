//! Smart spacing (feature 017): the spaces at the two edges of a dictation are fitted to the
//! text around the selection it replaces, so the inserted words never merge with the field's
//! words and spaces never pile up. Only the horizontal whitespace at the two edges of the
//! dictation may change; its words, punctuation and line breaks never do. Independent design
//! (clean room): see the spec's "Independent design" and "Provenance". Everything runs locally.
//!
//! - `junction`: pure rule deciding one junction (space or not).
//! - this module: the two edges of the dictation for an [`InsertionContext`].
//!
//! It runs after smart capitalization (015) in `smart_capitalization::prepare_text`, the
//! extension point of the insertion's final preparation.
pub mod junction;

use crate::smart_capitalization::context::{AfterContext, InsertionContext};
use crate::smart_capitalization::decision::{is_horizontal_space, CursorContext};
pub use junction::{is_unspaced_script, needs_space, suits_trailing_space};

/// The one space smart spacing may add at an edge.
const SPACE: &str = " ";

/// What smart spacing needs besides the text and the context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpacingInputs {
    /// `append_trailing_space`: a dictation that ends the field gets one trailing space (spec
    /// decision `trailing_space_articulation`).
    pub space_at_field_end: bool,
}

/// A dictation with its edges fitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spaced {
    pub text: String,
    /// True when smart spacing decided the end of the text: the insertion method must then
    /// add no trailing space of its own (`append_trailing_space`), so there is never a second.
    pub trailing_decided: bool,
}

// @spec FR-002: Edges fitted to the context — .specs/features/017-smart-spacing/spec.md#fr-002
/// `text` with its leading and trailing horizontal whitespace fitted to `context`.
///
/// Business rules (spec AC-002 to AC-007): a text of whitespace only, or a focus that is not
/// a text field, is left as is and the method keeps its own trailing space; an unavailable
/// context deduces nothing and only secures one final space; a read field fits each edge whose
/// side was read, and falls back to the final space when only the side after is unknown.
pub fn space_junctions(text: &str, context: &InsertionContext, inputs: SpacingInputs) -> Spaced {
    if text.trim().is_empty() || *context == InsertionContext::NotText {
        return Spaced {
            text: text.to_owned(),
            trailing_decided: false,
        };
    }
    let (lead, body) = match context {
        InsertionContext::Field { before, .. } => leading_edge(text, before),
        InsertionContext::Unavailable | InsertionContext::NotText => ("", text),
    };
    let (body, tail) = match context {
        InsertionContext::Field { after, .. } => trailing_edge(body, after, inputs),
        InsertionContext::Unavailable | InsertionContext::NotText => (body, fallback_tail(body)),
    };
    Spaced {
        text: [lead, body, tail].concat(),
        trailing_decided: true,
    }
}

// @spec AC-002: Leading edge, AC-007: Empty field — .specs/features/017-smart-spacing/spec.md#ac-002
/// The space before `text` and its body without leading horizontal whitespace, from the text
/// `before` the selection. An empty field (or the field start) needs none.
fn leading_edge<'a>(text: &'a str, before: &CursorContext) -> (&'static str, &'a str) {
    let CursorContext::Before {
        text: field,
        reaches_start,
    } = before
    else {
        return ("", text);
    };
    // Nothing read that does not reach the field start proves nothing: the edge is kept.
    if field.is_empty() && !reaches_start {
        return ("", text);
    }
    let body = text.trim_start_matches(is_horizontal_space);
    (if needs_space(field, body) { SPACE } else { "" }, body)
}

// @spec AC-005: Trailing edge — .specs/features/017-smart-spacing/spec.md#ac-005
/// The body of `text` and the space after it, from the text `after` the selection: the
/// junction rule against the following text; at the end of the field, the user's
/// `append_trailing_space`; when the side after is unknown, the fallback.
fn trailing_edge<'a>(
    text: &'a str,
    after: &AfterContext,
    inputs: SpacingInputs,
) -> (&'a str, &'static str) {
    let AfterContext::After {
        text: field,
        reaches_end,
    } = after
    else {
        return (text, fallback_tail(text));
    };
    let body = text.trim_end_matches(is_horizontal_space);
    let space = match (field.is_empty(), *reaches_end) {
        (false, _) => needs_space(body, field),
        (true, true) => inputs.space_at_field_end && suits_trailing_space(body),
        // An empty answer that does not reach the end tells nothing: the fallback applies.
        (true, false) => return (text, fallback_tail(text)),
    };
    (body, if space { SPACE } else { "" })
}

// @spec AC-006: Unavailable context fallback — .specs/features/017-smart-spacing/spec.md#ac-006
/// Fallback without context: one final space, unless `text` already ends with whitespace or
/// its language is written without spaces; nothing else is deduced.
fn fallback_tail(text: &str) -> &'static str {
    if suits_trailing_space(text) {
        SPACE
    } else {
        ""
    }
}

#[cfg(test)]
#[path = "spacing_tests.rs"]
mod tests;
