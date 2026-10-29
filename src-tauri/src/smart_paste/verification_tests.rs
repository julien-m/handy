//! AC-019 decisions over every stage, observation and retry permit, the
//! outcome each verdict reports, and the window budget at its boundaries.
use super::*;
use FieldObservation::{Changed, Unchanged, Unreadable};
use VerifyStage::{AfterRetry, First};

const STAGES: [VerifyStage; 2] = [First, AfterRetry];
const OBSERVATIONS: [FieldObservation; 3] = [Changed, Unchanged, Unreadable];
const PERMITS: [RetryPermit; 3] = [
    RetryPermit::Allowed,
    RetryPermit::ClipboardChanged,
    RetryPermit::DestinationLost,
];

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// The spec table, written independently of `next_step`.
fn expected(stage: VerifyStage, observation: FieldObservation, permit: RetryPermit) -> VerifyStep {
    use VerifyStep::{Done, Retry};
    match (observation, stage, permit) {
        (Changed, First, _) => Done(VerifyVerdict::Verified),
        (Changed, AfterRetry, _) => Done(VerifyVerdict::RetriedVerified),
        (Unreadable, _, _) => Done(VerifyVerdict::Unreadable),
        (Unchanged, First, RetryPermit::Allowed) => Retry,
        (Unchanged, First, refused) => {
            Done(VerifyVerdict::Failed(VerifyFailure::RetryRefused(refused)))
        }
        (Unchanged, AfterRetry, _) => {
            Done(VerifyVerdict::Failed(VerifyFailure::UnchangedAfterRetry))
        }
    }
}

#[test]
fn every_stage_observation_and_permit_matches_the_spec_table() {
    for stage in STAGES {
        for observation in OBSERVATIONS {
            for permit in PERMITS {
                assert_eq!(
                    next_step(stage, observation, || permit),
                    expected(stage, observation, permit),
                    "{stage:?} {observation:?} {permit:?}"
                );
            }
        }
    }
}

#[test]
fn the_retry_permit_is_asked_only_when_a_retry_is_due() {
    for stage in STAGES {
        for observation in OBSERVATIONS {
            let mut asked = false;
            let _ = next_step(stage, observation, || {
                asked = true;
                RetryPermit::Allowed
            });
            let is_due = stage == First && observation == Unchanged;
            assert_eq!(asked, is_due, "{stage:?} {observation:?}");
        }
    }
}

#[test]
fn only_a_failure_turns_read_observed_into_field_unchanged() {
    let failures = [
        VerifyFailure::RetryRefused(RetryPermit::ClipboardChanged),
        VerifyFailure::RetryRefused(RetryPermit::DestinationLost),
        VerifyFailure::RetryChordFailed,
        VerifyFailure::UnchangedAfterRetry,
    ];
    for failure in failures {
        let kind = VerifyVerdict::Failed(failure).outcome_kind();
        assert_eq!(kind, PasteOutcomeKind::FieldUnchanged);
        assert!(kind.requires_recovery());
        assert!(!kind.is_auto_submit_eligible());
    }
    for verdict in [
        VerifyVerdict::Verified,
        VerifyVerdict::RetriedVerified,
        VerifyVerdict::Unreadable,
    ] {
        assert_eq!(verdict.outcome_kind(), PasteOutcomeKind::ReadObserved);
    }
}

#[test]
fn labels_match_the_debug_log_contract() {
    let labels = [
        VerifyVerdict::Verified,
        VerifyVerdict::RetriedVerified,
        VerifyVerdict::Failed(VerifyFailure::RetryChordFailed),
        VerifyVerdict::Unreadable,
    ]
    .map(VerifyVerdict::label);
    assert_eq!(
        labels,
        ["Verified", "RetriedVerified", "Failed", "Unreadable"]
    );
}

#[test]
fn an_observation_compares_with_the_value_before_the_paste() {
    let before = utf16("Posez une question");
    assert_eq!(
        FieldObservation::of::<()>(&before, Ok(before.clone())),
        Unchanged
    );
    assert_eq!(
        FieldObservation::of::<()>(&before, Ok(utf16("Posez une question hello"))),
        Changed
    );
    assert_eq!(FieldObservation::of(&before, Err(())), Unreadable);
}

#[test]
fn only_a_change_settles_a_window_early() {
    assert!(VerifyWindow::is_settled_by(Changed));
    assert!(!VerifyWindow::is_settled_by(Unchanged));
    assert!(!VerifyWindow::is_settled_by(Unreadable));
}

#[test]
fn the_window_polls_at_the_interval_and_stops_at_its_end() {
    let start = Instant::now();
    let window = VerifyWindow::starting_at(start);
    let poll = VERIFY_POLL_INTERVAL;
    assert_eq!(window.pause_before_next_read(start), Some(poll));
    let near_end = start + VERIFY_WINDOW - poll / 2;
    assert_eq!(window.pause_before_next_read(near_end), Some(poll / 2));
    assert_eq!(window.pause_before_next_read(start + VERIFY_WINDOW), None);
    assert_eq!(
        window.pause_before_next_read(start + VERIFY_WINDOW + poll),
        None
    );
}
