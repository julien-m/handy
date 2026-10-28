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
        context: None,
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
// @spec AC-003: Copy and drag carry exactly the inserted text — .specs/features/010-recovery-bubble/spec.md#ac-003
#[test]
fn the_insertion_settings_carry_the_trailing_space_to_the_recovery_slot() {
    for append_trailing_space in [true, false] {
        let (manager, ticket) = processing_manager();
        let mut settings = crate::settings::get_default_settings();
        settings.append_trailing_space = append_trailing_space;
        let slot = RecoverySlot::default();
        // As `deliver` then `publish`: one snapshot, a failed report, the real slot.
        let text = FinalText::of("short safe phrase".to_owned(), &settings);
        let report = report(InsertionStatus::Failed, None);
        let retained = slot.retain_failed(ticket.dictation_id, report.status, text.clone());
        let built = outcome(ticket, report, HistoryRecord::FAILED, retained);
        let reference = built.transcript_ref.unwrap_or(TranscriptRef(0));
        let expected = if append_trailing_space {
            "short safe phrase "
        } else {
            "short safe phrase"
        };
        assert_eq!(slot.read_inserted(reference).as_deref(), Ok(expected));
        assert_eq!(slot.read(reference).as_deref(), Ok("short safe phrase"));
        // A dictation that did not fail keeps nothing and leaves the retained text in place.
        let attempted = InsertionStatus::InsertionAttempted;
        assert_eq!(
            slot.retain_failed(ticket.dictation_id, attempted, text),
            None
        );
        assert_eq!(slot.read_inserted(reference).as_deref(), Ok(expected));
        manager.finish(ticket.dictation_id, ticket.destination_generation);
    }
}
