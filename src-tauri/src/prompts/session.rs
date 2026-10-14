//! Prompt of the dictation in progress (005 FR-002, AC-003, AC-006): the trigger decides the
//! starting prompt (plain dictation → None, post-processing shortcut → the default prompt, a
//! prompt shortcut → that prompt), the indicator may change it while recording, and stopping
//! freezes it for the AI step. The prompt and its provider are captured from one settings view
//! when chosen, so a later edit or deletion never changes a captured prompt. Choice and stop are
//! serialized by one mutex: a choice that arrives after the stop is refused, never applied to
//! another session. The default prompt is never changed here.
use super::library;
use super::processor::ResolvedPrompt;
use super::provider::{resolve, ReadinessIssue};
use crate::settings::AppSettings;
use crate::shortcut::prompt_targets::{prompt_of, POST_PROCESS_ACTION};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// What started a dictation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    /// The plain dictation shortcut (or its external trigger).
    Plain,
    /// The post-processing shortcut: the default prompt.
    PostProcess,
    /// A prompt shortcut.
    Prompt(String),
}
impl Trigger {
    /// Trigger of the action `binding_id` that started the recording.
    pub fn of_binding(binding_id: &str) -> Self {
        match prompt_of(binding_id) {
            Some(prompt_id) => Self::Prompt(prompt_id.to_owned()),
            None if binding_id == POST_PROCESS_ACTION => Self::PostProcess,
            None => Self::Plain,
        }
    }
}

/// The indicator's view of the dictation prompt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Type)]
pub struct DictationPromptView {
    /// Session the choice must name (stale UI events of another session are refused).
    pub session_id: u64,
    /// Prompt of this dictation; `None` means no AI processing.
    pub prompt_id: Option<String>,
    pub prompt_name: Option<String>,
    /// False when AI post-processing is off: the menu is hidden and the prompt stays None.
    pub selectable: bool,
}

/// Why a choice on the indicator was refused; nothing changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PromptChoiceRefusal {
    /// No recording of that session any more (stopped, cancelled or another session).
    SessionEnded,
    /// AI post-processing is off for this dictation.
    ProcessingOff,
    /// The prompt is not in the library.
    PromptMissing,
    /// The prompt has issues and cannot be used until edited.
    PromptNotUsable,
}

struct Session {
    id: u64,
    selectable: bool,
    choice: Option<ResolvedPrompt>,
    frozen: bool,
}
impl Session {
    fn view(&self) -> DictationPromptView {
        DictationPromptView {
            session_id: self.id,
            prompt_id: self.choice.as_ref().map(|prompt| prompt.id.clone()),
            prompt_name: self.choice.as_ref().map(|prompt| prompt.name.clone()),
            selectable: self.selectable && !self.frozen,
        }
    }
}

/// The dictation sessions of the process (at most one at a time).
#[derive(Default)]
pub struct PromptSessions {
    state: Mutex<(u64, Option<Session>)>,
}

/// Prompt `id` captured from `settings` (text and provider from this one view); a missing id is
/// kept as a failure, so the dictation records `prompt_missing` instead of silently skipping.
fn capture(settings: &AppSettings, id: &str) -> ResolvedPrompt {
    match library::find(settings, id) {
        Some(prompt) => ResolvedPrompt {
            id: prompt.id.clone(),
            name: prompt.name.clone(),
            text: prompt.prompt.clone(),
            choice: resolve(settings, prompt),
        },
        None => ResolvedPrompt {
            id: id.to_owned(),
            name: String::new(),
            text: String::new(),
            choice: Err(ReadinessIssue::PromptMissing),
        },
    }
}

// @spec FR-002: Prompt of a dictation from its trigger — .specs/features/005-custom-ai-prompts/spec.md#fr-002
/// Starting prompt of a dictation: none when AI post-processing is off (it wins over every
/// trigger), otherwise none, the default prompt or the shortcut's prompt.
fn starting_choice(settings: &AppSettings, trigger: &Trigger) -> Option<ResolvedPrompt> {
    if !settings.post_process_enabled {
        return None;
    }
    match trigger {
        Trigger::Plain => None,
        Trigger::PostProcess => default_prompt(settings),
        Trigger::Prompt(id) => Some(capture(settings, id)),
    }
}

