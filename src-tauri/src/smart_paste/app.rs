//! Tauri glue of smart paste (step 7): the actor thread owned by the app
//! (started at setup when the rollout gate makes smart paste active, settled
//! at a normal exit), the insertion method handed to 006, the insertion
//! without a 006 session, and the outcome follow-ups: the Screen Sharing
//! hand-over message, the automatic Enter (the 009 hook) and the late-read
//! diagnostic for the recovery bubble (the 010 hook). Events carry
//! identifiers only, never the transcript.

use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use super::actor::shutdown_allowance;
use super::destination_bridge::DestinationChecks;
use super::dispatch::{
    follow_ups, insertion_route, ActorSlot, FollowUps, ForegroundInsertion, InsertionInputs,
    InsertionRoute, SmartPasteMethod,
};
use super::exit_gate::{ExitDecision, ExitGate};
use super::outcome::{PasteDiagnostic, PasteOutcome};
use super::request::DiagnosticSink;
use super::setting_policy::Rollout;
use crate::destination::context::DestinationContext as SessionContext;
use crate::destination::workspace::WorkspacePort;
use crate::settings::{AppSettings, AutoSubmitKey};

/// UI event: Screen Sharing received the transcript on the clipboard.
pub const HAND_OVER_EVENT: &str = "smart-paste-hand-over";
/// UI event for 010: a read arrived after "not read" (`PasteDiagnostic`).
pub const LATE_READ_EVENT: &str = "smart-paste-late-read";
/// Existing UI event of a failed insertion (toast); never carries the transcript.
const PASTE_ERROR_EVENT: &str = "paste-error";
/// Same settle as the classic path between the paste and the submit key.
const AUTO_SUBMIT_SETTLE: Duration = Duration::from_millis(50);
/// A responsive main thread runs one key press well within a second.
const MAIN_THREAD_WAIT: Duration = Duration::from_secs(1);

/// Payload of [`HAND_OVER_EVENT`].
#[derive(Clone, Copy, Debug, serde::Serialize)]
struct HandOverNotice {
    dictation_id: u64,
}

/// The smart-paste services of a running app.
pub struct SmartPasteRuntime {
    slot: ActorSlot,
    checks: Arc<DestinationChecks>,
    workspace: Arc<dyn WorkspacePort>,
    exit: ExitGate,
}

// @spec FR-006: Actor owned by the app, settled at exit — .specs/features/007-smart-paste/spec.md#fr-006
/// Starts the actor when smart paste is active in this build (macOS, gate
/// open). Side effects: one thread and one managed state; the pasteboard and
/// the helper process are untouched until the first insertion.
pub fn install(app: &AppHandle) {
    if !Rollout::CURRENT.is_active() {
        return;
    }
    let checks = Arc::new(DestinationChecks::default());
    match spawn_actor(app, Arc::clone(&checks)) {
        Ok(slot) => {
            let workspace = crate::destination::native_workspace();
            let exit = ExitGate::default();
            app.manage(Arc::new(SmartPasteRuntime {
                slot,
                checks,
                workspace,
                exit,
            }));
        }
        Err(error) => log::error!("Smart paste could not start: {error}"),
    }
}

#[cfg(target_os = "macos")]
fn spawn_actor(app: &AppHandle, checks: Arc<DestinationChecks>) -> std::io::Result<ActorSlot> {
    let diagnostics = Box::new(EventDiagnostics(app.clone()));
    super::macos_ports::spawn_actor(checks, diagnostics).map(ActorSlot::new)
}
#[cfg(not(target_os = "macos"))]
fn spawn_actor(app: &AppHandle, checks: Arc<DestinationChecks>) -> std::io::Result<ActorSlot> {
    // Smart paste is a macOS method: `Rollout::is_active` keeps other platforms out.
    let _ = (app, checks);
    Err(std::io::ErrorKind::Unsupported.into())
}

fn runtime(app: &AppHandle) -> Option<Arc<SmartPasteRuntime>> {
    app.try_state::<Arc<SmartPasteRuntime>>()
        .map(|state| Arc::clone(state.inner()))
}

/// 006's insertion method for this dictation when smart paste applies
/// (both gates open), with `settings` read once for the insertion.
pub fn session_method(
    app: &AppHandle,
    text: &str,
    settings: &AppSettings,
) -> Option<SmartPasteMethod> {
    let route = insertion_route(Rollout::CURRENT, true, settings.paste_method);
    let runtime = runtime(app).filter(|_| route == InsertionRoute::SmartPasteInSession)?;
    let final_text = crate::clipboard::with_trailing_space(text.to_owned(), settings);
    let (handle, auto_submit) = (
        app.clone(),
        (settings.auto_submit, settings.auto_submit_key),
    );
    Some(SmartPasteMethod {
        submitter: Arc::new(runtime.slot.clone()),
        checks: Arc::clone(&runtime.checks),
        inputs: InsertionInputs::new(final_text, settings),
        follow_up: Box::new(move |outcome, context| {
            let follow = follow_ups(outcome.kind, auto_submit.0);
            present_hand_over(&handle, follow, outcome);
            if follow.auto_submit && claim_enter(&handle, context) {
                submit_enter(&handle, auto_submit.1);
            }
        }),
    })
}

/// 009 hook: the Enter of an eligible outcome is claimed through 006's
/// single disposition veto; 009 replaces this rule when delivered.
fn claim_enter(app: &AppHandle, context: &SessionContext) -> bool {
    let manager = crate::destination::app::manager(app);
    manager.is_some_and(|manager| {
        let claim = manager.claim_enter(context.dictation_id, context.destination_generation);
        claim.is_ok()
    })
}

