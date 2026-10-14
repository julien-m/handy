//! `handy settings replacements`: the ordered word replacement rules of feature 002, listed,
//! changed one rule at a time, imported from and exported to the exchange format. Every change
//! runs the rule-set edits of the settings UI (`word_replacements::service`): same validator,
//! same merge/replace preview, one atomic commit through the shared path.
use super::{
    ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest, ReportFn, ReportedChange,
};
use crate::settings::AppSettings;
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use crate::settings_service::public::PublicSettings;
use crate::word_replacements::{
    exchange::{serialize_exchange, MAX_IMPORT_BYTES},
    model::{ReplacementRule, RuleId, RuleIdSource, UuidRuleIdSource},
    preview::{ImportMode, PreviewLine, PreviewStatus, ReplacementPreview},
    service::{self, PlannedImport, RuleSetError},
    transfer::{self, StageError},
};
use serde_json::{json, Value};

pub use super::replacements_refusal::refusal;

/// Settings key of the rules; validation errors name it in `details.field`.
pub const RULES_KEY: &str = "word_replacements";

/// The replacements domain.
pub struct ReplacementsDomain;
impl Domain for ReplacementsDomain {
    fn name(&self) -> &'static str {
        "replacements"
    }
    /// Public actions, then the private transfer steps the CLI uses when a list, a `get` or an
    /// import does not fit one channel message (`read-page`, `read-settings`, `stage-begin`,
    /// `stage-append`).
    fn actions(&self) -> &'static [&'static str] {
        &[
            "list",
            "add",
            "update",
            "remove",
            "import",
            "export",
            "read-page",
            "read-settings",
            "stage-begin",
            "stage-append",
        ]
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
            ("import", [mode, token]) => import(rules, mode, Some(&staged_input(token)?)),
            ("read-page", [start]) => read_page(rules, start),
            ("read-settings", []) => settings_without_rules(settings),
            ("stage-begin", [length, sha256]) => stage_begin(length, sha256),
            ("stage-append", [token, index]) => stage_append(token, index, request.input),
            _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
        }
    }
}

/// Encoded rules per page: half a channel message, so a page reply always fits one message.
pub const MAX_PAGE_BYTES: usize = 512 * 1024;

/// Rules from position `start` (0-based) up to `MAX_PAGE_BYTES` of JSON (at least one), with
/// the total and the start of the next page (`null` after the last). The envelope revision lets
/// the reader refuse pages of different revisions.
fn read_page(rules: &[ReplacementRule], start: &str) -> Result<DomainPlan, SettingsError> {
    let start: usize = start.parse().map_err(|_| usage())?;
    let mut page = Vec::new();
    let mut bytes = 0;
    for rule in rules.iter().skip(start) {
        let value = rule_value(rule);
        bytes += value.to_string().len() + 1;
        // Loop exit: the page is full; the first rule always fits (a rule is a few KiB at most).
        if bytes > MAX_PAGE_BYTES && !page.is_empty() {
            break;
        }
        page.push(value);
    }
    let end = start.saturating_add(page.len());
    let next = (end < rules.len()).then_some(end);
    Ok(DomainPlan::Read(
        json!({ "rules": page, "total": rules.len(), "next": next }),
    ))
}
/// The public settings of `get` without `word_replacements`: a `get` of every setting reads
/// them here and the rules in pages of the same revision (see `replacements_transfer`).
fn settings_without_rules(settings: &AppSettings) -> Result<DomainPlan, SettingsError> {
    let mut public = serde_json::to_value(PublicSettings::from_settings(settings))
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    if let Some(object) = public.as_object_mut() {
        object.remove(RULES_KEY);
    }
    Ok(DomainPlan::Read(public))
}
/// Reserve a staged import input (see `word_replacements::transfer`).
fn stage_begin(length: &str, sha256: &str) -> Result<DomainPlan, SettingsError> {
    let length: usize = length.parse().map_err(|_| usage())?;
    let token = transfer::staging()
        .begin(length, sha256)
        .map_err(stage_refusal)?;
    transfer::schedule_sweep();
    Ok(DomainPlan::Read(json!({ "token": token })))
}
fn stage_append(
    token: &str,
    index: &str,
    chunk: Option<&str>,
) -> Result<DomainPlan, SettingsError> {
    let index: u64 = index.parse().map_err(|_| usage())?;
    let chunk = chunk.ok_or_else(usage)?;
    transfer::staging()
        .append(token, index, chunk)
        .map_err(stage_refusal)?;
    Ok(DomainPlan::Read(json!({ "received": chunk.len() })))
}
/// The complete staged input of `token`, removed from staging.
fn staged_input(token: &str) -> Result<String, SettingsError> {
    transfer::staging().take(token).map_err(stage_refusal)
}
/// Stable code of a staging refusal: start again (unavailable), fix the request (usage) or the
/// input (limit).
fn stage_refusal(error: StageError) -> SettingsError {
    let code = match error {
        StageError::Unknown | StageError::Saturated => SettingsErrorCode::Unavailable,
        StageError::Mismatch => SettingsErrorCode::UsageError,
        StageError::TooLarge => SettingsErrorCode::LimitExceeded,
    };
    SettingsError::new(code)
}
fn usage() -> SettingsError {
    SettingsError::new(SettingsErrorCode::UsageError)
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
/// to this read's revision, its new identities are allocated now, and its set is reused at the
/// commit instead of computing the preview again under the settings lock.
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
    let (planned, preview) =
        PlannedImport::new(rules, input.to_owned(), mode, &mut UuidRuleIdSource)
            .map_err(refusal)?;
    let summary = import_report(&preview);
    Ok(DomainPlan::Reported(ReportedChange {
        change: rules_change(move |saved| planned.apply(saved)),
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
/// Mode, status counts and one `{line?, status, id?}` per rule line or deletion (an applicable
/// preview lists them all); ignored lines are only counted and rule text is left out, so the
/// report stays far below one reply frame.
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
        "ignored": preview.ignored_lines,
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
#[cfg(test)]
#[path = "replacements_tests.rs"]
mod tests;
