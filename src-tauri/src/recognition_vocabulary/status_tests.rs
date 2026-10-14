//! Status matrix (SC-001): every engine family and mode of the clarification table, the five
//! statuses and the technical error, and never "used" without a term in the request.
use super::*;
use crate::recognition_vocabulary::{
    budget::{fit_whisper_prefix, TokenCounter},
    capability::ContextCapabilities,
    context::{prepare_context, FittedContext},
};

struct CharCounter;
impl TokenCounter for CharCounter {
    fn count_tokens(&self, text: &str) -> Result<usize, String> {
        Ok(text.chars().count())
    }
}
fn list(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
fn prepared(caps: ContextCapabilities, stored: &[String]) -> LoadedContext {
    let eligible = eligible_terms(stored)
        .into_iter()
        .map(|term| term.text)
        .collect::<Vec<_>>();
    match prepare_context(caps, &eligible, &CharCounter) {
        Ok(preparation) => LoadedContext::Prepared(preparation),
        Err(_) => LoadedContext::Unanswered,
    }
}
fn whisper() -> ContextCapabilities {
    ContextCapabilities {
        initial_prompt: true,
        whisper_extension: true,
        ..ContextCapabilities::default()
    }
}
fn report(
    stored: &[String],
    streaming: bool,
    loaded: LoadedContext,
) -> Result<VocabularyStatusReport, ContextBudgetUnverified> {
    status_report(StatusInput {
        model_id: "model",
        streaming,
        stored,
        loaded,
    })
}

#[test]
fn whisper_batch_uses_the_vocabulary_and_lists_what_reaches_the_engine() -> Result<(), String> {
    let stored = list(&["Claude Code", "<bad>", "OpenClaw"]);
    let status =
        report(&stored, false, prepared(whisper(), &stored)).map_err(|e| format!("{e:?}"))?;
    assert_eq!(status.status, VocabularyStatus::UsedDuringRecognition);
    assert_eq!(status.terms_sent, list(&["Claude Code", "OpenClaw"]));
    assert_eq!(
        status.mechanism,
        Some(ContextMechanism::WhisperInitialPrompt)
    );
    assert_eq!(status.budget.map(|b| b.value), Some(223));
    assert_eq!(
        status.terms_not_sent,
        vec![ExcludedTerm {
            index: 1,
            term: "<bad>".into(),
            reason: ExclusionReason::ForbiddenCharacter
        }]
    );
    Ok(())
}

#[test]
fn every_family_without_context_is_not_supported_and_sends_nothing() -> Result<(), String> {
    let stored = list(&["Claude Code"]);
    // ONNX engines (Parakeet, Moonshine, SenseVoice, GigaAM, Canary, Cohere) and transcribe-cpp
    // families without a declared mechanism (Canary, Voxtral, Parakeet, Moonshine).
    let voxtral = ContextCapabilities {
        initial_prompt: true,
        ..ContextCapabilities::default()
    };
    for loaded in [
        LoadedContext::NoContextApi,
        prepared(ContextCapabilities::default(), &stored),
        prepared(voxtral, &stored),
    ] {
        let status = report(&stored, false, loaded).map_err(|e| format!("{e:?}"))?;
        assert_eq!(status.status, VocabularyStatus::NotSupported);
        assert!(status.terms_sent.is_empty());
        assert_eq!(status.terms_not_sent[0].reason, ExclusionReason::NoContext);
    }
    Ok(())
}

#[test]
fn streaming_never_uses_the_vocabulary_even_on_a_supporting_model() -> Result<(), String> {
    let stored = list(&["Claude Code"]);
    let status =
        report(&stored, true, prepared(whisper(), &stored)).map_err(|e| format!("{e:?}"))?;
    assert_eq!(status.status, VocabularyStatus::NotUsedStreaming);
    assert_eq!(status.mode, RecognitionMode::Streaming);
    assert!(status.terms_sent.is_empty());
    Ok(())
}

#[test]
fn unknown_has_a_distinct_reason_for_not_loaded_and_unanswered() -> Result<(), String> {
    let stored = list(&["Claude Code"]);
    for (loaded, reason) in [
        (LoadedContext::NotLoaded, StatusReason::ModelNotLoaded),
        (
            LoadedContext::Unanswered,
            StatusReason::CapabilityQueryUnanswered,
        ),
    ] {
        let status = report(&stored, true, loaded).map_err(|e| format!("{e:?}"))?;
        assert_eq!(status.status, VocabularyStatus::UnknownUntilModelLoads);
        assert_eq!(status.reason, Some(reason));
        assert!(status.terms_sent.is_empty());
    }
    Ok(())
}

#[test]
fn supporting_model_with_nothing_to_send_reports_why() -> Result<(), String> {
    let cases = [
        (list(&[]), StatusReason::EmptyVocabulary),
        (list(&["<a>", ""]), StatusReason::InvalidTermsOnly),
        (vec!["x".repeat(100); 3], StatusReason::OverBudget),
    ];
    for (stored, reason) in cases {
        // 100-character terms overflow a 10-token budget; the others never reach the counter.
        let fitted = fit_whisper_prefix(&stored, 10, &CharCounter)?;
        let loaded = LoadedContext::Prepared(ContextPreparation::Fitted(FittedContext {
            mechanism: ContextMechanism::WhisperInitialPrompt,
            budget: ContextBudget::WHISPER,
            sent: fitted.sent,
            tokens: fitted.tokens,
        }));
        let status = report(&stored, false, loaded).map_err(|e| format!("{e:?}"))?;
        assert_eq!(status.status, VocabularyStatus::NoTermSent);
        assert_eq!(status.reason, Some(reason));
        assert!(status.terms_sent.is_empty());
    }
    Ok(())
}

#[test]
fn undocumented_budget_is_a_technical_error_not_a_status() {
    let stored = list(&["Claude Code"]);
    let qwen = ContextCapabilities {
        vocabulary: true,
        ..ContextCapabilities::default()
    };
    assert_eq!(
        report(&stored, false, prepared(qwen, &stored)),
        Err(ContextBudgetUnverified(ContextMechanism::GenericVocabulary))
    );
    // A streaming model receives no context anyway: one of the five statuses, no error.
    let streamed = report(&stored, true, prepared(qwen, &stored)).map(|status| status.status);
    assert_eq!(streamed, Ok(VocabularyStatus::NotUsedStreaming));
}

#[test]
fn terms_after_the_first_overflow_are_listed_as_over_budget() -> Result<(), String> {
    let stored = list(&["ab", "a-very-long-term", "c"]);
    let loaded = LoadedContext::Prepared(ContextPreparation::Fitted(FittedContext {
        mechanism: ContextMechanism::WhisperInitialPrompt,
        budget: ContextBudget::WHISPER,
        sent: 1,
        tokens: 3,
    }));
    let status = report(&stored, false, loaded).map_err(|e| format!("{e:?}"))?;
    assert_eq!(status.terms_sent, list(&["ab"]));
    let excluded = status
        .terms_not_sent
        .iter()
        .map(|t| (t.index, t.reason))
        .collect::<Vec<_>>();
    assert_eq!(
        excluded,
        vec![
            (1, ExclusionReason::OverBudget),
            (2, ExclusionReason::OverBudget)
        ]
    );
    Ok(())
}
