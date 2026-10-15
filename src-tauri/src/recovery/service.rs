//! `RecoveryService`: 010's `OutcomeConsumer` and `RecoveryPort` for 006, and the actions of
//! the recovery window. It turns the manager's answers into surface, slot, clipboard and
//! fallback effects (plan "Presentation hand-off", "Copy, close and settings").
use super::classifier::{classify, FocusClassification, FocusedElementPort};
use super::decision::decide_recovery;
use super::instance::Admission;
use super::manager::{CopyEnd, DragEnd, LateReadEnd, RecoveryManager, Resolved, Retired};
use super::ports::{
    CopyError, CopyPort, FallbackPort, RecoverySurfacePort, SessionFacts, TimerPort,
    TranscriptSource,
};
use super::types::{BubbleId, BubbleSpec, RecoveryBubbleView, RecoveryPlatform};
use super::verdict::{classification_target, verdict_of};
use crate::destination::context::{
    DestinationContext, DictationId, PreInsertionVerdict, RecoveryPort,
};
use crate::destination::outcome::InsertionOutcome;
use crate::destination::recovery::{ConsumerAck, OutcomeConsumer, PresentationReceipt};
use crate::destination::workspace::Clock;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::Duration;

/// A surface silent this long after `consume` is hung, not slow: its window is destroyed and
/// the failure falls back to `paste-error` (plan "Presentation hand-off").
pub const PRESENTATION_HANG_LIMIT: Duration = Duration::from_secs(10);

/// Every port of the service.
pub struct RecoveryPorts {
    pub surface: Arc<dyn RecoverySurfacePort>,
    pub transcripts: Arc<dyn TranscriptSource>,
    pub facts: Arc<dyn SessionFacts>,
    pub copy: Arc<dyn CopyPort>,
    pub timers: Arc<dyn TimerPort>,
    pub fallback: Arc<dyn FallbackPort>,
    pub focus: Mutex<Box<dyn FocusedElementPort>>,
    pub clock: Arc<dyn Clock>,
}

/// The recovery bubble of a gate-open build.
pub struct RecoveryService {
    manager: RecoveryManager,
    ports: RecoveryPorts,
    platform: RecoveryPlatform,
    me: Weak<RecoveryService>,
}

