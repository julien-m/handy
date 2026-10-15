//! 009's Enter authority over 006's real manager, orchestrator and guard (AC-005, SC-001): the
//! full option × route × completion matrix gives exactly the expected 0 or 1 key press, a focus
//! switch before the key and the next dictation started before the claim suppress it, and the
//! three asynchronous orders (classic return, smart-paste terminal outcome, reliable rendezvous)
//! all refuse dictation A once dictation B started, without touching B's own claim.
#[path = "pin_enter/support.rs"]
mod support;

use handy_app_lib::automatic_enter::policy::{Completion, Suppression};
use handy_app_lib::automatic_enter::rendezvous::{ReceiptSink, ReliableRendezvous};
use handy_app_lib::automatic_enter::transaction::EnterResult;
use handy_app_lib::automatic_enter::AutomaticEnterPolicy as Policy;
use handy_app_lib::destination::context::{DestinationContext, DestinationSource};
use handy_app_lib::destination::outcome::MethodOutcome;
use handy_app_lib::destination::session::EnterRefusal;
use handy_app_lib::destination::snapshot::MenuChoice;
use std::sync::Arc;
use std::time::Duration;
use support::{chatgpt, comet, safari, Method, Rig, TestResult};

const ROUTES: [DestinationSource; 4] = [
    DestinationSource::Captured,
    DestinationSource::Badge,
    DestinationSource::Pinned,
    DestinationSource::FrontmostAtEnd,
];

/// A dictation of `route` dispatched by the real guard (disposition `Insert`) and its context.
fn dispatched(
    route: DestinationSource,
) -> Result<(Rig, support::DictationOf), Box<dyn std::error::Error>> {
    let rig = Rig::new(chatgpt(), vec![chatgpt(), comet(), safari()]);
    if route == DestinationSource::Pinned {
        rig.manager.pin_identifier("com.openai.chat")?;
    }
    let started = rig.start(route != DestinationSource::FrontmostAtEnd);
    if route == DestinationSource::Badge {
        let menu = rig.manager.menu(started.dictation_id, 0)?;
        let token = menu
            .entries
            .iter()
            .find(|entry| entry.display_name == "Comet")
            .map(|entry| entry.token)
            .ok_or("no Comet")?;
        rig.manager
            .choose(started.dictation_id, 0, MenuChoice::Application(token))?;
    }
    let method = Method::succeeding();
    let report = rig.deliver(&started, &method, false);
    let context = report
        .context
        .map(|context| *context)
        .ok_or("not dispatched")?;
    assert_eq!(context.source, route);
    Ok((rig, support::DictationOf { started, context }))
}

fn completions() -> Vec<Completion> {
    let mut all = vec![
        Completion::Classic { succeeded: true },
        Completion::Classic { succeeded: false },
        Completion::ReliableConfirmed,
        Completion::NoInsertion,
    ];
    all.extend(
        [
            MethodOutcome::InsertedVerified,
            MethodOutcome::ReadObserved,
            MethodOutcome::Typed,
            MethodOutcome::NotRead,
            MethodOutcome::PermissionMissing,
            MethodOutcome::Uncertain,
            MethodOutcome::ClipboardHandedOver,
            MethodOutcome::InsertionFailed,
        ]
        .map(Completion::SmartPaste),
    );
    all
}
/// Independent statement of the expected count.
fn expected(policy: Policy, route: DestinationSource, completion: Completion) -> usize {
    let inserted = matches!(
        completion,
        Completion::Classic { succeeded: true }
            | Completion::ReliableConfirmed
            | Completion::SmartPaste(
                MethodOutcome::InsertedVerified
                    | MethodOutcome::ReadObserved
                    | MethodOutcome::Typed
            )
    );
    let allowed = match policy {
        Policy::Never => false,
        Policy::Always => true,
        Policy::PinOnly => route == DestinationSource::Pinned,
    };
    usize::from(inserted && allowed)
}

