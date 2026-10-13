//! Table-driven AC-006/AC-007 matching fixtures (SC-001) and output invariants.

use super::matcher::{CompiledReplacements, OutputSegment};
use super::model::ReplacementRule;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// One exact matching oracle: ordered rules, recognized text, expected output.
struct Case {
    name: &'static str,
    rules: &'static [(&'static str, &'static str)],
    input: &'static str,
    expected: &'static str,
}

const CASES: &[Case] = &[
    Case {
        name: "spec: case-insensitive, replacement as written",
        rules: &[("cloud code", "Claude Code")],
        input: "CLOUD CODE, ready.",
        expected: "Claude Code, ready.",
    },
    Case {
        name: "spec: whole words only",
        rules: &[("art", "ART")],
        input: "smart art",
        expected: "smart ART",
    },
    Case {
        name: "spec: longest rule wins",
        rules: &[("Cloud", "Claude"), ("Cloud Code", "Claude Code CLI")],
        input: "cloud code and cloud",
        expected: "Claude Code CLI and Claude",
    },
    Case {
        name: "spec: no chaining",
        rules: &[("A1", "B1"), ("B1", "C1")],
        input: "A1",
        expected: "B1",
    },
    Case {
        name: "spec: story 1 dictation",
        rules: &[("Cloud Code", "Claude Code")],
        input: "open cloud code now",
        expected: "open Claude Code now",
    },
    Case {
        name: "empty rule set",
        rules: &[],
        input: "cloud code",
        expected: "cloud code",
    },
    Case {
        name: "empty text",
        rules: &[("a", "b")],
        input: "",
        expected: "",
    },
    Case {
        name: "whole text match",
        rules: &[("art", "ART")],
        input: "art",
        expected: "ART",
    },
    Case {
        name: "whitespace run of spaces and tab",
        rules: &[("cloud code", "X")],
        input: "cloud  \t code",
        expected: "X",
    },
    Case {
        name: "no-break and em spaces",
        rules: &[("cloud code", "X")],
        input: "cloud\u{A0}\u{2003}code",
        expected: "X",
    },
    Case {
        name: "line break is whitespace",
        rules: &[("cloud code", "X")],
        input: "cloud\ncode",
        expected: "X",
    },
    Case {
        name: "hyphen is not whitespace",
        rules: &[("cloud code", "X")],
        input: "cloud-code",
        expected: "cloud-code",
    },
    Case {
        name: "trailing digit blocks",
        rules: &[("art", "ART")],
        input: "art2 2art",
        expected: "art2 2art",
    },
    Case {
        name: "underscore is a boundary",
        rules: &[("art", "ART")],
        input: "art_x",
        expected: "ART_x",
    },
    Case {
        name: "punctuation kept",
        rules: &[("art", "ART")],
        input: "(art).",
        expected: "(ART).",
    },
    Case {
        name: "apostrophe kept",
        rules: &[("art", "ART")],
        input: "art's",
        expected: "ART's",
    },
    Case {
        name: "accented letter blocks",
        rules: &[("art", "ART")],
        input: "artéfact",
        expected: "artéfact",
    },
    Case {
        name: "sharp s folds to ss",
        rules: &[("straße", "Street")],
        input: "STRASSE Strasse straße",
        expected: "Street Street Street",
    },
    Case {
        name: "whole expansion consumed",
        rules: &[("ss", "X")],
        input: "ß",
        expected: "X",
    },
    Case {
        name: "no partial expansion",
        rules: &[("s", "X")],
        input: "ß",
        expected: "ß",
    },
    Case {
        name: "ligature folds",
        rules: &[("file", "File")],
        input: "\u{FB01}le",
        expected: "File",
    },
    Case {
        name: "final sigma folds",
        rules: &[("ΟΔΟΣ", "road")],
        input: "οδος",
        expected: "road",
    },
    Case {
        name: "accent is literal",
        rules: &[("café", "X")],
        input: "cafe CAFÉ",
        expected: "cafe X",
    },
    Case {
        name: "decomposed accent differs",
        rules: &[("café", "X")],
        input: "cafe\u{301}",
        expected: "cafe\u{301}",
    },
    Case {
        name: "dotted capital I is not Turkic",
        rules: &[("istanbul", "X")],
        input: "İstanbul",
        expected: "İstanbul",
    },
    Case {
        name: "Han without spaces",
        rules: &[("東京", "Tokyo")],
        input: "私は東京に住む",
        expected: "私はTokyoに住む",
    },
    Case {
        name: "kana without spaces",
        rules: &[("クロード", "Claude")],
        input: "これはクロードです",
        expected: "これはClaudeです",
    },
    Case {
        name: "hangul without spaces",
        rules: &[("클로드", "Claude")],
        input: "나는클로드를",
        expected: "나는Claude를",
    },
    Case {
        name: "supplementary ideograph",
        rules: &[("𠀀", "X")],
        input: "a𠀀b",
        expected: "aXb",
    },
    Case {
        name: "halfwidth kana",
        rules: &[("ｶﾀ", "KATA")],
        input: "テｶﾀス",
        expected: "テKATAス",
    },
    Case {
        name: "one-sided CJK exemption",
        rules: &[("東京tower", "X")],
        input: "私は東京tower! 東京towers",
        expected: "私はX! 東京towers",
    },
    Case {
        name: "Latin rule next to Han",
        rules: &[("code", "CODE")],
        input: "東京code code",
        expected: "東京code CODE",
    },
    Case {
        name: "shorter rule when longer fails boundary",
        rules: &[("cloud code", "A"), ("cloud", "B")],
        input: "cloud codex",
        expected: "B codex",
    },
    Case {
        name: "replacement contains recognized",
        rules: &[("Code", "Claude Code")],
        input: "code code",
        expected: "Claude Code Claude Code",
    },
    Case {
        name: "literal symbols",
        rules: &[("C++", "CPP")],
        input: "I like c++. c++x",
        expected: "I like CPP. c++x",
    },
    Case {
        name: "digit key",
        rules: &[("v2", "version two")],
        input: "use v2. v22",
        expected: "use version two. v22",
    },
    Case {
        name: "emoji neighbour",
        rules: &[("ok", "OK")],
        input: "ok😀",
        expected: "OK😀",
    },
    Case {
        name: "adjacent words",
        rules: &[("art", "ART")],
        input: "artart art art",
        expected: "artart ART ART",
    },
    Case {
        name: "longer key listed later",
        rules: &[("a b", "1"), ("a b c", "2")],
        input: "a b c",
        expected: "2",
    },
    Case {
        name: "replacement may contain separators",
        rules: &[("x", "a;b #c")],
        input: "x",
        expected: "a;b #c",
    },
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
        name: "rule with several internal spaces",
        rules: &[("cloud   code", "Claude Code")],
        input: "cloud code and cloud \t\u{A0} code",
        expected: "Claude Code and Claude Code",
    },
    Case {
        name: "no match starts inside a sharp s expansion",
        rules: &[("se", "X")],
        input: "ße",
        expected: "ße",
    },
];

