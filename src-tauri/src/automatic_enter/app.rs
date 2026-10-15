//! Tauri glue of 009's Enter authority: the transaction of one dictation built from the
//! insertion's settings snapshot, the native key press on the main thread (Enigo's home), and the
//! reliable-paste rendezvous that starts the Enter worker off the thread delivering the read.
use super::policy::Completion;
use super::rendezvous::ReliableRendezvous;
use super::transaction::{EnterPorts, EnterSnapshot, EnterTransaction, KeyEmitter};
use crate::destination::delivery::DictationTicket;
use crate::destination::guard::DispatchGuard;
use crate::destination::manager::DestinationManager;
use crate::destination::workspace::SystemClock;
use crate::settings::{AppSettings, AutoSubmitKey};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;

/// A responsive main thread runs one key press well within a second; a later run is refused
/// (an Enter late by more than this would land in another context).
const MAIN_THREAD_WAIT: Duration = Duration::from_secs(1);

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
    let ports = EnterPorts {
        claims: Arc::clone(manager) as Arc<dyn super::transaction::EnterClaims>,
        workspace: manager.workspace(),
        keys: Arc::new(MainThreadKeys(app.clone())),
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

/// Presses the key on the main thread through the shared Enigo instance.
struct MainThreadKeys(AppHandle);
impl KeyEmitter for MainThreadKeys {
    fn send(&self, key: AutoSubmitKey) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        let handle = self.0.clone();
        let deadline = Instant::now() + MAIN_THREAD_WAIT;
        self.0
            .run_on_main_thread(move || {
                // Too late: the worker already reported a failure; never press it now.
                let sent = if Instant::now() > deadline {
                    Err("the main thread ran the Enter press too late".to_owned())
                } else {
                    crate::clipboard::send_auto_submit(&handle, key)
                };
                // A dropped receiver means the worker stopped waiting.
                let _ = sender.send(sent);
            })
            .map_err(|error| error.to_string())?;
        receiver
            .recv_timeout(MAIN_THREAD_WAIT)
            .map_err(|error| format!("Enter press not confirmed: {error}"))?
    }
}
