//! Glue between the dictation pipeline and the prompt session (005 FR-002, FR-004): the session
//! starts when a recording begins, is frozen at the stop, runs the AI step with the frozen
//! prompt, reports a failure for the indicator notice, and ends with the processing (or at a
//! cancel). Insertion and history stay with the pipeline; this module only decides the text.
use super::output::OutputDecision;
use super::processor::{process, FailureReason, ResolvedPrompt, PROCESSING_BUDGET};
use super::session::{default_prompt, DictationPromptView, PromptSessions, Trigger};
use crate::managers::history_store::RetriedText;
use crate::settings::AppSettings;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

/// How long the indicator shows the failure notice at least (AC-005).
pub const FAILURE_NOTICE_MS: u64 = 4000;

/// The indicator's prompt changed (`None`: no dictation in progress).
#[derive(Clone, Debug, Serialize, Type, tauri_specta::Event)]
pub struct DictationPromptChanged {
    pub prompt: Option<DictationPromptView>,
}

// @spec AC-005: Failure notice says the original text was used — .specs/features/005-custom-ai-prompts/spec.md#ac-005
/// The AI step of a dictation failed and the original text was used for insertion. The notice
/// (translated key `overlay.fallbackNotice`, English source "AI processing failed. The original
/// text was used.") never says whether the insertion succeeded: 006, 007 and 010 report that.
#[derive(Clone, Debug, Serialize, Type, tauri_specta::Event)]
pub struct PromptProcessingFailed {
    pub session_id: u64,
    pub prompt_name: Option<String>,
    pub reason: FailureReason,
    /// Minimum display time of the notice, in milliseconds.
    pub notice_ms: u64,
}

/// The session frozen at a stop.
pub struct FrozenSession {
    pub session_id: Option<u64>,
    /// Prompt of the dictation; `None` means no AI processing.
    pub prompt: Option<ResolvedPrompt>,
}

fn emit_changed(app: &AppHandle, prompt: Option<DictationPromptView>) {
    if let Err(error) = (DictationPromptChanged { prompt }).emit(app) {
        log::warn!("Could not notify the indicator of the dictation prompt: {error}");
    }
}

/// Start the prompt session of a recording that began with `binding_id`.
pub fn start_session(app: &AppHandle, settings: &AppSettings, binding_id: &str) {
    let Some(sessions) = app.try_state::<PromptSessions>() else {
        return;
    };
    let view = sessions.start(settings, &Trigger::of_binding(binding_id));
    emit_changed(app, Some(view));
}

/// Freeze the current session at the stop (no session: no processing).
pub fn freeze_session(app: &AppHandle) -> FrozenSession {
    let frozen = app
        .try_state::<PromptSessions>()
        .and_then(|sessions| sessions.freeze_current());
    if let Some(view) = app.try_state::<PromptSessions>().and_then(|s| s.current()) {
        emit_changed(app, Some(view));
    }
    match frozen {
        Some((id, prompt)) => FrozenSession {
            session_id: Some(id),
            prompt,
        },
        None => FrozenSession {
            session_id: None,
            prompt: None,
        },
    }
}

/// End session `session_id` when its processing is over.
pub fn end_session(app: &AppHandle, session_id: Option<u64>) {
    if let (Some(sessions), Some(id)) = (app.try_state::<PromptSessions>(), session_id) {
        sessions.end(id);
        emit_changed(app, sessions.current());
    }
}

/// End the session at a cancel: later choices and late results are refused.
pub fn cancel_session(app: &AppHandle) {
    if let Some(sessions) = app.try_state::<PromptSessions>() {
        sessions.cancel();
        emit_changed(app, None);
    }
}

// @spec FR-004: Raw transcript kept when the AI step fails — .specs/features/005-custom-ai-prompts/spec.md#fr-004
/// Output of a dictation with its frozen prompt: the AI step within the 30-second budget, or
/// the transcript as is when the prompt is None.
pub async fn process_frozen(
    transcription: &str,
    prompt: Option<&ResolvedPrompt>,
) -> OutputDecision {
    let Some(prompt) = prompt else {
        return OutputDecision::raw(transcription);
    };
    let port = super::native::NativeProviders;
    let report = process(prompt, transcription, &port, PROCESSING_BUDGET, &|| false).await;
    let decision = OutputDecision::after(transcription, prompt, report);
    if let Some(reason) = decision.failure {
        log::warn!(
            "AI processing failed ({}); the original text is used",
            reason.code()
        );
    }
    decision
}

/// Tell the indicator that the AI step of `session_id` failed (shown at least 4 seconds).
pub fn notify_failure(app: &AppHandle, session_id: Option<u64>, decision: &OutputDecision) {
    let (Some(session_id), Some(reason)) = (session_id, decision.failure) else {
        return;
    };
    let event = PromptProcessingFailed {
        session_id,
        prompt_name: decision.processing.post_process_prompt_name.clone(),
        reason,
        notice_ms: FAILURE_NOTICE_MS,
    };
    if let Err(error) = event.emit(app) {
        log::warn!("Could not publish the AI failure notice: {error}");
    }
}

/// New text of a retried history row: the AI step with the default prompt when
/// `post_process` is set, as before 005 (a retry does not depend on the switch).
pub async fn retried_text(
    settings: &AppSettings,
    transcription: String,
    post_process: bool,
) -> RetriedText {
    let prompt = post_process.then(|| default_prompt(settings)).flatten();
    let decision = process_frozen(&transcription, prompt.as_ref()).await;
    RetriedText {
        transcription_text: transcription,
        post_processed_text: decision.post_processed_text,
        post_process_prompt: decision.post_process_prompt,
        processing: decision.processing,
    }
}

#[cfg(test)]
mod tests {
    // @spec AC-005: Notice wording never claims the insertion succeeded — .specs/features/005-custom-ai-prompts/spec.md#ac-005
    #[test]
    fn the_failure_notice_says_the_ai_failed_and_the_original_was_used_never_that_it_was_inserted()
    {
        let english: serde_json::Value = serde_json::from_str(include_str!(
            "../../../src/i18n/locales/en/translation.json"
        ))
        .unwrap_or_default();
        let notice = english["overlay"]["fallbackNotice"]
            .as_str()
            .unwrap_or_default();
        assert_eq!(notice, "AI processing failed. The original text was used.");
        for claim in ["insert", "paste", "success", "typed", "sent"] {
            assert!(!notice.to_lowercase().contains(claim), "{claim}");
        }
        assert_eq!(super::FAILURE_NOTICE_MS, 4000);
    }
}
