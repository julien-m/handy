//! Tauri glue of the insertion step: run the orchestrator on a blocking worker, dispatch the
//! existing insertion method on the main thread with the pre-dispatch guard, then publish one
//! terminal outcome (metadata event `insertion-outcome`, optional consumers) and keep a failed
//! transcript in the recovery slot. History is saved by the caller; its actual result is passed
//! in and never overstated. The transcript never enters an event or a log. Without a session
//! (rollout gate closed, see `rollout`) the insertion of `dev` before 006 runs unchanged.
use super::context::{DestinationSource, DictationId, NoRecoveryConsumer};
use super::guard::DispatchGuard;
use super::insertion::{InsertionPort, InsertionReport, InsertionRequest, Orchestrator};
use super::manager::DestinationManager;
use super::outcome::{
    DestinationFailure, HistoryStatus, InsertionOutcome, InsertionStatus, MethodResult,
    TranscriptRef,
};
use super::recovery::{OutcomeRegistry, RecoverySlot};
use super::workspace::SystemClock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// UI event of a terminal insertion outcome (metadata only).
pub const INSERTION_OUTCOME: &str = "insertion-outcome";
/// Existing UI event of a failed insertion (toast); never carries the transcript.
const PASTE_ERROR: &str = "paste-error";

/// The dictation an insertion belongs to, read when the recording stops.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DictationTicket {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
}
/// Actual history result of this dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryRecord {
    pub status: HistoryStatus,
    pub entry_id: Option<i64>,
}
impl HistoryRecord {
    /// No history write was attempted (cancellation observed before the write).
    pub const NOT_SAVED: Self = Self {
        status: HistoryStatus::NotSaved,
        entry_id: None,
    };
    /// `save_entry` failed.
    pub const FAILED: Self = Self {
        status: HistoryStatus::Failed,
        entry_id: None,
    };
    /// Saved as history entry `entry_id`.
    pub fn saved(entry_id: i64) -> Self {
        Self {
            status: HistoryStatus::Saved,
            entry_id: Some(entry_id),
        }
    }
}

/// The recording stopped: keep the destination through processing and AI, and return the
/// ticket of this dictation (none before the manager is installed).
pub fn ticket_at_stop(app: &AppHandle) -> Option<DictationTicket> {
    if !super::rollout::Rollout::CURRENT.opens_sessions() {
        return None;
    }
    let manager = super::app::manager(app)?;
    manager.mark_processing();
    let snapshot = manager.snapshot()?;
    Some(DictationTicket {
        dictation_id: snapshot.dictation_id,
        destination_generation: snapshot.destination_generation,
    })
}
/// End a dictation that inserts nothing (empty text, transcription error, cancellation).
pub fn abandon(app: &AppHandle) {
    let Some(manager) = super::app::manager(app) else {
        return;
    };
    if let Some(snapshot) = manager.cancel() {
        super::app::emit_changed(app, &snapshot);
    }
}

