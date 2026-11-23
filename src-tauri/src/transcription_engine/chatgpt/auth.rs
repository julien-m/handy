//! ChatGPT sign-in read from the Codex CLI auth file (`$CODEX_HOME/auth.json`,
//! `~/.codex/auth.json` by default). Handy only reads it: Codex and the ChatGPT
//! app own sign-in and token refresh. The token is never logged or stored.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::transcription_engine::LiveSessionError;

/// Name of the auth file inside the Codex home directory.
const AUTH_FILE_NAME: &str = "auth.json";
/// Codex's own override of its home directory (documented by the Codex CLI).
const CODEX_HOME_ENV: &str = "CODEX_HOME";

/// Whether a ChatGPT sign-in is available to the ChatGPT engine. A present
/// sign-in may still be expired; the server then refuses the session.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ChatGptSignInStatus {
    SignedIn,
    SignedOut,
}

/// Credentials of one session. `Debug` redacts the token.
pub(super) struct SignIn {
    access_token: String,
    account_id: Option<String>,
}

impl SignIn {
    pub(super) fn access_token(&self) -> &str {
        &self.access_token
    }

    pub(super) fn account_id(&self) -> Option<&str> {
        self.account_id.as_deref()
    }
}

impl fmt::Debug for SignIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignIn")
            .field("access_token", &"<redacted>")
            .field("has_account_id", &self.account_id.is_some())
            .finish()
    }
}

#[derive(Deserialize)]
struct AuthFile {
    tokens: Option<AuthTokens>,
}

#[derive(Deserialize)]
struct AuthTokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

/// Location of the Codex auth file, honoring `CODEX_HOME`.
fn auth_file_path() -> Option<PathBuf> {
    let codex_home = std::env::var_os(CODEX_HOME_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))?;
    Some(codex_home.join(AUTH_FILE_NAME))
}

/// Parse the Codex auth file; only a non-empty access token is accepted.
fn parse_sign_in(raw: &str) -> Result<SignIn, LiveSessionError> {
    let file: AuthFile = serde_json::from_str(raw)
        .map_err(|e| LiveSessionError::Unavailable(format!("unreadable Codex auth file: {e}")))?;
    let tokens = file.tokens.ok_or_else(|| {
        LiveSessionError::Unavailable("Codex is not signed in with ChatGPT".into())
    })?;
    let access_token = tokens
        .access_token
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| LiveSessionError::Unavailable("no ChatGPT access token".into()))?;
    let account_id = tokens.account_id.filter(|id| !id.trim().is_empty());
    Ok(SignIn {
        access_token,
        account_id,
    })
}

fn read_sign_in_at(path: &Path) -> Result<SignIn, LiveSessionError> {
    // The path is not logged: it contains the user's home directory.
    let raw = std::fs::read_to_string(path).map_err(|e| {
        LiveSessionError::Unavailable(format!("cannot read the Codex auth file: {}", e.kind()))
    })?;
    parse_sign_in(&raw)
}

/// Read the current ChatGPT sign-in for a new session.
pub(super) fn read_sign_in() -> Result<SignIn, LiveSessionError> {
    let path = auth_file_path()
        .ok_or_else(|| LiveSessionError::Unavailable("no home directory".into()))?;
    read_sign_in_at(&path)
}

// @spec FR-002: ChatGPT sign-in status — .specs/features/024-transcription-engine/spec.md#fr-002
/// Report whether the ChatGPT engine has a sign-in to use, without exposing it.
pub fn sign_in_status() -> ChatGptSignInStatus {
    match read_sign_in() {
        Ok(_) => ChatGptSignInStatus::SignedIn,
        Err(_) => ChatGptSignInStatus::SignedOut,
    }
}

/// A sign-in for network tests that never reach ChatGPT.
#[cfg(test)]
pub(super) fn test_sign_in() -> SignIn {
    SignIn {
        access_token: "test-token".into(),
        account_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sign_in_accepts_token_and_account() {
        let sign_in =
            parse_sign_in(r#"{"tokens":{"access_token":"tok","account_id":"acc"}}"#).unwrap();
        assert_eq!(sign_in.access_token(), "tok");
        assert_eq!(sign_in.account_id(), Some("acc"));
    }

    #[test]
    fn parse_sign_in_rejects_missing_or_empty_token() {
        assert!(parse_sign_in(r#"{"OPENAI_API_KEY":null}"#).is_err());
        assert!(parse_sign_in(r#"{"tokens":{"access_token":"  "}}"#).is_err());
        assert!(parse_sign_in("not json").is_err());
    }

    #[test]
    fn debug_output_redacts_the_token() {
        let sign_in = parse_sign_in(r#"{"tokens":{"access_token":"secret-value"}}"#).unwrap();
        let debug = format!("{sign_in:?}");
        assert!(!debug.contains("secret-value"));
        assert!(debug.contains("redacted"));
    }

    #[test]
    fn read_sign_in_at_missing_file_is_unavailable() {
        let missing = std::env::temp_dir().join("handy-no-such-dir-024/auth.json");
        assert!(matches!(
            read_sign_in_at(&missing),
            Err(LiveSessionError::Unavailable(_))
        ));
    }
}
