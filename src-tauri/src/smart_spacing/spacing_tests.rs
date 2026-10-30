//! Edges of a dictation for every kind of context (spec AC-001 to AC-008): field start,
//! middle and end, selection replacement, unavailable, partial and empty contexts, existing
//! spaces and line breaks, punctuation, Unicode, French and scripts without spaces.
use super::*;

const NO_END_SPACE: SpacingInputs = SpacingInputs {
    space_at_field_end: false,
};
const END_SPACE: SpacingInputs = SpacingInputs {
    space_at_field_end: true,
};

/// A read field around the selection: `before` (reaching the field start when `at_start`)
/// and `after` (reaching the field end when `at_end`).
fn field(before: &str, at_start: bool, after: &str, at_end: bool) -> InsertionContext {
    InsertionContext::Field {
        before: CursorContext::Before {
            text: before.to_owned(),
            reaches_start: at_start,
        },
        after: AfterContext::After {
            text: after.to_owned(),
            reaches_end: at_end,
        },
    }
}

/// A read field whose side after the selection is unknown.
fn before_only(before: &str) -> InsertionContext {
    InsertionContext::Field {
        before: CursorContext::Before {
            text: before.to_owned(),
            reaches_start: true,
        },
        after: AfterContext::Absent,
    }
}

fn spaced(text: &str, context: &InsertionContext, inputs: SpacingInputs) -> String {
    space_junctions(text, context, inputs).text
}

// @spec AC-011: Reference example "Bonjour|demain" — .specs/features/017-smart-spacing/spec.md#ac-011
#[test]
fn reference_example_bonjour_demain() {
    let context = field("Bonjour", true, "demain", true);
    let inserted = spaced("à tous", &context, NO_END_SPACE);
    assert_eq!(inserted, " à tous ");
    assert_eq!(format!("Bonjour{inserted}demain"), "Bonjour à tous demain");
}

#[test]
fn middle_of_a_field_between_two_spaces_adds_nothing() {
    let context = field("Bonjour ", true, " demain", true);
    assert_eq!(spaced("à tous", &context, END_SPACE), "à tous");
}

#[test]
fn spaces_of_the_dictation_never_pile_up() {
    let context = field("Bonjour ", true, " demain", true);
    assert_eq!(spaced("  à tous  ", &context, END_SPACE), "à tous");
    let joined = field("Bonjour", true, "demain", true);
    assert_eq!(spaced("  à tous  ", &joined, END_SPACE), " à tous ");
}

// @spec AC-007: Empty field — .specs/features/017-smart-spacing/spec.md#ac-007
#[test]
fn empty_field_gets_no_leading_space() {
    let empty = field("", true, "", true);
    assert_eq!(spaced("Hello there", &empty, NO_END_SPACE), "Hello there");
    assert_eq!(spaced(" Hello there", &empty, NO_END_SPACE), "Hello there");
    assert_eq!(spaced("Hello there", &empty, END_SPACE), "Hello there ");
}

#[test]
fn start_of_a_field_before_text() {
    let context = field("", true, "world", false);
    assert_eq!(spaced("Hello", &context, NO_END_SPACE), "Hello ");
    let spaced_after = field("", true, " world", false);
    assert_eq!(spaced("Hello", &spaced_after, NO_END_SPACE), "Hello");
}

#[test]
fn end_of_a_field_follows_append_trailing_space() {
    let context = field("It works.", true, "", true);
    assert_eq!(spaced("Next step", &context, NO_END_SPACE), " Next step");
    assert_eq!(spaced("Next step", &context, END_SPACE), " Next step ");
    assert_eq!(spaced("Next step  ", &context, END_SPACE), " Next step ");
}

#[test]
fn short_dictation() {
    let context = field("Say", true, "now", true);
    assert_eq!(spaced("yes", &context, NO_END_SPACE), " yes ");
    assert_eq!(spaced("a", &field("", true, "", true), NO_END_SPACE), "a");
}

#[test]
fn selection_replacement_uses_both_sides_of_the_selection() {
    // "Bonjour [tout le monde] demain": the selection is replaced, its sides are read.
    let context = field("Bonjour ", true, " demain", true);
    assert_eq!(spaced("à tous", &context, END_SPACE), "à tous");
    let tight = field("Bonjour", true, "demain", true);
    assert_eq!(spaced("à tous", &tight, END_SPACE), " à tous ");
}

#[test]
fn punctuation_on_the_field_side() {
    assert_eq!(
        spaced("pears", &field("apples,", true, ".", true), END_SPACE),
        " pears"
    );
    assert_eq!(
        spaced("see", &field("(", true, ")", true), END_SPACE),
        "see"
    );
    assert_eq!(
        spaced("note", &field("[", true, "]", true), END_SPACE),
        "note"
    );
}