// @spec FR-002: Async activation, then the configured method — .specs/features/006-destination-application/spec.md#fr-002
/// Insert `text` for `ticket` and publish its outcome; `None` only without a destination manager.
/// The insertion settings are read once here and the same snapshot drives the activation route
/// and the insertion method, so a concurrent settings change cannot split them.
pub async fn deliver(
    app: AppHandle,
    ticket: DictationTicket,
    text: String,
    history: HistoryRecord,
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
) -> Option<InsertionOutcome> {
    let manager = super::app::manager(&app)?;
    let settings = crate::settings::get_settings(&app);
    let request = InsertionRequest {
        dictation_id: ticket.dictation_id,
        destination_generation: ticket.destination_generation,
        paste_method_none: settings.paste_method == crate::settings::PasteMethod::None,
        cancelled,
    };
    let method = MainThreadPaste {
        app: app.clone(),
        text: text.clone(),
        settings,
    };
    let worker_manager = Arc::clone(&manager);
    let joined = tauri::async_runtime::spawn_blocking(move || {
        orchestrator(worker_manager).deliver(request, &method)
    })
    .await;
    let report = joined.unwrap_or_else(|error| worker_failed(&manager, ticket, &error));
    Some(publish(&app, &manager, ticket, report, text, history))
}
/// The insertion worker ended without a report (a panic in the orchestrator or the method): the
/// dictation is reported as an insertion failure, so `publish` keeps its transcript for recovery
/// and ends the session; a dictation already cancelled is reported cancelled.
fn worker_failed(
    manager: &DestinationManager,
    ticket: DictationTicket,
    error: &dyn std::fmt::Display,
) -> InsertionReport {
    let source = manager.fail_insertion(ticket.dictation_id, ticket.destination_generation);
    log::error!("Destination insertion worker failed ({error}); reported as an insertion failure");
    let (status, reason) = match source {
        Some(_) => (
            InsertionStatus::Failed,
            Some(DestinationFailure::InsertionFailed),
        ),
        None => (InsertionStatus::Cancelled, None),
    };
    InsertionReport {
        status,
        reason,
        method_result: None,
        target: None,
        source: source.unwrap_or(DestinationSource::FrontmostAtEnd),
    }
}
/// Insert the final text of a stopped dictation, then end the recording indicator: through its
/// destination session (006, rollout gate open), or exactly as before 006 without a session.
pub async fn insert_final_text(
    app: &AppHandle,
    (ticket, text, history): (Option<DictationTicket>, String, HistoryRecord),
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
) {
    let Some(ticket) = ticket else {
        return paste_as_before(app, text, cancelled);
    };
    let started = std::time::Instant::now();
    match deliver(app.clone(), ticket, text, history, cancelled).await {
        Some(outcome) => report_outcome(app, &outcome, started.elapsed()),
        None => {
            log::error!(
                "Destination manager missing: transcription not inserted (history {:?})",
                history.status
            );
            let _ = app.emit(PASTE_ERROR, ());
        }
    }
    end_indicator(app);
}
/// Payload of `paste-error` for a destination failure: whether the transcript is in history.
/// Never carries the transcript.
#[derive(Clone, Copy, Debug, serde::Serialize)]
struct PasteErrorPayload {
    history_saved: bool,
}
/// Log the outcome truthfully and show the interim failure toast (010 presents reasons later).
fn report_outcome(app: &AppHandle, outcome: &InsertionOutcome, elapsed: std::time::Duration) {
    if outcome.status != InsertionStatus::Failed {
        log::debug!("Insertion finished as {:?} in {elapsed:?}", outcome.status);
        return;
    }
    log::error!(
        "Transcription not inserted ({:?}); retained for recovery: {}; history: {:?}",
        outcome.reason,
        outcome.transcript_ref.is_some(),
        outcome.history_status
    );
    // Double failure (not inserted and not in history): until 010 reads the retained transcript,
    // the user cannot reach it, so the toast says it was not saved either (text never shown).
    let history_saved = outcome.history_status == HistoryStatus::Saved;
    let _ = app.emit(PASTE_ERROR, PasteErrorPayload { history_saved });
}
/// Hide the recording overlay and set the tray back to idle.
fn end_indicator(app: &AppHandle) {
    crate::utils::hide_recording_overlay(app);
    crate::tray::set_tray_state(app, crate::tray::TrayIconState::Idle);
}
/// The insertion of `dev` before 006, unchanged: on the main thread, a cancellation check, the
/// configured method with the settings read there, the `paste-error` toast, then the indicator.
fn paste_as_before(app: &AppHandle, text: String, cancelled: Box<dyn Fn() -> bool + Send + Sync>) {
    let handle = app.clone();
    let paste_time = std::time::Instant::now();
    let scheduled = app.run_on_main_thread(move || {
        if cancelled() {
            log::debug!("Transcription operation cancelled before paste");
            return end_indicator(&handle);
        }
        let settings = crate::settings::get_settings(&handle);
        match crate::utils::paste(text, handle.clone(), &settings, &|| Ok(())) {
            Ok(()) => log::debug!("Text pasted successfully in {:?}", paste_time.elapsed()),
            Err(error) => {
                log::error!("Failed to paste transcription: {error}");
                let _ = handle.emit(PASTE_ERROR, ());
            }
        }
        end_indicator(&handle);
    });
    if let Err(error) = scheduled {
        log::error!("Failed to run paste on main thread: {error:?}");
        end_indicator(app);
    }
}
fn orchestrator(manager: Arc<DestinationManager>) -> Orchestrator {
    Orchestrator {
        manager,
        clock: Arc::new(SystemClock),
        pause: Arc::new(SystemClock),
        recovery: Arc::new(NoRecoveryConsumer),
    }
}

