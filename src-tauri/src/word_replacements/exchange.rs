//! Two-column exchange format: bounded UTF-8 input, line diagnostics and canonical export.

use std::io::Read;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    model::ReplacementRule,
    unicode::{UnicodeData, UnicodeDataError},
    validation::{validate_fields, FieldIssue},
};

/// Inclusive AC-005 import cap; "1 MB" is read as decimal bytes (plan decision).
pub const MAX_IMPORT_BYTES: usize = 1_000_000;
/// Canonical header written by export and always shown by the bulk editor.
pub const EXPORT_HEADER: &str = "recognized;replacement";
/// AC-004 header field pairs, compared per field after trimming and Unicode
/// case folding, whatever the separator (tab or `;`).
const ACCEPTED_HEADERS: [(&str, &str); 2] = [
    ("texte_reconnu", "remplacement"),
    ("recognized", "replacement"),
];
const BYTE_ORDER_MARK: char = '\u{FEFF}';

/// Why raw file or stdin bytes cannot become exchange text; never carries content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExchangeInputError {
    /// The source exceeds `MAX_IMPORT_BYTES`.
    TooLarge { max_bytes: usize },
    /// The source is not UTF-8; the offset is the first invalid byte.
    InvalidUtf8 { byte_offset: usize },
    /// The source could not be read; only the sanitized I/O kind is kept.
    Unreadable { io_kind: String },
}

impl std::fmt::Display for ExchangeInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { max_bytes } => {
                write!(formatter, "Import exceeds {max_bytes} bytes")
            }
            Self::InvalidUtf8 { byte_offset } => {
                write!(formatter, "Import is not UTF-8 at byte {byte_offset}")
            }
            Self::Unreadable { io_kind } => write!(formatter, "Import is unreadable ({io_kind})"),
        }
    }
}

impl std::error::Error for ExchangeInputError {}

/// Why a line carries no rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum IgnoredLine {
    Blank,
    Comment,
    Header,
}

/// Parsed meaning of one input line, before comparison with saved rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedLineContent {
    Ignored {
        reason: IgnoredLine,
    },
    /// Trimmed fields that passed the shared field validator.
    Rule {
        recognized: String,
        replacement: String,
    },
    Invalid {
        issues: Vec<FieldIssue>,
    },
}

/// One input line with its original 1-based number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ParsedLine {
    pub line_number: usize,
    pub content: ParsedLineContent,
}

/// Read at most `MAX_IMPORT_BYTES` from a file or stdin and decode it.
///
/// # Errors
///
/// - `ExchangeInputError::Unreadable` when the reader fails.
/// - `ExchangeInputError::TooLarge` when more than the cap is available.
/// - `ExchangeInputError::InvalidUtf8` when the bytes are not UTF-8.
pub fn read_exchange_input<R: Read>(reader: R) -> Result<String, ExchangeInputError> {
    let mut bytes = Vec::new();
    // Read one byte past the cap so an oversized source is detected without
    // ever buffering more than the cap plus one byte.
    reader
        .take(MAX_IMPORT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ExchangeInputError::Unreadable {
            io_kind: format!("{:?}", error.kind()),
        })?;
    decode_exchange_bytes(bytes)
}

// @spec AC-005: Bounded UTF-8 import — .specs/features/002-word-replacements/spec.md#ac-005
/// Enforce the byte cap and strict UTF-8; the text is returned unchanged.
///
/// A leading byte order mark is kept here: `parse_exchange` removes exactly one
/// for every source (file, stdin or pasted text), so it is never removed twice.
///
/// # Errors
///
/// - `ExchangeInputError::TooLarge` above `MAX_IMPORT_BYTES`.
/// - `ExchangeInputError::InvalidUtf8` with the first invalid byte offset.
pub fn decode_exchange_bytes(bytes: Vec<u8>) -> Result<String, ExchangeInputError> {
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err(ExchangeInputError::TooLarge {
            max_bytes: MAX_IMPORT_BYTES,
        });
    }
    String::from_utf8(bytes).map_err(|error| ExchangeInputError::InvalidUtf8 {
        byte_offset: error.utf8_error().valid_up_to(),
    })
}

