//! `handy settings vocabulary` (feature 003): list, add, remove, import and export the recognition
//! vocabulary (the custom-words list) with the validator and commit path of the settings UI, and
//! the status of the selected model, answered only by the running instance (a CLI process never
//! loads a model).
use super::{
    live_operation::{managed_state, LiveAnswer, LiveCall, LiveContext, LiveOperation},
    Domain, DomainPlan, DomainRequest,
};
use crate::managers::transcription::TranscriptionManager;
use crate::recognition_vocabulary::{
    edits::VocabularyEdit, occurrences::inspect, preview::ImportMode,
};
use crate::settings::AppSettings;
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use serde_json::{json, Value};
use std::sync::Arc;

/// Only export target of this build: stdout (a file is a shell redirection).
const STDOUT_TARGET: &str = "-";

/// The vocabulary domain.
pub struct VocabularyDomain;
impl Domain for VocabularyDomain {
    fn name(&self) -> &'static str {
        "vocabulary"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["list", "add", "remove", "import", "export", "status"]
    }
    // @spec FR-005: Vocabulary CLI operations — .specs/features/003-recognition-vocabulary/spec.md#fr-005
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        match (request.action, request.arguments) {
            ("list", []) => Ok(DomainPlan::Read(list(settings)?)),
            ("export", [target]) if target == STDOUT_TARGET => {
                Ok(DomainPlan::Read(export(settings)))
            }
            ("add", [term]) => change(settings, VocabularyEdit::Add(term.clone())),
            ("remove", [term]) => change(settings, VocabularyEdit::Remove(term.clone())),
            ("import", [mode]) => import(request, settings, mode),
            ("status", []) => Ok(DomainPlan::Live(Box::new(VocabularyStatus))),
            _ => Err(usage()),
        }
    }
}

fn usage() -> SettingsError {
    SettingsError::new(SettingsErrorCode::UsageError)
}
fn internal() -> SettingsError {
    SettingsError::new(SettingsErrorCode::InternalError)
}

/// `{terms: [{index, term, issue}], count}`: every stored occurrence, unchanged, with the reason
/// it is not sent to recognition.
fn list(settings: &AppSettings) -> Result<Value, SettingsError> {
    let terms = serde_json::to_value(inspect(&settings.custom_words)).map_err(|_| internal())?;
    Ok(json!({ "terms": terms, "count": settings.custom_words.len() }))
}

/// `{content, count}`: one stored term per line.
fn export(settings: &AppSettings) -> Value {
    let content = settings
        .custom_words
        .iter()
        .map(|term| format!("{term}\n"))
        .collect::<String>();
    json!({ "content": content, "count": settings.custom_words.len() })
}

/// Refuse early with the commit's own validator, then let the shared path commit it on the
/// locked latest settings (or preview it for a dry run).
fn change(settings: &AppSettings, edit: VocabularyEdit) -> Result<DomainPlan, SettingsError> {
    edit.apply_to(&settings.custom_words)?;
    Ok(DomainPlan::Change {
        change: edit.into_change(),
        keys: vec!["custom_words"],
        render: None,
    })
}

fn import(
    request: &DomainRequest<'_>,
    settings: &AppSettings,
    mode: &str,
) -> Result<DomainPlan, SettingsError> {
    let mode = ImportMode::parse(mode).ok_or_else(usage)?;
    let input = request.input.ok_or_else(usage)?.to_owned();
    change(settings, VocabularyEdit::Import { input, mode })
}

/// `vocabulary status`: the live status of the selected model in the running instance.
struct VocabularyStatus;
impl LiveOperation for VocabularyStatus {
    // @spec FR-005: Live vocabulary status — .specs/features/003-recognition-vocabulary/spec.md#fr-005
    fn run(
        &self,
        context: &dyn LiveContext,
        _call: &LiveCall,
    ) -> Result<LiveAnswer, SettingsError> {
        // `_call` is unused on purpose: a status read has no dry run and writes no revision.
        let read = context.read()?;
        let manager = managed_state::<Arc<TranscriptionManager>>(context)?;
        let report = manager.vocabulary_status(&read.snapshot.settings)?;
        let data = serde_json::to_value(report).map_err(|_| internal())?;
        Ok(LiveAnswer::Read(data))
    }
}

#[cfg(test)]
#[path = "vocabulary_tests.rs"]
mod tests;
