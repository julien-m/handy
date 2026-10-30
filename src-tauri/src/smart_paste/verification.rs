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
//! Finding N28: a terminal (cmux) exposes a readable focused element whose
//! value does not follow the paste, so an unchanged value proved nothing and
//! the retry pasted the transcript twice. Only web text is verified: an
//! element below an `AXWebArea` (Chrome and Safari pages, Electron apps),
//! whose value the engine keeps in sync with edits. Any other element is
//! "unreadable" and keeps the current behavior. Right before the retry the
//! value is read once more, so a late paste is never sent twice.
//!
//! This module only decides; the coordinator performs every native effect.
//! Field values are compared, never logged.

use std::time::{Duration, Instant};

use super::ax_port::AncestryLimits;
use super::direct::PER_CALL_TIMEOUT;
use super::outcome::PasteOutcomeKind;

// @spec FR-012: Paste verification decisions — .specs/features/007-smart-paste/spec.md#fr-012

/// How long the field is polled after the read (and again after the retry).
/// N11 measured a 4/6 success rate with a 250 ms paste delay and 5/6 at 1 s:
/// 300 ms lets a slow page apply the paste without delaying Enter noticeably.
pub const VERIFY_WINDOW: Duration = Duration::from_millis(300);
/// Pause between two reads of the field: fifteen reads per window at most,
/// enough to see a page update without flooding it with AX requests.
pub const VERIFY_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Bounds of the web-area search (N28). A text input in a busy page sits
/// a few dozen levels below its `AXWebArea`; 48 parents and 100 ms cover it
/// while keeping a slow tree from delaying the first Cmd+V for long.
pub const WEB_ANCESTRY_LIMITS: AncestryLimits = AncestryLimits {
    per_call: PER_CALL_TIMEOUT,
    max_depth: 48,
    budget: Duration::from_millis(100),
};

/// Why the verification did not apply (logged with the `Unreadable`
/// verdict, never with any content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnreadableReason {
    /// No focused element of the destination with a readable value, or a
    /// password field.
    NoReadableField,
    /// The element is not web text: its value is not known to follow a
    /// paste (terminals, N28), so an unchanged value proves nothing.
    NotWebText,
    /// A read of the value failed during the verification.
    ReadFailed,
}

/// Business rule (N28): only web text is verified. `web_ancestry` is the
/// result of the bounded web-area search; a timeout or an exhausted budget
/// is no proof, so it is treated like an element outside any web area.
pub fn web_text_scope<E>(web_ancestry: Result<bool, E>) -> Result<(), UnreadableReason> {
    match web_ancestry {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(UnreadableReason::NotWebText),
    }
}

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
    /// The verification did not apply, for the given reason.
    Unreadable(UnreadableReason),
}

impl VerifyVerdict {
    /// The terminal outcome replacing "read observed": only a failure
    /// changes it, so an unreadable field keeps the current behavior.
    pub fn outcome_kind(self) -> PasteOutcomeKind {
        match self {
            Self::Failed(_) => PasteOutcomeKind::FieldUnchanged,
            Self::Verified | Self::RetriedVerified | Self::Unreadable(_) => {
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
            Self::Unreadable(_) => "Unreadable",
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
        (_, FieldObservation::Unreadable) => {
            done(VerifyVerdict::Unreadable(UnreadableReason::ReadFailed))
        }
        (VerifyStage::AfterRetry, FieldObservation::Unchanged) => {
            done(VerifyVerdict::Failed(VerifyFailure::UnchangedAfterRetry))
        }
        (VerifyStage::First, FieldObservation::Unchanged) => match permit() {
            RetryPermit::Allowed => VerifyStep::Retry,
            refused => done(VerifyVerdict::Failed(VerifyFailure::RetryRefused(refused))),
        },
    }
}

/// What to do with the single retry, given one last read of the field taken
/// right before the repeated Cmd+V.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryGate {
    /// Still unchanged: send the repeated Cmd+V.
    Send,
    /// No second Cmd+V: the verification ends with this verdict.
    Skip(VerifyVerdict),
}

/// Business rule (N28): a page that applied the paste after the window must
/// not receive it twice. A change seen by the last read is a verified paste;
/// a failed read gives no proof that the field is still empty, so nothing
/// is sent and the current behavior is kept.
pub fn gate_retry(last_read: FieldObservation) -> RetryGate {
    match last_read {
        FieldObservation::Unchanged => RetryGate::Send,
        FieldObservation::Changed => RetryGate::Skip(VerifyVerdict::Verified),
        FieldObservation::Unreadable => {
            RetryGate::Skip(VerifyVerdict::Unreadable(UnreadableReason::ReadFailed))
        }
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
