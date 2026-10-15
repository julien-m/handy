//! Serialized smart-paste coordinator: the insertion chain of AC-018 and the
//! clipboard lifecycle of AC-009, over injected native ports.
//!
//! Chain: permission and destination preflight; dedicated strategy for
//! remote apps (AC-010); otherwise verified AX insertion, then the clipboard
//! paste. The clipboard paste snapshots, publishes an on-demand item,
//! switches the layout, waits `paste_delay`, re-checks the destination and
//! any late AX write, records the key-down instant, posts Cmd+V, restores the
//! layout, then waits for the first qualifying read or the no-read deadline.
//! The terminal outcome is returned once; the restoration lifecycle then
//! continues through [`SmartPasteCoordinator::handle_read`],
//! [`SmartPasteCoordinator::handle_deadline`] and
//! [`SmartPasteCoordinator::shutdown`]. Every method takes `&mut self`: the
//! owner (one actor thread) serializes clipboard ownership, chords and
//! input-source cleanup.

use std::sync::Arc;
use std::time::Instant;

use super::ax_port::AxTextPort;
use super::clock::Clock;
use super::direct::{
    attempt_direct_insertion, DirectAttempt, FallbackGuard, GuardVerdict, SkipReason,
};
use super::ids::{TransactionId, Transcript};
use super::key_events::{KeyEventPort, PasteKey, CHORD_HOLD};
use super::keyboard::{restore_after_chord, switch_for_paste, InputSourcePort, LayoutSwitch};
use super::lifecycle::{DeadlineOutcome, ReadClassification, ReadLifecycle, ReadUpdate};
use super::outcome::{InsertionPath, PasteDiagnostic, PasteOutcome, PasteOutcomeKind};
use super::pasteboard_port::{PasteboardError, PasteboardPort, ReadEvent};
use super::pending::PendingClipboard;
use super::request::{DestinationGuard, DiagnosticSink, InsertionRequest, ReadEvents};
use super::settlement::{
    snapshot_source, ClipboardPolicy, SettlementAction, SettlementTrigger, SnapshotSource,
};
use super::snapshot::{PasteboardSnapshot, SnapshotBudget, SnapshotError};
use super::strategy::InsertionStrategy;
use super::timing::ClipboardTimings;

// @spec FR-006: Async restore, copy, exit and succession — .specs/features/007-smart-paste/spec.md#fr-006
// @spec FR-008: Terminal and late event delivery — .specs/features/007-smart-paste/spec.md#fr-008
// @spec FR-011: AX, clipboard, recovery ordering — .specs/features/007-smart-paste/spec.md#fr-011

/// Native ports used by the coordinator.
pub struct SmartPastePorts {
    /// Monotonic time for every timer.
    pub clock: Arc<dyn Clock>,
    /// Accessibility (permission and direct insertion).
    pub ax: Box<dyn AxTextPort>,
    /// Pasteboard snapshot, publication and guarded writes.
    pub pasteboard: Box<dyn PasteboardPort>,
    /// Text Input Sources.
    pub input_sources: Box<dyn InputSourcePort>,
    /// Key events.
    pub keys: Box<dyn KeyEventPort>,
    /// Destination revalidation (006).
    pub destination: Box<dyn DestinationGuard>,
    /// Late-read diagnostics (010).
    pub diagnostics: Box<dyn DiagnosticSink>,
}

/// Serialized coordinator owning the pending clipboard transaction.
pub struct SmartPasteCoordinator {
    ports: SmartPastePorts,
    next_transaction: u64,
    pending: Option<PendingClipboard>,
    /// Layout switched for the chord in progress, restored exactly once.
    layout_to_restore: Option<LayoutSwitch>,
}

type SnapshotChoice = (Arc<PasteboardSnapshot>, i64);

impl SmartPasteCoordinator {
    /// Creates a coordinator with no pending transaction.
    pub fn new(ports: SmartPastePorts) -> Self {
        Self {
            ports,
            next_transaction: 1,
            pending: None,
            layout_to_restore: None,
        }
    }

    /// Transaction still owning the pasteboard, if any.
    pub fn pending_transaction(&self) -> Option<TransactionId> {
        self.pending.as_ref().map(|pending| pending.transaction)
    }

