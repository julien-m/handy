//! Settings window commands of the replacement rules (feature 002): the published rules with
//! their revision, the shared validator, the bulk/import preview, and the two commits of the
//! editor (list Save of the complete draft, bulk/import Apply of a previewed input). Each commit
//! goes through the settings service once, only at the revision the editor started from, with
//! the same rule-set edits and refusals as `handy settings replacements`; a refusal reaches the
//! editor as the shared typed settings error (`SettingsCommandError`), never as a value.

use crate::settings::AppSettings;
use crate::settings_service::{
    command_error::SettingsCommandError,
    domains::replacements::refusal,
    errors::{SettingsError, SettingsErrorCode},
    protocol::Revision,
    tauri_adapter,
};
use crate::word_replacements::{
    exchange::MAX_IMPORT_BYTES,
    model::{ReplacementRule, RuleId, RuleIdSource, UuidRuleIdSource},
    preview::{preview_exchange, ImportMode, ReplacementPreview},
    service::{PlannedImport, PreallocatedIds, RuleSetError},
    validation::{validate_rules, RuleIssue},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

/// Saved rules in order, with the settings revision they were read at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ReplacementRulesView {
    pub revision: u64,
    pub rules: Vec<ReplacementRule>,
}

/// One rule of the editor's draft; a new rule has no id until it is saved.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
pub struct RuleDraft {
    pub id: Option<RuleId>,
    pub recognized: String,
    pub replacement: String,
}

/// A bulk or import preview against the rules saved at `revision`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ReplacementPreviewView {
    pub revision: u64,
    pub preview: ReplacementPreview,
}

#[tauri::command]
#[specta::specta]
/// The published rules and their revision.
pub fn get_word_replacements(app: AppHandle) -> ReplacementRulesView {
    let (revision, settings) = tauri_adapter::view(&app);
    ReplacementRulesView {
        revision: revision.0,
        rules: settings.word_replacements,
    }
}

// @spec FR-002: Shared typed field validation — .specs/features/002-word-replacements/spec.md#fr-002
#[tauri::command]
#[specta::specta]
/// Issues of a draft list (fields, duplicate keys, set size), from the shared validator; empty
/// when the draft can be saved.
pub fn check_word_replacements(rules: Vec<RuleDraft>) -> Vec<RuleIssue> {
    validate_rules(&draft_rules(&rules, &fresh_ids(&rules)))
        .err()
        .unwrap_or_default()
}

// @spec FR-004: Merge/replace preview without writes — .specs/features/002-word-replacements/spec.md#fr-004
#[tauri::command]
#[specta::specta]
/// Preview `input` (bulk text or imported file) in `mode` against the published rules, on a
/// blocking worker: a 1,000,000-byte input never stalls the main thread.
///
/// # Errors
/// `limit_exceeded` above 1,000,000 bytes, like Apply: nothing is previewed.
pub async fn preview_word_replacements(
    app: AppHandle,
    input: String,
    mode: ImportMode,
) -> Result<ReplacementPreviewView, SettingsCommandError> {
    tauri_adapter::blocking(move || {
        let (revision, settings) = tauri_adapter::view(&app);
        preview_view(revision, &settings.word_replacements, &input, mode)
            .map_err(SettingsCommandError::from)
    })
    .await
}

