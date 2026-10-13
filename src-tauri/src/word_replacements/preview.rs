//! Deterministic merge/replace previews of exchange text against a saved rule set.
//!
//! A preview never writes state. Binding it to a settings revision and
//! committing it belong to the 008 settings transaction adapter.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;

pub use super::materialize::{materialize_candidate, PreviewRejected};

use super::{
    exchange::{parse_exchange, IgnoredLine, ParsedLine, ParsedLineContent},
    model::{ReplacementRule, RuleId},
    unicode::UnicodeData,
    validation::{validate_rules, Field, FieldIssue, IssueKind, MAX_RULES},
};

/// How imported rules combine with the saved set (AC-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    /// Update matching keys in place and append new rules in input order.
    Merge,
    /// The input becomes the whole set; the bulk editor always uses this mode.
    Replace,
}

/// Per-line outcome shown before Apply (AC-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PreviewStatus {
    Add,
    Update,
    Delete,
    Unchanged,
    Ignored,
    Error,
}

/// One preview row; deletions have no input line number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct PreviewLine {
    pub line_number: Option<usize>,
    pub status: PreviewStatus,
    pub ignored: Option<IgnoredLine>,
    /// Existing rule updated, kept unchanged or deleted by this row.
    pub rule_id: Option<RuleId>,
    /// Resulting content for add/update/unchanged, removed content for delete.
    pub recognized: Option<String>,
    pub replacement: Option<String>,
    pub issues: Vec<FieldIssue>,
    /// Other input lines sharing this line's normalized key.
    pub duplicate_lines: Vec<usize>,
}

/// Candidate rule; new rules receive a durable ID only when committed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct CandidateRule {
    pub existing_id: Option<RuleId>,
    pub recognized: String,
    pub replacement: String,
}

/// Complete preview; `candidate` is exactly the ordered set Apply would commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ReplacementPreview {
    pub mode: ImportMode,
    /// SHA-256 of the exact input text, so Apply can prove it used this input.
    pub input_sha256: String,
    pub lines: Vec<PreviewLine>,
    pub set_issues: Vec<IssueKind>,
    pub candidate: Vec<CandidateRule>,
    /// The candidate equals the saved set in content, identity and order.
    pub is_noop: bool,
}

impl ReplacementPreview {
    /// Apply is available only without any line or set error (AC-003).
    pub fn can_apply(&self) -> bool {
        self.set_issues.is_empty()
            && self
                .lines
                .iter()
                .all(|line| line.status != PreviewStatus::Error)
    }
}

struct InputRule {
    line_index: usize,
    key: String,
    recognized: String,
    replacement: String,
}

// @spec FR-004: Merge/replace preview without writes — .specs/features/002-word-replacements/spec.md#fr-004
/// Compare exchange text with the saved set in `mode`; pure and deterministic.
pub fn preview_exchange(
    saved: &[ReplacementRule],
    input: &str,
    mode: ImportMode,
) -> ReplacementPreview {
    let input_sha256 = format!("{:x}", Sha256::digest(input.as_bytes()));
    let Ok((data, parsed)) =
        UnicodeData::load().and_then(|data| Ok((data, parse_exchange(input)?)))
    else {
        return unavailable_preview(mode, input_sha256);
    };
    let mut lines: Vec<PreviewLine> = parsed.iter().map(initial_line).collect();
    let input_rules = collect_input_rules(data, &parsed);
    let candidate = classify_input(data, saved, &input_rules, mode, &mut lines);
    ReplacementPreview {
        set_issues: candidate_set_issues(saved, &candidate),
        is_noop: is_same_set(saved, &candidate),
        mode,
        input_sha256,
        lines,
        candidate,
    }
}

