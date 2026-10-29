//! AC-019 decisions over every stage, observation and retry permit, the
//! outcome each verdict reports, the window budget at its boundaries, and
//! the N28 rules: only web text is verified, a late paste is never re-sent.
use super::*;
use crate::smart_paste::ax_port::{search_web_area, AxError};
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
        (Unreadable, _, _) => Done(VerifyVerdict::Unreadable(UnreadableReason::ReadFailed)),
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
        VerifyVerdict::Unreadable(UnreadableReason::NoReadableField),
        VerifyVerdict::Unreadable(UnreadableReason::NotWebText),
        VerifyVerdict::Unreadable(UnreadableReason::ReadFailed),
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
        VerifyVerdict::Unreadable(UnreadableReason::NotWebText),
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

#[test]
fn only_an_element_proven_to_be_web_text_is_verified() {
    assert_eq!(web_text_scope::<AxError>(Ok(true)), Ok(()));
    let not_web = Err(UnreadableReason::NotWebText);
    assert_eq!(web_text_scope::<AxError>(Ok(false)), not_web);
    assert_eq!(web_text_scope(Err(AxError::Timeout)), not_web);
    assert_eq!(web_text_scope(Err(AxError::NotTrusted)), not_web);
}

#[test]
fn a_change_seen_right_before_the_retry_sends_nothing_and_verifies() {
    assert_eq!(gate_retry(Unchanged), RetryGate::Send);
    assert_eq!(
        gate_retry(Changed),
        RetryGate::Skip(VerifyVerdict::Verified)
    );
    let unreadable = VerifyVerdict::Unreadable(UnreadableReason::ReadFailed);
    assert_eq!(gate_retry(Unreadable), RetryGate::Skip(unreadable));
}

/// Roles of a scripted ancestry, element first; `true` marks the web area.
fn walk(roles: &[bool], max_depth: usize) -> (Result<bool, AxError>, Vec<usize>) {
    let mut visited = Vec::new();
    let found = search_web_area(max_depth, |depth| {
        visited.push(depth);
        Ok(roles.get(depth).copied())
    });
    (found, visited)
}

#[test]
fn the_web_area_is_found_on_the_element_or_any_ancestor_within_the_depth() {
    assert_eq!(walk(&[true], 4), (Ok(true), vec![0]));
    assert_eq!(
        walk(&[false, false, true, false], 4),
        (Ok(true), vec![0, 1, 2])
    );
    assert_eq!(walk(&[false, false], 4), (Ok(false), vec![0, 1, 2]));
}

#[test]
fn the_web_area_search_stops_at_its_depth_and_on_the_first_error() {
    let deep = [vec![false; 6], vec![true]].concat();
    assert_eq!(walk(&deep, 4), (Ok(false), vec![0, 1, 2, 3, 4]));
    let mut visits = 0;
    let found = search_web_area(4, |depth| {
        visits += 1;
        if depth == 1 {
            Err(AxError::Timeout)
        } else {
            Ok(Some(false))
        }
    });
    assert_eq!(found, Err(AxError::Timeout));
    assert_eq!(visits, 2);
}

#[test]
fn the_web_area_search_is_bounded() {
    let limits = WEB_ANCESTRY_LIMITS;
    assert!(limits.max_depth > 0);
    assert!(limits.budget <= VERIFY_WINDOW);
    assert!(limits.per_call <= limits.budget);
}
