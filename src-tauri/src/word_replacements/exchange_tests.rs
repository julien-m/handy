//! Unit tests for the exchange parser, byte decoder and canonical serializer.

use std::io::Read;

use super::exchange::{
    decode_exchange_bytes, parse_exchange, read_exchange_input, serialize_exchange,
    ExchangeInputError, IgnoredLine, ParsedLineContent, MAX_IMPORT_BYTES,
};
use super::model::ReplacementRule;
use super::validation::{Field, FieldIssue, IssueKind};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn contents(text: &str) -> Result<Vec<ParsedLineContent>, Box<dyn std::error::Error>> {
    Ok(parse_exchange(text)?
        .into_iter()
        .map(|line| line.content)
        .collect())
}

fn rule(recognized: &str, replacement: &str) -> ParsedLineContent {
    ParsedLineContent::Rule {
        recognized: recognized.into(),
        replacement: replacement.into(),
    }
}

fn ignored(reason: IgnoredLine) -> ParsedLineContent {
    ParsedLineContent::Ignored { reason }
}

fn invalid(field: Field, kind: IssueKind) -> ParsedLineContent {
    ParsedLineContent::Invalid {
        issues: vec![FieldIssue { field, kind }],
    }
}

#[test]
fn parse_spec_example_with_french_header() -> TestResult {
    assert_eq!(
        contents("texte_reconnu;remplacement\nCloud Code;Claude Code\nCloud Sonnet;Claude Sonnet")?,
        vec![
            ignored(IgnoredLine::Header),
            rule("Cloud Code", "Claude Code"),
            rule("Cloud Sonnet", "Claude Sonnet"),
        ]
    );
    Ok(())
}

#[test]
fn recognize_header_case_insensitively_after_blank_and_comment_lines() -> TestResult {
    assert_eq!(
        contents("\n# rules\n  RECOGNIZED;Replacement  \nrecognized;replacement\n")?,
        vec![
            ignored(IgnoredLine::Blank),
            ignored(IgnoredLine::Comment),
            ignored(IgnoredLine::Header),
            rule("recognized", "replacement"),
        ]
    );
    Ok(())
}

#[test]
fn recognize_tab_separated_header_copied_from_a_spreadsheet() -> TestResult {
    assert_eq!(
        contents("recognized\treplacement\nCloud Code\tClaude Code\n")?,
        vec![
            ignored(IgnoredLine::Header),
            rule("Cloud Code", "Claude Code")
        ]
    );
    assert_eq!(
        contents("Texte_Reconnu\tRemplacement\r\nOpen Claw\tOpenClaw")?,
        vec![ignored(IgnoredLine::Header), rule("Open Claw", "OpenClaw")]
    );
    Ok(())
}

#[test]
fn recognize_header_fields_after_trimming_whatever_the_separator() -> TestResult {
    let header = vec![ignored(IgnoredLine::Header)];
    assert_eq!(contents("recognized ; replacement")?, header);
    assert_eq!(contents(" recognized \t replacement ")?, header);
    Ok(())
}

#[test]
fn keep_header_lookalikes_and_later_tab_headers_as_rules() -> TestResult {
    assert_eq!(
        contents("recognized;replacement;extra\nrecognized\treplacement")?,
        vec![
            rule("recognized", "replacement;extra"),
            rule("recognized", "replacement"),
        ]
    );
    assert_eq!(
        contents("recognized\treplacement\trest")?,
        vec![invalid(Field::Replacement, IssueKind::ForbiddenCharacter)]
    );
    Ok(())
}

#[test]
fn split_tab_lines_on_first_tab_even_after_semicolon() -> TestResult {
    assert_eq!(
        contents("Cloud\tClaude; Code\nA;B\tC")?,
        vec![
            rule("Cloud", "Claude; Code"),
            invalid(Field::Recognized, IssueKind::ForbiddenCharacter),
        ]
    );
    Ok(())
}

#[test]
fn keep_later_semicolons_and_hashes_literal_in_replacement() -> TestResult {
    assert_eq!(contents("C;a;b;c")?, vec![rule("C", "a;b;c")]);
    assert_eq!(
        contents("C;#not a comment")?,
        vec![rule("C", "#not a comment")]
    );
    Ok(())
}

#[test]
fn report_missing_separator_with_line_number() -> TestResult {
    let lines = parse_exchange("Cloud Code;Claude Code\nCloud Sonnet;Claude Sonnet\nCloud Opus")?;
    assert_eq!(lines[2].line_number, 3);
    assert_eq!(
        lines[2].content,
        invalid(Field::Line, IssueKind::MissingSeparator)
    );
    Ok(())
}

