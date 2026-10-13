//! One smart-paste request and the coordinator's remaining ports
//! (destination validity, late-read diagnostics, read-event source).

use std::time::{Duration, Instant};

use super::ids::{DestinationGeneration, DictationId, Transcript};
use super::outcome::PasteDiagnostic;
use super::pasteboard_port::ReadEvent;
use super::settlement::ClipboardPolicy;

/// Destination captured by the destination feature (006): the process and
/// generation smart paste must insert into. 007 only carries and checks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationContext {
    /// Destination process identifier.
    pub pid: i32,
    /// Destination bundle identifier, which selects the strategy.
    pub bundle_id: Option<String>,
    /// Destination generation, owned by 006.
    pub generation: DestinationGeneration,
}

/// Immutable input of one insertion.
#[derive(Debug, Clone)]
pub struct InsertionRequest {
    /// Dictation being inserted.
    pub dictation_id: DictationId,
    /// Final text after trailing-space handling (never logged).
    pub transcript: Transcript,
    /// Where to insert.
    pub destination: DestinationContext,
    /// Clipboard handling setting.
    pub clipboard_policy: ClipboardPolicy,
    /// Existing delay before the paste keystroke (`paste_delay_ms`).
    pub paste_delay: Duration,
}

/// Revalidates the destination before AX writes and key posting (006).
pub trait DestinationGuard: Send {
    /// Whether `destination` is still the frontmost target of its generation.
    fn is_current(&mut self, destination: &DestinationContext) -> bool;
}

/// Receives late-read diagnostics for the recovery bubble (010).
pub trait DiagnosticSink: Send {
    /// Called once per transaction whose item was read after "not read".
    fn late_read(&mut self, diagnostic: PasteDiagnostic);
}

/// Source of served-read events while the coordinator waits.
pub trait ReadEvents {
    /// Returns the next read stamped at or before `deadline`, waiting until
    /// it arrives; `None` once the deadline is reached. A read stamped after
    /// the deadline must be kept for later, so timers and reads are
    /// processed in timestamp order.
    fn next_read(&mut self, deadline: Instant) -> Option<ReadEvent>;
}
