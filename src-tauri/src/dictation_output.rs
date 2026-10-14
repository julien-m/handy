//! Output stages of one finished dictation, after the native text pipeline (recognition,
//! language and script, word replacements, approximate correction, fillers, normalization): the
//! optional AI post-processing, the history entry (only when the recording's WAV was saved) and
//! the text handed to insertion. The stages run behind ports, so their order and the text each
//! one receives are testable without Tauri; the shortcut action supplies the real ports.

use std::future::Future;

/// Text after the AI post-processing stage.
pub(crate) struct ProcessedTranscription {
    /// Text handed to insertion: the AI result, or the native text without AI.
    pub final_text: String,
    /// AI result recorded in history, if AI post-processing ran and answered.
    pub post_processed_text: Option<String>,
    /// Prompt of that AI result, recorded in history with it.
    pub post_process_prompt: Option<String>,
}

/// Native side of the output stages.
pub(crate) trait OutputPorts {
    /// AI post-processing of `native` (or `native` unchanged without AI); `None` when the
    /// dictation was cancelled while it ran.
    fn post_process(&self, native: &str) -> impl Future<Output = Option<ProcessedTranscription>>;
    /// True once the dictation was cancelled.
    fn is_cancelled(&self) -> bool;
    /// Record the history entry: the native text and the optional AI result.
    fn save_history(&self, native: String, processed: &ProcessedTranscription);
    /// Hand `text` to insertion (an attempted paste).
    fn insert(&self, text: String);
}

/// How the output stages of a dictation ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Delivery {
    /// Cancelled during or right after AI post-processing: no history entry, no insertion.
    Cancelled,
    /// The final text is empty: history only (when the WAV was saved).
    NothingToInsert,
    /// The final text was handed to insertion.
    Inserted,
}

// @spec AC-007: Replacement pipeline position — .specs/features/002-word-replacements/spec.md#ac-007
/// Run the output stages of `native`, the text of the native pipeline (replacements included).
///
/// Order matters: AI post-processing first (it may be cancelled), then the history entry, so a
/// saved recording keeps its entry even when insertion fails, then insertion.
pub(crate) async fn deliver(ports: &impl OutputPorts, native: String, wav_saved: bool) -> Delivery {
    let Some(processed) = ports.post_process(&native).await else {
        return Delivery::Cancelled;
    };
    if ports.is_cancelled() {
        return Delivery::Cancelled;
    }
    // Business rule: history keeps only recordings whose WAV was saved (retry needs the audio).
    if wav_saved {
        ports.save_history(native, &processed);
    }
    if processed.final_text.is_empty() {
        return Delivery::NothingToInsert;
    }
    ports.insert(processed.final_text);
    Delivery::Inserted
}

#[cfg(test)]
#[path = "dictation_output_tests.rs"]
mod tests;
