//! Normal-exit gate of the smart-paste actor. The actor settles the pending
//! clipboard transaction from a worker while the main run loop keeps
//! running (its settlement write needs that loop), so the first exit
//! request is held, the actor is shut down off the main thread, and the
//! exit is requested again once settled.

use std::sync::atomic::{AtomicU8, Ordering};

const IDLE: u8 = 0;
const SETTLING: u8 = 1;
const SETTLED: u8 = 2;

/// What to do with one exit request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitDecision {
    /// Let the exit proceed.
    Proceed,
    /// Prevent the exit and settle the actor, then exit again.
    SettleThenExit,
    /// Prevent the exit: settlement is in progress and exits by itself.
    Hold,
}

/// Exit state of the app's actor: idle, settling or settled.
#[derive(Debug, Default)]
pub struct ExitGate(AtomicU8);

impl ExitGate {
    /// Decides for one exit request. `restart` is a restart request, which
    /// cannot be prevented: it proceeds and the actor settles on drop
    /// without waiting.
    pub fn on_exit_requested(&self, restart: bool) -> ExitDecision {
        if restart {
            return ExitDecision::Proceed;
        }
        match self
            .0
            .compare_exchange(IDLE, SETTLING, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(_) => ExitDecision::SettleThenExit,
            Err(SETTLING) => ExitDecision::Hold,
            Err(_) => ExitDecision::Proceed,
        }
    }

    /// The actor has settled (or could not): the next exit proceeds.
    pub fn mark_settled(&self) {
        self.0.store(SETTLED, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_exit_settles_later_ones_wait_and_the_final_one_proceeds() {
        let gate = ExitGate::default();
        assert_eq!(gate.on_exit_requested(false), ExitDecision::SettleThenExit);
        assert_eq!(gate.on_exit_requested(false), ExitDecision::Hold);
        gate.mark_settled();
        assert_eq!(gate.on_exit_requested(false), ExitDecision::Proceed);
    }

    #[test]
    fn a_restart_is_never_held() {
        let gate = ExitGate::default();
        assert_eq!(gate.on_exit_requested(true), ExitDecision::Proceed);
        assert_eq!(gate.on_exit_requested(false), ExitDecision::SettleThenExit);
        assert_eq!(gate.on_exit_requested(true), ExitDecision::Proceed);
    }
}
