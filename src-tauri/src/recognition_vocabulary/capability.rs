//! Recognition-context features a loaded model declares, read by the engine worker at load, and
//! the mechanism Handy may use with them. Support comes from the loaded model's own probes, never
//! from a list of model names.
use serde::{Deserialize, Serialize};

/// Context features of the loaded model, as its probes answered them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextCapabilities {
    /// `Feature::InitialPrompt`: the Whisper run extension's initial prompt.
    pub initial_prompt: bool,
    /// The model accepts the Whisper run extension on the run slot. Some non-Whisper families
    /// declare `InitialPrompt` yet reject that extension, so both are required.
    pub whisper_extension: bool,
    /// `Feature::Vocabulary`: `RunOptions::vocabulary` is formatted for this model.
    pub vocabulary: bool,
    /// `Feature::ContextPrompt`: `RunOptions::prompt` reaches a conditioning slot.
    pub context_prompt: bool,
}

/// Probe the context features of a loaded model (worker side).
pub fn probe_capabilities(model: &transcribe_cpp::Model) -> ContextCapabilities {
    use transcribe_cpp::{sys::TRANSCRIBE_EXT_KIND_WHISPER_RUN, ExtSlot, Feature};
    ContextCapabilities {
        initial_prompt: model.supports(Feature::InitialPrompt),
        whisper_extension: model.accepts_ext(ExtSlot::Run, TRANSCRIBE_EXT_KIND_WHISPER_RUN),
        vocabulary: model.supports(Feature::Vocabulary),
        context_prompt: model.supports(Feature::ContextPrompt),
    }
}

/// How the vocabulary reaches the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ContextMechanism {
    /// Whisper run extension `initial_prompt`, terms joined with ", ".
    WhisperInitialPrompt,
    /// Generic `RunOptions::vocabulary` (Qwen3-ASR, Fun-ASR, Granite with the GGUF capability).
    GenericVocabulary,
    /// Generic `RunOptions::prompt` used as recognition context.
    ContextPrompt,
}

/// What Handy may do with a model's declared context features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanismSupport {
    /// No recognition-context mechanism: the vocabulary is never sent.
    None,
    /// A mechanism with a documented budget (`contracts/context-budget.md`).
    Documented(ContextMechanism),
    /// The model declares a mechanism whose budget is not documented for this build: it is
    /// never sent unchecked text, and the status reports `context_budget_unverified`.
    Unverified(ContextMechanism),
}

/// The mechanism of `capabilities`, the Whisper initial prompt first: it is the only one with a
/// documented budget, and the generic fields cannot be attached together with it.
// @spec FR-002: Loaded capability decides context support — .specs/features/003-recognition-vocabulary/spec.md#fr-002
pub fn resolve_mechanism(capabilities: ContextCapabilities) -> MechanismSupport {
    if capabilities.initial_prompt && capabilities.whisper_extension {
        return MechanismSupport::Documented(ContextMechanism::WhisperInitialPrompt);
    }
    // Business rule: the generic budgets depend on per-request audio tokens and on constants the
    // pinned engine does not expose, so these mechanisms stay disabled until documented.
    if capabilities.vocabulary {
        return MechanismSupport::Unverified(ContextMechanism::GenericVocabulary);
    }
    if capabilities.context_prompt {
        return MechanismSupport::Unverified(ContextMechanism::ContextPrompt);
    }
    MechanismSupport::None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(
        initial_prompt: bool,
        whisper_extension: bool,
        vocabulary: bool,
    ) -> ContextCapabilities {
        ContextCapabilities {
            initial_prompt,
            whisper_extension,
            vocabulary,
            context_prompt: false,
        }
    }

    #[test]
    fn whisper_needs_the_feature_and_the_accepted_extension() {
        assert_eq!(
            resolve_mechanism(caps(true, true, true)),
            MechanismSupport::Documented(ContextMechanism::WhisperInitialPrompt)
        );
        // Voxtral-like: declares InitialPrompt but rejects the Whisper extension.
        assert_eq!(
            resolve_mechanism(caps(true, false, false)),
            MechanismSupport::None
        );
        assert_eq!(
            resolve_mechanism(caps(false, true, false)),
            MechanismSupport::None
        );
    }

    #[test]
    fn generic_mechanisms_are_unverified_until_documented() {
        assert_eq!(
            resolve_mechanism(caps(false, false, true)),
            MechanismSupport::Unverified(ContextMechanism::GenericVocabulary)
        );
        let prompt_only = ContextCapabilities {
            context_prompt: true,
            ..ContextCapabilities::default()
        };
        assert_eq!(
            resolve_mechanism(prompt_only),
            MechanismSupport::Unverified(ContextMechanism::ContextPrompt)
        );
        assert_eq!(
            resolve_mechanism(ContextCapabilities::default()),
            MechanismSupport::None
        );
    }
}
