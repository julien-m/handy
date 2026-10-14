//! `handy settings replacements`: the ordered word replacement rules of feature 002, listed,
//! changed one rule at a time, imported from and exported to the exchange format. Every change
//! runs the rule-set edits of the settings UI (`word_replacements::service`): same validator,
//! same merge/replace preview, one atomic commit through the shared path.
use super::{
    ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest, ReportFn, ReportedChange,
};
use crate::settings::AppSettings;
use crate::settings_service::errors::{ItemIssue, SettingsError, SettingsErrorCode};
use crate::word_replacements::{
    exchange::{serialize_exchange, MAX_IMPORT_BYTES},
    model::{ReplacementRule, RuleId, RuleIdSource, UuidRuleIdSource},
    preview::{preview_exchange, ImportMode, PreviewLine, PreviewStatus, ReplacementPreview},
    service::{self, new_rule_count, PreallocatedIds, RuleSetError},
    validation::RuleIssue,
};
use serde_json::{json, Value};

/// Settings key of the rules; validation errors name it in `details.field`.
pub const RULES_KEY: &str = "word_replacements";

/// The replacements domain.
pub struct ReplacementsDomain;
impl Domain for ReplacementsDomain {
    fn name(&self) -> &'static str {
        "replacements"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["list", "add", "update", "remove", "import", "export"]
    }
    // @spec FR-006: Replacement CLI operations — .specs/features/002-word-replacements/spec.md#fr-006
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        let rules = &settings.word_replacements;
        match (request.action, request.arguments) {
            ("list", []) => Ok(DomainPlan::Read(json!({ "rules": rule_values(rules) }))),
            ("export", []) => Ok(DomainPlan::Read(export(rules))),
            ("add", [recognized, replacement]) => Ok(add(recognized, replacement)),
            ("update", [id, recognized, replacement]) => Ok(update(id, recognized, replacement)),
            ("remove", [id]) => Ok(remove(id)),
            ("import", [mode]) => import(rules, mode, request.input),
            _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
        }
    }
}

