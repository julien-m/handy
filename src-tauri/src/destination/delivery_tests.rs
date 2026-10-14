//! Outcome construction of the insertion glue: the actual history result, the exact method
//! kind, unique outcome ids and a metadata-only event.
use super::*;
use crate::destination::context::ApplicationIdentity;
use crate::destination::outcome::MethodOutcome;

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

fn processing_manager() -> (DestinationManager, DictationTicket) {
    use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
    use crate::destination::workspace::Foreground;
    use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
    let notes = app(5, "com.apple.Notes", "Notes");
    let workspace = FakeWorkspace::new(Foreground::Application(notes.clone()), vec![notes]);
    let manager = DestinationManager::new(
        Arc::new(workspace),
        Arc::new(FakeClock::new()),
        PlatformCapabilities::only(&[PlatformCapability::Destination]),
    );
    let started = manager.commit_start(manager.capture(true));
    manager.mark_processing();
    let ticket = DictationTicket {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
    };
    (manager, ticket)
}
#[test]
fn a_failed_worker_is_an_insertion_failure_whose_transcript_is_kept() {
    let (manager, ticket) = processing_manager();
    let report = worker_failed(&manager, ticket, &"worker panicked");
    assert_eq!(
        (report.status, report.reason, report.source),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::InsertionFailed),
            DestinationSource::Captured
        )
    );
    // `publish` retains the transcript of every failed report and ends the session.
    let retained = (report.status == InsertionStatus::Failed).then_some(TranscriptRef(1));
    let built = outcome(ticket, report, HistoryRecord::FAILED, retained);
    assert_eq!(built.transcript_ref, Some(TranscriptRef(1)));
    manager.finish(ticket.dictation_id, ticket.destination_generation);
    assert!(!manager.is_current(ticket.dictation_id, ticket.destination_generation));
}
#[test]
fn a_failed_worker_of_a_cancelled_dictation_is_reported_cancelled() {
    let (manager, ticket) = processing_manager();
    manager.cancel();
    let report = worker_failed(&manager, ticket, &"worker panicked");
    assert_eq!(
        (report.status, report.reason),
        (InsertionStatus::Cancelled, None)
    );
}