// @spec FR-003: Parse exchange lines with diagnostics — .specs/features/002-word-replacements/spec.md#fr-003
/// Parse exchange text line by line; never executes or interprets content.
/// Only the initial byte order mark is removed; a second one stays content.
///
/// # Errors
///
/// `UnicodeDataError` when the embedded Unicode tables are unusable.
pub fn parse_exchange(text: &str) -> Result<Vec<ParsedLine>, UnicodeDataError> {
    let data = UnicodeData::load()?;
    let text = text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text);
    let mut is_header_candidate = true;
    let lines = split_lines(text)
        .enumerate()
        .map(|(index, line)| ParsedLine {
            line_number: index + 1,
            content: parse_line(data, line, &mut is_header_candidate),
        })
        .collect();
    Ok(lines)
}

fn split_lines(text: &str) -> impl Iterator<Item = &str> {
    // LF and CRLF both end a line; a final line without LF is still a line.
    text.split_inclusive('\n')
        .map(|line| match line.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line,
        })
}

fn parse_line(data: &UnicodeData, line: &str, is_header_candidate: &mut bool) -> ParsedLineContent {
    let trimmed = data.trim_whitespace(line);
    if trimmed.is_empty() {
        return ParsedLineContent::Ignored {
            reason: IgnoredLine::Blank,
        };
    }
    if trimmed.starts_with('#') {
        return ParsedLineContent::Ignored {
            reason: IgnoredLine::Comment,
        };
    }
    // AC-004: only the first content line may be a header, never a later one.
    let was_header_candidate = std::mem::replace(is_header_candidate, false);
    if was_header_candidate && is_header(data, line) {
        return ParsedLineContent::Ignored {
            reason: IgnoredLine::Header,
        };
    }
    parse_rule_line(line)
}

// @spec AC-004: Header recognized with tab or semicolon — .specs/features/002-word-replacements/spec.md#ac-004
/// Whether a line's two fields, split like a rule line and trimmed, are an
/// accepted header pair, so a spreadsheet header row is never imported.
fn is_header(data: &UnicodeData, line: &str) -> bool {
    split_fields(line).is_some_and(|(recognized, replacement)| {
        let fields = (
            data.normalized_key(recognized),
            data.normalized_key(replacement),
        );
        ACCEPTED_HEADERS
            .iter()
            .any(|(left, right)| fields.0 == *left && fields.1 == *right)
    })
}

/// Split on the first tab when the line has one, otherwise on the first `;`.
fn split_fields(line: &str) -> Option<(&str, &str)> {
    // AC-004: a tab wins over `;` so spreadsheet rows keep semicolons literal.
    let separator = if line.contains('\t') { '\t' } else { ';' };
    line.split_once(separator)
}

fn parse_rule_line(line: &str) -> ParsedLineContent {
    let Some((recognized, replacement)) = split_fields(line) else {
        return ParsedLineContent::Invalid {
            issues: vec![FieldIssue {
                field: super::validation::Field::Line,
                kind: super::validation::IssueKind::MissingSeparator,
            }],
        };
    };
    match validate_fields(recognized, replacement) {
        Ok((recognized, replacement)) => ParsedLineContent::Rule {
            recognized,
            replacement,
        },
        Err(issues) => ParsedLineContent::Invalid { issues },
    }
}

// @spec AC-004: Canonical lossless export — .specs/features/002-word-replacements/spec.md#ac-004
/// Serialize the canonical header and one `recognized;replacement` LF line per rule.
/// Valid rules never contain the separators, so the output round-trips.
pub fn serialize_exchange(rules: &[ReplacementRule]) -> String {
    let capacity = rules
        .iter()
        .map(|rule| rule.recognized.len() + rule.replacement.len() + 2)
        .sum::<usize>()
        + EXPORT_HEADER.len()
        + 1;
    let mut output = String::with_capacity(capacity);
    output.push_str(EXPORT_HEADER);
    output.push('\n');
    for rule in rules {
        output.push_str(&rule.recognized);
        output.push(';');
        output.push_str(&rule.replacement);
        output.push('\n');
    }
    output
}
