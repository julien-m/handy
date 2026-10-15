//! Conservation of a failed dictation and hand-off to optional consumers (009 Enter policy, 010
//! recovery bubble, whose presentation is answered later through a `PresentationReceipt`). The final text stays in one bounded volatile slot behind an opaque
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
    /// What the method would have inserted when it differs from `text` (trailing space).
    inserted: Option<String>,
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
        self.retain_final(dictation_id, text, false)
    }
    /// Retain `text` of `dictation_id` and the exact text its method would have inserted: `text`
    /// plus a trailing space when `append_trailing_space` was on for this insertion. Supersedes
    /// any older transcript.
    pub fn retain_final(
        &self,
        dictation_id: DictationId,
        text: String,
        append_trailing_space: bool,
    ) -> TranscriptRef {
        let inserted = append_trailing_space
            .then(|| crate::clipboard::trailing_spaced(text.clone(), append_trailing_space));
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.last_reference += 1;
        let reference = TranscriptRef(state.last_reference);
        state.retained = Some(Retained {
            reference,
            dictation_id,
            text,
            inserted,
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
    /// The exact text the method would have inserted for `reference` (Copy and drag payload).
    ///
    /// # Errors
    /// `ExpiredReference` once superseded or released.
    pub fn read_inserted(&self, reference: TranscriptRef) -> Result<String, ExpiredReference> {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        match &state.retained {
            Some(retained) if retained.reference == reference => Ok(retained
                .inserted
                .clone()
                .unwrap_or_else(|| retained.text.clone())),
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

/// What the recovery surface answered for one presented reason (010 "Presentation hand-off").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationResult {
    /// The recovery window is shown and its view reported the first render of that bubble.
    Presented,
    /// Confirmed impossibility (no window, creation or show error, hung surface destroyed).
    Unpresentable,
    /// A new dictation closed the bubble before it became visible.
    Superseded,
}
/// How the wait for a presentation ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationWait {
    /// The surface answered before the guard.
    Resolved(PresentationResult),
    /// The consumer dropped its end without an answer: treated as a confirmed failure.
    Abandoned,
    /// The guard expired first; the presentation goes on without this waiter.
    GuardExpired,
}
/// The waiting end of one pending presentation, handed to 006 by the consumer.
#[derive(Debug)]
pub struct PresentationReceipt(tokio::sync::oneshot::Receiver<PresentationResult>);
impl PresentationReceipt {
    /// A receipt and the sender the consumer resolves later.
    pub fn pending() -> (tokio::sync::oneshot::Sender<PresentationResult>, Self) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        (sender, Self(receiver))
    }
    /// A receipt already resolved with `result` (a consumer that presented synchronously).
    pub fn resolved(result: PresentationResult) -> Self {
        let (sender, receipt) = Self::pending();
        // The receiver is alive in `receipt`, so this send cannot fail.
        let _ = sender.send(result);
        receipt
    }
    /// Wait for the answer at most `guard`, without blocking a thread. At expiry the receipt
    /// is closed and drained, so an answer sent at the very boundary is still reported here
    /// and a later one fails on the consumer's side, which then reports it itself.
    pub async fn wait(mut self, guard: std::time::Duration) -> PresentationWait {
        match tokio::time::timeout(guard, &mut self.0).await {
            Ok(Ok(result)) => PresentationWait::Resolved(result),
            Ok(Err(_)) => PresentationWait::Abandoned,
            Err(_) => after_guard(&mut self.0),
        }
    }
}

// @spec FR-002: No failure lost at the guard boundary — .specs/features/010-recovery-bubble/spec.md#fr-002
/// The guard expired: close the channel first, then take an answer that arrived meanwhile.
/// Once closed, every later send fails, so exactly one side reports the failure.
fn after_guard(
    receiver: &mut tokio::sync::oneshot::Receiver<PresentationResult>,
) -> PresentationWait {
    receiver.close();
    match receiver.try_recv() {
        Ok(result) => PresentationWait::Resolved(result),
        Err(_) => PresentationWait::GuardExpired,
    }
}

/// What a consumer did with an outcome.
#[derive(Debug)]
pub enum ConsumerAck {
    /// The consumer presented the failure reason synchronously.
    ReasonPresented,
    /// Received; the consumer does not present the reason.
    Received,
    /// The consumer took the reason and answers its presentation through the receipt (010).
    PresentationPending(PresentationReceipt),
}
/// A registered consumer of terminal outcomes (009, 010).
pub trait OutcomeConsumer: Send + Sync {
    /// Receive one terminal outcome (each `outcome_id` at most once). Never waits: a consumer
    /// that presents later returns `PresentationPending`.
    fn consume(&self, outcome: &InsertionOutcome) -> ConsumerAck;
}
/// Result of one publication.
#[derive(Debug)]
pub enum Delivery {
    /// Already published under this `outcome_id`: nothing was delivered again.
    Duplicate,
    /// Delivered; `presentation` when a consumer took the reason, so the caller waits for its
    /// answer before deciding on the `paste-error` fallback.
    Delivered {
        presentation: Option<PresentationReceipt>,
    },
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
        // Every consumer receives the outcome; the first one that takes the reason answers it.
        let presentation = consumers
            .iter()
            .map(|consumer| consumer.consume(outcome))
            .fold(None, |taken, ack| taken.or_else(|| receipt_of(ack)));
        Delivery::Delivered { presentation }
    }
}

/// The receipt of a consumer that took the reason, if it did.
fn receipt_of(ack: ConsumerAck) -> Option<PresentationReceipt> {
    match ack {
        ConsumerAck::ReasonPresented => {
            Some(PresentationReceipt::resolved(PresentationResult::Presented))
        }
        ConsumerAck::PresentationPending(receipt) => Some(receipt),
        ConsumerAck::Received => None,
    }
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
