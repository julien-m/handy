//! Output stages of one finished dictation, after the native text pipeline (recognition,
//! language and script, word replacements, approximate correction, fillers, normalization): the
//! optional AI post-processing, the history entry (text-only when the recording's WAV was not
//! saved, 006) and the text handed to insertion. The stages run behind ports, so their order and
//! the text each one receives are testable without Tauri; the shortcut action supplies the real
//! ports.

use std::future::Future;

/// Text after the AI post-processing stage (005): the text handed to insertion (the AI result,
/// or the native text unchanged without AI or on an AI failure), the AI result and prompt
/// recorded in history, the processing record and failure, and whether the AI step was
/// cancelled.
pub(crate) use crate::prompts::output::OutputDecision as ProcessedTranscription;

/// Native side of the output stages.
pub(crate) trait OutputPorts {
    /// Actual result of the history write, handed to insertion with the final text.
    type History;
    /// AI post-processing of `native` (or `native` unchanged without AI); `None` when the
    /// dictation was cancelled while it ran.
    fn post_process(&self, native: &str) -> impl Future<Output = Option<ProcessedTranscription>>;
    /// True once the dictation was cancelled.
    fn is_cancelled(&self) -> bool;
    /// Record the history entry (with its audio, or text-only when the WAV was not saved): the
    /// native text and the optional AI result; returns what actually happened.
    fn save_history(&self, native: String, processed: &ProcessedTranscription) -> Self::History;
    /// End a dictation that has nothing to insert.
    fn abandon(&self);
    /// Hand `text` to insertion with the history result of its dictation, then end the
    /// recording indicator. `case_decided` is true when `text` is the AI result (005), whose
    /// case smart capitalization (015) leaves as is.
    fn insert(
        &self,
        text: String,
        case_decided: bool,
        history: Self::History,
    ) -> impl Future<Output = ()>;
}

/// How the output stages of a dictation ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Delivery {
    /// Cancelled during or right after AI post-processing: no history entry, no insertion.
    Cancelled,
    /// The final text is empty: history only.
    NothingToInsert,
    /// The final text was handed to insertion.
    Inserted,
}

// @spec AC-007: Replacement pipeline position — .specs/features/002-word-replacements/spec.md#ac-007
/// Run the output stages of `native`, the text of the native pipeline (replacements included).
///
/// Order matters: AI post-processing first (it may be cancelled), then the history entry, so a
/// dictation keeps its entry even when insertion fails and insertion reports the actual history
/// result (006), then insertion.
pub(crate) async fn deliver<P: OutputPorts>(ports: &P, native: String) -> Delivery {
    let Some(processed) = ports.post_process(&native).await else {
        return Delivery::Cancelled;
    };
    if ports.is_cancelled() || processed.cancelled {
        return Delivery::Cancelled;
    }
    let history = ports.save_history(native, &processed);
    if processed.final_text.is_empty() {
        ports.abandon();
        return Delivery::NothingToInsert;
    }
    // @spec AC-003: AI-decided case is kept — .specs/features/015-smart-capitalization/spec.md#ac-003
    let case_decided =
        processed.post_processed_text.as_deref() == Some(processed.final_text.as_str());
    ports
        .insert(processed.final_text, case_decided, history)
        .await;
    Delivery::Inserted
}

#[cfg(test)]
#[path = "dictation_output_tests.rs"]
mod tests;
