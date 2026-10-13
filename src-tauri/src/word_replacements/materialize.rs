//! Commit-time materialization of an error-free preview into an ordered rule set.

use super::{
    model::{ReplacementRule, RuleIdSource},
    preview::ReplacementPreview,
    validation::{validate_rules, RuleIssue},
};

/// Why a preview cannot become a committed rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewRejected {
    /// The preview reports errors; nothing may be committed.
    HasErrors,
    /// The materialized set failed the shared validator (e.g. colliding new IDs).
    Invalid(Vec<RuleIssue>),
}

impl std::fmt::Display for PreviewRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HasErrors => formatter.write_str("Replacement preview contains errors"),
            Self::Invalid(issues) => write!(
                formatter,
                "Replacement candidate failed validation ({} issues)",
                issues.len()
            ),
        }
    }
}

impl std::error::Error for PreviewRejected {}

/// Turn an error-free preview into the exact ordered set to commit, allocating
/// IDs only for new rules; content and order never differ from the preview.
///
/// # Errors
///
/// - `PreviewRejected::HasErrors` when the preview cannot be applied.
/// - `PreviewRejected::Invalid` when the final set fails the shared validator.
pub fn materialize_candidate(
    preview: &ReplacementPreview,
    ids: &mut dyn RuleIdSource,
) -> Result<Vec<ReplacementRule>, PreviewRejected> {
    if !preview.can_apply() {
        return Err(PreviewRejected::HasErrors);
    }
    let rules: Vec<ReplacementRule> = preview
        .candidate
        .iter()
        .map(|entry| ReplacementRule {
            id: entry.existing_id.clone().unwrap_or_else(|| ids.next_id()),
            recognized: entry.recognized.clone(),
            replacement: entry.replacement.clone(),
        })
        .collect();
    validate_rules(&rules).map_err(PreviewRejected::Invalid)
}
