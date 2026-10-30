//! Ordering of the actor queue: in-flight reads before timers, shutdown
//! before queued insertions.

use std::time::Duration;

use super::*;
use crate::smart_paste::clock::ManualClock;
use crate::smart_paste::ids::{DestinationGeneration, DictationId, TransactionId, Transcript};
use crate::smart_paste::request::DestinationContext;
use crate::smart_paste::settlement::ClipboardPolicy;

/// Generous grace so the delayed sender below always lands inside it.
const TEST_GRACE: Duration = Duration::from_secs(5);
const DELIVERY_DELAY: Duration = Duration::from_millis(30);

fn new_queue(grace: Duration) -> (ActorQueue, Sender<ActorMessage>, ManualClock) {
    let clock = ManualClock::new();
    let (sender, receiver) = mpsc::channel();
    let mut queue = ActorQueue::new(receiver, Arc::new(clock.clone()));
    queue.read_grace = grace;
    (queue, sender, clock)
}

fn read_at(at: Instant) -> ActorMessage {
    ActorMessage::Read(ReadEvent {
        transaction: TransactionId(1),
        at,
    })
}

/// Sends `message` from another thread after a delay, as a preempted main
/// thread would after stamping a read.
fn send_late(sender: &Sender<ActorMessage>, message: ActorMessage) {
    let sender = sender.clone();
    std::thread::spawn(move || {
        std::thread::sleep(DELIVERY_DELAY);
        let _ = sender.send(message);
    });
}

fn insert_message() -> ActorMessage {
    let request = InsertionRequest {
        dictation_id: DictationId(1),
        transcript: Transcript::new("x"),
        destination: DestinationContext {
            pid: Some(1),
            bundle_id: None,
            generation: DestinationGeneration(1),
        },
        clipboard_policy: ClipboardPolicy::DontModify,
        paste_delay: Duration::ZERO,
        verifies_paste: false,
    };
    let (reply, _) = mpsc::channel();
    ActorMessage::Insert(Box::new(request), reply)
}

#[test]
fn a_read_stamped_before_the_no_read_deadline_but_delivered_after_it_still_counts() {
    let (mut queue, sender, clock) = new_queue(TEST_GRACE);
    let deadline = clock.now();
    send_late(&sender, read_at(deadline - Duration::from_millis(1)));
    let read = queue.next_read(deadline);
    assert_eq!(
        read.map(|event| event.at),
        Some(deadline - Duration::from_millis(1))
    );
}

#[test]
fn without_a_grace_the_late_delivered_read_would_be_missed() {
    let (mut queue, sender, clock) = new_queue(Duration::ZERO);
    let deadline = clock.now();
    send_late(&sender, read_at(deadline));
    assert_eq!(queue.next_read(deadline), None);
}

#[test]
fn reads_stamped_after_the_deadline_wait_for_later() {
    let (mut queue, sender, clock) = new_queue(Duration::ZERO);
    let deadline = clock.now();
    let _ = sender.send(read_at(deadline + Duration::from_millis(1)));
    assert_eq!(queue.next_read(deadline), None);
    assert!(matches!(
        queue.next(None),
        Next::Message(ActorMessage::Read(_))
    ));
}

#[test]
fn an_idle_timer_first_takes_a_read_still_in_flight() {
    let (mut queue, sender, clock) = new_queue(TEST_GRACE);
    let due = clock.now();
    send_late(&sender, read_at(due));
    assert!(matches!(
        queue.next(Some(due)),
        Next::Message(ActorMessage::Read(_))
    ));
    let (mut quiet, _sender, clock) = new_queue(Duration::ZERO);
    assert!(matches!(quiet.next(Some(clock.now())), Next::Deadline));
}

#[test]
fn shutdown_is_served_before_queued_insertions() {
    let (mut queue, sender, _clock) = new_queue(Duration::ZERO);
    let _ = sender.send(insert_message());
    let (reply, _) = mpsc::channel();
    let _ = sender.send(ActorMessage::Shutdown(reply));
    assert!(matches!(
        queue.next(None),
        Next::Message(ActorMessage::Shutdown(_))
    ));
    assert!(matches!(
        queue.next(None),
        Next::Message(ActorMessage::Insert(..))
    ));
}

#[test]
fn the_shutdown_allowance_covers_the_longest_insertion() {
    let allowance = shutdown_allowance(Duration::from_millis(60));
    assert!(allowance >= JUMP_NO_READ_DECISION + Duration::from_millis(60));
    assert!(allowance < Duration::from_secs(10));
}