#[test]
fn report_empty_fields_and_forbidden_tab_in_replacement() -> TestResult {
    assert_eq!(
        contents(" ;x\nx; \nx\ty\tz")?,
        vec![
            invalid(Field::Recognized, IssueKind::TooShort),
            invalid(Field::Replacement, IssueKind::TooShort),
            invalid(Field::Replacement, IssueKind::ForbiddenCharacter),
        ]
    );
    Ok(())
}

#[test]
fn accept_crlf_bom_and_keep_original_line_numbers() -> TestResult {
    let lines = parse_exchange("\u{FEFF}recognized;replacement\r\n\r\nA;B\r\n")?;
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].content, ignored(IgnoredLine::Header));
    assert_eq!(lines[1].content, ignored(IgnoredLine::Blank));
    assert_eq!(
        (lines[2].line_number, &lines[2].content),
        (3, &rule("A", "B"))
    );
    Ok(())
}

#[test]
fn reject_standalone_carriage_return_inside_a_field() -> TestResult {
    assert_eq!(
        contents("a\rb;c")?,
        vec![invalid(Field::Recognized, IssueKind::ForbiddenCharacter)]
    );
    Ok(())
}

#[test]
fn treat_csv_quotes_as_literal_text_and_empty_input_as_no_lines() -> TestResult {
    assert_eq!(
        contents("\"Cloud\";\"Claude, Code\"")?,
        vec![rule("\"Cloud\"", "\"Claude, Code\"")]
    );
    assert!(contents("")?.is_empty());
    Ok(())
}

#[test]
fn decode_bytes_with_cap_bom_and_utf8_offset() {
    assert_eq!(
        decode_exchange_bytes(b"\xEF\xBB\xBFA;B".to_vec()),
        Ok("\u{FEFF}A;B".into())
    );
    assert_eq!(
        decode_exchange_bytes(b"A;\xFFB".to_vec()),
        Err(ExchangeInputError::InvalidUtf8 { byte_offset: 2 })
    );
    assert_eq!(
        decode_exchange_bytes(vec![b'a'; MAX_IMPORT_BYTES + 1]),
        Err(ExchangeInputError::TooLarge {
            max_bytes: MAX_IMPORT_BYTES
        })
    );
    assert!(decode_exchange_bytes(vec![b'a'; MAX_IMPORT_BYTES]).is_ok());
}

#[test]
fn remove_only_the_initial_bom_once_across_decode_and_parse() -> TestResult {
    let decoded = decode_exchange_bytes(b"\xEF\xBB\xBF\xEF\xBB\xBFA;B\n\xEF\xBB\xBFC;D".to_vec())?;
    assert_eq!(decoded, "\u{FEFF}\u{FEFF}A;B\n\u{FEFF}C;D");
    assert_eq!(
        contents(&decoded)?,
        vec![rule("\u{FEFF}A", "B"), rule("\u{FEFF}C", "D")]
    );
    assert_eq!(contents("\u{FEFF}A;B")?, vec![rule("A", "B")]);
    Ok(())
}

#[test]
fn report_reader_failure_without_content() {
    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        }
    }
    assert_eq!(
        read_exchange_input(FailingReader),
        Err(ExchangeInputError::Unreadable {
            io_kind: "PermissionDenied".into()
        })
    );
}

fn stored(id: &str, recognized: &str, replacement: &str) -> ReplacementRule {
    ReplacementRule {
        id: id.into(),
        recognized: recognized.into(),
        replacement: replacement.into(),
    }
}

#[test]
fn serialize_canonical_header_and_order_with_lf() {
    let rules = vec![
        stored("b", "Cloud Code", "Claude Code"),
        stored("a", "recognized", "a;b"),
    ];
    assert_eq!(
        serialize_exchange(&rules),
        "recognized;replacement\nCloud Code;Claude Code\nrecognized;a;b\n"
    );
    assert_eq!(serialize_exchange(&[]), "recognized;replacement\n");
}

#[test]
fn round_trip_a_first_rule_that_looks_like_the_header() -> TestResult {
    let rules = vec![stored("x", "recognized", "replacement")];
    assert_eq!(
        contents(&serialize_exchange(&rules))?,
        vec![
            ignored(IgnoredLine::Header),
            rule("recognized", "replacement")
        ]
    );
    Ok(())
}
