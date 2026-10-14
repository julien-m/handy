//! Settings UI commands of the prompt library (005): read the library with its issues, create,
//! edit and delete prompts, choose the default and the provider override. Each change is one
//! typed edit of the settings service, decided by the same `prompts::library` rules as
//! `handy settings prompts`, so both surfaces refuse with the same code and reason.
use crate::prompts::library;
use crate::prompts::native::NativeProviders;
use crate::prompts::processor::{ProviderPort, PROCESSING_BUDGET};
use crate::prompts::provider::NativeAvailability;
use crate::prompts::test_run::{run_test, PromptTestRequest, PromptTestResult, TestGate};
use crate::prompts::types::PromptError;
use crate::prompts::view::{library_view, PromptLibraryView};
use crate::settings::{AppSettings, LLMPrompt, PromptProviderOverride};
use crate::settings_service::{
    command_error::SettingsCommandError, errors::SettingsError,
    platform_capability::PlatformCapabilities, tauri_adapter as live_settings,
};
use crate::shortcut::prompt_targets;
use tauri::AppHandle;

type CommandResult<T> = Result<T, SettingsCommandError>;

/// Commit one library edit in the write order; a refusal is typed and nothing is written.
fn commit(
    app: &AppHandle,
    edit: &dyn Fn(&mut AppSettings) -> Result<(), PromptError>,
) -> CommandResult<()> {
    live_settings::edit(app, &|settings| edit(settings).map_err(SettingsError::from)).map(|_| ())
}

// @spec FR-001: Prompt library read for the settings — .specs/features/005-custom-ai-prompts/spec.md#fr-001
#[tauri::command]
#[specta::specta]
/// The library as published: each prompt with its issues, shortcuts and provider readiness, and
/// the default prompt. Runs on the blocking pool (the Apple Intelligence check is synchronous).
///
/// # Errors
/// `internal_error` if the worker could not run.
pub async fn get_prompt_library(app: AppHandle) -> CommandResult<PromptLibraryView> {
    live_settings::blocking(move || {
        Ok(library_view(
            &live_settings::current(&app),
            &NativeAvailability,
        ))
    })
    .await
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

// @spec AC-006: Deleting a prompt removes its shortcuts — .specs/features/005-custom-ai-prompts/spec.md#ac-006
#[tauri::command]
#[specta::specta]
/// Delete a prompt and its shortcuts (the UI confirms first, naming them); a deleted default
/// becomes None. Its shortcuts are released before the commit and registered again if the
/// commit is refused. Returns the removed shortcuts.
///
/// # Errors
/// `not_found`; `unavailable` (`shortcut_session_busy`) during a dictation when the prompt has
/// shortcuts; or a commit refusal. Nothing is removed then.
pub async fn delete_post_process_prompt(app: AppHandle, id: String) -> CommandResult<Vec<String>> {
    live_settings::blocking(move || {
        let removed = prompt_targets::shortcuts_of(&live_settings::current(&app), &id);
        let native = crate::shortcut::live_change::bindings(app.clone(), None);
        let apply = |settings: &mut AppSettings| {
            library::remove(settings, &id).map_err(SettingsError::from)
        };
        live_settings::edit_prepared(&app, "bindings", &apply, native)?;
        Ok(removed)
    })
    .await
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

/// The settings window's single prompt test slot.
static TEST_GATE: TestGate = TestGate::new();

// @spec FR-005: Prompt test without insertion or history — .specs/features/005-custom-ai-prompts/spec.md#fr-005
#[tauri::command]
#[specta::specta]
/// Run the editor's draft prompt on the sample within the 30-second budget and return the
/// result, provider, model and duration, or the typed failure. Never inserts text or writes
/// history; a second test while one runs answers `busy`.
///
/// # Errors
/// `validation_failed` for a draft the library would refuse (empty text, incomplete or unknown
/// override, `unsupported_platform`).
pub async fn test_post_process_prompt(
    app: AppHandle,
    request: PromptTestRequest,
) -> CommandResult<PromptTestResult> {
    let settings = live_settings::current(&app);
    let ports = (
        &NativeProviders as &dyn ProviderPort,
        PROCESSING_BUDGET,
        PlatformCapabilities::current(),
    );
    run_test(&settings, &request, ports, &TEST_GATE)
        .await
        .map_err(|error| SettingsError::from(error).into())
}

#[tauri::command]
#[specta::specta]
/// Cancel the running prompt test (closing the editor); its late result reads `cancelled`.
pub fn cancel_post_process_prompt_test() {
    TEST_GATE.cancel();
}
