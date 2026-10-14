//! Bounded ownership of one native microphone open. The caller waits at most its timeout for the
//! capture worker's handshake; a timed-out attempt is abandoned, never joined (a blocked native
//! open cannot be killed), and its worker drops any stream it opens late instead of publishing
//! it. A ledger counts abandoned workers still running so repeated stalls cannot create an
//! unbounded number of detached threads: past the cap, a new open is refused.
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::time::Duration;

/// Most abandoned opens that may still run before a new open is refused. One resolution can
/// abandon three opens (two configured choices and the system default, each after its full
/// timeout); four leaves room for one more stall still draining without allowing unbounded
/// growth.
pub const MAX_STALLED_OPENS: usize = 4;

const PENDING: u8 = 0;
const PUBLISHED: u8 = 1;
const EXITED: u8 = 2;
const ABANDONED: u8 = 3;

/// Count of abandoned capture workers that are still running.
#[derive(Debug)]
pub struct OpenLedger {
    stalled: AtomicUsize,
    cap: usize,
}
impl OpenLedger {
    /// Ledger refusing new opens once `cap` abandoned workers are still running.
    pub fn new(cap: usize) -> Arc<Self> {
        Arc::new(Self {
            stalled: AtomicUsize::new(0),
            cap,
        })
    }
    /// Abandoned workers still running.
    pub fn stalled(&self) -> usize {
        self.stalled.load(Ordering::Acquire)
    }
}

/// Why an attempt produced no stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptFailure {
    /// Too many abandoned opens are still running; nothing was started.
    CapReached,
    /// The worker reported this initialization error.
    Failed(String),
    /// The handshake did not arrive in time; the worker was abandoned.
    TimedOut,
    /// The worker ended without a handshake.
    WorkerLost,
}

/// Shared publication state of one attempt.
#[derive(Debug)]
struct Fence(AtomicU8);
impl Fence {
    fn transition(&self, from: u8, to: u8) -> Result<(), u8> {
        self.0
            .compare_exchange(from, to, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
    }
}

/// Worker side of an attempt; dropped when the worker ends.
#[derive(Debug)]
pub struct AttemptWorker {
    fence: Arc<Fence>,
    ledger: Arc<OpenLedger>,
}
impl AttemptWorker {
    /// Claim the opened stream for the caller. `false` means the caller already gave up: the
    /// worker must drop the stream at once and end without sending its handshake.
    pub fn publish(&self) -> bool {
        self.fence.transition(PENDING, PUBLISHED).is_ok()
    }
}
impl Drop for AttemptWorker {
    /// Order matters: an attempt still pending is marked exited first, so a caller timing out
    /// afterwards cannot count a worker that is already gone.
    fn drop(&mut self) {
        if let Err(ABANDONED) = self.fence.transition(PENDING, EXITED) {
            self.ledger.stalled.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

/// Caller side of an attempt.
#[derive(Debug)]
pub struct AttemptOwner {
    fence: Arc<Fence>,
    ledger: Arc<OpenLedger>,
}

// @spec FR-003: Bounded open attempts with owned cleanup — .specs/features/001-microphone-priority/spec.md#fr-003
/// Start one attempt: the owner waits, the worker publishes.
///
/// # Errors
/// `CapReached` when the ledger's cap of stalled opens is reached; nothing is started.
pub fn begin(ledger: &Arc<OpenLedger>) -> Result<(AttemptOwner, AttemptWorker), AttemptFailure> {
    if ledger.stalled() >= ledger.cap {
        return Err(AttemptFailure::CapReached);
    }
    let fence = Arc::new(Fence(AtomicU8::new(PENDING)));
    let worker = AttemptWorker {
        fence: Arc::clone(&fence),
        ledger: Arc::clone(ledger),
    };
    let owner = AttemptOwner {
        fence,
        ledger: Arc::clone(ledger),
    };
    Ok((owner, worker))
}

impl AttemptOwner {
    /// Wait up to `timeout` for the worker's handshake on `handshake`.
    ///
    /// # Errors
    /// `Failed` with the worker's message, `TimedOut` (the worker is abandoned and counted until
    /// it ends) or `WorkerLost`.
    pub fn wait<R>(
        self,
        handshake: &mpsc::Receiver<Result<R, String>>,
        timeout: Duration,
    ) -> Result<R, AttemptFailure> {
        match handshake.recv_timeout(timeout) {
            Ok(result) => result.map_err(AttemptFailure::Failed),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(AttemptFailure::WorkerLost),
            Err(mpsc::RecvTimeoutError::Timeout) => self.abandon(handshake),
        }
    }
    /// Give up on the worker unless it published (or failed) in the meantime.
    fn abandon<R>(
        self,
        handshake: &mpsc::Receiver<Result<R, String>>,
    ) -> Result<R, AttemptFailure> {
        // Count before the transition: a worker ending right after it must find itself counted.
        self.ledger.stalled.fetch_add(1, Ordering::AcqRel);
        match self.fence.transition(PENDING, ABANDONED) {
            Ok(()) => Err(AttemptFailure::TimedOut),
            Err(state) => {
                self.ledger.stalled.fetch_sub(1, Ordering::AcqRel);
                Self::late_handshake(handshake, state)
            }
        }
    }
    /// The worker published just before the timeout: its handshake follows immediately. A worker
    /// that exited has already sent its error, or none.
    fn late_handshake<R>(
        handshake: &mpsc::Receiver<Result<R, String>>,
        state: u8,
    ) -> Result<R, AttemptFailure> {
        let received = if state == PUBLISHED {
            handshake.recv().ok()
        } else {
            handshake.try_recv().ok()
        };
        match received {
            Some(result) => result.map_err(AttemptFailure::Failed),
            None => Err(AttemptFailure::WorkerLost),
        }
    }
}

#[cfg(test)]
#[path = "attempt_tests.rs"]
mod tests;
