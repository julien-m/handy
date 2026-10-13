//! Script-boundary AC-006 fixtures (SC-001): combining marks in Indic scripts
//! and decomposed accents, zero-width joiners in Persian, and kana marks on a
//! CJK side. `matcher_tests` runs them with the general fixtures.

use super::matcher_tests::Case;

pub(super) const SCRIPT_CASES: &[Case] = &[
    Case {
        name: "dependent vowel sign blocks Devanagari prefix",
        rules: &[("भारत", "India")],
        input: "भारतीय संविधान",
        expected: "भारतीय संविधान",
    },
    Case {
        name: "dependent vowel sign blocks Devanagari suffix",
        rules: &[("रत", "Z")],
        input: "भारत",
        expected: "भारत",
    },
    Case {
        name: "virama blocks Devanagari conjunct split",
        rules: &[("स", "S")],
        input: "स्त",
        expected: "स्त",
    },
    Case {
        name: "whole Devanagari words replaced",
        rules: &[("भारत", "India"), ("भारतीय", "Indian")],
        input: "भारत और भारतीय संविधान।",
        expected: "India और Indian संविधान।",
    },
    Case {
        name: "decomposed accent after match blocks",
        rules: &[("cafe", "X")],
        input: "cafe\u{301} ok",
        expected: "cafe\u{301} ok",
    },
    Case {
        name: "precomposed accent does not match base letters",
        rules: &[("cafe", "X")],
        input: "café",
        expected: "café",
    },
    Case {
        name: "decomposed accent before match blocks",
        rules: &[("ok", "OK")],
        input: "e\u{301}ok",
        expected: "e\u{301}ok",
    },
    Case {
        name: "whole decomposed word replaced",
        rules: &[("cafe\u{301}", "Café")],
        input: "un CAFE\u{301}, merci",
        expected: "un Café, merci",
    },
    Case {
        name: "zero-width non-joiner keeps a Persian prefix in its word",
        rules: &[("\u{645}\u{6CC}", "X")],
        input: "\u{645}\u{6CC}\u{200C}\u{62E}\u{648}\u{627}\u{647}\u{645}",
        expected: "\u{645}\u{6CC}\u{200C}\u{62E}\u{648}\u{627}\u{647}\u{645}",
    },
    Case {
        name: "zero-width non-joiner keeps a Persian stem in its word",
        rules: &[("\u{62E}\u{648}\u{627}\u{647}\u{645}", "X")],
        input: "\u{645}\u{6CC}\u{200C}\u{62E}\u{648}\u{627}\u{647}\u{645}",
        expected: "\u{645}\u{6CC}\u{200C}\u{62E}\u{648}\u{627}\u{647}\u{645}",
    },
    Case {
        name: "whole Persian word separated by a space replaced",
        rules: &[("\u{645}\u{6CC}", "X")],
        input: "\u{645}\u{6CC} \u{62E}\u{648}\u{627}\u{647}\u{645}",
        expected: "X \u{62E}\u{648}\u{627}\u{647}\u{645}",
    },
    Case {
        name: "zero-width joiner blocks a match",
        rules: &[("ok", "X")],
        input: "ok\u{200D}x \u{200D}ok",
        expected: "ok\u{200D}x \u{200D}ok",
    },
    Case {
        name: "NFD dakuten after kana blocks on a CJK side",
        rules: &[("\u{304B}", "X")],
        input: "\u{304B}\u{3099}\u{304D} \u{304B}\u{3099}",
        expected: "\u{304B}\u{3099}\u{304D} \u{304B}\u{3099}",
    },
    Case {
        name: "kana inside a kana word still replaced",
        rules: &[("\u{304B}", "X")],
        input: "\u{3042}\u{304B}\u{3044}",
        expected: "\u{3042}X\u{3044}",
    },
    Case {
        name: "combining mark before a CJK side blocks",
        rules: &[("\u{304B}\u{3044}", "X")],
        input: "\u{304B}\u{3099}\u{304B}\u{3044}",
        expected: "\u{304B}\u{3099}\u{304B}\u{3044}",
    },
    Case {
        name: "rule ending with an NFD dakuten keeps the CJK exemption",
        rules: &[("\u{304B}\u{3099}", "GA")],
        input: "\u{304B}\u{3099}\u{304D}",
        expected: "GA\u{304D}",
    },
];
