//! Experimental paste verification (AC-019, finding N11): pure decisions.
//!
//! A read observed after Cmd+V only proves that the target asked for the
//! transcript; some pages (Google's AI mode in Chrome) receive the `paste`
//! event and then insert nothing. With the `paste_verification` setting on,
//! the coordinator compares the focused field value read before the chord
//! with the value read during a bounded window after the read: a change is a
//! verified paste; no change gets exactly one more Cmd+V while the clipboard
//! still owns the transcript; still no change is a failure handed to the
//! recovery bubble. A field that cannot be read keeps the current behavior.
//!
//! This module only decides; the coordinator performs every native effect.
//! Field values are compared, never logged.

use std::time::{Duration, Instant};

use super::outcome::PasteOutcomeKind;

// @spec FR-012: Paste verification decisions — .specs/features/007-smart-paste/spec.md#fr-012

/// How long the field is polled after the read (and again after the retry).
/// N11 measured a 4/6 success rate with a 250 ms paste delay and 5/6 at 1 s:
/// 300 ms lets a slow page apply the paste without delaying Enter noticeably.
pub const VERIFY_WINDOW: Duration = Duration::from_millis(300);
/// Pause between two reads of the field: fifteen reads per window at most,
/// enough to see a page update without flooding it with AX requests.
pub const VERIFY_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What one read of the focused field showed, compared with the value read
/// right before the first Cmd+V.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldObservation {
    /// The value differs from the value before the paste.
    Changed,
    /// The value equals the value before the paste.
    Unchanged,
    /// The value could not be read.
    Unreadable,
}

impl FieldObservation {
    /// Compares a fresh read with the value before the paste.
    pub fn of<E>(before: &[u16], read: Result<Vec<u16>, E>) -> Self {
        match read {
            Ok(value) if value == before => Self::Unchanged,
            Ok(_) => Self::Changed,
            Err(_) => Self::Unreadable,
        }
    }
}

/// Which verification window just ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStage {
    /// The window after the first Cmd+V.
    First,
    /// The window after the single repeated Cmd+V.
    AfterRetry,
}

/// Whether the single repeated Cmd+V may be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryPermit {
    /// The clipboard still owns the transcript and the destination is current.
    Allowed,
    /// The clipboard no longer holds this dictation's item (user copy,
    /// settlement): a retry would paste something else.
    ClipboardChanged,
    /// The destination is no longer the frontmost target (006).
    DestinationLost,
}

/// Why a verified paste failed (logged; the outcome is "field unchanged").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyFailure {
    /// Unchanged after the first window, and the retry was not allowed.
    RetryRefused(RetryPermit),
    /// The repeated Cmd+V could not be posted.
    RetryChordFailed,
    /// Unchanged after the repeated Cmd+V too.
    UnchangedAfterRetry,
}

/// Terminal verdict of one verification, as logged
/// (`verify dictation N: <verdict> in Xms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyVerdict {
    /// The field changed after the first Cmd+V.
    Verified,
    /// The field changed after the repeated Cmd+V.
    RetriedVerified,
    /// The field stayed unchanged.
    Failed(VerifyFailure),
    /// No readable field: the verification did not apply.
    Unreadable,
}

impl VerifyVerdict {
    /// The terminal outcome replacing "read observed": only a failure
    /// changes it, so an unreadable field keeps the current behavior.
    pub fn outcome_kind(self) -> PasteOutcomeKind {
        match self {
            Self::Failed(_) => PasteOutcomeKind::FieldUnchanged,
            Self::Verified | Self::RetriedVerified | Self::Unreadable => {
                PasteOutcomeKind::ReadObserved
            }
        }
    }

    /// Log label without the failure detail.
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "Verified",
            Self::RetriedVerified => "RetriedVerified",
            Self::Failed(_) => "Failed",
            Self::Unreadable => "Unreadable",
        }
    }
}

/// Next step once a window ended with `observation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStep {
    /// Send the single repeated Cmd+V, then verify again.
    Retry,
    /// The verification is over.
    Done(VerifyVerdict),
}

/// Business rule (AC-019): a change verifies the paste; an unreadable field
/// keeps the current behavior and is never retried (the text may be there);
/// an unchanged field gets one retry, only when `permit` allows it, and
/// fails after it. `permit` is asked only when a retry is due.
pub fn next_step(
    stage: VerifyStage,
    observation: FieldObservation,
    permit: impl FnOnce() -> RetryPermit,
) -> VerifyStep {
    let done = VerifyStep::Done;
    match (stage, observation) {
        (VerifyStage::First, FieldObservation::Changed) => done(VerifyVerdict::Verified),
        (VerifyStage::AfterRetry, FieldObservation::Changed) => {
            done(VerifyVerdict::RetriedVerified)
        }
        (_, FieldObservation::Unreadable) => done(VerifyVerdict::Unreadable),
        (VerifyStage::AfterRetry, FieldObservation::Unchanged) => {
            done(VerifyVerdict::Failed(VerifyFailure::UnchangedAfterRetry))
        }
        (VerifyStage::First, FieldObservation::Unchanged) => match permit() {
            RetryPermit::Allowed => VerifyStep::Retry,
            refused => done(VerifyVerdict::Failed(VerifyFailure::RetryRefused(refused))),
        },
    }
}

/// One bounded polling window: reads stop at `end`, a change ends it early.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyWindow {
    end: Instant,
}

impl VerifyWindow {
    /// A window of [`VERIFY_WINDOW`] starting at `start`.
    pub fn starting_at(start: Instant) -> Self {
        Self {
            end: start + VERIFY_WINDOW,
        }
    }

    /// Wait before the next read, or `None` once the window is over (the
    /// last read was the final one).
    pub fn pause_before_next_read(self, now: Instant) -> Option<Duration> {
        let remaining = self.end.saturating_duration_since(now);
        (!remaining.is_zero()).then(|| remaining.min(VERIFY_POLL_INTERVAL))
    }

    /// Whether a read observing `observation` ends the window now.
    pub fn is_settled_by(observation: FieldObservation) -> bool {
        observation == FieldObservation::Changed
    }
}

#[cfg(test)]
#[path = "verification_tests.rs"]
mod tests;
