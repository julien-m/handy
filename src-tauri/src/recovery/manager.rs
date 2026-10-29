//! `RecoveryManager`: the one current bubble and its lifecycle (plan "State Model"). Pure
//! state under one lock, no platform call: the service turns its answers into surface,
//! clipboard and slot effects. Every action names the exact bubble; a closed bubble's identity
//! is never reused, so a late callback of a closed bubble cannot affect a newer one.
use super::instance::{Admission, Instance};
use super::types::{BubbleId, BubbleKey, DragResult, Phase, RecoveryBubbleView, RecoveryPlatform};
use crate::destination::context::DictationId;
use crate::destination::outcome::{HistoryStatus, TranscriptRef};
use crate::destination::recovery::{PresentationReceipt, PresentationResult};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

/// A bubble that left the manager: what the service must hide and release.
#[derive(Debug, Eq, PartialEq)]
pub struct Retired {
    pub key: BubbleKey,
    /// The surface showed it (or was asked to): hide the window.
    pub shown: bool,
    /// 006's slot entry to release, if still held.
    pub release: Option<TranscriptRef>,
}
/// A presentation that resolved.
#[derive(Debug, Eq, PartialEq)]
pub struct Resolved {
    pub key: BubbleKey,
    pub release: Option<TranscriptRef>,
    /// 006 was still waiting for the answer.
    pub waiter_answered: bool,
    pub history: HistoryStatus,
    pub decided_at: Option<Instant>,
    pub consumed_at: Instant,
}
/// Outcome of a Copy completion.
#[derive(Debug, Eq, PartialEq)]
pub enum CopyEnd {
    Closed(Retired),
    /// The write failed: the bubble stays open and unchanged.
    KeptOpen,
    /// The bubble was closed or replaced meanwhile: the completion is ignored.
    Ignored,
}
/// Outcome of a drag end.
#[derive(Debug, Eq, PartialEq)]
pub enum DragEnd {
    Accepted(Retired),
    /// Released where no target took the text: show the same bubble again.
    Refused(RecoveryBubbleView),
    /// Cancelled by Escape: the bubble closes like Close, history untouched (N25).
    Dismissed(Retired),
    Ignored,
}
/// Outcome of a late-read diagnostic.
#[derive(Debug, Eq, PartialEq)]
pub enum LateReadEnd {
    /// The matching bubble now says the text may have been pasted after all.
    Updated(RecoveryBubbleView),
    /// Kept until its outcome is admitted.
    Buffered,
    Ignored,
}
/// A Copy admitted for one bubble.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyTicket {
    pub bubble_id: BubbleId,
    seq: u64,
}

#[derive(Default)]
struct State {
    last_bubble: u64,
    current: Option<Instance>,
    newest_dictation: Option<DictationId>,
    /// A late read received before its outcome was admitted (one entry).
    buffered_late_read: Option<(DictationId, String)>,
    /// "Show the bubble when no text field is focused" read at the accepted dictation start.
    frozen_setting: Option<FrozenSetting>,
}
#[derive(Clone, Copy)]
struct FrozenSetting {
    dictation_id: Option<DictationId>,
    enabled: bool,
}

/// The current recovery bubble of this process.
pub struct RecoveryManager {
    state: Mutex<State>,
    platform: RecoveryPlatform,
}

