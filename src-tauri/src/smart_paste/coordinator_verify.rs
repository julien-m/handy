//! Experimental paste verification of the coordinator (AC-019): the native
//! side of [`super::verification`]. The focused field is read right before
//! the first Cmd+V, then polled after the read; one more Cmd+V is posted
//! only for web text (N28), while this transaction still owns the clipboard
//! (no rewrite), the destination is current and a last read right before it
//! still shows the field unchanged. Field values are compared, never logged.

use std::fmt;
use std::time::Duration;

use super::*;
use crate::smart_paste::ax_port::AxField;
use crate::smart_paste::direct::{MAX_FIELD_UTF16_UNITS, PER_CALL_TIMEOUT};
use crate::smart_paste::keyboard::{KeyboardError, LAYOUT_RESTORE_DELAY, LAYOUT_SETTLE_DELAY};
use crate::smart_paste::verification::{
    gate_retry, next_step, web_text_scope, FieldObservation, RetryGate, RetryPermit,
    UnreadableReason, VerifyFailure, VerifyStage, VerifyStep, VerifyVerdict, VerifyWindow,
    VERIFY_WINDOW, WEB_ANCESTRY_LIMITS,
};

// @spec FR-012: Paste verification and single retry — .specs/features/007-smart-paste/spec.md#fr-012

/// Longest time a verification adds to an insertion: the probe reads, the
/// web-area search (which may overrun its budget by one request), two
/// windows (each may overrun by one read), the last read before the retry,
/// and the repeated chord with its layout switch and restoration. Counted in
/// the actor's shutdown allowance.
pub fn verification_allowance() -> Duration {
    let probe = PER_CALL_TIMEOUT * 4;
    let ancestry = WEB_ANCESTRY_LIMITS.budget + WEB_ANCESTRY_LIMITS.per_call;
    let windows = (VERIFY_WINDOW + PER_CALL_TIMEOUT) * 2;
    let last_read = PER_CALL_TIMEOUT;
    probe + ancestry + windows + last_read + LAYOUT_SETTLE_DELAY + CHORD_HOLD + LAYOUT_RESTORE_DELAY
}

/// The destination's focused field and its value right before the first
/// Cmd+V.
pub(super) struct FieldProbe {
    field: Box<dyn AxField>,
    before: Vec<u16>,
}

impl fmt::Debug for FieldProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Field values are user content and must never reach a log.
        formatter.write_str("FieldProbe(<field value redacted>)")
    }
}

impl FieldProbe {
    fn observe(&mut self) -> FieldObservation {
        let read = self
            .field
            .value_utf16(PER_CALL_TIMEOUT, MAX_FIELD_UTF16_UNITS);
        FieldObservation::of(&self.before, read)
    }
}

impl SmartPasteCoordinator {
    /// Reads the focused field before the first Cmd+V when verification is
    /// on (with it off, nothing is probed and the error is ignored).
    /// Business rule (AC-019, N28): only a non-secure web-text field of the
    /// destination process with a readable value is verified; anything else
    /// keeps the current behavior, with the reason logged.
    pub(super) fn probe_field(
        &mut self,
        request: &InsertionRequest,
    ) -> Result<FieldProbe, UnreadableReason> {
        let no_field = UnreadableReason::NoReadableField;
        if !request.verifies_paste {
            return Err(no_field);
        }
        let pid = request.destination.pid.ok_or(no_field)?;
        let mut field = (self.ports.ax)
            .focused_field(pid, PER_CALL_TIMEOUT)
            .map_err(|_| no_field)?;
        let is_destination_field = field.pid().map_err(|_| no_field)? == pid;
        let is_secure = field.is_secure(PER_CALL_TIMEOUT).map_err(|_| no_field)?;
        if !is_destination_field || is_secure {
            return Err(no_field);
        }
        web_text_scope(field.is_in_web_area(WEB_ANCESTRY_LIMITS))?;
        let before = field
            .value_utf16(PER_CALL_TIMEOUT, MAX_FIELD_UTF16_UNITS)
            .map_err(|_| no_field)?;
        Ok(FieldProbe { field, before })
    }

    /// Replaces a "read observed" outcome by the verification verdict when
    /// verification is on; every other outcome is returned unchanged.
    pub(super) fn verify_read(
        &mut self,
        request: &InsertionRequest,
        read: PasteOutcome,
        probe: Result<FieldProbe, UnreadableReason>,
        key_down_at: Instant,
        events: &mut dyn ReadEvents,
    ) -> PasteOutcome {
        if !request.verifies_paste || read.kind != PasteOutcomeKind::ReadObserved {
            return read;
        }
        let verdict = match probe {
            Ok(mut probe) => self.run_verification(request, read, &mut probe, events),
            Err(reason) => VerifyVerdict::Unreadable(reason),
        };
        let elapsed = self
            .ports
            .clock
            .now()
            .saturating_duration_since(key_down_at);
        log_verdict(request.dictation_id.0, verdict, elapsed);
        PasteOutcome {
            kind: verdict.outcome_kind(),
            ..read
        }
    }

