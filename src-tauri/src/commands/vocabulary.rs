//! Typed settings UI commands of the recognition vocabulary (feature 003): read the saved list
//! with its legacy flags, preview a paste, and commit additions, imports and per-occurrence
//! removals through the same validator and commit path as `handy settings vocabulary`.
use crate::managers::transcription::TranscriptionManager;
use crate::recognition_vocabulary::{
    edits::{refuse_preview, VocabularyEdit},
    occurrences::{inspect, Occurrence},
    preview::{prepare_import, ImportMode, ImportPreview},
    status::VocabularyStatusReport,
};
use crate::settings::AppSettings;
use crate::settings_service::{
    command_error::SettingsCommandError,
    errors::{SettingsError, SettingsErrorCode},
    protocol::Revision,
    tauri_adapter as live_settings,
};
use serde::Serialize;
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// The saved vocabulary at one revision: every stored occurrence, unchanged, with the reason it
/// is excluded from the recognition context.
#[derive(Clone, Debug, Serialize, Type)]
pub struct VocabularyList {
    pub revision: u64,
    pub terms: Vec<Occurrence>,
}

/// A paste preview against the saved list at `revision`; nothing is written.
#[derive(Clone, Debug, Serialize, Type)]
pub struct VocabularyImportPreview {
    pub revision: u64,
    pub preview: ImportPreview,
    /// True when Apply can commit the whole candidate.
    pub applicable: bool,
}

type VocabularyResult = Result<VocabularyList, SettingsCommandError>;

fn listed(revision: Revision, settings: &AppSettings) -> VocabularyList {
    VocabularyList {
        revision: revision.0,
        terms: inspect(&settings.custom_words),
    }
}

/// Commit `edit` with the UI's base revision on the blocking pool.
async fn commit(app: AppHandle, edit: VocabularyEdit, expected_revision: u64) -> VocabularyResult {
    live_settings::blocking(move || {
        let change = edit.into_change();
        let commit = live_settings::write_change(&app, &change, Some(Revision(expected_revision)))?;
        Ok(listed(commit.revision, &commit.after))
    })
    .await
}

/// Saved vocabulary and its revision.
#[tauri::command]
#[specta::specta]
pub fn get_vocabulary(app: AppHandle) -> VocabularyList {
    let (revision, settings) = live_settings::view(&app);
    listed(revision, &settings)
}

/// Preview a bulk input (one term per line) against the saved list; pure, nothing is written.
// @spec FR-001: Paste preview command — .specs/features/003-recognition-vocabulary/spec.md#fr-001
#[tauri::command]
#[specta::specta]
pub fn preview_vocabulary_import(
    app: AppHandle,
    input: String,
    mode: ImportMode,
) -> VocabularyImportPreview {
    let (revision, settings) = live_settings::view(&app);
    let preview = prepare_import(&settings.custom_words, &input, mode);
    VocabularyImportPreview {
        revision: revision.0,
        applicable: refuse_preview(&preview).is_ok(),
        preview,
    }
}

/// Apply a previewed bulk input as one atomic candidate at the preview's revision.
#[tauri::command]
#[specta::specta]
pub async fn apply_vocabulary_import(
    app: AppHandle,
    input: String,
    mode: ImportMode,
    expected_revision: u64,
) -> VocabularyResult {
    commit(
        app,
        VocabularyEdit::Import { input, mode },
        expected_revision,
    )
    .await
}

/// Add one term.
#[tauri::command]
#[specta::specta]
pub async fn add_vocabulary_term(
    app: AppHandle,
    term: String,
    expected_revision: u64,
) -> VocabularyResult {
    commit(app, VocabularyEdit::Add(term), expected_revision).await
}

/// Remove the occurrence at `index`, which must still hold `term` (legacy duplicates are removed
/// one row at a time).
// @spec FR-001: Per-occurrence removal command — .specs/features/003-recognition-vocabulary/spec.md#fr-001
#[tauri::command]
#[specta::specta]
pub async fn remove_vocabulary_occurrence(
    app: AppHandle,
    index: u32,
    term: String,
    expected_revision: u64,
) -> VocabularyResult {
    let edit = VocabularyEdit::RemoveOccurrence {
        index: index as usize,
        term,
    };
    commit(app, edit, expected_revision).await
}

/// Status of the vocabulary for the selected model (one of five), from the loaded model and the
/// exact preparation of the next request; never loads a model.
///
/// # Errors
/// `internal_error` with reason `context_budget_unverified` for a model whose declared mechanism
/// has no documented budget; `unavailable` before the transcription manager exists.
// @spec FR-002: Status command for the settings UI — .specs/features/003-recognition-vocabulary/spec.md#fr-002
#[tauri::command]
#[specta::specta]
pub async fn get_vocabulary_status(
    app: AppHandle,
) -> Result<VocabularyStatusReport, SettingsCommandError> {
    live_settings::blocking(move || {
        let manager = app
            .try_state::<Arc<TranscriptionManager>>()
            .ok_or_else(|| SettingsError::new(SettingsErrorCode::Unavailable))?;
        let settings = live_settings::current(&app);
        Ok(manager
            .vocabulary_status(&settings)
            .map_err(SettingsError::from)?)
    })
    .await
}
