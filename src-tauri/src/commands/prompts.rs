//! Settings UI commands of the prompt library (005): read the library with its issues, create,
//! edit and delete prompts, choose the default and the provider override. Each change is one
//! typed edit of the settings service, decided by the same `prompts::library` rules as
//! `handy settings prompts`, so both surfaces refuse with the same code and reason.
use crate::prompts::library;
use crate::prompts::types::PromptError;
use crate::prompts::view::{library_view, PromptLibraryView};
use crate::settings::{LLMPrompt, PromptProviderOverride};
use crate::settings_service::{
    command_error::SettingsCommandError, errors::SettingsError,
    platform_capability::PlatformCapabilities, tauri_adapter as live_settings,
};
use tauri::AppHandle;

type CommandResult<T> = Result<T, SettingsCommandError>;

/// Commit one library edit in the write order; a refusal is typed and nothing is written.
fn commit(
    app: &AppHandle,
    edit: &dyn Fn(&mut crate::settings::AppSettings) -> Result<(), PromptError>,
) -> CommandResult<()> {
    live_settings::edit(app, &|settings| edit(settings).map_err(SettingsError::from)).map(|_| ())
}

// @spec FR-001: Prompt library read for the settings — .specs/features/005-custom-ai-prompts/spec.md#fr-001
#[tauri::command]
#[specta::specta]
/// The library as published, with each prompt's issues and the default prompt.
pub fn get_prompt_library(app: AppHandle) -> PromptLibraryView {
    let (revision, settings) = live_settings::view(&app);
    library_view(&settings, revision.0)
}

#[tauri::command]
#[specta::specta]
/// Create a prompt; the id is generated once, outside the (re-runnable) edit.
///
/// # Errors
/// `validation_failed` with a prompt reason, `limit_exceeded` at 50 prompts, or a commit refusal.
pub async fn add_post_process_prompt(
    app: AppHandle,
    name: String,
    prompt: String,
) -> CommandResult<LLMPrompt> {
    live_settings::blocking(move || {
        let id = library::generate_id();
        commit(&app, &|settings| {
            library::create(settings, &id, &name, &prompt)
        })?;
        let settings = live_settings::current(&app);
        library::find(&settings, &id)
            .cloned()
            .ok_or_else(|| SettingsError::from(PromptError::NotFound).into())
    })
    .await
}

#[tauri::command]
#[specta::specta]
/// Rename and rewrite a prompt; the edited prompt must be valid.
///
/// # Errors
/// `not_found`, `validation_failed` with a prompt reason, or a commit refusal.
pub async fn update_post_process_prompt(
    app: AppHandle,
    id: String,
    name: String,
    prompt: String,
) -> CommandResult<()> {
    live_settings::blocking(move || {
        commit(&app, &|settings| {
            library::update(settings, &id, Some(&name), Some(&prompt))
        })
    })
    .await
}

#[tauri::command]
#[specta::specta]
/// Delete a prompt (the UI confirms first); a deleted default becomes None.
///
/// # Errors
/// `not_found`, or a commit refusal.
pub async fn delete_post_process_prompt(app: AppHandle, id: String) -> CommandResult<()> {
    live_settings::blocking(move || commit(&app, &|settings| library::remove(settings, &id))).await
}

#[tauri::command]
#[specta::specta]
/// Choose the default prompt of the post-processing shortcut, or None.
///
/// # Errors
/// `not_found`, `validation_failed` (`prompt_not_usable`) for a prompt with issues.
pub async fn set_post_process_selected_prompt(
    app: AppHandle,
    id: Option<String>,
) -> CommandResult<()> {
    live_settings::blocking(move || {
        commit(&app, &|settings| {
            library::set_default(settings, id.as_deref())
        })
    })
    .await
}

// @spec FR-006: Per-prompt provider override — .specs/features/005-custom-ai-prompts/spec.md#fr-006
#[tauri::command]
#[specta::specta]
/// Set the provider and model a prompt uses, or return it to the default provider with `None`.
///
/// # Errors
/// `not_found`; `validation_failed` with `prompt_override_incomplete`, `prompt_unknown_provider`
/// or `unsupported_platform` (Apple Intelligence off Apple silicon).
pub async fn set_post_process_prompt_provider_override(
    app: AppHandle,
    id: String,
    provider_override: Option<PromptProviderOverride>,
) -> CommandResult<()> {
    live_settings::blocking(move || {
        let platform = PlatformCapabilities::current();
        commit(&app, &|settings| {
            library::set_override(settings, &id, provider_override.clone(), platform)
        })
    })
    .await
}
