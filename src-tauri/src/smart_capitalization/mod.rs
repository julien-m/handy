//! Smart capitalization (feature 015): the first letter of a dictation is adapted to the text
//! before the cursor in the destination field: a capital where a field, a line, a sentence or a
//! list item starts, a lower-case letter where a sentence goes on, and the case received when
//! the context is absent or does not decide. Independent design (clean room): see the spec's
//! "Independent design" and "Provenance". Everything runs locally.
//!
//! - `decision`: pure context → [`CaseDecision`].
//! - `transform`: pure first-letter adaptation with protections and Unicode mappings.
//! - `reader`: bounded context read behind an accessibility port; `macos`: its adapter.
//! - `app`: Tauri glue at the final preparation of the insertion.

pub mod app;
pub mod decision;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod reader;
pub mod transform;

pub use decision::{decide, CaseDecision, CursorContext};
pub use transform::{adapt_first_letter, CaseInputs};

// @spec AC-009: Preparation order and smart spacing slot — .specs/features/015-smart-capitalization/spec.md#ac-009
/// The final preparation of an inserted text, given its destination `context`: the ordered
/// formatting steps, each a pure function of the text and the context.
///
/// Order: capitalization first. Extension point: the future smart spacing step (a separate
/// feature) is added after it, here, receiving the same `context`, so a space it adds never
/// changes the capitalization decision.
pub fn prepare_text(text: &str, context: &CursorContext, inputs: &CaseInputs<'_>) -> String {
    adapt_first_letter(text, decide(context), inputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_context_changes_nothing() {
        let inputs = CaseInputs::default();
        for text in ["Hello there", "hello there", "API", "東京"] {
            assert_eq!(prepare_text(text, &CursorContext::Absent, &inputs), text);
        }
    }

    #[test]
    fn preparation_adapts_from_the_context() {
        let inputs = CaseInputs::default();
        let continuation = CursorContext::Before {
            text: "I bought apples, ".to_owned(),
            reaches_start: true,
        };
        assert_eq!(
            prepare_text("Pears and plums", &continuation, &inputs),
            "pears and plums"
        );
        let empty = CursorContext::Before {
            text: String::new(),
            reaches_start: true,
        };
        assert_eq!(prepare_text("hello", &empty, &inputs), "Hello");
    }
}
