//! AC-006 combining-mark and join-control boundary fixtures (SC-001): Indic
//! vowel signs and virama, decomposed accents, Persian ZWNJ, ZWJ, kana marks
//! and join controls on a CJK side and the accepted CJK-side false positives
//! (halfwidth sound marks, NFD hangul jamo). Plain Han, kana and hangul
//! boundaries stay in `matcher_tests`, which runs these fixtures with the
//! general ones.

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
        name: "zero-width joiner after a match blocks on a CJK side",
        rules: &[("\u{304B}", "X")],
        input: "\u{304B}\u{200D}\u{304D}",
        expected: "\u{304B}\u{200D}\u{304D}",
    },
    Case {
        name: "zero-width non-joiner after a match blocks on a CJK side",
        rules: &[("\u{304B}", "X")],
        input: "\u{304B}\u{200C}\u{304D}",
        expected: "\u{304B}\u{200C}\u{304D}",
    },
    Case {
        name: "join controls before a match block on a CJK side",
        rules: &[("\u{304B}", "X")],
        input: "\u{304D}\u{200D}\u{304B} \u{304D}\u{200C}\u{304B}",
        expected: "\u{304D}\u{200D}\u{304B} \u{304D}\u{200C}\u{304B}",
    },
    Case {
        name: "rule ending with an NFD dakuten keeps the CJK exemption",
        rules: &[("\u{304B}\u{3099}", "GA")],
        input: "\u{304B}\u{3099}\u{304D}",
        expected: "GA\u{304D}",
    },
    Case {
        name: "accepted false positive: halfwidth voiced sound mark U+FF9E is a letter",
        rules: &[("\u{FF76}", "X")],
        input: "\u{FF76}\u{FF9E}\u{FF77}",
        expected: "X\u{FF9E}\u{FF77}",
    },
    Case {
        name: "accepted false positive: halfwidth semi-voiced sound mark U+FF9F is a letter",
        rules: &[("\u{FF8A}", "X")],
        input: "\u{FF8A}\u{FF9F}",
        expected: "X\u{FF9F}",
    },
    Case {
        name: "accepted false positive: NFD hangul trailing jamo is a letter",
        rules: &[("\u{1112}\u{1161}", "X")],
        input: "\u{1112}\u{1161}\u{11AB}",
        expected: "X\u{11AB}",
    },
];