impl RecoveryManager {
    /// An empty manager for `platform`.
    pub fn new(platform: RecoveryPlatform) -> Self {
        Self {
            state: Mutex::new(State::default()),
            platform,
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
    /// A fresh identity for an outcome of `dictation_id` (never reused).
    pub fn next_key(&self, dictation_id: DictationId, destination_generation: u64) -> BubbleKey {
        let mut state = self.lock();
        state.last_bubble += 1;
        BubbleKey {
            bubble_id: BubbleId(state.last_bubble),
            dictation_id,
            destination_generation,
        }
    }
    /// True when an outcome of `dictation_id` would be stale (an equal or newer dictation was
    /// already admitted).
    pub fn is_stale(&self, dictation_id: DictationId) -> bool {
        already_admitted(&self.lock(), dictation_id)
    }

    // @spec FR-002: Admit the bubble with its full transcript — .specs/features/010-recovery-bubble/spec.md#fr-002
    /// Admit a bubble in phase `Presenting`; returns its receipt, its view and the bubble it
    /// replaces, if any. `None` for a stale dictation.
    pub fn admit(
        &self,
        admission: Admission,
    ) -> Option<(PresentationReceipt, RecoveryBubbleView, Option<Retired>)> {
        let mut state = self.lock();
        let dictation_id = admission.key.dictation_id;
        if already_admitted(&state, dictation_id) {
            return None;
        }
        state.newest_dictation = Some(dictation_id);
        let replaced = retire(&mut state, PresentationResult::Superseded);
        let late_read = take_buffered(&mut state, &admission);
        let (waiter, receipt) = PresentationReceipt::pending();
        let instance = Instance {
            admission,
            phase: Phase::Presenting,
            late_read,
            waiter: Some(waiter),
            copy_seq: 0,
        };
        let view = instance.view(self.platform);
        state.current = Some(instance);
        Some((receipt, view, replaced))
    }
    /// The surface reported the first render of `bubble_id`: `Presented`.
    pub fn rendered(&self, bubble_id: BubbleId) -> Option<Resolved> {
        let mut state = self.lock();
        let instance = current_in(&mut state, bubble_id, &[Phase::Presenting])?;
        instance.phase = Phase::Visible;
        let waiter_answered = instance.answer(PresentationResult::Presented);
        Some(resolved(instance, waiter_answered))
    }
    /// The bubble cannot be presented (confirmed): it is discarded.
    pub fn unpresentable(&self, bubble_id: BubbleId) -> Option<Resolved> {
        let mut state = self.lock();
        current_in(&mut state, bubble_id, &[Phase::Presenting])?;
        let mut instance = state.current.take()?;
        let waiter_answered = instance.answer(PresentationResult::Unpresentable);
        Some(resolved(&mut instance, waiter_answered))
    }
    /// True while `bubble_id` still waits for its first render.
    pub fn is_presenting(&self, bubble_id: BubbleId) -> bool {
        current_in(&mut self.lock(), bubble_id, &[Phase::Presenting]).is_some()
    }
    /// The projection of the current bubble, if one is presented or shown.
    pub fn view(&self) -> Option<RecoveryBubbleView> {
        let state = self.lock();
        state
            .current
            .as_ref()
            .map(|instance| instance.view(self.platform))
    }

    // @spec AC-005: New dictation, Close, no timeout — .specs/features/010-recovery-bubble/spec.md#ac-005
    /// A new dictation really started: the bubble closes at once, whatever its age or phase,
    /// and a buffered late read is dropped. Called only once the recording started, so a
    /// refused start never loses the transcript of the bubble.
    pub fn new_dictation(&self) -> Option<Retired> {
        let mut state = self.lock();
        state.buffered_late_read = None;
        retire(&mut state, PresentationResult::Superseded)
    }
    /// "Show the bubble when no text field is focused" read at a dictation start, bound later
    /// to the dictation that actually starts (`bind_setting`).
    pub fn freeze_setting(&self, enabled: bool) {
        self.lock().frozen_setting = Some(FrozenSetting {
            dictation_id: None,
            enabled,
        });
    }
    /// Close the shown bubble `bubble_id` (Close, also during Copy or drag).
    pub fn close(&self, bubble_id: BubbleId) -> Option<Retired> {
        let mut state = self.lock();
        let shown = [Phase::Visible, Phase::Dragging, Phase::Copying];
        current_in(&mut state, bubble_id, &shown)?;
        retire(&mut state, PresentationResult::Superseded)
    }

    /// Bind the frozen setting to the dictation that actually started.
    pub fn bind_setting(&self, dictation_id: DictationId) {
        if let Some(frozen) = self.lock().frozen_setting.as_mut() {
            frozen.dictation_id.get_or_insert(dictation_id);
        }
    }
    /// The setting frozen for `dictation_id`; off when unknown (no classification on doubt).
    pub fn setting_for(&self, dictation_id: DictationId) -> bool {
        self.lock()
            .frozen_setting
            .is_some_and(|frozen| frozen.enabled && frozen.dictation_id == Some(dictation_id))
    }

    /// Admit a deliberate Copy of the visible bubble; returns the exact text that would have
    /// been inserted, to write.
    pub fn begin_copy(&self, bubble_id: BubbleId, generation: u64) -> Option<(CopyTicket, String)> {
        let mut state = self.lock();
        let instance = current_in(&mut state, bubble_id, &[Phase::Visible])?;
        if instance.key().destination_generation != generation {
            return None;
        }
        instance.phase = Phase::Copying;
        instance.copy_seq += 1;
        let ticket = CopyTicket {
            bubble_id,
            seq: instance.copy_seq,
        };
        Some((ticket, instance.admission.inserted.clone()))
    }
    /// The Copy of `ticket` finished: success closes, failure keeps the bubble unchanged.
    pub fn finish_copy(&self, ticket: CopyTicket, written: bool) -> CopyEnd {
        let mut state = self.lock();
        let Some(instance) = current_in(&mut state, ticket.bubble_id, &[Phase::Copying]) else {
            return CopyEnd::Ignored;
        };
        if instance.copy_seq != ticket.seq {
            return CopyEnd::Ignored;
        }
        if !written {
            instance.phase = Phase::Visible;
            return CopyEnd::KeptOpen;
        }
        retire(&mut state, PresentationResult::Superseded).map_or(CopyEnd::Ignored, CopyEnd::Closed)
    }

    // @spec AC-004: Identity guarded native drag — .specs/features/010-recovery-bubble/spec.md#ac-004
    /// Admit a native drag of the visible bubble (macOS); returns the exact text that would
    /// have been inserted.
    pub fn begin_drag(&self, bubble_id: BubbleId) -> Option<String> {
        if !self.platform.offers_drag() {
            return None;
        }
        let mut state = self.lock();
        let instance = current_in(&mut state, bubble_id, &[Phase::Visible])?;
        instance.phase = Phase::Dragging;
        Some(instance.admission.inserted.clone())
    }
    /// The drag could not start: the bubble stays visible and unchanged.
    pub fn drag_failed(&self, bubble_id: BubbleId) -> bool {
        let mut state = self.lock();
        let Some(instance) = current_in(&mut state, bubble_id, &[Phase::Dragging]) else {
            return false;
        };
        instance.phase = Phase::Visible;
        true
    }
    /// The drag of `bubble_id` ended with `result`: a drop or an Escape closes the bubble,
    /// a release without a target shows it again.
    pub fn drag_ended(&self, bubble_id: BubbleId, result: DragResult) -> DragEnd {
        let mut state = self.lock();
        let Some(instance) = current_in(&mut state, bubble_id, &[Phase::Dragging]) else {
            return DragEnd::Ignored;
        };
        // Escape is a deliberate dismissal (N25): the same retirement as Close.
        let closed = match result {
            DragResult::Released => {
                instance.phase = Phase::Visible;
                return DragEnd::Refused(instance.view(self.platform));
            }
            DragResult::Dropped => DragEnd::Accepted,
            DragResult::CancelledByEscape => DragEnd::Dismissed,
        };
        retire(&mut state, PresentationResult::Superseded).map_or(DragEnd::Ignored, closed)
    }

    /// 007 saw a read after "not read" for `dictation_id`/`transaction_id`.
    pub fn late_read(&self, dictation_id: DictationId, transaction_id: &str) -> LateReadEnd {
        let mut state = self.lock();
        let platform = self.platform;
        if let Some(instance) = state.current.as_mut() {
            if instance.key().dictation_id == dictation_id {
                return mark_late_read(instance, transaction_id, platform);
            }
        }
        if already_admitted(&state, dictation_id) {
            return LateReadEnd::Ignored;
        }
        state.buffered_late_read = Some((dictation_id, transaction_id.to_owned()));
        LateReadEnd::Buffered
    }
}

/// True when `dictation_id` or a newer dictation was already admitted (ids only grow).
fn already_admitted(state: &State, dictation_id: DictationId) -> bool {
    state
        .newest_dictation
        .is_some_and(|newest| dictation_id.0 <= newest.0)
}
/// The current instance when it is `bubble_id` in one of `phases`.
fn current_in<'a>(
    state: &'a mut State,
    bubble_id: BubbleId,
    phases: &[Phase],
) -> Option<&'a mut Instance> {
    state.current.as_mut().filter(|instance| {
        instance.key().bubble_id == bubble_id && phases.contains(&instance.phase)
    })
}
/// Remove the current instance; a presentation still pending is answered with `result`.
fn retire(state: &mut State, result: PresentationResult) -> Option<Retired> {
    let mut instance = state.current.take()?;
    let pending = instance.phase == Phase::Presenting;
    if pending {
        instance.answer(result);
    }
    Some(Retired {
        key: instance.key(),
        shown: true,
        // A pending presentation still holds 006's slot; a shown one released it already.
        release: pending
            .then_some(instance.admission.transcript_ref)
            .flatten(),
    })
}
fn resolved(instance: &mut Instance, waiter_answered: bool) -> Resolved {
    let admission = &instance.admission;
    Resolved {
        key: admission.key,
        release: admission.transcript_ref,
        waiter_answered,
        history: admission.history,
        decided_at: admission.decided_at,
        consumed_at: admission.consumed_at,
    }
}
/// The late read buffered for this admission, if it names the same dictation and transaction.
fn take_buffered(state: &mut State, admission: &Admission) -> bool {
    let buffered = state.buffered_late_read.take();
    buffered.is_some_and(|(dictation_id, transaction_id)| {
        dictation_id == admission.key.dictation_id
            && admission.transaction_id.as_deref() == Some(transaction_id.as_str())
    })
}
fn mark_late_read(
    instance: &mut Instance,
    transaction_id: &str,
    platform: RecoveryPlatform,
) -> LateReadEnd {
    let matches = instance.admission.transaction_id.as_deref() == Some(transaction_id);
    if !matches || instance.late_read {
        return LateReadEnd::Ignored;
    }
    instance.late_read = true;
    LateReadEnd::Updated(instance.view(platform))
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
