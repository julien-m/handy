//! Junction rule of spec AC-003 and AC-004: one table per family of characters.
use super::*;

#[test]
fn two_words_need_a_space() {
    for (left, right) in [
        ("Bonjour", "à tous"),
        ("tous", "demain"),
        ("Done.", "Next"),
        ("apples,", "pears"),
        ("Hello!", "How"),
        ("item 3", "4"),
        ("(see above)", "then"),
        ("« Bonjour »", "dit-il"),
        ("Привет", "мир"),
        ("مرحبا", "بالعالم"),
        ("안녕하세요", "여러분"),
        ("emoji 🙂", "ok"),
        ("# ", "x"),
    ] {
        let expected = !left.ends_with(' ');
        assert_eq!(needs_space(left, right), expected, "{left:?} | {right:?}");
    }
}

#[test]
fn whitespace_already_present_is_respected() {
    for (left, right) in [
        ("Bonjour ", "à tous"),
        ("Bonjour\u{a0}", "à tous"),
        ("Bonjour\u{202f}", "!"),
        ("line\n", "next"),
        ("tab\t", "next"),
        ("word", " next"),
        ("word", "\nnext"),
        ("word", "\u{202f}?"),
    ] {
        assert!(!needs_space(left, right), "{left:?} | {right:?}");
    }
}

#[test]
fn an_empty_side_needs_no_space() {
    assert!(!needs_space("", "Hello"));
    assert!(!needs_space("Hello", ""));
}

#[test]
fn no_space_before_closing_punctuation() {
    for right in [
        ",",
        ".",
        ")",
        "]",
        "}",
        "…",
        "%",
        "»",
        "”",
        "’s",
        "'s",
        "\" he said",
    ] {
        assert!(!needs_space("word", right), "{right:?}");
    }
}

// @spec AC-004: French high punctuation — .specs/features/017-smart-spacing/spec.md#ac-004
#[test]
fn french_high_punctuation_gets_no_invented_space() {
    for right in [";", ":", "!", "?", "! Oui", "? Non"] {
        assert!(!needs_space("Bonjour", right), "{right:?}");
    }
    // The space the user typed before it is whitespace, so it is kept and nothing is added.
    assert!(!needs_space("Bonjour ", "!"));
}

/// Left texts ending with an opening mark, an opening straight quote or an opening apostrophe.
const OPENING_ENDS: [&str; 15] = [
    "(",
    "see [",
    "{",
    "¿",
    "¡",
    "“",
    "‘",
    "„",
    "«",
    "‹",
    "「",
    "（",
    r#"said ""#,
    r#"""#,
    "'",
];

#[test]
fn no_space_after_opening_marks() {
    for left in OPENING_ENDS {
        assert!(!needs_space(left, "word"), "{left:?}");
    }
}

#[test]
fn a_closing_straight_quote_is_followed_by_a_space() {
    assert!(needs_space("\"Hello\"", "she"));
    assert!(needs_space("end.'", "Then"));
}

#[test]
fn an_opening_straight_quote_on_the_right_is_preceded_by_a_space() {
    assert!(needs_space("He said", "\"hello\""));
}

#[test]
fn french_elision_joins() {
    for left in ["l'", "d’", "qu'", "Aujourd'", "jusqu’"] {
        assert!(!needs_space(left, "arbre"), "{left:?}");
    }
}

#[test]
fn attached_hyphen_and_slash_join() {
    assert!(!needs_space("anti-", "inflammatoire"));
    assert!(!needs_space("and/", "or"));
    assert!(!needs_space("celle", "-ci"));
    // A spaced dash is a separator; a minus before a digit is a signed number.
    assert!(needs_space("Bonjour -", "à tous"));
    assert!(needs_space("is", "-5 degrees"));
}

#[test]
fn decimal_numbers_join() {
    assert!(!needs_space("3.", "5"));
    assert!(!needs_space("1,", "000"));
    assert!(needs_space("Item 3.", "Next"));
}

// @spec AC-008: Languages without spaces — .specs/features/017-smart-spacing/spec.md#ac-008
#[test]
fn scripts_without_spaces_join() {
    for (left, right) in [
        ("東京", "大学"),
        ("今日は", "晴れ"),
        ("カタカナ", "テスト"),
        ("你好。", "世界"),
        ("สวัสดี", "ครับ"),
        ("ສະບາຍດີ", "ເຈົ້າ"),
        ("ជំរាបសួរ", "លោក"),
        ("မင်္ဂလာပါ", "ခင်ဗျာ"),
        ("東京", "Tower"),
        ("Tokyo", "タワー"),
        ("ＡＢＣ", "ＤＥＦ"),
    ] {
        assert!(!needs_space(left, right), "{left:?} | {right:?}");
    }
}

#[test]
fn trailing_space_suits_spaced_scripts_only() {
    assert!(suits_trailing_space("Hello."));
    assert!(suits_trailing_space("안녕"));
    assert!(suits_trailing_space("Bonjour !"));
    assert!(!suits_trailing_space("你好。"));
    assert!(!suits_trailing_space("สวัสดี"));
    assert!(!suits_trailing_space("ends with space "));
    assert!(!suits_trailing_space("line\n"));
    assert!(!suits_trailing_space("open ("));
    assert!(!suits_trailing_space(""));
}
