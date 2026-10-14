//! Bulk input (paste box, CLI import): one term per line, previewed as a complete candidate list
//! before an explicit apply. Preparing a preview is pure: it never writes, commits or notifies.
use super::validation::{term_key, validate_term, TermIssue, MAX_TERMS};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// How bulk input combines with the saved list. Exported to TypeScript under a vocabulary name:
/// word replacements (002) export their own `ImportMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[specta(rename = "VocabularyImportMode")]
pub enum ImportMode {
    /// Keep the saved list and append the new unique terms in input order.
    Merge,
    /// Make the input the whole list, in input order.
    Replace,
}

impl ImportMode {
    /// The mode named `name` (`merge` or `replace`).
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "merge" => Some(Self::Merge),
            "replace" => Some(Self::Replace),
            _ => None,
        }
    }
}

/// One input line kept or skipped, with its zero-based line index. Exported to TypeScript under
/// a vocabulary name: word replacements (002) export their own `PreviewLine`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[specta(rename = "VocabularyPreviewLine")]
pub struct PreviewLine {
    pub index: u32,
    /// Validated text (trimmed, whitespace collapsed).
    pub term: String,
}

/// One refused input line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub struct LineIssue {
    pub index: u32,
    pub issue: TermIssue,
}

/// The candidate list a bulk input would produce, and why it could not be applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub struct ImportPreview {
    pub mode: ImportMode,
    /// New unique terms, in input order.
    pub added: Vec<PreviewLine>,
    /// Lines skipped because their key repeats an earlier line or (merge) a saved term.
    pub duplicates: Vec<PreviewLine>,
    /// Invalid lines: any of them refuses the whole input.
    pub issues: Vec<LineIssue>,
    /// The complete list applying this preview would save.
    pub candidate: Vec<String>,
    /// Applying would exceed `MAX_TERMS` (merge: any addition while the list is full).
    pub over_capacity: bool,
}

impl ImportPreview {
    /// True when the whole candidate can be applied.
    pub fn is_applicable(&self) -> bool {
        self.issues.is_empty() && !self.over_capacity
    }
}

/// Lines of `input`: CRLF and LF separators, one trailing terminator ignored. Interior blank
/// lines are kept (and refused as empty terms); a lone CR inside a line is a line break.
pub fn split_lines(input: &str) -> Vec<&str> {
    let body = input
        .strip_suffix("\r\n")
        .or_else(|| input.strip_suffix('\n'))
        .unwrap_or(input);
    if body.is_empty() {
        return Vec::new();
    }
    body.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
}

/// Preview `input` against the saved `current` list in `mode`: the first spelling of each key
/// wins, later duplicates are skipped without error, invalid lines are reported by index.
// @spec FR-001: Bulk paste preview and atomic candidate — .specs/features/003-recognition-vocabulary/spec.md#fr-001
pub fn prepare_import(current: &[String], input: &str, mode: ImportMode) -> ImportPreview {
    let mut preview = classify_lines(current, input, mode);
    let added = preview.added.iter().map(|line| line.term.clone());
    preview.candidate = match mode {
        ImportMode::Merge => current.iter().cloned().chain(added).collect(),
        ImportMode::Replace => added.collect(),
    };
    preview.over_capacity = match mode {
        ImportMode::Merge => !preview.added.is_empty() && preview.candidate.len() > MAX_TERMS,
        ImportMode::Replace => preview.candidate.len() > MAX_TERMS,
    };
    preview
}

/// Sort the lines of `input` into added, duplicate and invalid ones (candidate not built yet).
fn classify_lines(current: &[String], input: &str, mode: ImportMode) -> ImportPreview {
    let mut seen: HashSet<String> = match mode {
        ImportMode::Merge => current.iter().map(|term| term_key(term)).collect(),
        ImportMode::Replace => HashSet::new(),
    };
    let mut preview = ImportPreview {
        mode,
        added: Vec::new(),
        duplicates: Vec::new(),
        issues: Vec::new(),
        candidate: Vec::new(),
        over_capacity: false,
    };
    for (index, line) in split_lines(input).into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        match validate_term(line) {
            Err(issue) => preview.issues.push(LineIssue { index, issue }),
            Ok(valid) if seen.insert(valid.key.clone()) => preview.added.push(PreviewLine {
                index,
                term: valid.text,
            }),
            Ok(valid) => preview.duplicates.push(PreviewLine {
                index,
                term: valid.text,
            }),
        }
    }
    preview
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn pasted_duplicates_are_skipped_first_spelling_wins() {
        let input = "Claude Code\nClaude Sonnet\nclaude code\nOpenClaw\n";
        let preview = prepare_import(&[], input, ImportMode::Merge);
        assert_eq!(
            preview.candidate,
            list(&["Claude Code", "Claude Sonnet", "OpenClaw"])
        );
        assert_eq!(
            preview.duplicates,
            vec![PreviewLine {
                index: 2,
                term: "claude code".into()
            }]
        );
        assert!(preview.is_applicable());
    }

    #[test]
    fn merge_keeps_saved_bytes_and_skips_saved_keys() {
        let current = list(&["  legacy  ", "Claude Opus"]);
        let preview = prepare_import(&current, "claude opus\r\nOpenClaw\r\n", ImportMode::Merge);
        assert_eq!(
            preview.candidate,
            list(&["  legacy  ", "Claude Opus", "OpenClaw"])
        );
        assert_eq!(preview.duplicates.len(), 1);
    }

    #[test]
    fn invalid_lines_are_reported_by_index_and_block_the_candidate() {
        let input = format!("ok\n\n{}\n<b>\nlast", "x".repeat(101));
        let preview = prepare_import(&[], &input, ImportMode::Replace);
        assert_eq!(
            preview.issues,
            vec![
                LineIssue {
                    index: 1,
                    issue: TermIssue::Empty
                },
                LineIssue {
                    index: 2,
                    issue: TermIssue::TooLong
                },
                LineIssue {
                    index: 3,
                    issue: TermIssue::ForbiddenCharacter
                },
            ]
        );
        assert!(!preview.is_applicable());
    }

    #[test]
    fn capacity_is_checked_on_the_complete_candidate() {
        let full = (0..500)
            .map(|index| format!("t{index}"))
            .collect::<Vec<_>>();
        assert!(prepare_import(&full, "new", ImportMode::Merge).over_capacity);
        assert!(!prepare_import(&full, "t1", ImportMode::Merge).over_capacity);
        let input = (0..501)
            .map(|index| format!("n{index}\n"))
            .collect::<String>();
        assert!(prepare_import(&[], &input, ImportMode::Replace).over_capacity);
        let exact = (0..500)
            .map(|index| format!("n{index}\n"))
            .collect::<String>();
        assert!(!prepare_import(&[], &exact, ImportMode::Replace).over_capacity);
    }

    #[test]
    fn empty_input_clears_on_replace_and_changes_nothing_on_merge() {
        let current = list(&["a"]);
        assert_eq!(
            prepare_import(&current, "", ImportMode::Merge).candidate,
            current
        );
        assert!(prepare_import(&current, "\n", ImportMode::Replace)
            .candidate
            .is_empty());
        assert_eq!(split_lines("a\nb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\r\nb\r\n"), vec!["a", "b"]);
        assert_eq!(split_lines("a\n\n"), vec!["a", ""]);
    }
}
