//! Read projection of the prompt library for the settings UI and the CLI list: the stored fields
//! of each prompt with its validation issues, never a provider credential.
use super::library;
use super::types::PromptIssue;
use crate::settings::{AppSettings, LLMPrompt, PromptProviderOverride};
use serde::Serialize;
use specta::Type;

/// One prompt as the settings show it.
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

/// The library with its default prompt (None when absent or no longer in the library).
#[derive(Clone, Debug, Serialize, Type)]
pub struct PromptLibraryView {
    /// Durable settings revision the view was read at.
    pub revision: u64,
    /// Default prompt of the post-processing shortcut, or `None`.
    pub default_prompt_id: Option<String>,
    pub prompts: Vec<PromptView>,
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

/// View of the whole library at `revision`.
pub fn library_view(settings: &AppSettings, revision: u64) -> PromptLibraryView {
    let default_prompt_id = settings
        .post_process_selected_prompt_id
        .clone()
        .filter(|id| library::find(settings, id).is_some());
    PromptLibraryView {
        revision,
        default_prompt_id,
        prompts: settings
            .post_process_prompts
            .iter()
            .map(|prompt| prompt_view(settings, prompt))
            .collect(),
    }
}