impl RecoveryService {
    /// A service for `platform` over `ports`.
    pub fn new(ports: RecoveryPorts, platform: RecoveryPlatform) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            manager: RecoveryManager::new(platform),
            ports,
            platform,
            me: me.clone(),
        })
    }
    /// The projection of the current bubble, for a recovery window that (re)loads.
    pub fn view(&self) -> Option<RecoveryBubbleView> {
        self.manager.view()
    }

    /// A dictation is starting: freeze "Show the bubble when no text field is focused" for it.
    /// The bubble stays until the recording really starts.
    pub fn dictation_starting(&self, setting_enabled: bool) {
        self.manager.freeze_setting(setting_enabled);
    }
    // @spec AC-005: A new dictation closes the bubble at once — .specs/features/010-recovery-bubble/spec.md#ac-005
    /// The recording of `dictation_id` started: close the bubble at once and bind the frozen
    /// setting to this dictation. A refused start never reaches here, so its bubble stays.
    pub fn dictation_started(&self, dictation_id: DictationId) {
        if let Some(retired) = self.manager.new_dictation() {
            self.retire(retired);
        }
        self.manager.bind_setting(dictation_id);
    }

    /// The recovery view reported its first render of `bubble_id`: `Presented`.
    pub fn rendered(&self, bubble_id: BubbleId) {
        let Some(resolved) = self.manager.rendered(bubble_id) else {
            return;
        };
        self.release(&resolved);
        let now = self.ports.clock.now();
        let since = |start: std::time::Instant| now.saturating_duration_since(start);
        log::debug!(
            "Recovery bubble {} presented {:?} after the recovery decision ({:?} after consume)",
            bubble_id.0,
            resolved.decided_at.map(since),
            since(resolved.consumed_at)
        );
    }
    /// The bubble `bubble_id` cannot be presented (confirmed): `paste-error` replaces it.
    pub fn presentation_failed(&self, bubble_id: BubbleId) {
        let Some(resolved) = self.manager.unpresentable(bubble_id) else {
            return;
        };
        log::warn!("Recovery bubble {} could not be presented", bubble_id.0);
        self.ports.surface.hide();
        self.release(&resolved);
        // 006 reports an answer it received; once its guard expired, 010 reports it.
        if !resolved.waiter_answered {
            self.ports.fallback.paste_error(resolved.history);
        }
    }
    /// The hang limit of `bubble_id` passed: destroy a window that never answered.
    pub fn hang_check(&self, bubble_id: BubbleId) {
        if self.manager.is_presenting(bubble_id) {
            log::warn!("Recovery surface silent for {PRESENTATION_HANG_LIMIT:?}: destroyed");
            self.ports.surface.destroy(bubble_id);
        }
    }
    /// The window of a hung presentation is destroyed: confirmed `Unpresentable`.
    pub fn window_destroyed(&self, bubble_id: BubbleId) {
        self.presentation_failed(bubble_id);
    }

    // @spec FR-004: Close never touches history — .specs/features/010-recovery-bubble/spec.md#fr-004
    /// Close `bubble_id` (history untouched).
    pub fn close(&self, bubble_id: BubbleId) {
        if let Some(retired) = self.manager.close(bubble_id) {
            self.retire(retired);
        }
    }
    // @spec AC-005: Copy writes the full transcript, then closes — .specs/features/010-recovery-bubble/spec.md#ac-005
    /// Deliberate Copy of `bubble_id`; blocks on the write, so never call it on the main
    /// thread. Success closes the bubble; a failure keeps it open and unchanged.
    pub fn copy(&self, bubble_id: BubbleId, generation: u64) -> Result<(), CopyError> {
        let Some((ticket, transcript)) = self.manager.begin_copy(bubble_id, generation) else {
            return Ok(());
        };
        let written = self.ports.copy.write(&transcript);
        if let Err(error) = written {
            log::warn!("Recovery Copy failed ({error:?}); the bubble stays open");
        }
        if let CopyEnd::Closed(retired) = self.manager.finish_copy(ticket, written.is_ok()) {
            self.retire(retired);
        }
        written
    }

    // @spec FR-003: Native drag lifecycle by bubble identity — .specs/features/010-recovery-bubble/spec.md#fr-003
    /// Start a native drag of `bubble_id` (macOS only).
    pub fn begin_drag(&self, bubble_id: BubbleId) {
        if let Some(transcript) = self.manager.begin_drag(bubble_id) {
            self.ports.surface.begin_drag(bubble_id, transcript);
        }
    }
    /// The drag of `bubble_id` could not start: the bubble stays visible and unchanged.
    pub fn drag_failed(&self, bubble_id: BubbleId) {
        if self.manager.drag_failed(bubble_id) {
            log::warn!("Recovery drag could not start for bubble {}", bubble_id.0);
        }
    }
    /// The drag of `bubble_id` ended; an accepted drop proves protocol acceptance only.
    pub fn drag_ended(&self, bubble_id: BubbleId, accepted: bool) {
        match self.manager.drag_ended(bubble_id, accepted) {
            DragEnd::Accepted(retired) => {
                log::info!("Recovery bubble {}: drop accepted", bubble_id.0);
                self.retire(retired);
            }
            DragEnd::Refused(view) => self.ports.surface.show_again(view),
            DragEnd::Ignored => {}
        }
    }

    // @spec AC-001: Late read updates the same bubble — .specs/features/010-recovery-bubble/spec.md#ac-001
    /// 007 saw a read after "not read" for `dictation_id` and its transaction.
    pub fn late_read(&self, dictation_id: DictationId, transaction_id: u64) {
        let transaction = transaction_id.to_string();
        if let LateReadEnd::Updated(view) = self.manager.late_read(dictation_id, &transaction) {
            self.ports.surface.update(view);
        }
    }

    fn admit(&self, outcome: &InsertionOutcome) -> Option<PresentationReceipt> {
        let spec = decide_recovery(
            outcome.status,
            outcome.reason,
            outcome.method_outcome,
            self.platform,
        )?;
        if self.manager.is_stale(outcome.dictation_id) {
            return None;
        }
        // The full copy exists before anything else; an expired reference admits nothing.
        let transcript = self.ports.transcripts.read(outcome.transcript_ref?)?;
        let admission = self.admission(outcome, spec, transcript);
        let (receipt, view, replaced) = self.manager.admit(admission)?;
        if let Some(reference) = replaced.and_then(|retired| retired.release) {
            self.ports.transcripts.release(reference);
        }
        let bubble_id = view.bubble_id;
        self.ports.surface.present(view);
        self.schedule_hang_check(bubble_id);
        Some(receipt)
    }
    fn admission(
        &self,
        outcome: &InsertionOutcome,
        spec: BubbleSpec,
        transcript: String,
    ) -> Admission {
        let key = self
            .manager
            .next_key(outcome.dictation_id, outcome.destination_generation);
        Admission {
            key,
            spec,
            transcript,
            transaction_id: outcome.transaction_id.clone(),
            history: outcome.history_status,
            app_name: self.ports.facts.app_name(outcome),
            transcript_ref: outcome.transcript_ref,
            decided_at: self.ports.facts.decided_at(outcome),
            consumed_at: self.ports.clock.now(),
        }
    }
    fn schedule_hang_check(&self, bubble_id: BubbleId) {
        let me = self.me.clone();
        let task = Box::new(move || {
            if let Some(service) = me.upgrade() {
                service.hang_check(bubble_id);
            }
        });
        self.ports.timers.after(PRESENTATION_HANG_LIMIT, task);
    }
    fn retire(&self, retired: Retired) {
        if retired.shown {
            self.ports.surface.hide();
        }
        if let Some(reference) = retired.release {
            self.ports.transcripts.release(reference);
        }
    }
    fn release(&self, resolved: &Resolved) {
        if let Some(reference) = resolved.release {
            self.ports.transcripts.release(reference);
        }
    }
    fn classify_focus(&self, pid: u32) -> FocusClassification {
        let started = self.ports.clock.now();
        let mut port = self
            .ports
            .focus
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let classification = classify(port.as_mut(), pid, self.ports.clock.as_ref());
        let elapsed = self.ports.clock.now().saturating_duration_since(started);
        log::debug!("Focused element classified {classification:?} in {elapsed:?}");
        classification
    }
}

// @spec FR-002: Take over the reason presentation from 006 — .specs/features/010-recovery-bubble/spec.md#fr-002
impl OutcomeConsumer for RecoveryService {
    fn consume(&self, outcome: &InsertionOutcome) -> ConsumerAck {
        match self.admit(outcome) {
            Some(receipt) => ConsumerAck::PresentationPending(receipt),
            None => ConsumerAck::Received,
        }
    }
}

// @spec FR-001: Pre-insertion verdict within the guard check — .specs/features/010-recovery-bubble/spec.md#fr-001
impl RecoveryPort for RecoveryService {
    fn before_insertion(&self, context: &DestinationContext) -> PreInsertionVerdict {
        let setting_enabled = self.manager.setting_for(context.dictation_id);
        let target = context.target.as_ref();
        match classification_target(self.platform, setting_enabled, target) {
            Some(pid) => verdict_of(self.classify_focus(pid)),
            None => PreInsertionVerdict::Insert,
        }
    }
}
