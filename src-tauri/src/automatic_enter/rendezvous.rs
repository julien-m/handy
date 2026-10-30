//! Reliable-paste readiness of 009 (macOS beta and Windows reliable paste on a destination
//! session): a rendezvous between two independently latched signals, the successful chord
//! completion and the first qualifying target read at or after injection start, in either order.
//! A read alone, a read before injection, or a chord failure after an early read never makes the
//! dictation eligible. The platform settle and flush keep their clipboard restoration; they never
//! discard, relabel or emit from this latch.
use std::sync::{Mutex, PoisonError};
use std::time::Instant;

// @spec FR-004: Reliable-paste signals for the Enter authority — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-004
/// Signals of one reliable transfer for 009's Enter authority (gate `destination-return` open,
/// dictation with a destination session). Given, the transaction sends no auto-submit Enter of
/// its own: the injection start, the actual chord result and every target read at or after the
/// injection start are reported here, in whatever order they happen; settle and flush keep only
/// their clipboard restoration. Absent (`None`), the transaction behaves exactly as on `dev`.
pub trait ReceiptSink: Send + Sync {
    /// The chord is about to be sent (reads from now on may be the target's).
    fn injection_started(&self, at: std::time::Instant);
    /// The chord returned: `succeeded` is its actual result, never the outer `Ok`.
    fn chord_completed(&self, succeeded: bool, at: std::time::Instant);
    /// A consumer read the transcript at or after the injection start.
    fn target_read(&self, at: std::time::Instant);
}

/// Callback of the rendezvous, run once with `t0 = max(chord completion, first qualifying read)`.
/// It must not block (it may run on the thread that delivered the read).
pub type ReadyFn = Box<dyn Fn(Instant) + Send + Sync>;

#[derive(Default)]
struct Latch {
    injection_started_at: Option<Instant>,
    chord: Option<(bool, Instant)>,
    first_read_at: Option<Instant>,
    fired: bool,
}

/// One dictation's rendezvous; see the module documentation.
pub struct ReliableRendezvous {
    latch: Mutex<Latch>,
    ready: ReadyFn,
}

impl ReliableRendezvous {
    /// Rendezvous calling `ready` once both signals exist.
    pub fn new(ready: ReadyFn) -> Self {
        Self {
            latch: Mutex::new(Latch::default()),
            ready,
        }
    }
    /// True once a reliable transfer started (the transcript was published and the chord sent):
    /// from then on only this rendezvous may make the dictation eligible.
    pub fn engaged(&self) -> bool {
        self.lock().injection_started_at.is_some()
    }
    /// Latch one signal and fire outside the lock when the rendezvous completes.
    fn update(&self, apply: impl FnOnce(&mut Latch)) {
        let ready_at = {
            let mut latch = self.lock();
            apply(&mut latch);
            ready_at(&mut latch)
        };
        if let Some(t0) = ready_at {
            (self.ready)(t0);
        }
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Latch> {
        self.latch.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

// @spec AC-005: Windows reliable paste needs a confirmed read — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
/// Business rule: ready once, when the chord succeeded and a read at or after injection start
/// exists; t0 is the later of the two (a read while the chord is held waits for its completion).
fn ready_at(latch: &mut Latch) -> Option<Instant> {
    let injected = latch.injection_started_at?;
    let read = latch.first_read_at.filter(|read| *read >= injected)?;
    let (succeeded, completed) = latch.chord?;
    if latch.fired || !succeeded {
        return None;
    }
    latch.fired = true;
    Some(completed.max(read))
}

impl ReceiptSink for ReliableRendezvous {
    fn injection_started(&self, at: Instant) {
        self.update(|latch| {
            latch.injection_started_at.get_or_insert(at);
        });
    }
    fn chord_completed(&self, succeeded: bool, at: Instant) {
        self.update(|latch| {
            latch.chord.get_or_insert((succeeded, at));
        });
    }
    fn target_read(&self, at: Instant) {
        self.update(|latch| {
            let qualifies = latch
                .injection_started_at
                .is_some_and(|injected| at >= injected);
            if qualifies && latch.first_read_at.is_none() {
                latch.first_read_at = Some(at);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    fn recording() -> (Arc<Mutex<Vec<Instant>>>, ReliableRendezvous) {
        let fired = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&fired);
        let rendezvous = ReliableRendezvous::new(Box::new(move |t0| {
            sink.lock().unwrap_or_else(PoisonError::into_inner).push(t0);
        }));
        (fired, rendezvous)
    }
    fn fired(log: &Mutex<Vec<Instant>>) -> Vec<Instant> {
        log.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    #[test]
    fn a_read_while_the_chord_is_held_waits_for_its_successful_completion() {
        let (log, rendezvous) = recording();
        let start = Instant::now();
        rendezvous.injection_started(start);
        rendezvous.target_read(start + Duration::from_millis(40));
        assert!(
            fired(&log).is_empty(),
            "a read alone never authorizes Enter"
        );
        rendezvous.chord_completed(true, start + Duration::from_millis(100));
        assert_eq!(fired(&log), vec![start + Duration::from_millis(100)]);
    }
    #[test]
    fn success_before_the_read_becomes_ready_when_the_read_arrives() {
        let (log, rendezvous) = recording();
        let start = Instant::now();
        rendezvous.injection_started(start);
        rendezvous.chord_completed(true, start + Duration::from_millis(100));
        assert!(fired(&log).is_empty());
        rendezvous.target_read(start + Duration::from_millis(180));
        assert_eq!(fired(&log), vec![start + Duration::from_millis(180)]);
    }
    #[test]
    fn a_chord_failure_after_an_early_read_never_fires() {
        let (log, rendezvous) = recording();
        let start = Instant::now();
        rendezvous.injection_started(start);
        rendezvous.target_read(start + Duration::from_millis(20));
        rendezvous.chord_completed(false, start + Duration::from_millis(100));
        rendezvous.target_read(start + Duration::from_millis(150));
        assert!(fired(&log).is_empty());
        assert!(rendezvous.engaged());
    }
    #[test]
    fn a_pre_injection_read_does_not_qualify() {
        let (log, rendezvous) = recording();
        let start = Instant::now();
        rendezvous.target_read(start);
        rendezvous.injection_started(start + Duration::from_millis(5));
        rendezvous.chord_completed(true, start + Duration::from_millis(100));
        assert!(fired(&log).is_empty());
    }
    #[test]
    fn duplicate_reads_and_completions_fire_at_most_once() {
        let (log, rendezvous) = recording();
        let start = Instant::now();
        rendezvous.injection_started(start);
        rendezvous.chord_completed(true, start);
        for offset in [10, 20, 30] {
            rendezvous.target_read(start + Duration::from_millis(offset));
        }
        rendezvous.chord_completed(true, start + Duration::from_millis(50));
        assert_eq!(fired(&log).len(), 1);
    }
    #[test]
    fn no_transfer_means_not_engaged() {
        let (_, rendezvous) = recording();
        assert!(!rendezvous.engaged());
    }
}
