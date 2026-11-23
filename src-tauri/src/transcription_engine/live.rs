//! Engine-neutral contract of a live (streaming) remote transcription session.

use std::fmt;

/// Live text split the way the overlay renders it: `committed` no longer
/// changes, `tentative` may still be rewritten by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LiveText {
    pub committed: String,
    pub tentative: String,
}

/// Why a live session could not start or continue. Every variant makes the
/// caller fall back to the local model for the same recording.
#[derive(Debug)]
pub enum LiveSessionError {
    /// The engine has no usable sign-in or configuration.
    Unavailable(String),
    /// The connection or the remote service failed.
    Stream(String),
    /// The remote service did not answer in time; names the stage.
    Timeout(&'static str),
}

impl fmt::Display for LiveSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(reason) => write!(f, "engine unavailable: {reason}"),
            Self::Stream(reason) => write!(f, "live stream failed: {reason}"),
            Self::Timeout(stage) => write!(f, "live stream timed out while {stage}"),
        }
    }
}

impl std::error::Error for LiveSessionError {}

/// One dictation streamed to a remote engine. A session is owned by a single
/// worker thread for its whole life; dropping it abandons the dictation.
pub trait LiveSession: Send {
    /// Send 16 kHz mono samples captured since the previous call.
    fn send_audio(&mut self, samples: &[f32]) -> Result<(), LiveSessionError>;

    /// Read pending engine events without blocking for long; returns the new
    /// live text when it changed.
    fn poll(&mut self) -> Result<Option<LiveText>, LiveSessionError>;

    /// End the dictation and wait (bounded) for the final text, consuming the
    /// session and closing its connection.
    fn finish(self: Box<Self>) -> Result<String, LiveSessionError>;
}