/// Settings of a dictation without a 006 session when smart paste applies to it.
pub fn foreground_settings(app: &AppHandle) -> Option<AppSettings> {
    runtime(app)?;
    let settings = crate::settings::get_settings(app);
    let route = insertion_route(Rollout::CURRENT, false, settings.paste_method);
    (route == InsertionRoute::SmartPasteAtForeground).then_some(settings)
}

// @spec FR-011: AX, clipboard, recovery ordering in dictation — .specs/features/007-smart-paste/spec.md#fr-011
/// Inserts `text` into the application in front (destination-return gate
/// closed), on a worker thread, then runs the follow-ups there.
pub async fn insert_at_foreground(
    app: &AppHandle,
    text: String,
    settings: AppSettings,
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
) {
    let Some(runtime) = runtime(app) else { return };
    let final_text = crate::clipboard::with_trailing_space(text, &settings);
    let insertion = ForegroundInsertion {
        submitter: Arc::new(runtime.slot.clone()),
        checks: Arc::clone(&runtime.checks),
        workspace: Arc::clone(&runtime.workspace),
        inputs: InsertionInputs::new(final_text, &settings),
    };
    let handle = app.clone();
    let joined = tauri::async_runtime::spawn_blocking(move || {
        let outcome = insertion.insert(cancelled)?;
        follow_without_session(&handle, &outcome, &settings);
        Some(outcome)
    })
    .await;
    if let Err(error) = joined {
        log::error!("Smart paste worker failed: {error}");
        // Best effort: without a listening window only the toast is lost; the error is logged.
        let _ = app.emit(PASTE_ERROR_EVENT, ());
    }
}

fn follow_without_session(app: &AppHandle, outcome: &PasteOutcome, settings: &AppSettings) {
    let follow = follow_ups(outcome.kind, settings.auto_submit);
    present_hand_over(app, follow, outcome);
    if follow.failure_notice {
        // 010 hook: until the recovery bubble exists, the classic failure toast (best effort,
        // like the classic path: the outcome itself is logged by the coordinator).
        let _ = app.emit(PASTE_ERROR_EVENT, ());
    }
    if follow.auto_submit {
        submit_enter(app, settings.auto_submit_key);
    }
}

/// AC-012 message for Screen Sharing; 006's indicator presentation waits
/// for the Penflow pass, so the UI shows it as a toast meanwhile.
fn present_hand_over(app: &AppHandle, follow: FollowUps, outcome: &PasteOutcome) {
    if !follow.hand_over_message {
        return;
    }
    let notice = HandOverNotice {
        dictation_id: outcome.dictation_id.0,
    };
    if let Err(error) = app.emit(HAND_OVER_EVENT, notice) {
        log::warn!("Could not show the smart paste hand-over message: {error}");
    }
}

/// Presses the automatic-submit key on the main thread (Enigo's home), after the settle.
fn submit_enter(app: &AppHandle, key: AutoSubmitKey) {
    std::thread::sleep(AUTO_SUBMIT_SETTLE);
    let (sender, receiver) = mpsc::channel();
    let handle = app.clone();
    let scheduled = app.run_on_main_thread(move || {
        // A dropped receiver means the worker stopped waiting; the key was still sent.
        let _ = sender.send(crate::clipboard::send_auto_submit(&handle, key));
    });
    let result = scheduled.map_err(|error| error.to_string()).and_then(|()| {
        let reply = receiver.recv_timeout(MAIN_THREAD_WAIT);
        reply
            .map_err(|error| error.to_string())
            .and_then(|sent| sent)
    });
    if let Err(error) = result {
        log::warn!("Smart paste inserted, but auto-submit failed: {error}");
    }
}

/// 010 hook: late reads are published as an event with identifiers only.
struct EventDiagnostics(AppHandle);
impl DiagnosticSink for EventDiagnostics {
    fn late_read(&mut self, diagnostic: PasteDiagnostic) {
        log::info!(
            "[smart-paste] dictation {}: read after not read",
            diagnostic.dictation_id.0
        );
        if let Err(error) = self.0.emit(LATE_READ_EVENT, diagnostic) {
            log::warn!("Could not publish the smart paste late read: {error}");
        }
    }
}

/// `RunEvent::ExitRequested`: hold the first normal exit while the actor
/// settles off the main thread, then exit again (see `exit_gate`).
pub fn settle_before_exit(app: &AppHandle, code: Option<i32>, api: &tauri::ExitRequestApi) {
    let Some(runtime) = runtime(app) else { return };
    let restart = code == Some(tauri::RESTART_EXIT_CODE);
    match runtime.exit.on_exit_requested(restart) {
        ExitDecision::Proceed => {}
        ExitDecision::Hold => api.prevent_exit(),
        ExitDecision::SettleThenExit => {
            api.prevent_exit();
            let app = app.clone();
            // Not joined on purpose: the main loop must keep running for the settlement.
            std::thread::spawn(move || settle_then_exit(&app, &runtime, code));
        }
    }
}

fn settle_then_exit(app: &AppHandle, runtime: &SmartPasteRuntime, code: Option<i32>) {
    let delay = Duration::from_millis(crate::settings::get_settings(app).paste_delay_ms);
    if let Some(mut actor) = runtime.slot.take() {
        if let Err(error) = actor.shutdown(shutdown_allowance(delay)) {
            log::warn!("Smart paste did not settle before exit: {error:?}");
        }
    }
    runtime.exit.mark_settled();
    app.exit(code.unwrap_or(0));
}
