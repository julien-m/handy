//! Conservation of a failed dictation and hand-off to optional consumers (009 Enter policy, 010
//! recovery bubble). The final text stays in one bounded volatile slot behind an opaque
//! reference until a later final result supersedes it, a consumer releases it or the process
//! exits; it is never broadcast in an event or a log. Consumers are optional: a missing one
//! never prevents the terminal outcome.
use super::context::DictationId;
use super::outcome::{InsertionOutcome, TranscriptRef};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

/// A reference that no longer names the retained transcript (superseded or released).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpiredReference;

struct Retained {
    reference: TranscriptRef,
    dictation_id: DictationId,
    text: String,
}
#[derive(Default)]
struct SlotState {
    last_reference: u64,
    retained: Option<Retained>,
}
/// The single volatile recovery slot (technical hand-off storage, not durable history).
#[derive(Default)]
pub struct RecoverySlot {
    state: Mutex<SlotState>,
}
impl RecoverySlot {
    /// Retain `text` of `dictation_id`, superseding any older transcript.
    pub fn retain(&self, dictation_id: DictationId, text: String) -> TranscriptRef {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.last_reference += 1;
        let reference = TranscriptRef(state.last_reference);
        state.retained = Some(Retained {
            reference,
            dictation_id,
            text,
        });
        reference
    }
    /// Text behind `reference`, for a trusted native consumer.
    ///
    /// # Errors
    /// `ExpiredReference` once superseded or released.
    pub fn read(&self, reference: TranscriptRef) -> Result<String, ExpiredReference> {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        match &state.retained {
            Some(retained) if retained.reference == reference => Ok(retained.text.clone()),
            _ => Err(ExpiredReference),
        }
    }
    /// Dictation whose transcript `reference` names, if still retained.
    pub fn dictation_of(&self, reference: TranscriptRef) -> Option<DictationId> {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state
            .retained
            .as_ref()
            .filter(|retained| retained.reference == reference)
            .map(|retained| retained.dictation_id)
    }
    /// Explicit release by a consumer; returns false for an expired reference.
    pub fn release(&self, reference: TranscriptRef) -> bool {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let held = state
            .retained
            .as_ref()
            .is_some_and(|retained| retained.reference == reference);
        if held {
            state.retained = None;
        }
        held
    }
}

/// What a consumer did with an outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumerAck {
    /// The consumer took ownership of presenting the failure reason (010).
    ReasonPresented,
    /// Received; the consumer does not present the reason.
    Received,
}
/// A registered consumer of terminal outcomes (009, 010).
pub trait OutcomeConsumer: Send + Sync {
    /// Receive one terminal outcome (each `outcome_id` at most once).
    fn consume(&self, outcome: &InsertionOutcome) -> ConsumerAck;
}
/// Result of one publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// Already published under this `outcome_id`: nothing was delivered again.
    Duplicate,
    /// Delivered; `reason_presented` when a consumer acknowledged presenting the reason, so the
    /// indicator's own reason may be suppressed.
    Delivered { reason_presented: bool },
}

// Remember this many recent outcome ids for deduplication: far more than the outcomes that can
// be in flight at once (one per dictation), while keeping the memory bounded.
const RECENT_OUTCOMES: usize = 64;

/// Optional consumers of terminal outcomes, with deduplication by `outcome_id`.
#[derive(Default)]
pub struct OutcomeRegistry {
    consumers: RwLock<Vec<Arc<dyn OutcomeConsumer>>>,
    recent: Mutex<VecDeque<u64>>,
}
impl OutcomeRegistry {
    /// Register a consumer (when its owning feature is present).
    pub fn register(&self, consumer: Arc<dyn OutcomeConsumer>) {
        self.consumers
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .push(consumer);
    }
    // @spec FR-004: Hand failures to the recovery features — .specs/features/006-destination-application/spec.md#fr-004
    /// Deliver `outcome` once to every registered consumer.
    pub fn publish(&self, outcome: &InsertionOutcome) -> Delivery {
        {
            let mut recent = self.recent.lock().unwrap_or_else(PoisonError::into_inner);
            if recent.contains(&outcome.outcome_id) {
                return Delivery::Duplicate;
            }
            if recent.len() == RECENT_OUTCOMES {
                recent.pop_front();
            }
            recent.push_back(outcome.outcome_id);
        }
        let consumers = self
            .consumers
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let acks = consumers.iter().map(|consumer| consumer.consume(outcome));
        let reason_presented = acks.fold(false, |presented, ack| {
            presented | (ack == ConsumerAck::ReasonPresented)
        });
        Delivery::Delivered { reason_presented }
    }
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