/// The existing insertion method, run on the main thread like before with the settings snapshot
/// of `deliver`; the guard is called by `clipboard::paste` after its configured delay. The
/// worker waits for its result.
struct MainThreadPaste {
    app: AppHandle,
    text: String,
    settings: crate::settings::AppSettings,
}
impl InsertionPort for MainThreadPaste {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        let (sender, receiver) = std::sync::mpsc::channel();
        let (app, text, settings) = (self.app.clone(), self.text.clone(), self.settings.clone());
        let scheduled = self.app.run_on_main_thread(move || {
            let check = || {
                guard
                    .check()
                    .map(|_| ())
                    .map_err(|refusal| refusal.to_string())
            };
            // A dropped receiver means the worker is gone; there is nobody left to inform.
            let _ = sender.send(crate::utils::paste(text, app, &settings, &check));
        });
        let result = match scheduled {
            Ok(()) => receiver
                .recv()
                .unwrap_or_else(|_| Err("Paste did not report a result".to_owned())),
            Err(error) => Err(format!("Failed to run paste on main thread: {error}")),
        };
        MethodResult::Legacy(result)
    }
}

static LAST_OUTCOME: AtomicU64 = AtomicU64::new(0);

// @spec FR-004: Preserve the transcript and report the reason — .specs/features/006-destination-application/spec.md#fr-004
/// Build, conserve and publish the terminal outcome, then end the session.
fn publish(
    app: &AppHandle,
    manager: &DestinationManager,
    ticket: DictationTicket,
    report: InsertionReport,
    text: String,
    history: HistoryRecord,
) -> InsertionOutcome {
    let transcript_ref = (report.status == InsertionStatus::Failed)
        .then(|| retain(app, ticket.dictation_id, text))
        .flatten();
    let outcome = outcome(ticket, report, history, transcript_ref);
    if let Some(registry) = app.try_state::<Arc<OutcomeRegistry>>() {
        registry.publish(&outcome);
    }
    if let Err(error) = app.emit(INSERTION_OUTCOME, &outcome) {
        log::warn!("Could not notify the UI of the insertion outcome: {error}");
    }
    manager.finish(ticket.dictation_id, ticket.destination_generation);
    if let Some(snapshot) = manager.snapshot() {
        super::app::emit_changed(app, &snapshot);
    }
    outcome
}
fn retain(app: &AppHandle, dictation_id: DictationId, text: String) -> Option<TranscriptRef> {
    let slot = app.try_state::<Arc<RecoverySlot>>()?;
    Some(slot.retain(dictation_id, text))
}
fn outcome(
    ticket: DictationTicket,
    report: InsertionReport,
    history: HistoryRecord,
    transcript_ref: Option<TranscriptRef>,
) -> InsertionOutcome {
    let method_outcome = report
        .method_result
        .as_ref()
        .and_then(MethodResult::method_outcome);
    let transaction_id = report
        .method_result
        .as_ref()
        .and_then(MethodResult::transaction_id);
    InsertionOutcome {
        dictation_id: ticket.dictation_id,
        destination_generation: ticket.destination_generation,
        outcome_id: LAST_OUTCOME.fetch_add(1, Ordering::Relaxed) + 1,
        target: report.target,
        destination_source: report.source,
        status: report.status,
        method_outcome,
        transaction_id,
        reason: report.reason,
        history_status: history.status,
        history_entry_id: history.entry_id,
        transcript_ref,
        completed_at: completed_at(),
    }
}
/// Milliseconds since the Unix epoch (0 if the system clock is before it).
fn completed_at() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "delivery_tests.rs"]
mod tests;
