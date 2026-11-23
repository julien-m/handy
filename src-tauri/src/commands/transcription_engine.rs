//! Commands of the transcription engine settings.

use crate::transcription_engine::{self, ChatGptSignInStatus};

// @spec FR-002: ChatGPT sign-in status — .specs/features/024-transcription-engine/spec.md#fr-002
/// Whether a ChatGPT sign-in (from Codex) is available to the ChatGPT engine.
/// Reads a local file only; never returns or logs the credentials.
#[tauri::command]
#[specta::specta]
pub fn get_chatgpt_sign_in_status() -> ChatGptSignInStatus {
    transcription_engine::sign_in_status()
}