/// Exchange text of the saved rules (header, then one line per rule in order) and their count.
fn export(rules: &[ReplacementRule]) -> Value {
    json!({ "content": serialize_exchange(rules), "count": rules.len() })
}
fn add(recognized: &str, replacement: &str) -> DomainPlan {
    // Allocated once: the edit may run twice (dry run or pre-check, then commit) on one identity.
    let id = UuidRuleIdSource.next_id();
    let (recognized, replacement) = (recognized.to_owned(), replacement.to_owned());
    let edit_id = id.clone();
    DomainPlan::Reported(ReportedChange {
        change: rules_change(move |saved| {
            service::add_rule(saved, edit_id.clone(), &recognized, &replacement)
        }),
        report: rule_report(id),
        bound: false,
    })
}
fn update(id: &str, recognized: &str, replacement: &str) -> DomainPlan {
    let id = RuleId::from(id);
    let (recognized, replacement) = (recognized.to_owned(), replacement.to_owned());
    let edit_id = id.clone();
    DomainPlan::Reported(ReportedChange {
        change: rules_change(move |saved| {
            service::update_rule(saved, &edit_id, &recognized, &replacement)
        }),
        report: rule_report(id),
        bound: false,
    })
}
fn remove(id: &str) -> DomainPlan {
    let id = RuleId::from(id);
    let edit_id = id.clone();
    DomainPlan::Reported(ReportedChange {
        change: rules_change(move |saved| service::remove_rule(saved, &edit_id)),
        report: Box::new(move |settings| {
            Ok(json!({ "removed": id.as_str(), "count": settings.word_replacements.len() }))
        }),
        bound: false,
    })
}
/// Import `input` in `mode`. The preview decided here is the one committed: the change is bound
/// to this read's revision and its new identities are allocated now.
fn import(
    rules: &[ReplacementRule],
    mode: &str,
    input: Option<&str>,
) -> Result<DomainPlan, SettingsError> {
    let mode = import_mode(mode)?;
    let input = input.ok_or_else(|| SettingsError::new(SettingsErrorCode::UsageError))?;
    // AC-005: 1 MB is 1,000,000 bytes inclusive, below the channel's 1 MiB input bound.
    if input.len() > MAX_IMPORT_BYTES {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    let preview = preview_exchange(rules, input, mode);
    if !preview.can_apply() {
        return Err(input_refusal(&preview));
    }
    let ids: Vec<RuleId> = (0..new_rule_count(&preview))
        .map(|_| UuidRuleIdSource.next_id())
        .collect();
    let summary = import_report(&preview);
    let input = input.to_owned();
    Ok(DomainPlan::Reported(ReportedChange {
        change: rules_change(move |saved| {
            service::import_rules(saved, &input, mode, &mut PreallocatedIds::new(&ids))
                .map(|outcome| outcome.rules)
        }),
        report: Box::new(move |settings| with_count(&summary, settings)),
        bound: true,
    }))
}
fn import_mode(mode: &str) -> Result<ImportMode, SettingsError> {
    match mode {
        "merge" => Ok(ImportMode::Merge),
        "replace" => Ok(ImportMode::Replace),
        _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
    }
}
/// A typed edit of the rules, applied by the shared path to the locked settings.
fn rules_change(
    edit: impl Fn(&[ReplacementRule]) -> Result<Vec<ReplacementRule>, RuleSetError>
        + Send
        + Sync
        + 'static,
) -> DomainChange {
    DomainChange(ChangeKind::Apply(Box::new(
        move |settings: &mut AppSettings| {
            settings.word_replacements = edit(&settings.word_replacements).map_err(refusal)?;
            Ok(())
        },
    )))
}
/// `{rule, position, count}` of rule `id` in the resulting settings.
fn rule_report(id: RuleId) -> ReportFn {
    Box::new(move |settings| {
        let rules = &settings.word_replacements;
        let position = rules
            .iter()
            .position(|rule| rule.id == id)
            .ok_or_else(|| SettingsError::new(SettingsErrorCode::InternalError))?;
        Ok(json!({
            "rule": rule_value(&rules[position]),
            "position": position + 1,
            "count": rules.len(),
        }))
    })
}
fn with_count(summary: &Value, settings: &AppSettings) -> Result<Value, SettingsError> {
    let mut report = summary.clone();
    report["count"] = json!(settings.word_replacements.len());
    Ok(report)
}
/// Mode, status counts and one `{line?, status, id?}` per rule line or deletion; ignored lines
/// are only counted and rule text is left out, so the report stays far below one reply frame.
fn import_report(preview: &ReplacementPreview) -> Value {
    let count = |status| {
        preview
            .lines
            .iter()
            .filter(|line| line.status == status)
            .count()
    };
    let lines: Vec<Value> = preview
        .lines
        .iter()
        .filter(|line| line.status != PreviewStatus::Ignored)
        .map(line_value)
        .collect();
    json!({
        "mode": mode_name(preview.mode),
        "added": count(PreviewStatus::Add),
        "updated": count(PreviewStatus::Update),
        "deleted": count(PreviewStatus::Delete),
        "unchanged": count(PreviewStatus::Unchanged),
        "ignored": count(PreviewStatus::Ignored),
        "lines": lines,
    })
}
fn line_value(line: &PreviewLine) -> Value {
    let mut value = json!({ "status": status_name(line.status) });
    if let Some(number) = line.line_number {
        value["line"] = json!(number);
    }
    if let Some(id) = &line.rule_id {
        value["id"] = json!(id.as_str());
    }
    value
}
fn mode_name(mode: ImportMode) -> &'static str {
    match mode {
        ImportMode::Merge => "merge",
        ImportMode::Replace => "replace",
    }
}
fn status_name(status: PreviewStatus) -> &'static str {
    match status {
        PreviewStatus::Add => "add",
        PreviewStatus::Update => "update",
        PreviewStatus::Delete => "delete",
        PreviewStatus::Unchanged => "unchanged",
        PreviewStatus::Ignored => "ignored",
        PreviewStatus::Error => "error",
    }
}
/// Rules as `{id, recognized, replacement}` objects, in saved order.
pub fn rule_values(rules: &[ReplacementRule]) -> Vec<Value> {
    rules.iter().map(rule_value).collect()
}
fn rule_value(rule: &ReplacementRule) -> Value {
    json!({
        "id": rule.id.as_str(),
        "recognized": rule.recognized,
        "replacement": rule.replacement,
    })
}
/// Typed refusal of a rule-set edit, with item locations and no rule text.
pub fn refusal(error: RuleSetError) -> SettingsError {
    match error {
        RuleSetError::NotFound => SettingsError::new(SettingsErrorCode::NotFound),
        RuleSetError::Invalid(issues) => rule_refusal(&issues),
        RuleSetError::InvalidInput(preview) => input_refusal(&preview),
    }
}
fn rule_refusal(issues: &[RuleIssue]) -> SettingsError {
    let items = issues
        .iter()
        .map(|issue| ItemIssue {
            line: None,
            position: issue.rule_index.map(|index| index as u64 + 1),
            field: issue.field.as_str(),
            kind: issue.kind.as_str(),
        })
        .collect();
    SettingsError::invalid_items(RULES_KEY, items)
}
/// Every issue of every error line in input order, then the set issues.
fn input_refusal(preview: &ReplacementPreview) -> SettingsError {
    let lines = preview.lines.iter().flat_map(|line| {
        line.issues.iter().map(move |issue| ItemIssue {
            line: line.line_number.map(|number| number as u64),
            position: None,
            field: issue.field.as_str(),
            kind: issue.kind.as_str(),
        })
    });
    let set = preview.set_issues.iter().map(|kind| ItemIssue {
        line: None,
        position: None,
        field: "rules",
        kind: kind.as_str(),
    });
    SettingsError::invalid_items(RULES_KEY, lines.chain(set).collect())
}
#[cfg(test)]
#[path = "replacements_tests.rs"]
mod tests;
