//! Tauri glue of 009's Enter authority: the transaction of one dictation built from the
//! insertion's settings snapshot, the guarded key press on the main thread (Enigo's home), and the
//! reliable-paste rendezvous that starts the Enter worker off the thread delivering the read.
use super::key_press::{GuardedKeyPress, MainThreadRunner, MAIN_THREAD_BOUND};
use super::policy::Completion;
use super::rendezvous::ReliableRendezvous;
use super::transaction::{EnterPorts, EnterSnapshot, EnterTransaction};
use crate::destination::delivery::DictationTicket;
use crate::destination::guard::DispatchGuard;
use crate::destination::manager::DestinationManager;
use crate::destination::workspace::SystemClock;
use crate::settings::AppSettings;
use std::sync::Arc;
use std::time::Instant;
use tauri::AppHandle;

// @spec FR-004: Policy and key frozen at insertion entry — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-004
/// The Enter transaction of `ticket`, with the policy and key of the insertion's `settings`
/// snapshot (a change made later applies to the next dictation).
pub fn transaction(
    app: &AppHandle,
    manager: &Arc<DestinationManager>,
    ticket: DictationTicket,
    settings: &AppSettings,
) -> Arc<EnterTransaction> {
    let frozen = EnterSnapshot {
        dictation_id: ticket.dictation_id,
        destination_generation: ticket.destination_generation,
        policy: super::policy_of(settings),
        key: settings.auto_submit_key,
    };
    let handle = app.clone();
    let keys = GuardedKeyPress {
        runner: Arc::new(MainThread(app.clone())),
        workspace: manager.workspace(),
        press: Arc::new(move |key| crate::clipboard::send_auto_submit(&handle, key)),
        bound: MAIN_THREAD_BOUND,
    };
    let ports = EnterPorts {
        claims: Arc::clone(manager) as Arc<dyn super::transaction::EnterClaims>,
        workspace: manager.workspace(),
        keys: Arc::new(keys),
        clock: Arc::new(SystemClock),
        pause: Arc::new(SystemClock),
    };
    Arc::new(EnterTransaction::new(frozen, ports))
}

/// The reliable-paste rendezvous of `enter`: once ready it decides the Enter on its own worker
/// thread with the context of `guard`.
pub fn rendezvous(
    enter: &Arc<EnterTransaction>,
    guard: &Arc<DispatchGuard>,
) -> Arc<ReliableRendezvous> {
    let (enter, guard) = (Arc::clone(enter), Arc::clone(guard));
    Arc::new(ReliableRendezvous::new(Box::new(move |t0: Instant| {
        let (enter, context) = (Arc::clone(&enter), guard.context());
        // Not joined: the read callback (main thread on macOS) must not wait for the pause.
        std::thread::spawn(move || {
            enter.complete(Completion::ReliableConfirmed, context.as_ref(), t0);
        });
    })))
}

/// The app's main thread.
struct MainThread(AppHandle);
impl MainThreadRunner for MainThread {
    fn run(&self, task: Box<dyn FnOnce() + Send>) -> Result<(), String> {
        self.0
            .run_on_main_thread(task)
            .map_err(|error| error.to_string())
    }
}