/// The preview of `input` at `revision`, refused above the import size bound.
fn preview_view(
    revision: Revision,
    saved: &[ReplacementRule],
    input: &str,
    mode: ImportMode,
) -> Result<ReplacementPreviewView, SettingsError> {
    if input.len() > MAX_IMPORT_BYTES {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    Ok(ReplacementPreviewView {
        revision: revision.0,
        preview: preview_exchange(saved, input, mode),
    })
}

// @spec AC-001: List draft saved once — .specs/features/002-word-replacements/spec.md#ac-001
#[tauri::command]
#[specta::specta]
/// Save the complete draft list as the rule set, in one commit at `expected_revision`, and
/// return the saved rules with their revision.
///
/// # Errors
/// A `SettingsCommandError`, nothing saved: `validation_failed` (`invalid_items`; the fields
/// come from `check_word_replacements`) for an invalid draft, `revision_conflict` with the
/// current revision when the settings moved on, and every refusal of the settings service.
pub async fn save_word_replacements(
    app: AppHandle,
    rules: Vec<RuleDraft>,
    expected_revision: u64,
) -> Result<ReplacementRulesView, SettingsCommandError> {
    let rules = draft_rules(&rules, &fresh_ids(&rules));
    tauri_adapter::blocking(move || {
        let edit = save_edit(rules);
        let commit = tauri_adapter::edit_at(&app, &edit, Some(Revision(expected_revision)))?;
        Ok(rules_view(commit.revision, commit.after))
    })
    .await
}

// @spec AC-003: Apply commits exactly the previewed set — .specs/features/002-word-replacements/spec.md#ac-003
#[tauri::command]
#[specta::specta]
/// Apply `input` in `mode` (the bulk editor's Apply or a file import) in one commit at
/// `expected_revision`, the revision its preview was computed at, and return the saved rules.
///
/// # Errors
/// A `SettingsCommandError`, nothing saved: `limit_exceeded` above 1,000,000 bytes,
/// `validation_failed` (`invalid_items`; the lines come from `preview_word_replacements`), and
/// the refusals of `save_word_replacements`.
pub async fn apply_word_replacements(
    app: AppHandle,
    input: String,
    mode: ImportMode,
    expected_revision: u64,
) -> Result<ReplacementRulesView, SettingsCommandError> {
    tauri_adapter::blocking(move || {
        let expected = Revision(expected_revision);
        let edit = import_edit_at(tauri_adapter::view(&app), input, mode, expected)?;
        let commit = tauri_adapter::edit_at(&app, &edit, Some(expected))?;
        Ok(rules_view(commit.revision, commit.after))
    })
    .await
}

/// Edit replacing the rule set with `rules`, refused unless they pass the shared validator.
fn save_edit(
    rules: Vec<ReplacementRule>,
) -> impl Fn(&mut AppSettings) -> Result<(), SettingsError> {
    move |settings| {
        settings.word_replacements =
            validate_rules(&rules).map_err(|issues| refusal(RuleSetError::Invalid(issues)))?;
        Ok(())
    }
}

/// `import_edit` for an Apply computed at `expected`, refused with `revision_conflict` unless the
/// published settings (`revision`, `settings`) are still at that revision.
///
/// # Errors
/// `revision_conflict` with the current revision, otherwise those of `import_edit`.
fn import_edit_at(
    (revision, settings): (Revision, AppSettings),
    input: String,
    mode: ImportMode,
    expected: Revision,
) -> Result<impl Fn(&mut AppSettings) -> Result<(), SettingsError>, SettingsError> {
    if revision != expected {
        return Err(SettingsError::revision_conflict(revision));
    }
    import_edit(&settings.word_replacements, input, mode)
}

/// Edit applying `input` in `mode` to the locked rules, decided now on the preview of `saved`
/// (the rules at the expected revision) with the identities of its new rules; under the lock the
/// decided set is reused while the rules are unchanged.
///
/// # Errors
/// `limit_exceeded` above 1,000,000 bytes; `validation_failed` when the preview cannot apply.
fn import_edit(
    saved: &[ReplacementRule],
    input: String,
    mode: ImportMode,
) -> Result<impl Fn(&mut AppSettings) -> Result<(), SettingsError>, SettingsError> {
    if input.len() > MAX_IMPORT_BYTES {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    let (planned, _preview) =
        PlannedImport::new(saved, input, mode, &mut UuidRuleIdSource).map_err(refusal)?;
    Ok(move |settings: &mut AppSettings| {
        settings.word_replacements = planned
            .apply(&settings.word_replacements)
            .map_err(refusal)?;
        Ok(())
    })
}

/// One fresh identity per draft rule without one, in draft order.
fn fresh_ids(rules: &[RuleDraft]) -> Vec<RuleId> {
    let count = rules.iter().filter(|rule| rule.id.is_none()).count();
    (0..count).map(|_| UuidRuleIdSource.next_id()).collect()
}

/// The draft as rules, new rules taking `ids` in order.
fn draft_rules(drafts: &[RuleDraft], ids: &[RuleId]) -> Vec<ReplacementRule> {
    let mut source = PreallocatedIds::new(ids);
    drafts
        .iter()
        .map(|draft| ReplacementRule {
            id: draft.id.clone().unwrap_or_else(|| source.next_id()),
            recognized: draft.recognized.clone(),
            replacement: draft.replacement.clone(),
        })
        .collect()
}

fn rules_view(revision: Revision, settings: AppSettings) -> ReplacementRulesView {
    ReplacementRulesView {
        revision: revision.0,
        rules: settings.word_replacements,
    }
}

#[cfg(test)]
#[path = "replacements_tests.rs"]
mod tests;
