//! Rule files of the settings window (feature 002): the native import and export dialogs of the
//! main window, a bounded UTF-8 read of the chosen import file and an atomic export of the saved
//! rules. The webview never receives a path or a file-system permission: it gets the chosen
//! file's name and text (import) or the export outcome, and previews or applies the text through
//! `preview_word_replacements` and `apply_word_replacements` like the bulk editor.

use crate::settings_service::{
    cli::export_failure,
    command_error::SettingsCommandError,
    errors::{SettingsError, SettingsErrorCode, ValidationReason},
    tauri_adapter,
};
use crate::word_replacements::{
    exchange::{read_exchange_input, serialize_exchange, ExchangeInputError},
    exchange_file::{write_exchange_file, Written},
};
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

/// File name proposed by the export dialog; the user can change it.
const DEFAULT_EXPORT_NAME: &str = "replacement-rules.txt";

/// A rule file chosen in the import dialog, read as bounded UTF-8 text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ReplacementImportFile {
    /// Name of the chosen file, without its directory.
    pub file_name: String,
    /// Exact file text (at most 1,000,000 bytes), previewed and applied by the editor.
    pub text: String,
}

/// A completed export of the saved rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ReplacementExportFile {
    /// Name of the written file, without its directory.
    pub file_name: String,
    /// Number of exported rules.
    pub count: u32,
    /// False when the file was replaced but the disk did not confirm the rename yet.
    pub durable: bool,
}

/// Refusal of a rule file: the typed settings error and the chosen file name (never its path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ReplacementFileError {
    pub file_name: String,
    pub error: SettingsCommandError,
}

// @spec AC-005: Bounded UTF-8 import — .specs/features/002-word-replacements/spec.md#ac-005
#[tauri::command]
#[specta::specta]
/// Open the import dialog of the main window and read the chosen file; `None` when cancelled.
/// Nothing is saved: the editor previews the text, then applies it at the preview's revision.
///
/// # Errors
/// `ReplacementFileError` with `limit_exceeded` above 1,000,000 bytes, `validation_failed`
/// (reason `invalid_encoding`) for a file that is not UTF-8, `usage_error` for an unreadable file.
pub async fn import_word_replacements_file(
    app: AppHandle,
) -> Result<Option<ReplacementImportFile>, ReplacementFileError> {
    on_worker(move || {
        let Some(path) = pick_import_path(&app) else {
            return Ok(None);
        };
        read_import_file(&path).map(Some)
    })
    .await
}

// @spec AC-004: Canonical lossless export — .specs/features/002-word-replacements/spec.md#ac-004
#[tauri::command]
#[specta::specta]
/// Open the export dialog of the main window and write the saved rules (canonical exchange
/// format) atomically to the chosen file; `None` when cancelled. Unsaved draft edits are not
/// exported.
///
/// # Errors
/// `ReplacementFileError` with `persistence_failed` (reason `export_symbolic_link`,
/// `export_read_only` or `export_not_written`); the destination is unchanged.
pub async fn export_word_replacements_file(
    app: AppHandle,
) -> Result<Option<ReplacementExportFile>, ReplacementFileError> {
    on_worker(move || {
        let Some(path) = pick_export_path(&app) else {
            return Ok(None);
        };
        let (_, settings) = tauri_adapter::view(&app);
        write_export_file(&path, &settings.word_replacements).map(Some)
    })
    .await
}

/// Run a dialog and its file I/O off the async runtime: the dialogs block until the user answers.
async fn on_worker<T: Send + 'static>(
    body: impl FnOnce() -> Result<T, ReplacementFileError> + Send + 'static,
) -> Result<T, ReplacementFileError> {
    tauri::async_runtime::spawn_blocking(body)
        .await
        .map_err(|error| {
            log::error!("Replacement file worker failed: {error}");
            file_error(
                Path::new(""),
                SettingsError::new(SettingsErrorCode::InternalError),
            )
        })?
}

/// Path chosen in the main window's open dialog, if any.
fn pick_import_path(app: &AppHandle) -> Option<PathBuf> {
    let mut dialog = app.dialog().file();
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    dialog.blocking_pick_file()?.into_path().ok()
}

/// Path chosen in the main window's save dialog, if any.
fn pick_export_path(app: &AppHandle) -> Option<PathBuf> {
    let mut dialog = app.dialog().file().set_file_name(DEFAULT_EXPORT_NAME);
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    dialog.blocking_save_file()?.into_path().ok()
}

/// Read `path` as bounded UTF-8 exchange text.
///
/// # Errors
/// See `import_word_replacements_file`.
pub fn read_import_file(path: &Path) -> Result<ReplacementImportFile, ReplacementFileError> {
    let text = std::fs::File::open(path)
        .map_err(|error| ExchangeInputError::Unreadable {
            io_kind: format!("{:?}", error.kind()),
        })
        .and_then(read_exchange_input)
        .map_err(|error| file_error(path, input_refusal(&error)))?;
    Ok(ReplacementImportFile {
        file_name: file_name(path),
        text,
    })
}

/// Write `rules` in the canonical exchange format to `path`, atomically.
///
/// # Errors
/// See `export_word_replacements_file`.
pub fn write_export_file(
    path: &Path,
    rules: &[crate::word_replacements::model::ReplacementRule],
) -> Result<ReplacementExportFile, ReplacementFileError> {
    let written = write_exchange_file(path, &serialize_exchange(rules))
        .map_err(|error| file_error(path, export_failure(&error)))?;
    Ok(ReplacementExportFile {
        file_name: file_name(path),
        // At most 1000 rules are ever saved (AC-002), so the count always fits.
        count: u32::try_from(rules.len()).unwrap_or(u32::MAX),
        durable: written == Written::Durable,
    })
}

/// Typed refusal of an import source, the same categories as the CLI's file input.
fn input_refusal(error: &ExchangeInputError) -> SettingsError {
    match error {
        ExchangeInputError::TooLarge { .. } => SettingsError::new(SettingsErrorCode::LimitExceeded),
        ExchangeInputError::InvalidUtf8 { byte_offset } => SettingsError::invalid_input(
            ValidationReason::InvalidEncoding,
            Some(*byte_offset as u64),
        ),
        ExchangeInputError::Unreadable { .. } => SettingsError::new(SettingsErrorCode::UsageError),
    }
}

fn file_error(path: &Path, error: SettingsError) -> ReplacementFileError {
    ReplacementFileError {
        file_name: file_name(path),
        error: SettingsCommandError::from(error),
    }
}

/// Last component of `path`; lossy so a non-UTF-8 name still names the file in the editor.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "replacements_files_tests.rs"]
mod tests;
