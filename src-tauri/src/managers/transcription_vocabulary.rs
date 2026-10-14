//! Recognition vocabulary in the transcription manager (feature 003): the context of the next
//! batch request, fitted by the engine worker for the loaded model, and the status of the
//! selected model computed from the same preparation. Logs carry counts and reasons only, never
//! term text.
use super::TranscriptionManager;
use crate::managers::model::EngineType;
use crate::recognition_vocabulary::{
    context::ContextPreparation,
    occurrences::eligible_terms,
    status::{
        status_report, ContextBudgetUnverified, LoadedContext, StatusInput, VocabularyStatusReport,
    },
};
use crate::settings::AppSettings;
use log::{debug, warn};
use std::time::Duration;

/// Longest wait for the engine's answer to a status query: a settings read never waits behind a
/// dictation, and the CLI answers well inside its 5 s deadline.
const STATUS_QUERY_WAIT: Duration = Duration::from_secs(2);

impl TranscriptionManager {
    /// Initial prompt of the next batch request on the loaded transcribe-cpp model: the eligible
    /// terms of `settings` (the snapshot this request keeps, whatever is committed meanwhile),
    /// fitted whole and in list order by the worker. `None` sends no context, and approximate
    /// correction then applies to the result.
    // @spec FR-003: Batch request carries the fitted terms — .specs/features/003-recognition-vocabulary/spec.md#fr-003
    pub(super) fn batch_context_prompt(&self, settings: &AppSettings) -> Option<String> {
        let terms = eligible_texts(&settings.custom_words);
        if terms.is_empty() {
            return None;
        }
        let preparation = match self.engine.prepare_context(terms.clone(), None) {
            Ok(preparation) => preparation,
            Err(error) => {
                warn!("Recognition context not prepared ({error}); recognizing without it");
                return None;
            }
        };
        if let ContextPreparation::Unverified(mechanism) = &preparation {
            warn!("Recognition context not sent: context_budget_unverified for {mechanism:?}");
        }
        let prompt = preparation.whisper_prompt(&terms);
        debug!(
            "Recognition context: {} of {} eligible terms sent",
            prompt.as_ref().map_or(0, |_| sent_count(&preparation)),
            terms.len()
        );
        prompt
    }

    /// Status of the vocabulary of `settings` for the selected model.
    ///
    /// # Errors
    /// `ContextBudgetUnverified` when the selected batch model declares a mechanism without a
    /// documented budget.
    // @spec FR-002: Status from the loaded model and mode — .specs/features/003-recognition-vocabulary/spec.md#fr-002
    pub fn vocabulary_status(
        &self,
        settings: &AppSettings,
    ) -> Result<VocabularyStatusReport, ContextBudgetUnverified> {
        let selected = settings.selected_model.as_str();
        let info = self.model_manager.get_model_info(selected);
        status_report(StatusInput {
            model_id: selected,
            streaming: info.as_ref().is_some_and(|info| info.supports_streaming),
            stored: &settings.custom_words,
            loaded: self.loaded_context(selected, info.map(|info| info.engine_type), settings),
        })
    }

    /// What the loaded engine says about `selected`, without loading it or starting a worker.
    fn loaded_context(
        &self,
        selected: &str,
        engine_type: Option<EngineType>,
        settings: &AppSettings,
    ) -> LoadedContext {
        if self.get_current_model().as_deref() != Some(selected) {
            return LoadedContext::NotLoaded;
        }
        if !matches!(engine_type, Some(EngineType::TranscribeCpp)) {
            return LoadedContext::NoContextApi;
        }
        if self.engine.loaded().is_none() {
            return LoadedContext::NotLoaded;
        }
        let terms = eligible_texts(&settings.custom_words);
        match self.engine.prepare_context(terms, Some(STATUS_QUERY_WAIT)) {
            Ok(preparation) => LoadedContext::Prepared(preparation),
            Err(error) => {
                debug!("Vocabulary status query unanswered: {error}");
                LoadedContext::Unanswered
            }
        }
    }
}

fn eligible_texts(stored: &[String]) -> Vec<String> {
    eligible_terms(stored)
        .into_iter()
        .map(|term| term.text)
        .collect()
}

fn sent_count(preparation: &ContextPreparation) -> usize {
    match preparation {
        ContextPreparation::Fitted(fitted) => fitted.sent,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::super::post_process_transcription_text;
    use crate::audio_toolkit::OutputLanguageEvidence;
    use crate::settings::AppSettings;
    use crate::word_replacements::matcher::CompiledReplacements;

    fn corrected(words: &[&str], context_applied: bool) -> String {
        let settings = AppSettings {
            custom_words: words.iter().map(|word| (*word).to_owned()).collect(),
            word_correction_threshold: 0.5,
            filler_word_removal_enabled: false,
            ..Default::default()
        };
        let evidence = OutputLanguageEvidence::UserSelected("en".to_owned());
        post_process_transcription_text(
            "we use charge bee".to_owned(),
            &settings,
            &CompiledReplacements::empty(),
            context_applied,
            &evidence,
            &[],
        )
    }

    #[test]
    fn approximate_correction_runs_once_only_when_no_context_reached_the_engine() {
        assert!(corrected(&["ChargeBee"], false).contains("ChargeBee"));
        // At least one term was sent: the whole result skips correction (AC-005).
        assert!(!corrected(&["ChargeBee"], true).contains("ChargeBee"));
        // An empty vocabulary disables correction (AC-006).
        assert!(!corrected(&[], false).contains("ChargeBee"));
    }
}
