//! Context preparation for the loaded model: which mechanism, which budget and how many of the
//! eligible terms fit. The engine worker runs it with the model's tokenizer; the same answer
//! drives the next-request status of the settings and the actual batch request.
use super::{
    budget::{fit_whisper_prefix, render_terms, ContextBudget, TokenCounter},
    capability::{resolve_mechanism, ContextCapabilities, ContextMechanism, MechanismSupport},
};
use serde::{Deserialize, Serialize};

/// Terms that fit a documented mechanism, with the evidence of the fit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FittedContext {
    pub mechanism: ContextMechanism,
    pub budget: ContextBudget,
    /// Number of leading eligible terms that fit (a whole prefix, in list order).
    pub sent: usize,
    /// Tokens of the rendered context of those terms, counted by the loaded tokenizer.
    pub tokens: usize,
}

/// What the loaded model can receive for a list of eligible terms.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextPreparation {
    /// The model declares no recognition-context mechanism.
    NoMechanism,
    /// The model declares a mechanism without a documented budget (`context_budget_unverified`).
    Unverified(ContextMechanism),
    /// A documented mechanism and the terms that fit it (possibly none).
    Fitted(FittedContext),
}

impl ContextPreparation {
    /// Initial prompt to attach to the run for `terms` (the eligible list this preparation was
    /// computed for), or `None` when no term is sent.
    pub fn whisper_prompt(&self, terms: &[String]) -> Option<String> {
        match self {
            Self::Fitted(fitted)
                if fitted.mechanism == ContextMechanism::WhisperInitialPrompt
                    && fitted.sent > 0
                    && fitted.sent <= terms.len() =>
            {
                Some(render_terms(&terms[..fitted.sent]))
            }
            _ => None,
        }
    }
}

/// Prepare the context of `terms` for a model with `capabilities`, counting tokens with
/// `counter` (the loaded model's tokenizer).
///
/// # Errors
/// A tokenizer failure: the request then runs without context.
// @spec FR-003: Exact request within documented budget — .specs/features/003-recognition-vocabulary/spec.md#fr-003
pub fn prepare_context(
    capabilities: ContextCapabilities,
    terms: &[String],
    counter: &dyn TokenCounter,
) -> Result<ContextPreparation, String> {
    match resolve_mechanism(capabilities) {
        MechanismSupport::None => Ok(ContextPreparation::NoMechanism),
        MechanismSupport::Unverified(mechanism) => Ok(ContextPreparation::Unverified(mechanism)),
        MechanismSupport::Documented(mechanism) => {
            let budget = ContextBudget::WHISPER;
            let fitted = fit_whisper_prefix(terms, budget.value, counter)?;
            Ok(ContextPreparation::Fitted(FittedContext {
                mechanism,
                budget,
                sent: fitted.sent,
                tokens: fitted.tokens,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CharCounter;
    impl TokenCounter for CharCounter {
        fn count_tokens(&self, text: &str) -> Result<usize, String> {
            Ok(text.chars().count())
        }
    }
    fn whisper() -> ContextCapabilities {
        ContextCapabilities {
            initial_prompt: true,
            whisper_extension: true,
            ..ContextCapabilities::default()
        }
    }
    fn terms() -> Vec<String> {
        vec!["Claude Code".to_owned(), "OpenClaw".to_owned()]
    }

    #[test]
    fn whisper_prompt_carries_exactly_the_fitted_terms() -> Result<(), String> {
        let prepared = prepare_context(whisper(), &terms(), &CharCounter)?;
        assert_eq!(
            prepared.whisper_prompt(&terms()).as_deref(),
            Some("Claude Code, OpenClaw")
        );
        Ok(())
    }

    #[test]
    fn unsupported_and_unverified_models_never_get_a_prompt() -> Result<(), String> {
        let none = prepare_context(ContextCapabilities::default(), &terms(), &CharCounter)?;
        assert_eq!(none, ContextPreparation::NoMechanism);
        assert_eq!(none.whisper_prompt(&terms()), None);
        let generic = ContextCapabilities {
            vocabulary: true,
            ..ContextCapabilities::default()
        };
        let unverified = prepare_context(generic, &terms(), &CharCounter)?;
        assert_eq!(
            unverified,
            ContextPreparation::Unverified(ContextMechanism::GenericVocabulary)
        );
        assert_eq!(unverified.whisper_prompt(&terms()), None);
        Ok(())
    }

    #[test]
    fn nothing_fitting_sends_no_prompt() -> Result<(), String> {
        let long = vec!["x".repeat(300)];
        let prepared = prepare_context(whisper(), &long, &CharCounter)?;
        assert!(matches!(&prepared, ContextPreparation::Fitted(f) if f.sent == 0));
        assert_eq!(prepared.whisper_prompt(&long), None);
        Ok(())
    }
}