/// Classify rule lines against the saved set and build the ordered candidate.
fn classify_input(
    data: &UnicodeData,
    saved: &[ReplacementRule],
    input_rules: &[InputRule],
    mode: ImportMode,
    lines: &mut Vec<PreviewLine>,
) -> Vec<CandidateRule> {
    let duplicated = mark_input_duplicates(input_rules, lines);
    let accepted: Vec<&InputRule> = input_rules
        .iter()
        .filter(|rule| !duplicated.contains(&rule.line_index))
        .collect();
    let saved_keys = index_saved_keys(data, saved);
    match mode {
        ImportMode::Merge => merge_candidate(saved, &saved_keys, &accepted, lines),
        ImportMode::Replace => {
            // Deletions follow the input rows, so input line indices stay valid.
            lines.extend(deletion_lines(data, saved, input_rules));
            replace_candidate(saved, &saved_keys, &accepted, lines)
        }
    }
}

fn candidate_set_issues(saved: &[ReplacementRule], candidate: &[CandidateRule]) -> Vec<IssueKind> {
    let mut issues = Vec::new();
    // A corrupt base cannot be reconciled by key; refuse Apply rather than guess.
    if validate_rules(saved).is_err() {
        issues.push(IssueKind::InvalidSavedRules);
    }
    if candidate.len() > MAX_RULES {
        issues.push(IssueKind::TooManyRules);
    }
    issues
}

fn unavailable_preview(mode: ImportMode, input_sha256: String) -> ReplacementPreview {
    ReplacementPreview {
        mode,
        input_sha256,
        lines: Vec::new(),
        set_issues: vec![IssueKind::UnicodeDataUnavailable],
        candidate: Vec::new(),
        is_noop: false,
    }
}

fn initial_line(line: &ParsedLine) -> PreviewLine {
    let (status, ignored, issues) = match &line.content {
        ParsedLineContent::Ignored { reason } => {
            (PreviewStatus::Ignored, Some(*reason), Vec::new())
        }
        ParsedLineContent::Invalid { issues } => (PreviewStatus::Error, None, issues.clone()),
        // Rule lines are classified once compared with the saved set.
        ParsedLineContent::Rule { .. } => (PreviewStatus::Add, None, Vec::new()),
    };
    PreviewLine {
        line_number: Some(line.line_number),
        status,
        ignored,
        rule_id: None,
        recognized: None,
        replacement: None,
        issues,
        duplicate_lines: Vec::new(),
    }
}

