//! Output decision of a dictation after its AI step (005 FR-004, AC-005): the text to insert
//! (the processed text, or the original transcript unchanged on any failure) and what history
//! records (prompt, provider, model, failure code). Pure: insertion and history writes belong to
//! the caller.
use super::processor::{FailureReason, ProcessingOutcome, ProcessingReport, ResolvedPrompt};
use crate::managers::history_store::ProcessingRecord;

/// What a dictation inserts and records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputDecision {
    /// Text handed to insertion: processed text, or the original transcript.
    pub final_text: String,
    pub post_processed_text: Option<String>,
    /// Prompt text that produced `post_processed_text` (historical history column).
    pub post_process_prompt: Option<String>,
    pub processing: ProcessingRecord,
    /// Why the AI step failed, when the original transcript is used because of a failure.
    pub failure: Option<FailureReason>,
    /// The user cancelled during the AI step: nothing is inserted.
    pub cancelled: bool,
}

impl OutputDecision {
    /// The transcript as is: no AI step (no prompt, or post-processing off).
    pub fn raw(transcription: &str) -> Self {
        Self {
            final_text: transcription.to_owned(),
            post_processed_text: None,
            post_process_prompt: None,
            processing: ProcessingRecord::default(),
            failure: None,
            cancelled: false,
        }
    }

    // @spec FR-004: Original transcript on any AI failure — .specs/features/005-custom-ai-prompts/spec.md#fr-004
    /// Decision for `transcription` after `prompt` ran with `report`. A failure keeps the
    /// original transcript unchanged and records the prompt and the failure code.
    pub fn after(transcription: &str, prompt: &ResolvedPrompt, report: ProcessingReport) -> Self {
        let mut decision = Self::raw(transcription);
        decision.processing = ProcessingRecord {
            post_process_prompt_id: Some(prompt.id.clone()).filter(|id| !id.is_empty()),
            post_process_prompt_name: Some(prompt.name.clone()).filter(|n| !n.is_empty()),
            post_process_provider_id: report.provider_id,
            post_process_model: report.model,
            post_process_failure: None,
        };
        match report.outcome {
            ProcessingOutcome::Processed(text) => {
                decision.final_text = text.clone();
                decision.post_processed_text = Some(text);
                decision.post_process_prompt = Some(prompt.text.clone());
            }
            ProcessingOutcome::Failed(reason) => {
                decision.processing.post_process_failure = Some(reason.code());
                decision.failure = Some(reason);
            }
            ProcessingOutcome::Cancelled => decision.cancelled = true,
            ProcessingOutcome::Skipped => {}
        }
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn prompt() -> ResolvedPrompt {
        ResolvedPrompt {
            id: "starter_notes".into(),
            name: "Notes".into(),
            text: "Notes: ${output}".into(),
            choice: Err(crate::prompts::provider::ReadinessIssue::MissingApiKey),
        }
    }
    fn report(outcome: ProcessingOutcome) -> ProcessingReport {
        ProcessingReport {
            outcome,
            provider_id: Some("openai".into()),
            model: Some("gpt-4o-mini".into()),
            duration: Duration::from_millis(5),
        }
    }

    #[test]
    fn a_failure_keeps_the_exact_transcript_and_records_prompt_and_reason() {
        let failed = report(ProcessingOutcome::Failed(FailureReason::Timeout));
        let decision = OutputDecision::after("  raw words, exactly ", &prompt(), failed);
        assert_eq!(decision.final_text, "  raw words, exactly ");
        assert_eq!(decision.post_processed_text, None);
        assert_eq!(decision.failure, Some(FailureReason::Timeout));
        let record = &decision.processing;
        assert_eq!(record.post_process_prompt_name.as_deref(), Some("Notes"));
        assert_eq!(record.post_process_failure.as_deref(), Some("timeout"));
        assert!(!decision.cancelled);
    }
    #[test]
    fn processed_text_replaces_the_transcript() {
        let done = report(ProcessingOutcome::Processed("- notes".into()));
        let decision = OutputDecision::after("raw", &prompt(), done);
        assert_eq!(decision.final_text, "- notes");
        assert_eq!(
            decision.post_process_prompt.as_deref(),
            Some("Notes: ${output}")
        );
        assert_eq!(decision.processing.post_process_failure, None);
    }
    #[test]
    fn cancellation_is_carried_for_the_caller_to_insert_nothing() {
        let cancelled =
            OutputDecision::after("raw", &prompt(), report(ProcessingOutcome::Cancelled));
        assert!(cancelled.cancelled);
        assert_eq!(
            OutputDecision::raw("raw").processing,
            ProcessingRecord::default()
        );
    }
}
