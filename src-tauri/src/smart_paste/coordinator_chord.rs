//! Keystroke, first-read wait and the pending lifecycle of the coordinator.

use super::*;

impl SmartPasteCoordinator {
    /// Switches the layout, waits `paste_delay`, re-checks, posts Cmd+V and
    /// waits for the terminal outcome of the published transaction.
    pub(super) fn paste_published(
        &mut self,
        request: &InsertionRequest,
        transaction: TransactionId,
        mut guard: Option<FallbackGuard>,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        let clock = Arc::clone(&self.ports.clock);
        let switch = switch_for_paste(self.ports.input_sources.as_mut(), clock.as_ref());
        clock.sleep(request.paste_delay);
        let cancelled = self.cancel_reason(request, guard.as_mut());
        let key = match cancelled {
            Some(_) => None,
            None => self.ports.keys.resolve_paste_key().ok(),
        };
        let Some(key) = key else {
            self.restore_layout(&switch, clock.now());
            self.cancel_publication();
            let (path, kind) = cancelled.unwrap_or((
                InsertionPath::ClipboardPaste,
                PasteOutcomeKind::InsertionFailed,
            ));
            return outcome(request, Some(transaction), path, kind);
        };
        self.post_chord(request, transaction, key, &switch, events)
    }

    fn post_chord(
        &mut self,
        request: &InsertionRequest,
        transaction: TransactionId,
        key: PasteKey,
        switch: &LayoutSwitch,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        // The keystroke instant is taken immediately before the first
        // key-down; reads served while the chord is held qualify (AC-008).
        let key_down_at = self.ports.clock.now();
        if let Some(pending) = self.pending.as_mut() {
            pending.lifecycle.record_key_down(key_down_at);
        }
        let posted = self.ports.keys.post_paste_chord(key, CHORD_HOLD);
        self.restore_layout(switch, self.ports.clock.now());
        if let Err(error) = posted {
            log::warn!("[smart-paste] paste chord failed: {error}");
            let kind = PasteOutcomeKind::InsertionFailed;
            return outcome(
                request,
                Some(transaction),
                InsertionPath::ClipboardPaste,
                kind,
            );
        }
        self.await_first_read(request, transaction, events)
    }

    /// Reasons not to post the chord: the destination is gone (failure
    /// before any key effect) or the earlier AX write showed up late (the
    /// text may already be there: never insert it twice).
    fn cancel_reason(
        &mut self,
        request: &InsertionRequest,
        guard: Option<&mut FallbackGuard>,
    ) -> Option<(InsertionPath, PasteOutcomeKind)> {
        if !self.ports.destination.is_current(&request.destination) {
            return Some((
                InsertionPath::ClipboardPaste,
                PasteOutcomeKind::InsertionFailed,
            ));
        }
        match guard.map(FallbackGuard::check) {
            Some(GuardVerdict::StillUnchanged) | None => None,
            Some(_) => Some((
                InsertionPath::DirectAccessibility,
                PasteOutcomeKind::Uncertain,
            )),
        }
    }

    fn restore_layout(&mut self, switch: &LayoutSwitch, last_key_up: Instant) {
        let clock = Arc::clone(&self.ports.clock);
        let input_sources = self.ports.input_sources.as_mut();
        if let Err(error) = restore_after_chord(input_sources, clock.as_ref(), switch, last_key_up)
        {
            log::error!("[smart-paste] previous input source could not be restored: {error}");
        }
    }

    /// No keystroke was posted after publication: no read can confirm
    /// anything, so the snapshot is discarded and the transcript made eager.
    fn cancel_publication(&mut self) {
        if let Some(mut pending) = self.pending.take() {
            pending.settle(
                SettlementTrigger::KeystrokeCancelled,
                self.ports.pasteboard.as_mut(),
            );
        }
    }