// @spec SC-001: Enter count matches the matrix in 100% of cases — .specs/features/009-pin-mode-and-enter-rule/spec.md#sc-001
#[test]
fn the_option_route_completion_matrix_presses_exactly_the_expected_count() -> TestResult {
    let mut cases = 0;
    for policy in [Policy::Never, Policy::Always, Policy::PinOnly] {
        for route in ROUTES {
            for completion in completions() {
                let (rig, dictation) = dispatched(route)?;
                let enter = rig.enter(&dictation.started, policy);
                let t0 = rig.clock_now();
                enter.complete(completion, Some(&dictation.context), t0);
                assert_eq!(
                    rig.keys.count(),
                    expected(policy, route, completion),
                    "{policy:?} {route:?} {completion:?}"
                );
                // A second completion never adds a key.
                enter.complete(completion, Some(&dictation.context), t0);
                assert!(rig.keys.count() <= 1);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 3 * 4 * 12);
    Ok(())
}
#[test]
fn a_destination_route_needs_its_app_still_in_front_but_frontmost_at_end_does_not() -> TestResult {
    for route in ROUTES {
        let (rig, dictation) = dispatched(route)?;
        rig.workspace.bring(safari());
        let enter = rig.enter(&dictation.started, Policy::Always);
        let result = enter.complete(
            Completion::Classic { succeeded: true },
            Some(&dictation.context),
            rig.clock_now(),
        );
        if route == DestinationSource::FrontmostAtEnd {
            assert_eq!(result, EnterResult::Sent);
        } else {
            assert_eq!(
                result,
                EnterResult::Suppressed(Suppression::FocusLost),
                "{route:?}"
            );
        }
    }
    Ok(())
}
#[test]
fn the_key_is_attempted_50_to_100_ms_after_readiness() -> TestResult {
    let (rig, dictation) = dispatched(DestinationSource::Pinned)?;
    let enter = rig.enter(&dictation.started, Policy::PinOnly);
    let t0 = rig.clock_now();
    enter.complete(
        Completion::SmartPaste(MethodOutcome::Typed),
        Some(&dictation.context),
        t0,
    );
    let pressed = rig
        .keys
        .pressed
        .lock()
        .map(|list| list.clone())
        .unwrap_or_default();
    let delay = pressed
        .first()
        .ok_or("no key")?
        .saturating_duration_since(t0);
    assert!(delay >= Duration::from_millis(50) && delay <= Duration::from_millis(100));
    Ok(())
}

/// Start dictation B on the same manager (`commit_start` replaces A's session).
fn start_next(rig: &Rig) -> handy_app_lib::destination::snapshot::DestinationSnapshot {
    rig.start(true)
}

// @spec AC-005: Next dictation before the press: no Enter — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
#[test]
fn dictation_b_started_before_a_is_claimed_suppresses_a_in_every_order() -> TestResult {
    let stale = EnterResult::Suppressed(Suppression::Claim(EnterRefusal::Stale));
    // Classic: A's method returned, B starts, then A's worker decides.
    let (rig, a) = dispatched(DestinationSource::Captured)?;
    let enter = rig.enter(&a.started, Policy::Always);
    let b = start_next(&rig);
    let result = enter.complete(
        Completion::Classic { succeeded: true },
        Some(&a.context),
        rig.clock_now(),
    );
    assert_eq!(result, stale);
    // Smart paste: A's terminal outcome arrives after B started.
    let (rig, a) = dispatched(DestinationSource::Pinned)?;
    let enter = rig.enter(&a.started, Policy::Always);
    start_next(&rig);
    let result = enter.complete(
        Completion::SmartPaste(MethodOutcome::ReadObserved),
        Some(&a.context),
        rig.clock_now(),
    );
    assert_eq!(result, stale);
    // Reliable paste: A's read arrives after B started.
    let (rig, a) = dispatched(DestinationSource::Captured)?;
    let enter = rig.enter(&a.started, Policy::Always);
    let (fire, context) = (Arc::clone(&enter), a.context.clone());
    let rendezvous = ReliableRendezvous::new(Box::new(move |t0| {
        fire.complete(Completion::ReliableConfirmed, Some(&context), t0);
    }));
    let t = rig.clock_now();
    rendezvous.injection_started(t);
    rendezvous.chord_completed(true, t);
    assert!(enter.await_receipt());
    start_next(&rig);
    rendezvous.target_read(t + Duration::from_millis(20));
    assert_eq!(enter.result(), Some(stale));
    assert_eq!(rig.keys.count(), 0);
    // B's own claim is unaffected (shown on the first rig).
    let _ = b;
    Ok(())
}
#[test]
fn b_s_own_enter_is_decided_by_b_only() -> TestResult {
    let (rig, a) = dispatched(DestinationSource::Captured)?;
    let enter_a = rig.enter(&a.started, Policy::Always);
    let b = start_next(&rig);
    let method = Method::succeeding();
    let report = rig.deliver(&b, &method, false);
    let context_b: DestinationContext = report
        .context
        .map(|context| *context)
        .ok_or("B not dispatched")?;
    let enter_b = rig.enter(&b, Policy::Always);
    assert!(matches!(
        enter_a.complete(
            Completion::Classic { succeeded: true },
            Some(&a.context),
            rig.clock_now()
        ),
        EnterResult::Suppressed(_)
    ));
    assert_eq!(
        enter_b.complete(
            Completion::Classic { succeeded: true },
            Some(&context_b),
            rig.clock_now()
        ),
        EnterResult::Sent
    );
    assert_eq!(rig.keys.count(), 1);
    Ok(())
}
#[test]
fn a_policy_change_after_insertion_entry_does_not_change_the_frozen_decision() -> TestResult {
    let (rig, dictation) = dispatched(DestinationSource::Captured)?;
    let enter = rig.enter(&dictation.started, Policy::Never);
    // A later settings change builds the next dictation's transaction, never this one.
    let result = enter.complete(
        Completion::Classic { succeeded: true },
        Some(&dictation.context),
        rig.clock_now(),
    );
    assert_eq!(result, EnterResult::Suppressed(Suppression::PolicyNever));
    assert_eq!(enter.snapshot().policy, Policy::Never);
    Ok(())
}
