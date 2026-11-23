//! Transcription engines: decides which engine transcribes a dictation and
//! opens live sessions for remote engines. The local engine itself stays in
//! [`crate::managers::transcription`]; each remote engine lives in its own
//! submodule behind the engine-neutral [`LiveSession`] contract, so adding a
//! provider means adding a submodule and a [`TranscriptionEngine`] variant.

mod chatgpt;
mod live;

pub use chatgpt::{sign_in_status, ChatGptSignInStatus};
pub use live::{LiveSession, LiveSessionError, LiveText};

use crate::settings::TranscriptionEngine;

// @spec FR-006: Engine-neutral live session contract — .specs/features/024-transcription-engine/spec.md#fr-006
/// Whether `engine` transcribes live through a remote session (and therefore
/// always uses the streaming dictation path, whatever the local model).
pub fn is_live(engine: TranscriptionEngine) -> bool {
    match engine {
        TranscriptionEngine::Local => false,
        TranscriptionEngine::ChatGpt => true,
    }
}

// @spec FR-008: Local model locked while an API engine is active — .specs/features/024-transcription-engine/spec.md#fr-008
/// Whether the user may switch the local model: only while the local engine
/// transcribes. With a live API engine the local model is the fallback and is
/// changed after going back to the local engine.
pub fn can_switch_local_model(engine: TranscriptionEngine) -> bool {
    !is_live(engine)
}

/// Open the live session of a remote engine for one dictation.
///
/// Returns `None` for the local engine. `language` is an ISO code, or `None`
/// for automatic detection. Blocks while connecting (bounded by the engine's
/// own timeout); call it from a worker thread.
pub fn open_live_session(
    engine: TranscriptionEngine,
    language: Option<&str>,
) -> Option<Result<Box<dyn LiveSession>, LiveSessionError>> {
    match engine {
        TranscriptionEngine::Local => None,
        TranscriptionEngine::ChatGpt => Some(chatgpt::connect(language)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_engine_opens_no_live_session() {
        assert!(!is_live(TranscriptionEngine::Local));
        assert!(open_live_session(TranscriptionEngine::Local, Some("fr")).is_none());
    }

    #[test]
    fn chatgpt_engine_is_live() {
        assert!(is_live(TranscriptionEngine::ChatGpt));
    }

    #[test]
    fn local_model_switch_is_refused_while_an_api_engine_is_active() {
        assert!(can_switch_local_model(TranscriptionEngine::Local));
        assert!(!can_switch_local_model(TranscriptionEngine::ChatGpt));
    }
}
