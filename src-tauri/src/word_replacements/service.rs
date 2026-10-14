//! Rule-set edits shared by the settings UI and the settings CLI: pure functions over the saved
//! ordered rules, checked by the one shared validator. New identities come from the caller, so
//! an edit can run twice (dry run or absent-store pre-check, then the locked commit) and produce
//! exactly the same set.

use super::{
    model::{ReplacementRule, RuleId, RuleIdSource},
    preview::{
        materialize_candidate, preview_exchange, ImportMode, PreviewRejected, ReplacementPreview,
    },
    validation::{validate_fields, validate_rules, RuleIssue},
};

/// Why an edit leaves the saved rules unchanged; never carries rule text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSetError {
    /// No saved rule has this identity.
    NotFound,
    /// The edited rule or the resulting set fails the shared validator.
    Invalid(Vec<RuleIssue>),
    /// The exchange input has error lines or set issues; the preview locates them.
    InvalidInput(Box<ReplacementPreview>),
}

/// An applicable import: its preview and the exact ordered set it commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    pub preview: ReplacementPreview,
    pub rules: Vec<ReplacementRule>,
}

// @spec FR-001: Ordered stable replacement rules — .specs/features/002-word-replacements/spec.md#fr-001
/// Append a rule with identity `id` after the saved rules.
///
/// # Errors
/// `Invalid` with the field issues of the new rule (at its future position) or the set issues
/// (duplicate key, more than 1000 rules).
pub fn add_rule(
    saved: &[ReplacementRule],
    id: RuleId,
    recognized: &str,
    replacement: &str,
) -> Result<Vec<ReplacementRule>, RuleSetError> {
    let (recognized, replacement) = checked_fields(recognized, replacement, saved.len())?;
    let mut rules = saved.to_vec();
    rules.push(ReplacementRule {
        id,
        recognized,
        replacement,
    });
    validated(rules)
}

/// Change both texts of the rule `id`, keeping its identity and position.
///
/// # Errors
/// `NotFound` for an unknown identity; otherwise as `add_rule`.
pub fn update_rule(
    saved: &[ReplacementRule],
    id: &RuleId,
    recognized: &str,
    replacement: &str,
) -> Result<Vec<ReplacementRule>, RuleSetError> {
    let position = position_of(saved, id)?;
    let (recognized, replacement) = checked_fields(recognized, replacement, position)?;
    let mut rules = saved.to_vec();
    rules[position].recognized = recognized;
    rules[position].replacement = replacement;
    validated(rules)
}

/// Remove the rule `id`; the others keep their order.
///
/// The remaining set is not revalidated: removing a rule cannot make a set invalid, and a set
/// damaged by a manual store edit must stay repairable one removal at a time.
///
/// # Errors
/// `NotFound` for an unknown identity.
pub fn remove_rule(
    saved: &[ReplacementRule],
    id: &RuleId,
) -> Result<Vec<ReplacementRule>, RuleSetError> {
    let position = position_of(saved, id)?;
    let mut rules = saved.to_vec();
    rules.remove(position);
    Ok(rules)
}

// @spec FR-004: Merge/replace preview without writes — .specs/features/002-word-replacements/spec.md#fr-004
/// Preview `input` against the saved rules in `mode` and materialize the set to commit, with
/// identities for new rules drawn from `ids` in candidate order.
///
/// # Errors
/// `InvalidInput` when the preview has an error line or a set issue (nothing can be applied);
/// `Invalid` when the materialized set fails the shared validator.
pub fn import_rules(
    saved: &[ReplacementRule],
    input: &str,
    mode: ImportMode,
    ids: &mut dyn RuleIdSource,
) -> Result<ImportOutcome, RuleSetError> {
    let preview = preview_exchange(saved, input, mode);
    match materialize_candidate(&preview, ids) {
        Ok(rules) => Ok(ImportOutcome { preview, rules }),
        Err(PreviewRejected::HasErrors) => Err(RuleSetError::InvalidInput(Box::new(preview))),
        Err(PreviewRejected::Invalid(issues)) => Err(RuleSetError::Invalid(issues)),
    }
}

/// Number of identities a preview allocates when it is committed (one per new rule).
pub fn new_rule_count(preview: &ReplacementPreview) -> usize {
    preview
        .candidate
        .iter()
        .filter(|entry| entry.existing_id.is_none())
        .count()
}

/// Identities allocated once before an edit and handed out in order, so every run of the edit
/// produces the same set. An exhausted pool yields an empty identity, which the shared validator
/// refuses (`empty_id`) instead of inventing one.
pub struct PreallocatedIds<'a> {
    ids: &'a [RuleId],
    next: usize,
}

impl<'a> PreallocatedIds<'a> {
    /// Hand out `ids` from the first one.
    pub fn new(ids: &'a [RuleId]) -> Self {
        Self { ids, next: 0 }
    }
}

impl RuleIdSource for PreallocatedIds<'_> {
    fn next_id(&mut self) -> RuleId {
        let id = self
            .ids
            .get(self.next)
            .cloned()
            .unwrap_or_else(|| "".into());
        self.next += 1;
        id
    }
}

fn position_of(saved: &[ReplacementRule], id: &RuleId) -> Result<usize, RuleSetError> {
    saved
        .iter()
        .position(|rule| &rule.id == id)
        .ok_or(RuleSetError::NotFound)
}

/// Trimmed fields, or their issues reported at `position` of the resulting set.
fn checked_fields(
    recognized: &str,
    replacement: &str,
    position: usize,
) -> Result<(String, String), RuleSetError> {
    validate_fields(recognized, replacement).map_err(|issues| {
        RuleSetError::Invalid(
            issues
                .into_iter()
                .map(|issue| RuleIssue {
                    rule_index: Some(position),
                    field: issue.field,
                    kind: issue.kind,
                })
                .collect(),
        )
    })
}

fn validated(rules: Vec<ReplacementRule>) -> Result<Vec<ReplacementRule>, RuleSetError> {
    validate_rules(&rules).map_err(RuleSetError::Invalid)
}
