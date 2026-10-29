//! Decision table of spec AC-002 and AC-005.
use super::*;

fn before(text: &str) -> CursorContext {
    CursorContext::Before {
        text: text.to_owned(),
        reaches_start: false,
    }
}

fn at_start(text: &str) -> CursorContext {
    CursorContext::Before {
        text: text.to_owned(),
        reaches_start: true,
    }
}

#[test]
fn absent_context_keeps() {
    assert_eq!(decide(&CursorContext::Absent), CaseDecision::Keep);
}

#[test]
fn empty_context_capitalizes() {
    assert_eq!(decide(&at_start("")), CaseDecision::Capitalize);
    assert_eq!(decide(&at_start("   ")), CaseDecision::Capitalize);
}

#[test]
fn absent_and_empty_differ() {
    assert_ne!(decide(&CursorContext::Absent), decide(&at_start("")));
}

#[test]
fn keeps_whitespace_window_not_at_start() {
    assert_eq!(decide(&before("        ")), CaseDecision::Keep);
}

#[test]
fn capitalizes_after_sentence_end() {
    for context in [
        "It works. ",
        "It works.",
        "Really? ",
        "Stop! ",
        "Wait… ",
        "Wait... ",
        "He said \"Done.\" ",
        "(As planned.) ",
        "終わった。",
        "¿Listo? ",
    ] {
        assert_eq!(
            decide(&before(context)),
            CaseDecision::Capitalize,
            "{context}"
        );
    }
}

#[test]
fn capitalizes_after_newline() {
    for context in ["First line\n", "First line\n  ", "Title\r\n", "a\u{2029}"] {
        assert_eq!(
            decide(&before(context)),
            CaseDecision::Capitalize,
            "{context:?}"
        );
    }
}

#[test]
fn capitalizes_after_list_markers() {
    for context in [
        "Groceries:\n- ",
        "\n* ",
        "\n• ",
        "\n  1. ",
        "\n12) ",
        "\n\t– ",
    ] {
        assert_eq!(
            decide(&before(context)),
            CaseDecision::Capitalize,
            "{context:?}"
        );
    }
}

#[test]
fn marker_at_field_start_is_a_list() {
    assert_eq!(decide(&at_start("- ")), CaseDecision::Capitalize);
    assert_eq!(decide(&at_start("  1) ")), CaseDecision::Capitalize);
}

#[test]
fn marker_at_window_edge_is_not_proven() {
    // The window may start on a dash in the middle of a sentence ("… — ").
    assert_eq!(decide(&before("— ")), CaseDecision::Keep);
}

#[test]
fn multi_byte_line_breaks_never_split_a_character() {
    assert_eq!(decide(&before("a\u{2028}hello ")), CaseDecision::Lower);
    assert_eq!(decide(&before("a\u{2029}- ")), CaseDecision::Capitalize);
    assert_eq!(decide(&before("a\u{85}1. ")), CaseDecision::Capitalize);
    assert_eq!(decide(&before("a\u{2028}  ")), CaseDecision::Capitalize);
}

#[test]
fn marker_without_space_or_inside_a_line_is_not_a_list() {
    assert_eq!(decide(&before("\n-")), CaseDecision::Keep);
    assert_eq!(decide(&before("\n1234. ")), CaseDecision::Capitalize);
    assert_eq!(decide(&before("a - ")), CaseDecision::Keep);
}

#[test]
fn lowers_after_comma() {
    for context in ["I bought apples, ", "Note: ", "first; ", "東京、", "a，"] {
        assert_eq!(decide(&before(context)), CaseDecision::Lower, "{context}");
    }
}

#[test]
fn lowers_after_a_word() {
    for context in [
        "We will meet ",
        "Je souhaite",
        "hel",
        "room 42 ",
        "(see above) ",
        "the users' ",
        "cafe\u{0301} ",
    ] {
        assert_eq!(decide(&before(context)), CaseDecision::Lower, "{context}");
    }
}

#[test]
fn keeps_after_opening_quote() {
    for context in ["He said \"", "He said « ", "(", "Price: $", "Done 🎉 "] {
        assert_eq!(decide(&before(context)), CaseDecision::Keep, "{context}");
    }
}

#[test]
fn context_debug_never_prints_the_text() {
    let printed = format!("{:?}", before("secret words"));
    assert!(!printed.contains("secret"));
}
