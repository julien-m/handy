//! ChatGPT engine: live dictation through the ChatGPT desktop dictation stream,
//! authenticated with the user's ChatGPT sign-in from Codex. Audio leaves the
//! device; any failure makes the caller fall back to the local model.

mod auth;
mod protocol;
mod session;
mod transcript;

pub use auth::{sign_in_status, ChatGptSignInStatus};

use crate::transcription_engine::{LiveSession, LiveSessionError};
use session::ChatGptSession;

// @spec FR-003: ChatGPT live dictation — .specs/features/024-transcription-engine/spec.md#fr-003
/// Open a ChatGPT dictation session with the current sign-in.
pub(super) fn connect(language: Option<&str>) -> Result<Box<dyn LiveSession>, LiveSessionError> {
    let sign_in = auth::read_sign_in()?;
    let session = ChatGptSession::connect(&sign_in, language)?;
    Ok(Box::new(session))
}
