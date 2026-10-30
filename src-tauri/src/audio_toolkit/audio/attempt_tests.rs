//! Bounded open attempts with real threads and channels: handshake, failure, timeout fencing of
//! a late stream, the stalled-worker cap and the publish-at-timeout race.
use super::{begin, AttemptFailure, OpenLedger, ABANDONED, EXITED, PUBLISHED};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SHORT: Duration = Duration::from_millis(20);

/// Stand-in for a native stream: records that its owner dropped it.
struct FakeStream(Arc<AtomicBool>);
impl Drop for FakeStream {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn a_ready_handshake_returns_the_stream_result() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    let handle = thread::spawn(move || {
        if worker.publish() {
            let _ = tx.send(Ok(48_000));
        }
    });
    assert_eq!(owner.wait(&rx, Duration::from_secs(5)), Ok(48_000));
    handle.join().map_err(|_| "worker panicked")?;
    assert_eq!(ledger.stalled(), 0);
    Ok(())
}

#[test]
fn a_worker_error_is_reported_without_counting_a_stall() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    let handle = thread::spawn(move || {
        let _ = tx.send(Err("device busy".to_owned()));
        drop(worker);
    });
    let outcome = owner.wait(&rx, Duration::from_secs(5));
    assert_eq!(
        outcome,
        Err(AttemptFailure::Failed("device busy".to_owned()))
    );
    handle.join().map_err(|_| "worker panicked")?;
    assert_eq!(ledger.stalled(), 0);
    Ok(())
}

// @spec AC-003: Each open attempt times out after 2 seconds — .specs/features/001-microphone-priority/spec.md#ac-003
#[test]
fn a_timed_out_attempt_is_abandoned_and_its_late_stream_dropped() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let dropped = Arc::new(AtomicBool::new(false));
    let stream = FakeStream(Arc::clone(&dropped));
    let handle = thread::spawn(move || {
        let _ = go_rx.recv();
        if worker.publish() {
            let _ = tx.send(Ok(1));
        }
        drop(stream);
        drop(worker);
    });
    assert_eq!(owner.wait(&rx, SHORT), Err(AttemptFailure::TimedOut));
    assert_eq!(ledger.stalled(), 1);
    go_tx.send(())?;
    handle.join().map_err(|_| "worker panicked")?;
    assert!(dropped.load(Ordering::SeqCst));
    assert!(
        rx.try_recv().is_err(),
        "a late stream must not be published"
    );
    assert_eq!(ledger.stalled(), 0);
    Ok(())
}

#[test]
fn the_stalled_open_cap_refuses_new_attempts_until_a_stall_ends() -> TestResult {
    let ledger = OpenLedger::new(1);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (_tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    assert_eq!(owner.wait(&rx, SHORT), Err(AttemptFailure::TimedOut));
    assert_eq!(begin(&ledger).err(), Some(AttemptFailure::CapReached));
    drop(worker);
    assert_eq!(ledger.stalled(), 0);
    assert!(begin(&ledger).is_ok());
    Ok(())
}

#[test]
fn a_worker_ending_without_handshake_is_lost() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    drop((tx, worker));
    assert_eq!(owner.wait(&rx, SHORT), Err(AttemptFailure::WorkerLost));
    assert_eq!(ledger.stalled(), 0);
    Ok(())
}

#[test]
fn a_publication_racing_the_timeout_is_accepted_not_abandoned() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    assert!(worker.publish());
    tx.send(Ok(7))?;
    assert_eq!(owner.abandon(&rx), Ok(7));
    assert_eq!(ledger.stalled(), 0);
    assert_eq!(worker.fence.0.load(Ordering::SeqCst), PUBLISHED);
    Ok(())
}

#[test]
fn a_worker_exiting_at_the_timeout_is_never_counted_as_stalled() -> TestResult {
    let ledger = OpenLedger::new(2);
    let (owner, worker) = begin(&ledger).map_err(|e| format!("{e:?}"))?;
    let (tx, rx) = mpsc::sync_channel::<Result<u32, String>>(1);
    tx.send(Err("no device".to_owned()))?;
    let fence = Arc::clone(&worker.fence);
    drop(worker);
    assert_eq!(fence.0.load(Ordering::SeqCst), EXITED);
    let outcome = owner.abandon(&rx);
    assert_eq!(outcome, Err(AttemptFailure::Failed("no device".to_owned())));
    assert_eq!(ledger.stalled(), 0);
    assert_ne!(fence.0.load(Ordering::SeqCst), ABANDONED);
    Ok(())
}
