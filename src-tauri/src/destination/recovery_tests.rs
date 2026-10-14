//! Volatile retention (supersession, release, expired references) and consumer delivery
//! (optional consumers, deduplication, reason acknowledgement).
use super::*;
use crate::destination::context::{DestinationSource, DictationId};
use crate::destination::outcome::{HistoryStatus, InsertionStatus};
use std::sync::atomic::{AtomicUsize, Ordering};

fn outcome(outcome_id: u64) -> InsertionOutcome {
    InsertionOutcome {
        dictation_id: DictationId(3),
        destination_generation: 5,
        outcome_id,
        target: None,
        destination_source: DestinationSource::Captured,
        status: InsertionStatus::Failed,
        method_outcome: None,
        transaction_id: None,
        reason: None,
        history_status: HistoryStatus::Saved,
        history_entry_id: Some(9),
        transcript_ref: Some(TranscriptRef(1)),
        completed_at: 0,
    }
}
struct Counting {
    calls: AtomicUsize,
    ack: ConsumerAck,
}
fn counting(ack: ConsumerAck) -> Arc<Counting> {
    Arc::new(Counting {
        calls: AtomicUsize::new(0),
        ack,
    })
}
const DELIVERED: Delivery = Delivery::Delivered {
    reason_presented: false,
};
impl OutcomeConsumer for Counting {
    fn consume(&self, outcome: &InsertionOutcome) -> ConsumerAck {
        // `outcome` is unused: this double only counts deliveries.
        let _ = outcome;
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.ack
    }
}

#[test]
fn a_later_dictation_supersedes_the_retained_transcript_and_old_references_expire() {
    let slot = RecoverySlot::default();
    let first = slot.retain(DictationId(1), "first phrase".to_owned());
    assert_eq!(slot.read(first), Ok("first phrase".to_owned()));
    let second = slot.retain(DictationId(2), "second phrase".to_owned());
    assert_eq!(slot.read(first), Err(ExpiredReference));
    assert_eq!(slot.dictation_of(second), Some(DictationId(2)));
    assert!(slot.release(second));
    assert!(!slot.release(second));
    assert_eq!(slot.read(second), Err(ExpiredReference));
}
#[test]
fn without_consumers_an_outcome_is_still_delivered_and_never_twice() {
    let registry = OutcomeRegistry::default();
    let delivered = registry.publish(&outcome(1));
    assert_eq!(
        delivered,
        Delivery::Delivered {
            reason_presented: false
        }
    );
    assert_eq!(registry.publish(&outcome(1)), Delivery::Duplicate);
}
#[test]
fn a_reason_is_suppressed_only_after_a_consumer_acknowledges_presenting_it() {
    let registry = OutcomeRegistry::default();
    let enter = counting(ConsumerAck::Received);
    registry.register(enter.clone());
    assert_eq!(registry.publish(&outcome(1)), DELIVERED);
    let bubble = counting(ConsumerAck::ReasonPresented);
    registry.register(bubble.clone());
    let presented = Delivery::Delivered {
        reason_presented: true,
    };
    assert_eq!(registry.publish(&outcome(2)), presented);
    registry.publish(&outcome(2));
    let calls = [&enter, &bubble].map(|consumer| consumer.calls.load(Ordering::SeqCst));
    assert_eq!(calls, [2, 1]);
}
