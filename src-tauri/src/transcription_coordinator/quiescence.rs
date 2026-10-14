//! Quiescence lease of the coordinator, taken by every change of the shortcut registrations
//! (list edit, post-processing toggle, keyboard switch), whichever surface asked for it.
//!
//! A registration change must never remove the shortcut a session still needs (the release of
//! a held key, the press that stops a locked recording, a press remembered while processing).
//! The lease is asked by message to the coordinator thread and answered there, in input order:
//! a key-down handled before the request makes the coordinator busy and the change is refused;
//! a key-down that arrives while the lease is held is dropped (gated), so it can never become
//! the owner of a registration being changed. External triggers (CLI flags, signals) are not
//! shortcuts and pass the gate.
use super::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Answer of the coordinator to one lease request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestState {
    Pending,
    Granted,
    Refused,
    /// The requester stopped waiting: a late answer must not grant a lease nobody releases.
    Abandoned,
}

/// One lease request, shared by the requester and the coordinator thread.
pub(super) struct LeaseRequest {
    state: Mutex<RequestState>,
    answered: Condvar,
}

impl LeaseRequest {
    /// A pending request, shared with the coordinator thread.
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(RequestState::Pending),
            answered: Condvar::new(),
        })
    }

    /// Coordinator side: answer the request. Returns whether the lease is now held, false when
    /// the coordinator is busy or the requester already gave up.
    pub(super) fn answer(&self, quiescent: bool) -> bool {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if *state != RequestState::Pending {
            return false;
        }
        *state = if quiescent {
            RequestState::Granted
        } else {
            RequestState::Refused
        };
        self.answered.notify_all();
        quiescent
    }

    /// Requester side: wait for the answer until `timeout`. Returns whether the lease was
    /// granted; on timeout the request is abandoned under the same lock, so a late answer
    /// cannot grant it.
    pub(super) fn wait(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        while *state == RequestState::Pending {
            let left = deadline.saturating_duration_since(Instant::now());
            // Exit when the deadline passed: the coordinator is busy running an effect.
            if left.is_zero() {
                *state = RequestState::Abandoned;
                return false;
            }
            state = self
                .answered
                .wait_timeout(state, left)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        *state == RequestState::Granted
    }
}

/// Keyboard input gate of the coordinator thread: closed while at least one lease is held.
#[derive(Debug, Default)]
pub(super) struct InputGate {
    holders: u32,
}

impl InputGate {
    /// Answer `request`; a granted lease closes the gate until it is released.
    pub(super) fn answer(&mut self, request: &LeaseRequest, quiescent: bool) {
        if request.answer(quiescent) {
            self.holders += 1;
        }
    }

    /// A lease was released.
    pub(super) fn release(&mut self) {
        self.holders = self.holders.saturating_sub(1);
    }

    /// Whether an input may reach the state machine: external triggers always, shortcuts only
    /// while no lease is held.
    pub(super) fn admits(&self, external: bool) -> bool {
        external || self.holders == 0
    }
}

/// A held quiescence lease: no session exists and shortcut input is gated until it is dropped.
pub struct QuiescenceLease {
    tx: Sender<Command>,
}

impl QuiescenceLease {
    /// A granted lease; dropping it reopens the gate through `tx`.
    pub(super) fn new(tx: Sender<Command>) -> Self {
        Self { tx }
    }
}

impl Drop for QuiescenceLease {
    fn drop(&mut self) {
        // A closed channel means the coordinator thread is gone: there is no gate to reopen.
        let _ = self.tx.send(Command::Resume);
    }
}

/// The coordinator owns a session (recording, processing, a remembered or deferred input).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionBusy;

#[cfg(test)]
#[path = "quiescence_tests.rs"]
mod tests;
