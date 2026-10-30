//! The Enter transaction's decisions over 006's real manager: a failed key never retried,
//! duplicate completions, the option and the pin source, return off with and without a pin, the
//! hand-over to a reliable rendezvous and the mapping of the orchestrator's reports.
use super::fixture::*;
use super::*;
use crate::automatic_enter::rendezvous::{ReceiptSink, ReliableRendezvous};
use crate::destination::context::DestinationSource;
use crate::destination::outcome::MethodOutcome;
use std::time::Duration;

#[test]
fn a_failed_key_is_reported_and_never_sent_again() {
    let fixture = fixture(DestinationSource::Captured, true);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
    let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert!(matches!(result, EnterResult::EmissionFailed(_)));
    enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert_eq!(presses(&fixture).len(), 1);
}
#[test]
fn concurrent_duplicate_completions_press_at_most_once() {
    let fixture = fixture(DestinationSource::Captured, false);
    let enter = Arc::new(transaction(&fixture, AutomaticEnterPolicy::Always, None));
    let threads = (0..8)
        .map(|_| {
            let (enter, context, t0) =
                (enter.clone(), fixture.context.clone(), fixture.clock.now());
            std::thread::spawn(move || enter.complete(CLASSIC, Some(&context), t0))
        })
        .collect::<Vec<_>>();
    let sent = threads
        .into_iter()
        .filter_map(|thread| thread.join().ok())
        .filter(|result| *result == EnterResult::Sent)
        .count();
    assert_eq!(sent, 1);
    assert_eq!(presses(&fixture).len(), 1);
}
#[test]
fn never_and_pin_only_without_the_pin_source_suppress_before_any_claim() {
    for (policy, source, reason) in [
        (
            AutomaticEnterPolicy::Never,
            DestinationSource::Pinned,
            Suppression::PolicyNever,
        ),
        (
            AutomaticEnterPolicy::PinOnly,
            DestinationSource::Captured,
            Suppression::NotPinned,
        ),
        (
            AutomaticEnterPolicy::PinOnly,
            DestinationSource::Badge,
            Suppression::NotPinned,
        ),
        (
            AutomaticEnterPolicy::PinOnly,
            DestinationSource::FrontmostAtEnd,
            Suppression::NotPinned,
        ),
    ] {
        let fixture = fixture(source, false);
        let enter = transaction(&fixture, policy, None);
        let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
        assert_eq!(result, EnterResult::Suppressed(reason));
        let (id, generation) = (
            fixture.context.dictation_id,
            fixture.context.destination_generation,
        );
        assert!(
            fixture.manager.claim_enter(id, generation).is_ok(),
            "claim untouched"
        );
    }
}
#[test]
fn return_off_with_a_pin_checks_focus_and_without_a_pin_does_not() {
    let pinned = fixture(DestinationSource::Pinned, false);
    let enter = transaction(&pinned, AutomaticEnterPolicy::Always, Some(safari()));
    let result = enter.complete(CLASSIC, Some(&pinned.context), pinned.clock.now());
    assert_eq!(result, EnterResult::Suppressed(Suppression::FocusLost));
    let unpinned = fixture(DestinationSource::FrontmostAtEnd, false);
    let enter = transaction(&unpinned, AutomaticEnterPolicy::Always, Some(safari()));
    let result = enter.complete(CLASSIC, Some(&unpinned.context), unpinned.clock.now());
    assert_eq!(result, EnterResult::Sent);
}
#[test]
fn a_reliable_transfer_hands_the_decision_to_its_rendezvous() {
    let fixture = fixture(DestinationSource::Captured, false);
    let enter = Arc::new(transaction(&fixture, AutomaticEnterPolicy::Always, None));
    let (fire, context) = (enter.clone(), fixture.context.clone());
    let rendezvous = ReliableRendezvous::new(Box::new(move |t0| {
        fire.complete(Completion::ReliableConfirmed, Some(&context), t0);
    }));
    let start = fixture.clock.now();
    rendezvous.injection_started(start);
    rendezvous.chord_completed(true, start);
    assert!(enter.await_receipt());
    let report = InsertionReport {
        status: crate::destination::outcome::InsertionStatus::InsertionAttempted,
        reason: None,
        method_result: Some(MethodResult::Legacy(Ok(()))),
        target: None,
        source: DestinationSource::Captured,
        context: Some(Box::new(fixture.context.clone())),
        addressed: false,
    };
    assert_eq!(
        enter.after_insertion(&report),
        None,
        "not a classic success"
    );
    assert!(presses(&fixture).is_empty());
    rendezvous.target_read(start + Duration::from_millis(30));
    assert_eq!(enter.result(), Some(EnterResult::Sent));
    assert_eq!(presses(&fixture).len(), 1);
}
#[test]
fn after_insertion_maps_skipped_failed_and_smart_paste_reports() {
    let cases = [
        (
            crate::destination::outcome::InsertionStatus::Skipped,
            None,
            Suppression::PasteMethodNone,
        ),
        (
            crate::destination::outcome::InsertionStatus::Failed,
            Some(MethodResult::Legacy(Err("refused".to_owned()))),
            Suppression::MethodFailed,
        ),
        (
            crate::destination::outcome::InsertionStatus::Failed,
            Some(MethodResult::Detailed {
                outcome: MethodOutcome::NotRead,
                transaction_id: None,
            }),
            Suppression::SmartPasteOutcome(MethodOutcome::NotRead),
        ),
    ];
    for (status, method_result, reason) in cases {
        let fixture = fixture(DestinationSource::Captured, false);
        let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
        let report = InsertionReport {
            status,
            reason: None,
            method_result,
            target: None,
            source: DestinationSource::Captured,
            context: Some(Box::new(fixture.context.clone())),
            addressed: false,
        };
        assert_eq!(
            enter.after_insertion(&report),
            Some(EnterResult::Suppressed(reason))
        );
        assert!(presses(&fixture).is_empty());
    }
}

// @spec AC-007: No automatic Enter after a terminal delivery — .specs/features/013-default-input-per-app/spec.md#ac-007
#[test]
fn a_terminal_delivery_never_presses_enter_even_when_always() {
    let fixture = fixture(DestinationSource::Pinned, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
    let report = InsertionReport {
        status: crate::destination::outcome::InsertionStatus::InsertionAttempted,
        reason: None,
        method_result: Some(MethodResult::Legacy(Ok(()))),
        target: None,
        source: DestinationSource::Pinned,
        context: Some(Box::new(fixture.context.clone())),
        addressed: true,
    };
    assert_eq!(
        enter.after_insertion(&report),
        Some(EnterResult::Suppressed(Suppression::AddressedTerminal))
    );
    assert!(presses(&fixture).is_empty());
}