#[test]
fn punctuation_on_the_dictation_side() {
    // A capital after keeps the final period (AC-017); a lower-case word drops it.
    let new_sentence = field("Bonjour", true, "Demain", true);
    assert_eq!(
        spaced(", à tous.", &new_sentence, NO_END_SPACE),
        ", à tous. "
    );
    let context = field("Bonjour", true, "demain", true);
    assert_eq!(spaced(", à tous.", &context, NO_END_SPACE), ", à tous ");
    assert_eq!(spaced("(à tous)", &context, NO_END_SPACE), " (à tous) ");
}

// @spec AC-004: French high punctuation — .specs/features/017-smart-spacing/spec.md#ac-004
#[test]
fn french_typography_is_never_invented_nor_removed() {
    let typed = field("Est-ce que ", true, " ?", true);
    assert_eq!(spaced("tu viens", &typed, NO_END_SPACE), "tu viens");
    let tight = field("Est-ce que", true, "?", true);
    assert_eq!(spaced("tu viens", &tight, NO_END_SPACE), " tu viens");
    let guillemets = field("« ", true, " »", true);
    assert_eq!(spaced("Bonjour", &guillemets, NO_END_SPACE), "Bonjour");
    // The dictation's own punctuation and spacing inside it are untouched.
    let after_word = field("Il a dit", true, "", true);
    assert_eq!(
        spaced("bonjour ! Ça va ?", &after_word, NO_END_SPACE),
        " bonjour ! Ça va ?"
    );
}

#[test]
fn line_breaks_are_respected() {
    let context = field("First line\n", true, "\nNext", true);
    assert_eq!(spaced("middle", &context, END_SPACE), "middle");
    let dictated_breaks = field("Title", true, "end", true);
    assert_eq!(
        spaced("\nbody\n", &dictated_breaks, NO_END_SPACE),
        "\nbody\n"
    );
}

#[test]
fn unicode_letters_and_marks() {
    let context = field("Café", true, "über", true);
    assert_eq!(spaced("Ελληνικά", &context, NO_END_SPACE), " Ελληνικά ");
    let combining = field("cafe\u{301}", true, "", true);
    assert_eq!(spaced("noir", &combining, NO_END_SPACE), " noir");
    let emoji = field("🙂", true, "", true);
    assert_eq!(spaced("ok", &emoji, NO_END_SPACE), " ok");
}

// @spec AC-008: Languages without spaces — .specs/features/017-smart-spacing/spec.md#ac-008
#[test]
fn cjk_and_thai_get_no_spaces() {
    let japanese = field("今日は", true, "です", true);
    assert_eq!(spaced("晴れ", &japanese, END_SPACE), "晴れ");
    let chinese = field("你好，", true, "", true);
    assert_eq!(spaced("世界。", &chinese, END_SPACE), "世界。");
    let thai = field("สวัสดี", true, "ครับ", true);
    assert_eq!(spaced("คุณ", &thai, END_SPACE), "คุณ");
    assert_eq!(
        spaced("世界。", &InsertionContext::Unavailable, END_SPACE),
        "世界。"
    );
}

#[test]
fn korean_is_spaced() {
    let korean = field("안녕하세요", true, "", true);
    assert_eq!(spaced("여러분", &korean, NO_END_SPACE), " 여러분");
}

// @spec AC-006: Unavailable context fallback — .specs/features/017-smart-spacing/spec.md#ac-006
#[test]
fn unavailable_context_secures_one_final_space_only() {
    let context = InsertionContext::Unavailable;
    for inputs in [NO_END_SPACE, END_SPACE] {
        let result = space_junctions("Hello there", &context, inputs);
        assert_eq!(result.text, "Hello there ");
        assert!(result.trailing_decided);
    }
    assert_eq!(spaced(" Hello", &context, NO_END_SPACE), " Hello ");
    assert_eq!(spaced("Hello ", &context, NO_END_SPACE), "Hello ");
    assert_eq!(spaced("Hello\n", &context, NO_END_SPACE), "Hello\n");
    assert_eq!(spaced("Hello  ", &context, NO_END_SPACE), "Hello  ");
}

#[test]
fn partial_context_fits_the_known_side_and_secures_the_end() {
    let context = before_only("Bonjour");
    assert_eq!(spaced("à tous", &context, NO_END_SPACE), " à tous ");
    assert_eq!(spaced("à tous ", &context, NO_END_SPACE), " à tous ");
    let incomplete = field("Bonjour", true, "", false);
    assert_eq!(spaced("à tous", &incomplete, NO_END_SPACE), " à tous ");
}

#[test]
fn nothing_read_before_that_misses_the_field_start_keeps_the_leading_edge() {
    let context = InsertionContext::Field {
        before: CursorContext::Before {
            text: String::new(),
            reaches_start: false,
        },
        after: AfterContext::After {
            text: "demain".to_owned(),
            reaches_end: true,
        },
    };
    assert_eq!(spaced(" à tous", &context, NO_END_SPACE), " à tous ");
    assert_eq!(spaced("à tous", &context, NO_END_SPACE), "à tous ");
}

