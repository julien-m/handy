//! Tauri glue of the insertion step: run the orchestrator on a blocking worker, dispatch the
//! existing insertion method on the main thread with the pre-dispatch guard, then publish one
//! terminal outcome (metadata event `insertion-outcome`, optional consumers) and keep a failed
//! transcript in the recovery slot. History is saved by the caller; its actual result is passed
//! in and never overstated. The transcript never enters an event or a log.
use super::context::{DictationId, NoRecoveryConsumer};
use super::guard::DispatchGuard;
use super::insertion::{InsertionPort, InsertionReport, InsertionRequest, Orchestrator};
use super::manager::DestinationManager;
use super::outcome::{
    HistoryStatus, InsertionOutcome, InsertionStatus, MethodResult, TranscriptRef,
};
use super::recovery::{OutcomeRegistry, RecoverySlot};
use super::workspace::SystemClock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// UI event of a terminal insertion outcome (metadata only).
pub const INSERTION_OUTCOME: &str = "insertion-outcome";

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
    /// History was not attempted (no WAV).
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
/// Insert `text` for `ticket` and publish its outcome; returns the outcome when one was produced.
pub async fn deliver(
    app: AppHandle,
    ticket: DictationTicket,
    text: String,
    history: HistoryRecord,
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
) -> Option<InsertionOutcome> {
    let manager = super::app::manager(&app)?;
    let paste_method_none =
        crate::settings::get_settings(&app).paste_method == crate::settings::PasteMethod::None;
    let request = InsertionRequest {
        dictation_id: ticket.dictation_id,
        destination_generation: ticket.destination_generation,
        paste_method_none,
        cancelled,
    };
    let method = MainThreadPaste {
        app: app.clone(),
        text: text.clone(),
    };
    let worker_manager = Arc::clone(&manager);
    let report = tauri::async_runtime::spawn_blocking(move || {
        orchestrator(worker_manager).deliver(request, &method)
    })
    .await
    .map_err(|error| log::error!("Destination insertion worker failed: {error}"))
    .ok()?;
    Some(publish(&app, &manager, ticket, report, text, history))
}
/// Insert the final text of a stopped dictation: through its destination (006), or into the
/// application in front as before when no destination session exists. A failure is also
/// reported through the existing `paste-error` event (staged fallback until 010 presents it).
pub async fn insert_final_text(
    app: &AppHandle,
    (ticket, text, history): (Option<DictationTicket>, String, HistoryRecord),
    cancelled: Box<dyn Fn() -> bool + Send + Sync>,
) {
    let started = std::time::Instant::now();
    let Some(ticket) = ticket else {
        return insert_without_destination(app, text, &*cancelled);
    };
    match deliver(app.clone(), ticket, text, history, cancelled).await {
        Some(outcome) if outcome.status != InsertionStatus::Failed => {
            log::debug!(
                "Insertion finished as {:?} in {:?}",
                outcome.status,
                started.elapsed()
            );
        }
        Some(_) | None => {
            log::error!("Transcription was not inserted; it is kept for recovery");
            let _ = app.emit("paste-error", ());
        }
    }
}
/// The previous path, kept for a dictation without session: paste on the main thread.
fn insert_without_destination(app: &AppHandle, text: String, cancelled: &dyn Fn() -> bool) {
    if cancelled() {
        return;
    }
    let handle = app.clone();
    let scheduled = app.run_on_main_thread(move || {
        if let Err(error) = crate::utils::paste(text, handle.clone(), &|| Ok(())) {
            log::error!("Failed to paste transcription: {error}");
            let _ = handle.emit("paste-error", ());
        }
    });
    if let Err(error) = scheduled {
        log::error!("Failed to run paste on main thread: {error:?}");
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

/// The existing insertion method, run on the main thread like before; the guard is called by
/// `clipboard::paste` after its configured delay. The worker waits for its result.
struct MainThreadPaste {
    app: AppHandle,
    text: String,
}
impl InsertionPort for MainThreadPaste {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        let (sender, receiver) = std::sync::mpsc::channel();
        let (app, text) = (self.app.clone(), self.text.clone());
        let scheduled = self.app.run_on_main_thread(move || {
            let check = || {
                guard
                    .check()
                    .map(|_| ())
                    .map_err(|refusal| refusal.to_string())
            };
            // A dropped receiver means the worker is gone; there is nobody left to inform.
            let _ = sender.send(crate::utils::paste(text, app, &check));
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
mod tests {
    use super::*;
    use crate::destination::context::{ApplicationIdentity, DestinationSource};
    use crate::destination::outcome::{DestinationFailure, MethodOutcome};

    fn ticket() -> DictationTicket {
        DictationTicket {
            dictation_id: DictationId(4),
            destination_generation: 9,
        }
    }
    fn report(status: InsertionStatus, method_result: Option<MethodResult>) -> InsertionReport {
        InsertionReport {
            status,
            reason: (status == InsertionStatus::Failed).then_some(DestinationFailure::AppClosed),
            method_result,
            target: Some(ApplicationIdentity::BundleId("com.apple.Notes".to_owned())),
            source: DestinationSource::Captured,
        }
    }

    #[test]
    fn the_outcome_carries_the_actual_history_result_and_the_exact_method_kind() {
        let history = HistoryRecord {
            status: HistoryStatus::Failed,
            entry_id: None,
        };
        let detailed = MethodResult::Detailed {
            outcome: MethodOutcome::ReadObserved,
            transaction_id: Some("tx-1".to_owned()),
        };
        let attempted = report(InsertionStatus::InsertionAttempted, Some(detailed));
        let built = outcome(ticket(), attempted, history, None);
        assert_eq!(
            (built.dictation_id, built.destination_generation),
            (DictationId(4), 9)
        );
        assert_eq!(built.method_outcome, Some(MethodOutcome::ReadObserved));
        assert_eq!(built.transaction_id.as_deref(), Some("tx-1"));
        assert_eq!(
            (built.history_status, built.history_entry_id),
            (HistoryStatus::Failed, None)
        );
    }
    #[test]
    fn a_failed_destination_names_its_target_and_ids_are_unique() {
        let history = HistoryRecord {
            status: HistoryStatus::Saved,
            entry_id: Some(12),
        };
        let first = outcome(
            ticket(),
            report(InsertionStatus::Failed, None),
            history,
            Some(TranscriptRef(3)),
        );
        let second = outcome(
            ticket(),
            report(InsertionStatus::Failed, None),
            history,
            None,
        );
        assert_eq!(first.reason, Some(DestinationFailure::AppClosed));
        assert_eq!(
            first.target,
            Some(ApplicationIdentity::BundleId("com.apple.Notes".to_owned()))
        );
        assert_eq!(
            (first.method_outcome, first.transcript_ref),
            (None, Some(TranscriptRef(3)))
        );
        assert_ne!(first.outcome_id, second.outcome_id);
        let event = serde_json::to_value(&first).unwrap_or_default();
        assert!(event.get("text").is_none() && event.get("transcript").is_none());
    }
}
