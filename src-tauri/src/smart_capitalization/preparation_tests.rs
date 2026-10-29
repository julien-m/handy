//! The insertion's final preparation: capitalization (015) then smart spacing (017), each step
//! only when its option is on, both from one context.
use super::*;

fn field(before: &str, after: &str) -> InsertionContext {
    InsertionContext::Field {
        before: CursorContext::Before {
            text: before.to_owned(),
            reaches_start: true,
        },
        after: AfterContext::After {
            text: after.to_owned(),
            reaches_end: true,
        },
    }
}

const BOTH: Preparation<'static> = Preparation {
    capitalization: Some(CaseInputs {
        case_decided: false,
        protected: &[],
    }),
    spacing: Some(SpacingInputs {
        space_at_field_end: false,
    }),
};

#[test]
fn absent_context_changes_nothing() {
    let capitalization_only = Preparation {
        capitalization: BOTH.capitalization,
        spacing: None,
    };
    for text in ["Hello there", "hello there", "API", "東京"] {
        let prepared = prepare_text(text, &InsertionContext::Unavailable, &capitalization_only);
        assert_eq!(prepared, PreparedText::unchanged(text.to_owned()));
    }
}

#[test]
fn preparation_adapts_from_the_context() {
    let steps = Preparation {
        capitalization: BOTH.capitalization,
        spacing: None,
    };
    let continuation = field("I bought apples, ", "");
    let prepared = prepare_text("Pears and plums", &continuation, &steps);
    assert_eq!(prepared.text, "pears and plums");
    assert!(prepared.case_changed);
    assert_eq!(prepare_text("hello", &field("", ""), &steps).text, "Hello");
}

// @spec AC-012: Reference example "Je souhaite" — .specs/features/017-smart-spacing/spec.md#ac-012
#[test]
fn reference_example_je_souhaite_with_both_steps() {
    let context = field("Je souhaite", "");
    let prepared = prepare_text("Vérifier ce point.", &context, &BOTH);
    assert_eq!(prepared.text, " vérifier ce point.");
    assert_eq!(
        format!("Je souhaite{}", prepared.text),
        "Je souhaite vérifier ce point."
    );
    let with_end_space = Preparation {
        spacing: Some(SpacingInputs {
            space_at_field_end: true,
        }),
        ..BOTH
    };
    assert_eq!(
        prepare_text("Vérifier ce point.", &context, &with_end_space).text,
        " vérifier ce point. "
    );
}

#[test]
fn reference_example_bonjour_demain_with_both_steps() {
    let prepared = prepare_text("À tous", &field("Bonjour", "demain"), &BOTH);
    assert_eq!(
        format!("Bonjour{}demain", prepared.text),
        "Bonjour à tous demain"
    );
}

#[test]
fn spacing_alone_keeps_the_case() {
    let steps = Preparation {
        capitalization: None,
        spacing: BOTH.spacing,
    };
    let prepared = prepare_text("Vérifier", &field("Je souhaite", ""), &steps);
    assert_eq!(prepared.text, " Vérifier");
    assert!(!prepared.case_changed);
    assert!(prepared.trailing_decided);
}

#[test]
fn spacing_off_never_decides_the_trailing_space() {
    let steps = Preparation {
        capitalization: BOTH.capitalization,
        spacing: None,
    };
    let prepared = prepare_text("word", &field("a", "b"), &steps);
    assert_eq!(prepared.text, "word");
    assert!(!prepared.trailing_decided);
}

#[test]
fn capitalization_decides_before_the_space_is_added() {
    // The space spacing adds after "It works." is not seen by capitalization: it reads the
    // field, which ends with a terminator, so the capital stays.
    let prepared = prepare_text("now test it", &field("It works.", ""), &BOTH);
    assert_eq!(prepared.text, " Now test it");
}

// @spec AC-015: Addressed terminal unspaced — .specs/features/017-smart-spacing/spec.md#ac-015
#[test]
fn an_addressed_terminal_gets_no_spacing_whatever_the_context() {
    for context in [
        InsertionContext::Unavailable,
        field("Bonjour", "demain"),
        InsertionContext::NotText,
    ] {
        let prepared = prepare_text(" ls -la ", &context, &BOTH);
        assert_eq!(prepared.for_addressed_terminal(), " ls -la ", "{context:?}");
    }
    // The paste text of an unavailable context carries the fallback space; the terminal not.
    let prepared = prepare_text("ls -la", &InsertionContext::Unavailable, &BOTH);
    assert_eq!(prepared.text, "ls -la ");
    assert_eq!(prepared.for_addressed_terminal(), "ls -la");
    // Same output as with smart spacing off.
    let off = Preparation {
        spacing: None,
        ..BOTH
    };
    let without = prepare_text("ls -la", &InsertionContext::Unavailable, &off);
    assert_eq!(prepared.for_addressed_terminal(), without.text);
    assert_eq!(
        PreparedText::unchanged("ls".to_owned()).for_addressed_terminal(),
        "ls"
    );
}

#[test]
fn not_a_text_field_is_left_to_the_method() {
    let prepared = prepare_text("ls -la", &InsertionContext::NotText, &BOTH);
    assert_eq!(prepared, PreparedText::unchanged("ls -la".to_owned()));
}

// @spec AC-017: Final period, option and route — .specs/features/017-smart-spacing/spec.md#ac-017
#[test]
fn n30_final_period_follows_capitalization_and_the_option() {
    let context = field("Bonjour", ", ça va");
    let prepared = prepare_text("Tout le monde.", &context, &BOTH);
    assert_eq!(
        format!("Bonjour{}, ça va", prepared.text),
        "Bonjour tout le monde, ça va"
    );
    let spacing_off = Preparation {
        spacing: None,
        ..BOTH
    };
    assert_eq!(
        prepare_text("Tout le monde.", &context, &spacing_off).text,
        "tout le monde."
    );
}

#[test]
fn n30_addressed_terminal_keeps_the_final_period() {
    let prepared = prepare_text("ls.", &field("x", "y"), &BOTH);
    assert_eq!(prepared.text, " ls ");
    assert_eq!(prepared.for_addressed_terminal(), "ls.");
}
