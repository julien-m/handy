//! One recovery instance: the full immutable transcript copied from 006's slot before anything
//! else, its identity, reason, history status, late-read flag and presentation waiter.
use super::types::{
    preview_of, BubbleKey, BubbleSpec, HistoryNote, Phase, RecoveryBubbleView, RecoveryPlatform,
};
use crate::destination::outcome::{HistoryStatus, TranscriptRef};
use crate::destination::recovery::PresentationResult;
use std::time::Instant;
use tokio::sync::oneshot::Sender;

/// Everything 010 keeps from one failed outcome.
pub struct Admission {
    pub key: BubbleKey,
    pub spec: BubbleSpec,
    /// Full text that would have been inserted (never logged, never broadcast).
    pub transcript: String,
    /// 007 transaction as 006 carries it (decimal string), for the late read.
    pub transaction_id: Option<String>,
    pub history: HistoryStatus,
    pub app_name: Option<String>,
    /// 006's slot entry, released once the presentation resolves.
    pub transcript_ref: Option<TranscriptRef>,
    /// When 006 set the disposition to `Recover` (QE latency start), if known.
    pub decided_at: Option<Instant>,
    pub consumed_at: Instant,
}

/// The current bubble inside the manager.
pub(super) struct Instance {
    pub(super) admission: Admission,
    pub(super) phase: Phase,
    pub(super) late_read: bool,
    pub(super) waiter: Option<Sender<PresentationResult>>,
    /// Sequence of the Copy in progress, so a late completion of an older one is ignored.
    pub(super) copy_seq: u64,
}
impl Instance {
    /// The identity a callback must name.
    pub(super) fn key(&self) -> BubbleKey {
        self.admission.key
    }
    /// The projection the recovery window renders.
    pub(super) fn view(&self, platform: RecoveryPlatform) -> RecoveryBubbleView {
        let admission = &self.admission;
        let (preview, truncated) = preview_of(&admission.transcript);
        RecoveryBubbleView {
            bubble_id: admission.key.bubble_id,
            destination_generation: admission.key.destination_generation,
            reason: admission.spec.reason,
            app_name: admission.app_name.clone(),
            preview,
            truncated,
            detail: admission.spec.detail,
            late_read: self.late_read,
            history: HistoryNote::from(admission.history),
            can_drag: platform.offers_drag(),
        }
    }
    /// Answer the presentation waiter once; false when nobody waits any more (006's guard
    /// expired) or it was answered already.
    pub(super) fn answer(&mut self, result: PresentationResult) -> bool {
        self.waiter
            .take()
            .is_some_and(|waiter| waiter.send(result).is_ok())
    }
}
