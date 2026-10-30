//! Smart capitalization (feature 015): the first letter of a dictation is adapted to the text
//! before the cursor in the destination field: a capital where a field, a line, a sentence or a
//! list item starts, a lower-case letter where a sentence goes on, and the case received when
//! the context is absent or does not decide. Independent design (clean room): see the spec's
//! "Independent design" and "Provenance". Everything runs locally.
//!
//! This module also holds the insertion's final preparation (`prepare_text`), whose second
//! step is smart spacing (017, module `smart_spacing`), and the context read both share.
//!
//! - `context`: what the read found around the selection (before it, and after it for 017).
//! - `decision`: pure context → [`CaseDecision`].
//! - `transform`: pure first-letter adaptation with protections and Unicode mappings.
//! - `reader`: bounded context read behind an accessibility port; `macos`: its adapter.
//! - `app`: Tauri glue at the final preparation of the insertion.

pub mod app;
pub mod context;
pub mod decision;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod reader;
pub mod transform;

pub use context::{AfterContext, InsertionContext};
pub use decision::{decide, CaseDecision, CursorContext};
pub use transform::{adapt_first_letter, CaseInputs};

use crate::smart_spacing::{space_junctions, SpacingInputs};

/// The formatting steps of one insertion, each `None` when its option is off.
#[derive(Clone, Copy, Debug, Default)]
pub struct Preparation<'a> {
    /// Smart capitalization (015).
    pub capitalization: Option<CaseInputs<'a>>,
    /// Smart spacing (017).
    pub spacing: Option<SpacingInputs>,
}

/// The text an insertion receives after its final preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedText {
    /// The text every paste method inserts.
    pub text: String,
    /// The text before smart spacing (capitalization applied): what 013's addressed terminal
    /// route types instead of `text` (017 AC-015).
    pub unspaced: String,
    /// Smart capitalization changed the first letter (logged as a flag only).
    pub case_changed: bool,
    /// Smart spacing decided the end of the text: the method adds no trailing space (017).
    pub trailing_decided: bool,
}

impl PreparedText {
    /// `text` as received: no step ran.
    pub fn unchanged(text: String) -> Self {
        Self {
            unspaced: text.clone(),
            text,
            case_changed: false,
            trailing_decided: false,
        }
    }

    // @spec AC-015: Addressed terminal unspaced — .specs/features/017-smart-spacing/spec.md#ac-015
    /// Business rule (017 AC-015): the text 013's terminal route types through its addressing
    /// tool (`cmux send`, no paste). A terminal is not a text field, so it gets no spacing
    /// adjustment at all, not even the fallback final space, whatever application is in front;
    /// that route never adds a trailing space of its own, so this is the output with smart
    /// spacing off.
    pub fn for_addressed_terminal(&self) -> &str {
        &self.unspaced
    }
}

// @spec AC-009: Preparation order and smart spacing slot — .specs/features/015-smart-capitalization/spec.md#ac-009
// @spec AC-009: After capitalization, same context — .specs/features/017-smart-spacing/spec.md#ac-009
/// The final preparation of an inserted text, given its destination `context`: the ordered
/// formatting steps, each a pure function of the text and the context.
///
/// Order matters: capitalization first, from the text before the cursor; smart spacing (017)
/// second, on the capitalized text and with the same `context`, so a space it adds never
/// changes the capitalization decision and the first letter it sees is final.
pub fn prepare_text(
    text: &str,
    context: &InsertionContext,
    steps: &Preparation<'_>,
) -> PreparedText {
    let capitalized = match &steps.capitalization {
        Some(inputs) => adapt_first_letter(text, decide(context.before()), inputs),
        None => text.to_owned(),
    };
    let case_changed = capitalized != text;
    let Some(inputs) = steps.spacing else {
        return PreparedText {
            unspaced: capitalized.clone(),
            text: capitalized,
            case_changed,
            trailing_decided: false,
        };
    };
    let spaced = space_junctions(&capitalized, context, inputs);
    PreparedText {
        text: spaced.text,
        unspaced: capitalized,
        case_changed,
        trailing_decided: spaced.trailing_decided,
    }
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;
