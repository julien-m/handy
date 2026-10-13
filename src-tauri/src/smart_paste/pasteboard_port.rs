//! Pasteboard port used by the smart-paste coordinator (AC-008, AC-009).
//!
//! Every write that could replace user content is guarded by the
//! `changeCount` recorded at publication: when it differs, the adapter
//! writes nothing and reports [`GuardedWrite::OwnershipChanged`].

use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use super::ids::{TransactionId, Transcript};
use super::snapshot::{PasteboardSnapshot, SnapshotBudget, SnapshotError};

/// One plain-text request served by the on-demand item of `transaction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadEvent {
    /// Transaction whose item was read.
    pub transaction: TransactionId,
    /// Instant at which the request was served.
    pub at: Instant,
}

/// Receives provider reads; called on the thread that serves the request.
pub type ReadSink = Arc<dyn Fn(ReadEvent) + Send + Sync>;

/// Errors of pasteboard operations. They never carry clipboard content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteboardError {
    /// The pasteboard changed between the snapshot and the publication.
    SourceChanged,
    /// The snapshot could not be captured within its bounds.
    SnapshotFailed(SnapshotError),
    /// AppKit refused the write.
    WriteFailed,
    /// The main thread did not run the operation in time.
    MainThreadUnavailable,
}

impl fmt::Display for PasteboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PasteboardError {}

/// How an eager transcript is marked when it replaces the on-demand item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptMarking {
    /// Current-host-only content with the transient, concealed and
    /// auto-generated markers, like the on-demand item it replaces.
    Private,
    /// Ordinary plain text: the deliberate "copy to clipboard" setting.
    Plain,
}

/// Result of an ownership-guarded write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedWrite {
    /// The pasteboard still belonged to the transaction and was written.
    Written,
    /// Someone wrote since publication: nothing was written.
    OwnershipChanged,
}

/// Pasteboard operations needed by smart paste.
pub trait PasteboardPort: Send {
    /// Current `changeCount`.
    fn change_count(&mut self) -> Result<i64, PasteboardError>;

    /// Hint that a capture may follow soon: an adapter may start its
    /// snapshot helper now so its startup stays out of the capture budget.
    /// Never reads the pasteboard. The default does nothing.
    fn prepare_capture(&mut self, _budget: SnapshotBudget) {}

    /// Captures a bounded snapshot of the current content.
    fn capture_snapshot(
        &mut self,
        budget: SnapshotBudget,
    ) -> Result<PasteboardSnapshot, PasteboardError>;

    /// Publishes `transcript` as an on-demand, current-host-only item marked
    /// transient, only if `changeCount` still equals `expected_source_count`.
    /// Returns the `changeCount` after publication.
    fn publish_lazy_transcript(
        &mut self,
        transaction: TransactionId,
        transcript: &Transcript,
        expected_source_count: i64,
    ) -> Result<i64, PasteboardError>;

    /// Puts back the saved items and types (or clears when it was empty) as
    /// current-host-only content, only if `changeCount` still equals
    /// `publication_count`.
    fn restore_snapshot(
        &mut self,
        snapshot: Arc<PasteboardSnapshot>,
        publication_count: i64,
    ) -> Result<GuardedWrite, PasteboardError>;

    /// Replaces the on-demand item with eager plain text marked as
    /// `marking` says, only if `changeCount` still equals `publication_count`.
    fn materialize_transcript(
        &mut self,
        transcript: &Transcript,
        publication_count: i64,
        marking: TranscriptMarking,
    ) -> Result<GuardedWrite, PasteboardError>;

    /// Deliberately writes plain text without markers (Screen Sharing
    /// hand-over, copy-to-clipboard after a non-clipboard insertion).
    fn write_plain_text(&mut self, transcript: &Transcript) -> Result<(), PasteboardError>;

    /// Releases the provider of a settled or superseded transaction.
    fn release_transaction(&mut self, transaction: TransactionId);
}