    fn await_first_read(
        &mut self,
        request: &InsertionRequest,
        transaction: TransactionId,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        let done = |kind| {
            outcome(
                request,
                Some(transaction),
                InsertionPath::ClipboardPaste,
                kind,
            )
        };
        // Each pass handles one read (in timestamp order) or the deadline,
        // until the first qualifying read or "not read" decides the outcome.
        while let Some(deadline) = self.next_deadline() {
            let update = match events.next_read(deadline) {
                Some(event) => self.record_pending_read(event, true),
                None => self.expire_no_read(true),
            };
            match update {
                Some(update) if update.declared_not_read => return done(PasteOutcomeKind::NotRead),
                Some(update) if update.classification == ReadClassification::FirstQualifying => {
                    return done(PasteOutcomeKind::ReadObserved)
                }
                _ => {}
            }
        }
        done(PasteOutcomeKind::NotRead)
    }

    fn expire_no_read(&mut self, reports_late_reads: bool) -> Option<ReadUpdate> {
        let now = self.ports.clock.now();
        let pending = self.pending.as_mut()?;
        let declared = pending.lifecycle.poll_deadline(now) == DeadlineOutcome::DeclaredNotRead;
        if declared {
            self.declare_not_read(reports_late_reads);
        }
        Some(ReadUpdate {
            declared_not_read: declared,
            classification: ReadClassification::BeforeKeyDown,
        })
    }

    /// Records a read of the current item; reads of older items never
    /// affect it. Emits the late-read diagnostic once after "not read".
    fn record_pending_read(
        &mut self,
        event: ReadEvent,
        reports_late_reads: bool,
    ) -> Option<ReadUpdate> {
        let pending = self.pending.as_mut()?;
        if pending.transaction != event.transaction {
            return None;
        }
        let update = pending.lifecycle.record_read(event.at);
        if update.declared_not_read {
            self.declare_not_read(reports_late_reads);
        }
        let is_first_late_read = update.classification
            == ReadClassification::LateAfterNotRead {
                first_late_read: true,
            };
        if is_first_late_read {
            self.report_late_read();
        }
        Some(update)
    }

    fn report_late_read(&mut self) {
        let Some(pending) = self
            .pending
            .as_ref()
            .filter(|pending| pending.reports_late_reads)
        else {
            return;
        };
        let diagnostic = PasteDiagnostic {
            dictation_id: pending.dictation_id,
            transaction_id: pending.transaction,
            late_read_observed: true,
        };
        self.ports.diagnostics.late_read(diagnostic);
    }

    /// "Not read": the snapshot is discarded for good. Without copy mode the
    /// on-demand item stays and keeps serving late reads; with copy mode the
    /// transcript becomes eager plain text and the transaction ends.
    fn declare_not_read(&mut self, reports_late_reads: bool) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        pending.reports_late_reads = reports_late_reads;
        let action = pending.settle(
            SettlementTrigger::NotReadDeclared,
            self.ports.pasteboard.as_mut(),
        );
        if action != SettlementAction::KeepLazyTranscript {
            self.pending = None;
        }
    }

    /// Handles a read served while no insertion is running.
    pub fn handle_read(&mut self, event: ReadEvent) {
        let _ = self.record_pending_read(event, false);
    }

    /// Applies the pending transaction's deadline if it is reached.
    pub fn handle_deadline(&mut self) {
        let now = self.ports.clock.now();
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        match pending.lifecycle.poll_deadline(now) {
            DeadlineOutcome::SettlementDue => {
                pending.settle(
                    SettlementTrigger::ReadsWentQuiet,
                    self.ports.pasteboard.as_mut(),
                );
                self.pending = None;
            }
            DeadlineOutcome::DeclaredNotRead => self.declare_not_read(false),
            DeadlineOutcome::Pending => {}
        }
    }

    /// Normal exit: restores only when a qualifying read happened and its
    /// quiet period (or cap) elapsed; otherwise makes the transcript eager.
    /// Writes only while the publication count still holds.
    pub fn shutdown(&mut self) {
        let now = self.ports.clock.now();
        if let Some(mut pending) = self.pending.take() {
            let is_restoration_due = pending.lifecycle.is_restoration_due(now);
            let trigger = SettlementTrigger::Exit { is_restoration_due };
            pending.settle(trigger, self.ports.pasteboard.as_mut());
        }
    }
}