    /// Next instant at which [`Self::handle_deadline`] has work to do.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.pending.as_ref()?.lifecycle.next_deadline()
    }

    /// 010's deliberate Copy: `transcript` as unmarked plain text (the primitive of the "copy
    /// to clipboard" setting). It changes the `changeCount` even for identical text, so the
    /// pending restoration, guarded by its publication count, abandons on its own.
    ///
    /// # Errors
    /// The pasteboard error; the bubble then stays open.
    pub fn deliberate_copy(&mut self, transcript: &Transcript) -> Result<(), PasteboardError> {
        self.ports.pasteboard.write_plain_text(transcript)
    }

    /// Runs one insertion and returns its single terminal outcome. Reads are
    /// pulled from `events` until the outcome is known.
    pub fn insert(
        &mut self,
        request: &InsertionRequest,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        let strategy = InsertionStrategy::for_bundle(request.destination.bundle_id.as_deref());
        let outcome = self.run_strategy(request, strategy, events);
        log::info!(
            "[smart-paste] dictation {}: {:?} via {:?}",
            request.dictation_id.0,
            outcome.kind,
            outcome.path
        );
        outcome
    }

    fn run_strategy(
        &mut self,
        request: &InsertionRequest,
        strategy: InsertionStrategy,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        // Permission is checked before the clipboard is touched (AC-012).
        if strategy.requires_accessibility_permission() && !self.ports.ax.is_trusted() {
            return outcome(
                request,
                None,
                InsertionPath::NotAttempted,
                PasteOutcomeKind::PermissionMissing,
            );
        }
        if !self.ports.destination.is_current(&request.destination) {
            return outcome(
                request,
                None,
                InsertionPath::NotAttempted,
                PasteOutcomeKind::InsertionFailed,
            );
        }
        match strategy {
            InsertionStrategy::UnicodeTyping => self.type_unicode(request),
            InsertionStrategy::ClipboardHandOver => self.hand_over(request),
            InsertionStrategy::DirectThenClipboard(timings) => {
                self.direct_then_clipboard(request, timings, events)
            }
        }
    }

    fn type_unicode(&mut self, request: &InsertionRequest) -> PasteOutcome {
        let kind = match self.ports.keys.type_unicode(request.transcript.as_str()) {
            Ok(()) => {
                self.copy_if_requested(request);
                PasteOutcomeKind::Typed
            }
            Err(_) => PasteOutcomeKind::InsertionFailed,
        };
        outcome(request, None, InsertionPath::UnicodeTyping, kind)
    }

    fn hand_over(&mut self, request: &InsertionRequest) -> PasteOutcome {
        // The hand-over replaces the clipboard on purpose: an older pending
        // restoration is abandoned without writing.
        self.supersede_pending();
        let kind = match self.ports.pasteboard.write_plain_text(&request.transcript) {
            Ok(()) => PasteOutcomeKind::ClipboardHandedOver,
            Err(_) => PasteOutcomeKind::InsertionFailed,
        };
        outcome(request, None, InsertionPath::ClipboardHandOver, kind)
    }

    /// Copy-to-clipboard after an insertion that did not use the clipboard
    /// (AC-013, AC-010): a deliberate plain-text write.
    fn copy_if_requested(&mut self, request: &InsertionRequest) {
        if request.clipboard_policy == ClipboardPolicy::CopyToClipboard {
            if let Err(error) = self.ports.pasteboard.write_plain_text(&request.transcript) {
                log::warn!("[smart-paste] copy to clipboard failed: {error}");
            }
        }
    }

    fn direct_then_clipboard(
        &mut self,
        request: &InsertionRequest,
        timings: ClipboardTimings,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        // The snapshot helper starts while AX runs; it reads nothing until
        // a capture is actually needed.
        let budget = SnapshotBudget::default();
        self.ports.pasteboard.prepare_capture(budget);
        match self.attempt_direct(request) {
            DirectAttempt::Verified => self.verified(request),
            DirectAttempt::Uncertain(reason) => {
                log::info!("[smart-paste] direct insertion uncertain: {reason:?}");
                let path = InsertionPath::DirectAccessibility;
                outcome(request, None, path, PasteOutcomeKind::Uncertain)
            }
            DirectAttempt::Skipped(reason) => {
                log::info!("[smart-paste] direct insertion skipped: {reason:?}");
                self.clipboard_paste(request, timings, None, events)
            }
            DirectAttempt::Unchanged(guard) => {
                self.clipboard_paste(request, timings, Some(guard), events)
            }
        }
    }

    /// Business rule (006 contract): without a named target process, direct
    /// insertion cannot check the focused field's owner, so it is skipped.
    fn attempt_direct(&mut self, request: &InsertionRequest) -> DirectAttempt {
        let Some(pid) = request.destination.pid else {
            return DirectAttempt::Skipped(SkipReason::NoTarget);
        };
        let transcript = request.transcript.to_utf16();
        let clock = Arc::clone(&self.ports.clock);
        attempt_direct_insertion(clock.as_ref(), self.ports.ax.as_mut(), pid, &transcript)
    }

    fn verified(&mut self, request: &InsertionRequest) -> PasteOutcome {
        self.copy_if_requested(request);
        outcome(
            request,
            None,
            InsertionPath::DirectAccessibility,
            PasteOutcomeKind::InsertedVerified,
        )
    }

    fn clipboard_paste(
        &mut self,
        request: &InsertionRequest,
        timings: ClipboardTimings,
        mut guard: Option<FallbackGuard>,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        let transaction = TransactionId(self.next_transaction);
        self.next_transaction += 1;
        // Nothing was published, so no transaction id is reported.
        let failed = |error: PasteboardError| {
            log::warn!("[smart-paste] clipboard transaction could not start: {error}");
            let path = InsertionPath::ClipboardPaste;
            outcome(request, None, path, PasteOutcomeKind::InsertionFailed)
        };
        let choice = match self.snapshot_for_new_transaction() {
            Ok(choice) => choice,
            Err(error) => return failed(error),
        };
        if let Some(late) = self.late_write_outcome(request, guard.as_mut()) {
            return late;
        }
        if let Err(error) = self.publish(request, timings, transaction, choice) {
            return failed(error);
        }
        self.paste_published(request, transaction, guard, events)
    }

    /// Final check right before publication: a late AX write that shows up
    /// during the snapshot keeps the clipboard untouched.
    fn late_write_outcome(
        &mut self,
        request: &InsertionRequest,
        guard: Option<&mut FallbackGuard>,
    ) -> Option<PasteOutcome> {
        match guard.map(FallbackGuard::check)? {
            GuardVerdict::AppliedLate => Some(self.verified(request)),
            GuardVerdict::ChangedUnexpectedly | GuardVerdict::Unreadable => Some(outcome(
                request,
                None,
                InsertionPath::DirectAccessibility,
                PasteOutcomeKind::Uncertain,
            )),
            GuardVerdict::StillUnchanged => None,
        }
    }

    /// Publishes and installs the new pending transaction. The previous
    /// pending one is superseded only once publication succeeded, so a
    /// failed start leaves its restoration intact.
    fn publish(
        &mut self,
        request: &InsertionRequest,
        timings: ClipboardTimings,
        transaction: TransactionId,
        (snapshot, source_count): SnapshotChoice,
    ) -> Result<(), PasteboardError> {
        if !snapshot.is_restorable() {
            // A nonempty clipboard with nothing saved must not be treated as
            // empty: refuse to start rather than lose it.
            return Err(PasteboardError::SnapshotFailed(SnapshotError::NothingSaved));
        }
        let publication_count = self.ports.pasteboard.publish_lazy_transcript(
            transaction,
            &request.transcript,
            source_count,
        )?;
        self.supersede_pending();
        self.pending = Some(PendingClipboard {
            dictation_id: request.dictation_id,
            transaction,
            lifecycle: ReadLifecycle::new(timings),
            snapshot: Some(snapshot),
            publication_count,
            transcript: request.transcript.clone(),
            policy: request.clipboard_policy,
            reports_late_reads: false,
        });
        Ok(())
    }

    fn snapshot_for_new_transaction(&mut self) -> Result<SnapshotChoice, PasteboardError> {
        let current = self.ports.pasteboard.change_count();
        let view = self.pending.as_ref().map(|pending| pending.view(current));
        let reusable = match (snapshot_source(view), self.pending.as_ref(), current) {
            (SnapshotSource::CarryPendingSnapshot, Some(pending), Ok(count)) => {
                pending.snapshot.clone().map(|snapshot| (snapshot, count))
            }
            // Handy's own earlier transcript item is marked transient and
            // concealed, so it is never restored either.
            (SnapshotSource::PendingTranscript, Some(_), Ok(count)) => {
                Some((Arc::new(PasteboardSnapshot::withheld(count)), count))
            }
            _ => None,
        };
        if let Some(choice) = reusable {
            return Ok(choice);
        }
        let snapshot = self
            .ports
            .pasteboard
            .capture_snapshot(SnapshotBudget::default())?;
        log_snapshot(&snapshot);
        let source_count = snapshot.source_change_count();
        let snapshot = snapshot.withhold_if_privacy_marked();
        if snapshot.is_restoration_withheld() {
            log::info!("[smart-paste] previous clipboard is concealed or transient: not restored");
        }
        Ok((Arc::new(snapshot), source_count))
    }

    fn supersede_pending(&mut self) {
        // No intermediate restoration: the newer transaction (or write)
        // takes over, and the older item's provider is released.
        if let Some(previous) = self.pending.take() {
            self.ports
                .pasteboard
                .release_transaction(previous.transaction);
        }
    }
}

/// Debug-logs what the snapshot could not keep, by type name only (spec
/// Edge Cases), so a partial restoration is visible without any content.
fn log_snapshot(snapshot: &PasteboardSnapshot) {
    for skipped in snapshot.skipped() {
        log::debug!(
            "[smart-paste] snapshot skipped type {} of item {}: {:?}",
            skipped.uti,
            skipped.item_index,
            skipped.cause
        );
    }
    let omitted = snapshot.omitted();
    log::debug!(
        "[smart-paste] snapshot saved {} types ({} bytes), skipped {}, omitted {} items and {} types beyond limits",
        snapshot.saved_type_count(),
        snapshot.saved_bytes(),
        snapshot.skipped().len(),
        omitted.items,
        omitted.types
    );
}

fn outcome(
    request: &InsertionRequest,
    transaction_id: Option<TransactionId>,
    path: InsertionPath,
    kind: PasteOutcomeKind,
) -> PasteOutcome {
    PasteOutcome {
        dictation_id: request.dictation_id,
        transaction_id,
        destination_generation: request.destination.generation,
        path,
        kind,
    }
}

#[path = "coordinator_chord.rs"]
mod chord;
