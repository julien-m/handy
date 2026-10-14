//! First pass of a preview: the parsed input lines become listed rows and counters, with a
//! bounded memory whatever the input holds. A 1,000,000-byte input can hold about a million
//! blank lines or half a million error lines; only rule lines (at most `MAX_RULES`), the first
//! `MAX_LISTED_OTHER_LINES` ignored or error lines and the counters are kept.
use super::{
    exchange::{IgnoredLine, ParsedLine, ParsedLineContent},
    preview::{PreviewLine, PreviewStatus},
    unicode::UnicodeData,
    validation::{FieldIssue, MAX_RULES},
};

/// Ignored and error rows listed by a preview; the rest are counted. 1000 rows stay far below
/// one channel message and readable in an editor, and name every line of a typical input.
pub const MAX_LISTED_OTHER_LINES: usize = 1000;
/// Other input lines named by a duplicated row; the first ones are enough to find the others.
pub const MAX_DUPLICATE_LINES: usize = 10;

/// A rule line kept for classification, with its listed row.
pub(super) struct InputRule {
    pub(super) line_number: usize,
    pub(super) row: usize,
    pub(super) key: String,
    pub(super) recognized: String,
    pub(super) replacement: String,
}

/// Listed rows, kept rule lines and counters of one input.
#[derive(Default)]
pub(super) struct Scan {
    pub(super) rows: Vec<PreviewLine>,
    pub(super) rules: Vec<InputRule>,
    pub(super) ignored_lines: usize,
    pub(super) invalid_lines: usize,
    pub(super) omitted_lines: usize,
    /// More than `MAX_RULES` rule lines: the input can never be applied.
    pub(super) has_excess_rules: bool,
    listed_other_lines: usize,
}

/// Scan `parsed` once, in input order.
pub(super) fn scan_input(data: &UnicodeData, parsed: impl Iterator<Item = ParsedLine>) -> Scan {
    let mut scan = Scan::default();
    for line in parsed {
        match line.content {
            ParsedLineContent::Rule {
                recognized,
                replacement,
            } => scan.push_rule(data, line.line_number, recognized, replacement),
            ParsedLineContent::Ignored { reason } => {
                scan.ignored_lines += 1;
                scan.push_other(line.line_number, Some(reason), Vec::new());
            }
            ParsedLineContent::Invalid { issues } => {
                scan.invalid_lines += 1;
                scan.push_other(line.line_number, None, issues);
            }
        }
    }
    scan
}

impl Scan {
    fn push_rule(
        &mut self,
        data: &UnicodeData,
        line_number: usize,
        recognized: String,
        replacement: String,
    ) {
        // Business rule: past MAX_RULES rule lines the input can never be applied (either some
        // lines are duplicates, or the resulting set exceeds MAX_RULES), so the rest is counted.
        if self.rules.len() == MAX_RULES {
            self.has_excess_rules = true;
            self.omitted_lines += 1;
            return;
        }
        self.rules.push(InputRule {
            line_number,
            row: self.rows.len(),
            key: data.normalized_key(&recognized),
            recognized,
            replacement,
        });
        // Rule lines are classified once compared with the saved set.
        self.rows
            .push(row(line_number, PreviewStatus::Add, None, Vec::new()));
    }

    fn push_other(
        &mut self,
        line_number: usize,
        ignored: Option<IgnoredLine>,
        issues: Vec<FieldIssue>,
    ) {
        if self.listed_other_lines == MAX_LISTED_OTHER_LINES {
            self.omitted_lines += 1;
            return;
        }
        self.listed_other_lines += 1;
        let status = if ignored.is_some() {
            PreviewStatus::Ignored
        } else {
            PreviewStatus::Error
        };
        self.rows.push(row(line_number, status, ignored, issues));
    }
}

fn row(
    line_number: usize,
    status: PreviewStatus,
    ignored: Option<IgnoredLine>,
    issues: Vec<FieldIssue>,
) -> PreviewLine {
    PreviewLine {
        line_number: Some(line_number),
        status,
        ignored,
        rule_id: None,
        recognized: None,
        replacement: None,
        issues,
        duplicate_lines: Vec::new(),
    }
}