#[test]
fn absent_and_empty_differ() {
    let empty = field("", true, "", true);
    let unavailable = InsertionContext::Unavailable;
    assert_eq!(spaced("Hello", &empty, NO_END_SPACE), "Hello");
    assert_eq!(spaced("Hello", &unavailable, NO_END_SPACE), "Hello ");
    assert_eq!(spaced(" Hello", &empty, NO_END_SPACE), "Hello");
    assert_eq!(spaced(" Hello", &unavailable, NO_END_SPACE), " Hello ");
}

// @spec AC-010: Not a text field — .specs/features/017-smart-spacing/spec.md#ac-010
#[test]
fn not_a_text_field_changes_nothing_and_leaves_the_method_space() {
    let result = space_junctions("ls -la", &InsertionContext::NotText, END_SPACE);
    assert_eq!(result.text, "ls -la");
    assert!(!result.trailing_decided);
}

#[test]
fn empty_or_blank_text_is_unchanged() {
    for text in ["", " ", "\n"] {
        let result = space_junctions(text, &field("a", true, "b", true), END_SPACE);
        assert_eq!(result.text, text);
        assert!(!result.trailing_decided);
    }
}

// @spec AC-001: Only edge whitespace changes — .specs/features/017-smart-spacing/spec.md#ac-001
#[test]
fn only_edge_whitespace_changes() {
    let contexts = [
        field("Bonjour", true, "demain", true),
        field("", true, "", true),
        before_only("x "),
        InsertionContext::Unavailable,
    ];
    let text = "  Un  texte,\tavec  espaces ! \n";
    for context in &contexts {
        let result = spaced(text, context, END_SPACE);
        assert!(
            result.contains("Un  texte,\tavec  espaces ! \n"),
            "{context:?}: {result:?}"
        );
        let core = result.trim_matches(|c: char| c == ' ' || c == '\t');
        assert_eq!(core, "Un  texte,\tavec  espaces ! \n", "{context:?}");
    }
}

#[test]
fn the_trailing_space_is_decided_once() {
    for context in [
        field("a", true, "", true),
        field("a", true, "b", true),
        before_only("a"),
        InsertionContext::Unavailable,
    ] {
        assert!(space_junctions("word", &context, END_SPACE).trailing_decided);
        let text = spaced("word", &context, END_SPACE);
        assert!(!text.ends_with("  "), "{context:?}: {text:?}");
    }
}

/// The field after inserting `text` at the selection of the read field `before|after`.
fn inserted(before: &str, text: &str, after: &str) -> String {
    let context = field(before, true, after, true);
    format!("{before}{}{after}", spaced(text, &context, NO_END_SPACE))
}

// @spec AC-017: Final period inside a sentence — .specs/features/017-smart-spacing/spec.md#ac-017
#[test]
fn n30_final_period_before_a_lower_case_word_is_dropped() {
    assert_eq!(
        inserted("Bonjour", "à tous.", "demain"),
        "Bonjour à tous demain"
    );
}

#[test]
fn n30_final_period_before_a_comma_is_dropped() {
    assert_eq!(
        inserted("Bonjour", "tout le monde.", ", ça va"),
        "Bonjour tout le monde, ça va"
    );
}

#[test]
fn n30_engine_spaces_after_the_period_do_not_hide_it() {
    assert_eq!(
        inserted("Bonjour ", " à tous. ", " demain"),
        "Bonjour à tous demain"
    );
}

#[test]
fn n30_new_sentence_line_or_field_end_keep_the_period() {
    assert_eq!(
        inserted("Bonjour", "à tous.", "Demain"),
        "Bonjour à tous. Demain"
    );
    assert_eq!(
        inserted("Bonjour", "à tous.", "\ndemain"),
        "Bonjour à tous.\ndemain"
    );
    assert_eq!(inserted("Bonjour", "à tous.", ""), "Bonjour à tous.");
}

#[test]
fn n30_abbreviation_and_ellipsis_keep_their_mark() {
    assert_eq!(
        inserted("Des fruits", "pommes, etc.", ", ça va"),
        "Des fruits pommes, etc., ça va"
    );
    assert_eq!(
        inserted("Bonjour", "à tous…", "demain"),
        "Bonjour à tous… demain"
    );
    assert_eq!(
        inserted("Bonjour", "chez M.", "demain"),
        "Bonjour chez M. demain"
    );
}

#[test]
fn n30_cjk_full_stop_is_not_concerned() {
    assert_eq!(inserted("你好", "世界。", "，好"), "你好世界。，好");
}

#[test]
fn n30_unknown_side_after_keeps_the_period() {
    let partial = before_only("Bonjour");
    assert_eq!(spaced("à tous.", &partial, END_SPACE), " à tous. ");
    let unread_end = field("Bonjour", true, "", false);
    assert_eq!(spaced("à tous.", &unread_end, END_SPACE), " à tous. ");
    assert_eq!(
        spaced("à tous.", &InsertionContext::Unavailable, END_SPACE),
        "à tous. "
    );
    let result = space_junctions("à tous.", &InsertionContext::NotText, END_SPACE);
    assert_eq!(result.text, "à tous.");
}
