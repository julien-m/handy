//! The Enter transaction's timing and claim order over 006's real manager: the 50 ms pause before
//! the claim, the claim before the focus check, focus lost during the pause or after the claim,
//! the next dictation started before the claim (`Stale`), cancellation and recovery.
use super::fixture::*;
use super::*;
use crate::destination::context::{DestinationSource, RecoveryDisposition};
use crate::destination::session::EnterRefusal;
use std::time::Duration;

#[test]
fn an_eligible_enter_is_pressed_once_50_to_100_ms_after_t0_with_the_frozen_key() {
    let fixture = fixture(DestinationSource::Pinned, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::PinOnly, None);
    let t0 = fixture.clock.now();
    assert_eq!(
        enter.complete(CLASSIC, Some(&fixture.context), t0),
        EnterResult::Sent
    );
    let pressed = presses(&fixture);
    assert_eq!(pressed.len(), 1);
    let (key, at) = pressed[0];
    assert_eq!(key, AutoSubmitKey::CtrlEnter);
    let delay = at.saturating_duration_since(t0);
    assert!(
        delay >= PRE_ENTER_SETTLE && delay <= Duration::from_millis(100),
        "{delay:?}"
    );
}
#[test]
fn time_already_spent_after_t0_counts_toward_the_pause() {
    let fixture = fixture(DestinationSource::Captured, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
    let t0 = fixture.clock.now();
    fixture.clock.advance(Duration::from_millis(30));
    enter.complete(CLASSIC, Some(&fixture.context), t0);
    let delay = presses(&fixture)[0].1.saturating_duration_since(t0);
    assert_eq!(delay, PRE_ENTER_SETTLE);
}
#[test]
fn focus_lost_during_the_pause_suppresses_and_never_retries() {
    let fixture = fixture(DestinationSource::Badge, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, Some(safari()));
    let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert_eq!(result, EnterResult::Suppressed(Suppression::FocusLost));
    assert!(presses(&fixture).is_empty());
    // The claim was consumed before the check: a retry is refused.
    let (id, generation) = (
        fixture.context.dictation_id,
        fixture.context.destination_generation,
    );
    assert_eq!(
        fixture.manager.claim_enter(id, generation),
        Err(EnterRefusal::AlreadyClaimed)
    );
    let again = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert_eq!(again, EnterResult::Suppressed(Suppression::AlreadyDecided));
}
#[test]
fn the_frontmost_at_end_route_needs_no_focus_check() {
    let fixture = fixture(DestinationSource::FrontmostAtEnd, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, Some(safari()));
    let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert_eq!(result, EnterResult::Sent);
}
#[test]
fn the_next_dictation_started_before_the_claim_makes_this_enter_stale() {
    let fixture = fixture(DestinationSource::Captured, false);
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
    let next = fixture.manager.commit_start(fixture.manager.capture(true));
    let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
    assert_eq!(
        result,
        EnterResult::Suppressed(Suppression::Claim(EnterRefusal::Stale))
    );
    assert!(presses(&fixture).is_empty());
    // The next dictation's own claim is unaffected.
    let (id, generation) = (next.dictation_id, next.destination_generation);
    fixture.manager.mark_processing();
    assert!(fixture.manager.freeze(id, generation).is_ok(), "frozen");
    let insert = fixture
        .manager
        .set_disposition(id, generation, RecoveryDisposition::Insert);
    assert!(insert.is_ok(), "insert");
    assert!(fixture.manager.claim_enter(id, generation).is_ok());
}
#[test]
fn recovery_or_cancellation_before_the_claim_suppresses() {
    for next in [RecoveryDisposition::Recover, RecoveryDisposition::Cancelled] {
        let fixture = fixture(DestinationSource::Captured, false);
        let (id, generation) = (
            fixture.context.dictation_id,
            fixture.context.destination_generation,
        );
        let forward = fixture.manager.set_disposition(id, generation, next);
        assert!(forward.is_ok(), "forward");
        let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
        let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
        assert_eq!(
            result,
            EnterResult::Suppressed(Suppression::Claim(EnterRefusal::Disposition(next)))
        );
    }
}
#[test]
fn a_cancellation_after_publication_makes_a_late_reliable_claim_stale() {
    let fixture = fixture(DestinationSource::Captured, false);
    let (id, generation) = (
        fixture.context.dictation_id,
        fixture.context.destination_generation,
    );
    fixture.manager.finish(id, generation);
    fixture.manager.cancel();
    let enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
    let result = enter.complete(
        Completion::ReliableConfirmed,
        Some(&fixture.context),
        fixture.clock.now(),
    );
    assert_eq!(
        result,
        EnterResult::Suppressed(Suppression::Claim(EnterRefusal::Stale))
    );
}
#[test]
fn the_emitter_outcome_is_reported_as_is_and_gets_the_expected_destination() {
    struct Scripted(KeyOutcome, Mutex<Vec<Option<ResolvedTarget>>>);
    impl KeyEmitter for Scripted {
        fn send(&self, _key: AutoSubmitKey, expected: Option<&ResolvedTarget>) -> KeyOutcome {
            self.1
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(expected.cloned());
            self.0.clone()
        }
    }
    let cases = [
        (
            KeyOutcome::FocusLost,
            EnterResult::Suppressed(Suppression::FocusLost),
        ),
        (
            KeyOutcome::Late,
            EnterResult::Suppressed(Suppression::KeyLate),
        ),
        (KeyOutcome::Sent, EnterResult::Sent),
    ];
    for (outcome, expected) in cases {
        let fixture = fixture(DestinationSource::Pinned, false);
        let keys = Arc::new(Scripted(outcome, Mutex::new(Vec::new())));
        let mut enter = transaction(&fixture, AutomaticEnterPolicy::Always, None);
        enter.ports.keys = keys.clone();
        let result = enter.complete(CLASSIC, Some(&fixture.context), fixture.clock.now());
        assert_eq!(result, expected);
        let asked = keys
            .1
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        assert_eq!(
            asked,
            vec![fixture.context.target.clone()],
            "checked on the input thread"
        );
    }
}
