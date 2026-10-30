//! Deliberate layouts kept unchanged (016 AC-005).

use super::{detect, Structure};

#[test]
fn plain_prose_has_no_structure() {
    for text in [
        "We met the client today. They agreed on the budget.",
        "-5 degrees this morning, so we stayed in.",
        "2024 was a good year. 3.5 percent growth.",
        "*Important* note for the team.",
        "1.5 million people watched it.",
    ] {
        assert_eq!(detect(text), None, "{text}");
    }
}

#[test]
fn any_line_break_is_kept() {
    assert_eq!(
        detect("First line\nsecond line"),
        Some(Structure::LineBreak)
    );
    assert_eq!(
        detect("First line\r\nsecond line"),
        Some(Structure::LineBreak)
    );
    assert_eq!(detect("Dear Anna,\n\nThanks."), Some(Structure::LineBreak));
}

#[test]
fn list_markers_are_kept() {
    for text in [
        "- buy milk and bread for the weekend",
        "* buy milk",
        "• buy milk",
        "1. buy milk",
        "12) buy milk",
        "  - indented item",
    ] {
        assert_eq!(detect(text), Some(Structure::ListItem), "{text}");
    }
}

#[test]
fn code_is_kept() {
    assert_eq!(
        detect("Run ```cargo test``` before you commit."),
        Some(Structure::CodeFence)
    );
    assert_eq!(detect("    let x = 1;"), Some(Structure::IndentedCode));
    assert_eq!(detect("\tlet x = 1;"), Some(Structure::IndentedCode));
}
