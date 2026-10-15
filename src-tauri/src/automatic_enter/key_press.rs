//! The Enter key press of 009 on the thread that owns the input (the main thread): the
//! destination is checked again there, immediately before the key, and the press is refused when
//! the main thread runs it too late. One shared state decides between the worker giving up and the
//! main thread taking the task, so the reported result is always the actual one: a key is never
//! sent after the worker reported it as not sent, and a sent key is never reported as failed.
use super::transaction::{KeyEmitter, KeyOutcome};
use crate::destination::context::ResolvedTarget;
use crate::destination::workspace::{Foreground, WorkspacePort};
use crate::settings::AutoSubmitKey;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Latest acceptable start of the main-thread press after its dispatch: the AC-005 QE budget
/// of 100 ms between readiness and the key; a later press would land in another context.
pub const MAIN_THREAD_BOUND: Duration = Duration::from_millis(100);

const PENDING: u8 = 0;
const TAKEN: u8 = 1;
const ABANDONED: u8 = 2;

/// Runs a task on the input thread (the main thread in production).
pub trait MainThreadRunner: Send + Sync {
    /// Schedule `task`; it may run later.
    ///
    /// # Errors
    /// The task could not be scheduled; it never runs.
    fn run(&self, task: Box<dyn FnOnce() + Send>) -> Result<(), String>;
}

/// The native key press itself (Enigo through `clipboard::send_auto_submit` in production).
pub type PressFn = Arc<dyn Fn(AutoSubmitKey) -> Result<(), String> + Send + Sync>;

/// Key emitter that checks the destination on the input thread right before pressing.
pub struct GuardedKeyPress {
    pub runner: Arc<dyn MainThreadRunner>,
    pub workspace: Arc<dyn WorkspacePort>,
    pub press: PressFn,
    pub bound: Duration,
}

// @spec AC-005: Destination checked immediately before the key — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
impl KeyEmitter for GuardedKeyPress {
    fn send(&self, key: AutoSubmitKey, expected: Option<&ResolvedTarget>) -> KeyOutcome {
        let state = Arc::new(AtomicU8::new(PENDING));
        let (sender, receiver) = mpsc::channel();
        let task = press_task(self, key, expected.cloned(), Arc::clone(&state), sender);
        if let Err(error) = self.runner.run(task) {
            return KeyOutcome::Failed(error);
        }
        await_outcome(&state, &receiver, self.bound)
    }
}

/// The main-thread part: take the task (unless the worker gave up), refuse a late start, check
/// the destination, then press once.
fn press_task(
    emitter: &GuardedKeyPress,
    key: AutoSubmitKey,
    expected: Option<ResolvedTarget>,
    state: Arc<AtomicU8>,
    sender: mpsc::Sender<KeyOutcome>,
) -> Box<dyn FnOnce() + Send> {
    let (workspace, press) = (Arc::clone(&emitter.workspace), Arc::clone(&emitter.press));
    let (dispatched, bound) = (Instant::now(), emitter.bound);
    Box::new(move || {
        let taken = state.compare_exchange(PENDING, TAKEN, Ordering::SeqCst, Ordering::SeqCst);
        if taken.is_err() {
            // The worker already reported the press as late: never press it now.
            return;
        }
        let outcome = if dispatched.elapsed() > bound {
            KeyOutcome::Late
        } else if !in_front(workspace.as_ref(), expected.as_ref()) {
            KeyOutcome::FocusLost
        } else {
            press(key).map_or_else(KeyOutcome::Failed, |()| KeyOutcome::Sent)
        };
        // A dropped receiver cannot happen once the task is taken (the worker waits for it).
        let _ = sender.send(outcome);
    })
}

/// The worker's wait: within the bound it gets the task's outcome; past it, either it abandons
/// the task (never pressed: `Late`) or the task was already taken and its actual outcome follows.
fn await_outcome(
    state: &AtomicU8,
    receiver: &mpsc::Receiver<KeyOutcome>,
    bound: Duration,
) -> KeyOutcome {
    if let Ok(outcome) = receiver.recv_timeout(bound) {
        return outcome;
    }
    match state.compare_exchange(PENDING, ABANDONED, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => KeyOutcome::Late,
        Err(_) => receiver
            .recv()
            .unwrap_or_else(|_| KeyOutcome::Failed("the key press did not report".to_owned())),
    }
}

/// The expected destination (PID and identity) is in front; no expectation for the
/// frontmost-at-the-end route.
fn in_front(workspace: &dyn WorkspacePort, expected: Option<&ResolvedTarget>) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    matches!(workspace.frontmost_now(), Foreground::Application(front)
        if front.pid == expected.pid && front.identity == expected.identity)
}

#[cfg(test)]
#[path = "key_press_tests.rs"]
mod tests;
