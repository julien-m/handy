//! Shared scalar-bound validation and normalized-key uniqueness for rule drafts.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    model::ReplacementRule,
    unicode::{UnicodeData, UnicodeDataError},
};

/// Inclusive AC-002 maximum for recognized Unicode scalar values.
pub const MAX_RECOGNIZED_SCALARS: usize = 100;
/// Inclusive AC-002 maximum for replacement Unicode scalar values.
pub const MAX_REPLACEMENT_SCALARS: usize = 500;
/// Inclusive AC-002 maximum number of ordered rules.
pub const MAX_RULES: usize = 1000;

/// Stable field identifier translated at UI/CLI boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Id,
    Recognized,
    Replacement,
    Rules,
    /// A whole exchange line, before it can be split into fields.
    Line,
}

/// Stable validation kind; diagnostics never contain user rule text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    EmptyId,
    DuplicateId,
    TooShort,
    TooLong,
    ForbiddenCharacter,
    LeadingComment,
    DuplicateKey,
    TooManyRules,
    UnicodeDataUnavailable,
    /// An exchange line has neither a tab nor a `;` separator.
    MissingSeparator,
    /// The saved base set is itself invalid, so no preview can be applied to it.
    InvalidSavedRules,
}

/// An issue in one input field before a rule has an ordered position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct FieldIssue {
    pub field: Field,
    pub kind: IssueKind,
}

/// Ordered rule issue; index is zero-based or absent for a whole-set issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct RuleIssue {
    pub rule_index: Option<usize>,
    pub field: Field,
    pub kind: IssueKind,
}

/// Derive a full default folded key, reporting unavailable Unicode data safely.
pub fn normalized_key(text: &str) -> Result<String, UnicodeDataError> {
    Ok(UnicodeData::load()?.normalized_key(text))
}

// @spec FR-002: Shared typed field validation — .specs/features/002-word-replacements/spec.md#fr-002
/// Trim fields then validate scalar lengths and forbidden exchange characters.
/// Returns trimmed content only when every field is valid; no state is written.
pub fn validate_fields(
    recognized: &str,
    replacement: &str,
) -> Result<(String, String), Vec<FieldIssue>> {
    let data = UnicodeData::load().map_err(|_| {
        vec![FieldIssue {
            field: Field::Rules,
            kind: IssueKind::UnicodeDataUnavailable,
        }]
    })?;
    validate_fields_with_data(data, recognized, replacement)
}

fn validate_fields_with_data(
    data: &UnicodeData,
    recognized: &str,
    replacement: &str,
) -> Result<(String, String), Vec<FieldIssue>> {
    let recognized = data.trim_whitespace(recognized);
    let replacement = data.trim_whitespace(replacement);
    let mut issues = Vec::new();
    check_field(
        recognized,
        Field::Recognized,
        MAX_RECOGNIZED_SCALARS,
        &mut issues,
    );
    check_field(
        replacement,
        Field::Replacement,
        MAX_REPLACEMENT_SCALARS,
        &mut issues,
    );
    if issues.is_empty() {
        Ok((recognized.to_owned(), replacement.to_owned()))
    } else {
        Err(issues)
    }
}

fn check_field(value: &str, field: Field, max: usize, issues: &mut Vec<FieldIssue>) {
    let count = value.chars().count();
    let is_recognized = field == Field::Recognized;
    let checks = [
        // AC-002: recognized text cannot start like an exchange comment line.
        (
            is_recognized && value.starts_with('#'),
            IssueKind::LeadingComment,
        ),
        (count == 0, IssueKind::TooShort),
        (count > max, IssueKind::TooLong),
        (
            value
                .chars()
                .any(|character| is_forbidden(character, is_recognized)),
            IssueKind::ForbiddenCharacter,
        ),
    ];
    issues.extend(
        checks
            .into_iter()
            .filter(|(has_failed, _)| *has_failed)
            .map(|(_, kind)| FieldIssue { field, kind }),
    );
}

