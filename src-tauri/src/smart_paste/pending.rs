//! The clipboard transaction still owned after an insertion returned: it
//! waits for reads to go quiet, keeps serving late reads, and is settled by
//! a timer, a newer dictation or a normal exit.

use std::sync::Arc;

use super::ids::{DictationId, TransactionId, Transcript};
use super::lifecycle::ReadLifecycle;
use super::pasteboard_port::{GuardedWrite, PasteboardError, PasteboardPort, TranscriptMarking};
use super::settlement::{
    settlement_action, ClipboardPolicy, PendingClipboardView, SettlementAction, SettlementTrigger,
};
use super::snapshot::PasteboardSnapshot;

// @spec FR-006: Guarded settlement of the pending transaction — .specs/features/007-smart-paste/spec.md#fr-006

/// A published on-demand item and everything needed to settle it.
#[derive(Debug)]
pub(super) struct PendingClipboard {
    pub(super) dictation_id: DictationId,
    pub(super) transaction: TransactionId,
    pub(super) lifecycle: ReadLifecycle,
    /// Previous content; discarded for good once "not read" is declared.
    pub(super) snapshot: Option<Arc<PasteboardSnapshot>>,
    pub(super) publication_count: i64,
    pub(super) transcript: Transcript,
    pub(super) policy: ClipboardPolicy,
    /// The terminal outcome was "not read": late reads produce a diagnostic.
    pub(super) reports_late_reads: bool,
}

impl PendingClipboard {
    /// What a new dictation may reuse from this transaction. A withheld
    /// snapshot is still carried: it keeps withholding the restoration.
    pub(super) fn view(&self, current_count: Result<i64, PasteboardError>) -> PendingClipboardView {
        PendingClipboardView {
            is_still_owned: current_count == Ok(self.publication_count),
            is_snapshot_restorable: self.snapshot.is_some()
                && self.policy == ClipboardPolicy::DontModify,
        }
    }

    /// Applies `trigger`, writing only while the publication count still
    /// holds, and returns the action actually taken. "Not read" discards
    /// the snapshot so it can never be restored later.
    pub(super) fn settle(
        &mut self,
        trigger: SettlementTrigger,
        pasteboard: &mut dyn PasteboardPort,
    ) -> SettlementAction {
        let action = settlement_action(trigger, self.policy);
        let snapshot = self.snapshot.take();
        let snapshot = (trigger != SettlementTrigger::NotReadDeclared)
            .then_some(snapshot)
            .flatten();
        let applied = self.apply(action, snapshot, pasteboard);
        if applied != SettlementAction::KeepLazyTranscript {
            self.lifecycle.mark_settled();
            pasteboard.release_transaction(self.transaction);
        }
        log::info!(
            "[smart-paste] settled transaction {} ({trigger:?}): {applied:?}",
            self.transaction.0
        );
        applied
    }

    fn apply(
        &self,
        action: SettlementAction,
        snapshot: Option<Arc<PasteboardSnapshot>>,
        pasteboard: &mut dyn PasteboardPort,
    ) -> SettlementAction {
        // Content marked concealed or transient is never restored
        // (decision 2026-10-13): the transcript stays instead, as eager text.
        let restorable = snapshot.filter(|snapshot| !snapshot.is_restoration_withheld());
        let result = match (action, restorable) {
            (SettlementAction::RestoreSnapshot, Some(snapshot)) => {
                pasteboard.restore_snapshot(snapshot, self.publication_count)
            }
            (SettlementAction::RestoreSnapshot, None)
            | (SettlementAction::MaterializeTranscript, _) => {
                return self.materialize(pasteboard);
            }
            (other, _) => return other,
        };
        written_or_left(action, result)
    }

    /// Makes the transcript eager: marked and current-host-only like the
    /// on-demand item, or ordinary plain text in copy mode (AC-013).
    fn materialize(&self, pasteboard: &mut dyn PasteboardPort) -> SettlementAction {
        let marking = match self.policy {
            ClipboardPolicy::DontModify => TranscriptMarking::Private,
            ClipboardPolicy::CopyToClipboard => TranscriptMarking::Plain,
        };
        let result =
            pasteboard.materialize_transcript(&self.transcript, self.publication_count, marking);
        written_or_left(SettlementAction::MaterializeTranscript, result)
    }
}

/// A guarded write that lost ownership, or failed, leaves the pasteboard as
/// it is: a failed write is never retried blindly.
fn written_or_left(
    action: SettlementAction,
    result: Result<GuardedWrite, PasteboardError>,
) -> SettlementAction {
    match result {
        Ok(GuardedWrite::Written) => action,
        Ok(GuardedWrite::OwnershipChanged) => SettlementAction::LeaveNewerContent,
        Err(error) => {
            log::warn!("[smart-paste] clipboard settlement failed: {error}");
            SettlementAction::LeaveNewerContent
        }
    }
}