/// The default prompt captured from `settings`, or `None` when the default is None.
pub fn default_prompt(settings: &AppSettings) -> Option<ResolvedPrompt> {
    let id = settings.post_process_selected_prompt_id.as_deref()?;
    Some(capture(settings, id))
}

impl PromptSessions {
    fn lock(&self) -> MutexGuard<'_, (u64, Option<Session>)> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Start the session of a recording that began; it replaces any previous one.
    pub fn start(&self, settings: &AppSettings, trigger: &Trigger) -> DictationPromptView {
        let mut state = self.lock();
        state.0 += 1;
        let session = Session {
            id: state.0,
            selectable: settings.post_process_enabled,
            choice: starting_choice(settings, trigger),
            frozen: false,
        };
        let view = session.view();
        state.1 = Some(session);
        view
    }

    // @spec FR-002: Per-dictation choice without changing the default — .specs/features/005-custom-ai-prompts/spec.md#fr-002
    /// Choose the prompt of recording `session_id` (`None`: no processing), captured from the
    /// latest `settings`. The default prompt is not changed.
    ///
    /// # Errors
    /// `SessionEnded` after the stop (or for another session), `ProcessingOff`, `PromptMissing`
    /// or `PromptNotUsable`; nothing changes.
    pub fn select(
        &self,
        session_id: u64,
        prompt_id: Option<&str>,
        settings: &AppSettings,
    ) -> Result<DictationPromptView, PromptChoiceRefusal> {
        let mut state = self.lock();
        let session = state.1.as_mut().filter(|s| s.id == session_id && !s.frozen);
        let session = session.ok_or(PromptChoiceRefusal::SessionEnded)?;
        if !session.selectable {
            return Err(PromptChoiceRefusal::ProcessingOff);
        }
        session.choice = match prompt_id {
            None => None,
            Some(id) => Some(usable_capture(settings, id)?),
        };
        Ok(session.view())
    }

    /// Freeze the prompt of session `session_id` at the stop: later choices are refused. Returns
    /// the frozen prompt (`None`: no processing), or `None` when the session is not current.
    pub fn freeze(&self, session_id: u64) -> Option<Option<ResolvedPrompt>> {
        let mut state = self.lock();
        let session = state.1.as_mut().filter(|s| s.id == session_id)?;
        session.frozen = true;
        Some(session.choice.clone())
    }

    /// Freeze the current session at the stop; its id and frozen prompt.
    pub fn freeze_current(&self) -> Option<(u64, Option<ResolvedPrompt>)> {
        let mut state = self.lock();
        let session = state.1.as_mut()?;
        session.frozen = true;
        Some((session.id, session.choice.clone()))
    }

    /// End session `session_id` (processing finished or cancelled); a newer one is kept.
    pub fn end(&self, session_id: u64) {
        let mut state = self.lock();
        if state.1.as_ref().is_some_and(|s| s.id == session_id) {
            state.1 = None;
        }
    }

    /// End whatever session is current (cancel).
    pub fn cancel(&self) {
        self.lock().1 = None;
    }

    /// The current session's view, if a dictation is in progress.
    pub fn current(&self) -> Option<DictationPromptView> {
        self.lock().1.as_ref().map(Session::view)
    }
}

/// Prompt `id` captured from `settings` if it exists and has no issue.
fn usable_capture(settings: &AppSettings, id: &str) -> Result<ResolvedPrompt, PromptChoiceRefusal> {
    let prompt = library::find(settings, id).ok_or(PromptChoiceRefusal::PromptMissing)?;
    if !library::is_usable(settings, prompt) {
        return Err(PromptChoiceRefusal::PromptNotUsable);
    }
    Ok(capture(settings, id))
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