/// Unicode hard line breaks and tabs are forbidden in both fields; `;` only in
/// recognized text, so replacements may contain it literally.
fn is_forbidden(character: char, is_recognized: bool) -> bool {
    matches!(
        character,
        '\r' | '\n' | '\t' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    ) || (is_recognized && character == ';')
}

// @spec AC-002: Rule bounds and duplicate keys — .specs/features/002-word-replacements/spec.md#ac-002
/// Validate an entire ordered set and report every participating duplicate rule.
/// Success preserves identities/order and trims content; failures return no set.
pub fn validate_rules(rules: &[ReplacementRule]) -> Result<Vec<ReplacementRule>, Vec<RuleIssue>> {
    let data = prepare_validation(rules.len())?;
    let mut issues = Vec::new();
    let mut normalized = Vec::with_capacity(rules.len());
    let mut keys: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, rule) in rules.iter().enumerate() {
        validate_ordered_rule(data, index, rule, &mut normalized, &mut issues);
        keys.entry(data.normalized_key(&rule.recognized))
            .or_default()
            .push(index);
    }
    add_duplicates(keys, &mut issues);
    validate_ids(rules, &mut issues);
    issues.sort_by_key(|issue| issue.rule_index);
    if issues.is_empty() {
        Ok(normalized)
    } else {
        Err(issues)
    }
}

fn prepare_validation(count: usize) -> Result<&'static UnicodeData, Vec<RuleIssue>> {
    let set_issue = |kind| {
        vec![RuleIssue {
            rule_index: None,
            field: Field::Rules,
            kind,
        }]
    };
    // Reject oversized input before allocating derived keys or folding its text.
    if count > MAX_RULES {
        return Err(set_issue(IssueKind::TooManyRules));
    }
    UnicodeData::load().map_err(|_| set_issue(IssueKind::UnicodeDataUnavailable))
}

fn validate_ordered_rule(
    data: &UnicodeData,
    index: usize,
    rule: &ReplacementRule,
    normalized: &mut Vec<ReplacementRule>,
    issues: &mut Vec<RuleIssue>,
) {
    match validate_fields_with_data(data, &rule.recognized, &rule.replacement) {
        Ok((recognized, replacement)) => normalized.push(ReplacementRule {
            id: rule.id.clone(),
            recognized,
            replacement,
        }),
        Err(fields) => issues.extend(fields.into_iter().map(|issue| RuleIssue {
            rule_index: Some(index),
            field: issue.field,
            kind: issue.kind,
        })),
    }
}

fn add_duplicates(keys: HashMap<String, Vec<usize>>, issues: &mut Vec<RuleIssue>) {
    // Annotate all participants rather than arbitrarily accepting the first duplicate.
    for indices in keys.into_values().filter(|indices| indices.len() > 1) {
        issues.extend(indices.into_iter().map(|index| RuleIssue {
            rule_index: Some(index),
            field: Field::Recognized,
            kind: IssueKind::DuplicateKey,
        }));
    }
}

// @spec FR-001: Unique opaque rule identities — .specs/features/002-word-replacements/spec.md#fr-001
fn validate_ids(rules: &[ReplacementRule], issues: &mut Vec<RuleIssue>) {
    let mut ids: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, rule) in rules.iter().enumerate() {
        let value = rule.id.as_str();
        if value.is_empty() {
            issues.push(RuleIssue {
                rule_index: Some(index),
                field: Field::Id,
                kind: IssueKind::EmptyId,
            });
        }
        ids.entry(value).or_default().push(index);
    }
    // Exact opaque IDs route UI edits: reject every participant, without folding or trimming.
    for indices in ids.into_values().filter(|indices| indices.len() > 1) {
        issues.extend(indices.into_iter().map(|index| RuleIssue {
            rule_index: Some(index),
            field: Field::Id,
            kind: IssueKind::DuplicateId,
        }));
    }
}
