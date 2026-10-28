//! Ports of the recovery service: the nonactivating surface, 006's retained transcript, the
//! destination facts of an outcome, the deliberate Copy writer, timers and the `paste-error`
//! fallback. The Tauri adapters live in `app`; tests inject fakes. None carries the transcript
//! into an event or a log.
use super::types::{BubbleId, RecoveryBubbleView};
use crate::destination::outcome::{HistoryStatus, InsertionOutcome, TranscriptRef};
use std::time::{Duration, Instant};

/// The recovery window. Calls return at once; the window answers through the service
/// (`rendered`, `presentation_failed`, `window_destroyed`, `drag_failed`, `drag_ended`).
pub trait RecoverySurfacePort: Send + Sync {
    /// Show `view` in place of the indicator without taking focus (window created if needed).
    fn present(&self, view: RecoveryBubbleView);
    /// Update the shown bubble in place (late read).
    fn update(&self, view: RecoveryBubbleView);
    /// Hide the recovery window.
    fn hide(&self);
    /// Destroy the recovery window of a hung presentation of `bubble_id`.
    fn destroy(&self, bubble_id: BubbleId);
    /// Start a native drag of `transcript` from the bubble `bubble_id` (macOS).
    fn begin_drag(&self, bubble_id: BubbleId, transcript: String);
    /// Show the bubble again after a refused or cancelled drop.
    fn show_again(&self, view: RecoveryBubbleView);
}

/// A retained failed dictation, read from 006's slot.
pub struct RetainedText {
    /// The transcript, shown in the preview without the paste's trailing space.
    pub transcript: String,
    /// Exactly what the method would have inserted (trailing space included when enabled),
    /// carried by Copy and the drag.
    pub inserted: String,
}

/// 006's volatile recovery slot.
pub trait TranscriptSource: Send + Sync {
    /// The retained text behind `reference`, `None` once expired.
    fn read(&self, reference: TranscriptRef) -> Option<RetainedText>;
    /// Release `reference` (harmless when a later dictation already superseded it).
    fn release(&self, reference: TranscriptRef);
}

/// Destination facts of an outcome, read while 006's session still exists.
pub trait SessionFacts: Send + Sync {
    /// The destination's display name, if one can be named.
    fn app_name(&self, outcome: &InsertionOutcome) -> Option<String>;
    /// When 006 set this dictation's disposition to `Recover` (start of the QE latency).
    fn decided_at(&self, outcome: &InsertionOutcome) -> Option<Instant>;
}

/// Why a deliberate Copy did not write (logged as a kind, never with text).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyError {
    /// 007's pasteboard port refused the write.
    Pasteboard,
    /// 007's actor stopped.
    ActorStopped,
    /// 007's actor did not answer within its bound.
    TimedOut,
    /// The clipboard plugin write of `dev` failed.
    Clipboard,
}

/// The deliberate Copy writer (007's actor when it runs, else `dev`'s clipboard write).
pub trait CopyPort: Send + Sync {
    /// Write `transcript` as plain text; may block (never called on the main thread).
    fn write(&self, transcript: &str) -> Result<(), CopyError>;
}

/// Delayed tasks (the presentation hang limit).
pub trait TimerPort: Send + Sync {
    /// Run `task` once after `delay`, off the main thread.
    fn after(&self, delay: Duration, task: Box<dyn FnOnce() + Send>);
}

/// 006's existing failure toast, the only fallback when the bubble cannot be presented.
pub trait FallbackPort: Send + Sync {
    /// Emit `paste-error` for a dictation whose history status is `history`.
    fn paste_error(&self, history: HistoryStatus);
}