fn collect_input_rules(data: &UnicodeData, parsed: &[ParsedLine]) -> Vec<InputRule> {
    parsed
        .iter()
        .enumerate()
        .filter_map(|(line_index, line)| match &line.content {
            ParsedLineContent::Rule {
                recognized,
                replacement,
            } => Some(InputRule {
                line_index,
                key: data.normalized_key(recognized),
                recognized: recognized.clone(),
                replacement: replacement.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// Mark every participant of a duplicated key as an error and return their indices.
fn mark_input_duplicates(input: &[InputRule], lines: &mut [PreviewLine]) -> HashSet<usize> {
    let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
    for rule in input {
        groups
            .entry(rule.key.as_str())
            .or_default()
            .push(rule.line_index);
    }
    let mut duplicated = HashSet::new();
    // AC-005: duplicates inside one input are errors for all participants, never first-wins.
    for indices in groups.into_values().filter(|indices| indices.len() > 1) {
        for &index in &indices {
            let line = &mut lines[index];
            line.status = PreviewStatus::Error;
            line.issues.push(FieldIssue {
                field: Field::Recognized,
                kind: IssueKind::DuplicateKey,
            });
            line.duplicate_lines = indices
                .iter()
                .filter(|&&other| other != index)
                .map(|&other| other + 1)
                .collect();
            duplicated.insert(index);
        }
    }
    duplicated
}

fn index_saved_keys(data: &UnicodeData, saved: &[ReplacementRule]) -> HashMap<String, usize> {
    let mut keys = HashMap::with_capacity(saved.len());
    for (position, rule) in saved.iter().enumerate() {
        // A valid base has unique keys; on a corrupt base keep the first so the
        // preview stays deterministic (Apply is refused by InvalidSavedRules).
        keys.entry(data.normalized_key(&rule.recognized))
            .or_insert(position);
    }
    keys
}

fn merge_candidate(
    saved: &[ReplacementRule],
    saved_keys: &HashMap<String, usize>,
    accepted: &[&InputRule],
    lines: &mut [PreviewLine],
) -> Vec<CandidateRule> {
    let mut candidate: Vec<CandidateRule> = saved.iter().map(candidate_from_saved).collect();
    for rule in accepted {
        let line = &mut lines[rule.line_index];
        // AC-005 merge: a matching key keeps saved spelling, ID and position.
        if let Some(&position) = saved_keys.get(&rule.key) {
            let existing = &mut candidate[position];
            line.status = if existing.replacement == rule.replacement {
                PreviewStatus::Unchanged
            } else {
                PreviewStatus::Update
            };
            existing.replacement = rule.replacement.clone();
            fill_line(line, existing);
        } else {
            let added = new_candidate(rule);
            line.status = PreviewStatus::Add;
            fill_line(line, &added);
            candidate.push(added);
        }
    }
    candidate
}

fn replace_candidate(
    saved: &[ReplacementRule],
    saved_keys: &HashMap<String, usize>,
    accepted: &[&InputRule],
    lines: &mut [PreviewLine],
) -> Vec<CandidateRule> {
    let mut candidate = Vec::with_capacity(accepted.len());
    for rule in accepted {
        let existing = saved_keys.get(&rule.key).map(|&position| &saved[position]);
        let (status, entry) = replace_entry(existing, rule);
        let line = &mut lines[rule.line_index];
        line.status = status;
        fill_line(line, &entry);
        candidate.push(entry);
    }
    candidate
}

/// AC-005 replace: input spelling and order win; a known key keeps its saved ID.
fn replace_entry(
    existing: Option<&ReplacementRule>,
    rule: &InputRule,
) -> (PreviewStatus, CandidateRule) {
    let Some(existing) = existing else {
        return (PreviewStatus::Add, new_candidate(rule));
    };
    let status =
        if existing.recognized == rule.recognized && existing.replacement == rule.replacement {
            PreviewStatus::Unchanged
        } else {
            PreviewStatus::Update
        };
    let entry = CandidateRule {
        existing_id: Some(existing.id.clone()),
        recognized: rule.recognized.clone(),
        replacement: rule.replacement.clone(),
    };
    (status, entry)
}

fn deletion_lines(
    data: &UnicodeData,
    saved: &[ReplacementRule],
    input_rules: &[InputRule],
) -> Vec<PreviewLine> {
    // Keys of every input rule line, duplicates included, are kept by the input.
    let input_keys: HashSet<&str> = input_rules.iter().map(|rule| rule.key.as_str()).collect();
    saved
        .iter()
        .filter(|rule| !input_keys.contains(data.normalized_key(&rule.recognized).as_str()))
        .map(|rule| PreviewLine {
            line_number: None,
            status: PreviewStatus::Delete,
            ignored: None,
            rule_id: Some(rule.id.clone()),
            recognized: Some(rule.recognized.clone()),
            replacement: Some(rule.replacement.clone()),
            issues: Vec::new(),
            duplicate_lines: Vec::new(),
        })
        .collect()
}

fn candidate_from_saved(rule: &ReplacementRule) -> CandidateRule {
    CandidateRule {
        existing_id: Some(rule.id.clone()),
        recognized: rule.recognized.clone(),
        replacement: rule.replacement.clone(),
    }
}

fn new_candidate(rule: &InputRule) -> CandidateRule {
    CandidateRule {
        existing_id: None,
        recognized: rule.recognized.clone(),
        replacement: rule.replacement.clone(),
    }
}

fn fill_line(line: &mut PreviewLine, rule: &CandidateRule) {
    line.rule_id = rule.existing_id.clone();
    line.recognized = Some(rule.recognized.clone());
    line.replacement = Some(rule.replacement.clone());
}

fn is_same_set(saved: &[ReplacementRule], candidate: &[CandidateRule]) -> bool {
    saved.len() == candidate.len()
        && saved.iter().zip(candidate).all(|(rule, entry)| {
            entry.existing_id.as_ref() == Some(&rule.id)
                && entry.recognized == rule.recognized
                && entry.replacement == rule.replacement
        })
}