    /// Polls after the first Cmd+V, then at most once after the retry.
    fn run_verification(
        &mut self,
        request: &InsertionRequest,
        read: PasteOutcome,
        probe: &mut FieldProbe,
        events: &mut dyn ReadEvents,
    ) -> VerifyVerdict {
        let mut stage = VerifyStage::First;
        // Two passes at most: the first window, then the window after the
        // single retry (`next_step` never asks for a second retry).
        loop {
            let observation = self.watch_field(probe, events);
            let step = next_step(stage, observation, || {
                self.retry_permit(request, read.transaction_id)
            });
            match step {
                VerifyStep::Done(verdict) => return verdict,
                VerifyStep::Retry => {
                    if let Some(verdict) = self.retry_once(probe) {
                        return verdict;
                    }
                    stage = VerifyStage::AfterRetry;
                }
            }
        }
    }

    /// Reads the field one last time (N28), then posts the repeated Cmd+V
    /// when it is still unchanged; returns the verdict when the
    /// verification ends here instead.
    fn retry_once(&mut self, probe: &mut FieldProbe) -> Option<VerifyVerdict> {
        if let RetryGate::Skip(verdict) = gate_retry(probe.observe()) {
            return Some(verdict);
        }
        if let Err(error) = self.post_retry_chord() {
            log::warn!("[smart-paste] repeated paste chord failed: {error}");
            return Some(VerifyVerdict::Failed(VerifyFailure::RetryChordFailed));
        }
        None
    }

    /// Reads the field until it changes or the window ends; served reads
    /// keep feeding the lifecycle while waiting. Returns the last read.
    fn watch_field(
        &mut self,
        probe: &mut FieldProbe,
        events: &mut dyn ReadEvents,
    ) -> FieldObservation {
        let window = VerifyWindow::starting_at(self.ports.clock.now());
        // Exits on a change, or after the last read of the window.
        loop {
            let observation = probe.observe();
            if VerifyWindow::is_settled_by(observation) {
                return observation;
            }
            let Some(pause) = window.pause_before_next_read(self.ports.clock.now()) else {
                return observation;
            };
            let until = self.ports.clock.now() + pause;
            while let Some(event) = events.next_read(until) {
                let _ = self.record_pending_read(event, true);
            }
        }
    }

    /// The retry reuses the published item: it is allowed only while this
    /// transaction still owns the pasteboard (publication count unchanged),
    /// so the clipboard is never rewritten and nothing else is pasted.
    fn retry_permit(
        &mut self,
        request: &InsertionRequest,
        transaction: Option<TransactionId>,
    ) -> RetryPermit {
        let current = self.ports.pasteboard.change_count();
        let is_owned = self.pending.as_ref().is_some_and(|pending| {
            Some(pending.transaction) == transaction && current == Ok(pending.publication_count)
        });
        if !is_owned {
            return RetryPermit::ClipboardChanged;
        }
        if !self.ports.destination.is_current(&request.destination) {
            return RetryPermit::DestinationLost;
        }
        RetryPermit::Allowed
    }

    /// Posts the repeated Cmd+V with the same layout handling as the first
    /// (AC-011): switch, resolve V for the active layout, chord, restore.
    fn post_retry_chord(&mut self) -> Result<(), KeyboardError> {
        let clock = Arc::clone(&self.ports.clock);
        let switch = switch_for_paste(self.ports.input_sources.as_mut(), clock.as_ref());
        self.layout_to_restore = Some(switch);
        let posted = self
            .ports
            .keys
            .resolve_paste_key()
            .and_then(|key| self.ports.keys.post_paste_chord(key, CHORD_HOLD));
        self.restore_layout(clock.now());
        posted
    }
}

/// Logs the verdict line of AC-019 (debug) and a failure's cause (info):
/// only the dictation id, the verdict and the duration, never any content.
fn log_verdict(dictation_id: u64, verdict: VerifyVerdict, elapsed: Duration) {
    log::debug!(
        "[smart-paste] verify dictation {dictation_id}: {} in {}ms",
        verdict.label(),
        elapsed.as_millis()
    );
    match verdict {
        VerifyVerdict::Failed(failure) => {
            log::info!("[smart-paste] verify dictation {dictation_id} failed: {failure:?}");
        }
        VerifyVerdict::Unreadable(reason) => {
            log::info!("[smart-paste] verify dictation {dictation_id} unreadable: {reason:?}");
        }
        VerifyVerdict::Verified | VerifyVerdict::RetriedVerified => {}
    }
}
