//! Read projection of the prompt library for the settings UI and `handy settings prompts list`:
//! the stored fields of each prompt with its validation issues, shortcuts and provider
//! readiness, never a provider credential.
use super::library;
use super::provider::{readiness, AvailabilityProbe, Readiness};
use super::types::PromptIssue;
use crate::settings::{AppSettings, LLMPrompt, PromptProviderOverride};
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use crate::shortcut::prompt_targets::shortcuts_of;
use serde::Serialize;
use specta::Type;

/// One prompt as stored, with its issues.
#[derive(Clone, Debug, Serialize, Type)]
pub struct PromptView {
    pub id: String,
    pub name: String,
    pub prompt: String,
    /// Provider and model override; `None` uses the default provider.
    pub provider_override: Option<PromptProviderOverride>,
    /// Added by the starter migration.
    pub starter: bool,
    /// Why the prompt cannot be used until edited; empty for a valid prompt.
    pub issues: Vec<PromptIssue>,
}

/// One prompt of the library listing: its view, its 0–2 shortcuts and its readiness.
#[derive(Clone, Debug, Serialize, Type)]
pub struct ListedPrompt {
    #[serde(flatten)]
    pub view: PromptView,
    pub shortcuts: Vec<String>,
    pub readiness: Readiness,
}

/// The library with its default prompt (None when absent or no longer in the library).
#[derive(Clone, Debug, Serialize, Type)]
pub struct PromptLibraryView {
    /// Default prompt of the post-processing shortcut, or `None`.
    pub default_prompt_id: Option<String>,
    /// The AI post-processing switch: off, no dictation is processed.
    pub post_process_enabled: bool,
    pub prompts: Vec<ListedPrompt>,
}

/// View of one stored prompt in `settings`.
pub fn prompt_view(settings: &AppSettings, prompt: &LLMPrompt) -> PromptView {
    PromptView {
        id: prompt.id.clone(),
        name: prompt.name.clone(),
        prompt: prompt.prompt.clone(),
        provider_override: prompt.provider_override.clone(),
        starter: prompt.starter,
        issues: library::issues_of(settings, prompt),
    }
}

/// The whole library with shortcuts and readiness (Apple availability through `probe`).
pub fn library_view(settings: &AppSettings, probe: &dyn AvailabilityProbe) -> PromptLibraryView {
    let default_prompt_id = settings
        .post_process_selected_prompt_id
        .clone()
        .filter(|id| library::find(settings, id).is_some());
    let prompts = settings
        .post_process_prompts
        .iter()
        .map(|prompt| ListedPrompt {
            view: prompt_view(settings, prompt),
            shortcuts: shortcuts_of(settings, &prompt.id),
            readiness: readiness(settings, prompt, probe),
        })
        .collect();
    PromptLibraryView {
        default_prompt_id,
        post_process_enabled: settings.post_process_enabled,
        prompts,
    }
}

/// `library_view` as the CLI envelope data.
///
/// # Errors
/// `internal_error` if the view cannot be serialized.
pub fn listed(
    settings: &AppSettings,
    probe: &dyn AvailabilityProbe,
) -> Result<serde_json::Value, SettingsError> {
    serde_json::to_value(library_view(settings, probe))
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))
}