fn compile(rules: &[(&str, &str)]) -> Result<CompiledReplacements, Box<dyn std::error::Error>> {
    let rules: Vec<ReplacementRule> = rules
        .iter()
        .enumerate()
        .map(|(index, (recognized, replacement))| ReplacementRule {
            id: format!("rule-{index}").into(),
            recognized: (*recognized).into(),
            replacement: (*replacement).into(),
        })
        .collect();
    Ok(CompiledReplacements::compile(&rules)?)
}

#[test]
fn match_every_exact_fixture() -> TestResult {
    assert!(CASES.len() >= 30, "SC-001 requires at least 30 fixtures");
    for case in CASES {
        let output = compile(case.rules)?.apply(case.input);
        assert_eq!(output.text(), case.expected, "{}", case.name);
    }
    Ok(())
}

#[test]
fn segments_cover_the_original_text_in_order() -> TestResult {
    for case in CASES {
        let output = compile(case.rules)?.apply(case.input);
        let mut covered = 0;
        for segment in &output.segments {
            let source = match segment {
                OutputSegment::Source { source, text } => {
                    assert_eq!(
                        case.input.get(source.clone()),
                        Some(text.as_str()),
                        "{}",
                        case.name
                    );
                    source
                }
                OutputSegment::Replacement { source, .. } => source,
            };
            assert_eq!(source.start, covered, "{}", case.name);
            covered = source.end;
        }
        assert_eq!(covered, case.input.len(), "{}", case.name);
    }
    Ok(())
}

#[test]
fn report_rule_identity_and_matched_range() -> TestResult {
    let output = compile(&[("cloud code", "Claude Code")])?.apply("open CLOUD  code!");
    assert!(output.has_replacements());
    assert_eq!(
        output.segments[1],
        OutputSegment::Replacement {
            rule_id: "rule-0".into(),
            source: 5..16,
            text: "Claude Code".into(),
        }
    );
    let spans = output.spans();
    let protected: Vec<bool> = spans.iter().map(|span| span.is_protected).collect();
    assert_eq!(protected, vec![false, true, false]);
    Ok(())
}

#[test]
fn reject_invalid_rule_sets_at_compile_time() {
    let duplicate = [("Same", "1"), ("SAME", "2")];
    assert!(compile(&duplicate).is_err());
    assert!(CompiledReplacements::empty().apply("text").segments.len() == 1);
}
