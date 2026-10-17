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
/// What a test consumer answers.
#[derive(Clone, Copy)]
enum Answer {
    Received,
    Presented,
    Pending,
}
struct Counting {
    calls: AtomicUsize,
    answer: Answer,
}
fn counting(answer: Answer) -> Arc<Counting> {
    Arc::new(Counting {
        calls: AtomicUsize::new(0),
        answer,
    })
}
impl OutcomeConsumer for Counting {
    fn consume(&self, outcome: &InsertionOutcome) -> ConsumerAck {
        // `outcome` is unused: this double only counts deliveries.
        let _ = outcome;
        self.calls.fetch_add(1, Ordering::SeqCst);
        match self.answer {
            Answer::Received => ConsumerAck::Received,
            Answer::Presented => ConsumerAck::ReasonPresented,
            Answer::Pending => ConsumerAck::PresentationPending(PresentationReceipt::pending().1),
        }
    }
}
/// True when `delivery` was delivered with (`Some(true)`) or without a taken reason.
fn delivered(delivery: &Delivery) -> Option<bool> {
    match delivery {
        Delivery::Delivered { presentation } => Some(presentation.is_some()),
        Delivery::Duplicate => None,
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
    assert_eq!(delivered(&registry.publish(&outcome(1))), Some(false));
    assert_eq!(delivered(&registry.publish(&outcome(1))), None);
}
#[test]
fn a_reason_is_suppressed_only_after_a_consumer_acknowledges_presenting_it() {
    let registry = OutcomeRegistry::default();
    let enter = counting(Answer::Received);
    registry.register(enter.clone());
    assert_eq!(delivered(&registry.publish(&outcome(1))), Some(false));
    let bubble = counting(Answer::Presented);
    registry.register(bubble.clone());
    assert_eq!(delivered(&registry.publish(&outcome(2))), Some(true));
    registry.publish(&outcome(2));
    let calls = [&enter, &bubble].map(|consumer| consumer.calls.load(Ordering::SeqCst));
    assert_eq!(calls, [2, 1]);
}
#[test]
fn a_pending_presentation_is_handed_to_the_publisher_with_every_consumer_called() {
    let registry = OutcomeRegistry::default();
    let enter = counting(Answer::Received);
    let bubble = counting(Answer::Pending);
    registry.register(enter.clone());
    registry.register(bubble.clone());
    assert_eq!(delivered(&registry.publish(&outcome(1))), Some(true));
    let calls = [&enter, &bubble].map(|consumer| consumer.calls.load(Ordering::SeqCst));
    assert_eq!(calls, [1, 1]);
}
#[test]
fn a_receipt_reports_the_answer_a_dropped_sender_or_the_guard_expiry() -> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let guard = std::time::Duration::from_millis(20);
    let (sender, receipt) = PresentationReceipt::pending();
    let _ = sender.send(PresentationResult::Superseded);
    let answered = runtime.block_on(receipt.wait(guard));
    assert_eq!(
        answered,
        PresentationWait::Resolved(PresentationResult::Superseded)
    );
    let (sender, receipt) = PresentationReceipt::pending();
    drop(sender);
    assert_eq!(
        runtime.block_on(receipt.wait(guard)),
        PresentationWait::Abandoned
    );
    let (_sender, receipt) = PresentationReceipt::pending();
    assert_eq!(
        runtime.block_on(receipt.wait(guard)),
        PresentationWait::GuardExpired
    );
    Ok(())
}
#[test]
fn an_answer_at_the_guard_boundary_is_never_lost() {
    let (sender, mut receipt) = PresentationReceipt::pending();
    assert_eq!(sender.send(PresentationResult::Unpresentable), Ok(()));
    assert_eq!(
        after_guard(&mut receipt.0),
        PresentationWait::Resolved(PresentationResult::Unpresentable),
        "sent just before expiry: 006 reports it"
    );
    let (sender, mut receipt) = PresentationReceipt::pending();
    assert_eq!(after_guard(&mut receipt.0), PresentationWait::GuardExpired);
    assert!(
        sender.send(PresentationResult::Unpresentable).is_err(),
        "sent after expiry: refused, so 010 reports it"
    );
}
// @spec AC-003: Copy and drag carry exactly the text that would have been inserted — .specs/features/010-recovery-bubble/spec.md#ac-003
#[test]
fn the_slot_keeps_the_exact_inserted_text_with_and_without_the_trailing_space() {
    let slot = RecoverySlot::default();
    let final_text = |append_trailing_space| FinalText {
        text: "kept phrase".to_owned(),
        append_trailing_space,
    };
    let spaced = slot.retain_final(DictationId(1), final_text(true));
    assert_eq!(slot.read(spaced), Ok("kept phrase".to_owned()));
    assert_eq!(slot.read_inserted(spaced), Ok("kept phrase ".to_owned()));
    let plain = slot.retain_final(DictationId(2), final_text(false));
    assert_eq!(slot.read(plain), Ok("kept phrase".to_owned()));
    assert_eq!(slot.read_inserted(plain), Ok("kept phrase".to_owned()));
    assert_eq!(slot.read_inserted(spaced), Err(ExpiredReference));
}
