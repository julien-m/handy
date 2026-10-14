//! Typed refusals of the replacements domain (002): every located rule or input issue becomes a
//! `details.items` entry of the shared closed list (`ErrorReason`, 006), never the rule text. An
//! item's `index` is zero-based: the input line of an import, or the rule position in the edited
//! list. Issues of the whole set (too many rules, invalid saved rules) have no position and
//! become `details.reason`.
use super::replacements::RULES_KEY;
use crate::settings_service::errors::{ErrorItem, ErrorReason, SettingsError, SettingsErrorCode};
use crate::word_replacements::{
    preview::ReplacementPreview,
    service::RuleSetError,
    validation::{Field, IssueKind, RuleIssue},
};

/// Typed refusal of a rule-set edit, with item locations and no rule text.
pub fn refusal(error: RuleSetError) -> SettingsError {
    match error {
        RuleSetError::NotFound => SettingsError::new(SettingsErrorCode::NotFound),
        RuleSetError::Invalid(issues) => rule_refusal(&issues),
        RuleSetError::InvalidInput(preview) => input_refusal(&preview),
    }
}

fn rule_refusal(issues: &[RuleIssue]) -> SettingsError {
    let items = issues.iter().filter_map(|issue| {
        let index = issue.rule_index?;
        Some(ErrorItem::new(
            index as u64,
            reason(issue.field, issue.kind),
        ))
    });
    let set_reason = issues
        .iter()
        .find(|issue| issue.rule_index.is_none())
        .map(|issue| reason(issue.field, issue.kind));
    SettingsError::invalid_items(RULES_KEY, items, set_reason)
}

/// Every issue of every error line of an import, then the first set issue as the reason.
fn input_refusal(preview: &ReplacementPreview) -> SettingsError {
    let items = preview.lines.iter().flat_map(|line| {
        // Input lines are 1-based; an error line always has one (deletions carry no issue).
        let index = line
            .line_number
            .map(|number| number.saturating_sub(1) as u64);
        line.issues
            .iter()
            .filter_map(move |issue| Some(ErrorItem::new(index?, reason(issue.field, issue.kind))))
    });
    let set_reason = preview
        .set_issues
        .first()
        .map(|kind| reason(Field::Rules, *kind));
    SettingsError::invalid_items(RULES_KEY, items, set_reason)
}

/// Closed reason of one validation issue. Length and character kinds are only produced for the
/// recognized and replacement fields, so any field other than `recognized` is the replacement.
fn reason(field: Field, kind: IssueKind) -> ErrorReason {
    let is_recognized = field == Field::Recognized;
    match kind {
        IssueKind::TooShort if is_recognized => ErrorReason::RecognizedEmpty,
        IssueKind::TooShort => ErrorReason::ReplacementEmpty,
        IssueKind::TooLong if is_recognized => ErrorReason::RecognizedTooLong,
        IssueKind::TooLong => ErrorReason::ReplacementTooLong,
        IssueKind::ForbiddenCharacter if is_recognized => ErrorReason::RecognizedForbiddenCharacter,
        IssueKind::ForbiddenCharacter => ErrorReason::ReplacementForbiddenCharacter,
        IssueKind::ControlCharacter if is_recognized => ErrorReason::RecognizedControlCharacter,
        IssueKind::ControlCharacter => ErrorReason::ReplacementControlCharacter,
        IssueKind::LeadingComment => ErrorReason::RecognizedLeadingComment,
        IssueKind::DuplicateKey => ErrorReason::DuplicateRecognized,
        IssueKind::EmptyId => ErrorReason::EmptyRuleId,
        IssueKind::DuplicateId => ErrorReason::DuplicateRuleId,
        IssueKind::MissingSeparator => ErrorReason::MissingSeparator,
        IssueKind::TooManyRules => ErrorReason::TooManyRules,
        IssueKind::InvalidSavedRules => ErrorReason::InvalidSavedRules,
        IssueKind::UnicodeDataUnavailable => ErrorReason::UnicodeDataUnavailable,
    }
}
