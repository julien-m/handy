//! The Enter transaction over 006's real manager and a fake workspace, clock and key spy: the
//! 50 ms pause before the claim, the claim before the focus check, focus lost during the pause or
//! after the claim, the next dictation started before the claim (`Stale`), cancellation and
//! recovery, duplicate completions, a failed key never retried, and the hand-over to a reliable
//! rendezvous.
use super::*;
use crate::automatic_enter::rendezvous::ReceiptSink;
use crate::automatic_enter::rendezvous::ReliableRendezvous;
use crate::destination::context::{DestinationSource, RecoveryDisposition, ResolvedTarget};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::outcome::MethodOutcome;
use crate::destination::session::EnterRefusal;
use crate::destination::workspace::RunningApp;
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::time::Duration;

/// Key spy: records the clock time of every press; `fails` makes the press fail.
struct Keys {
    clock: Arc<FakeClock>,
    pressed: Mutex<Vec<(AutoSubmitKey, Instant)>>,
    fails: bool,
}
impl KeyEmitter for Keys {
    fn send(&self, key: AutoSubmitKey) -> Result<(), String> {
        self.pressed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((key, self.clock.now()));
        if self.fails {
            Err("injected input failure".to_owned())
        } else {
            Ok(())
        }
    }
}
/// Pause that switches the foreground half-way, when asked to.
struct SwitchingPause {
    clock: Arc<FakeClock>,
    workspace: Arc<FakeWorkspace>,
    switch_to: Option<RunningApp>,
}
impl Pause for SwitchingPause {
    fn pause(&self, duration: Duration) {
        self.clock.advance(duration);
        if let Some(app) = &self.switch_to {
            self.workspace
                .set_foreground(Foreground::Application(app.clone()));
        }
    }
}

struct Fixture {
    clock: Arc<FakeClock>,
    workspace: Arc<FakeWorkspace>,
    manager: Arc<DestinationManager>,
    keys: Arc<Keys>,
    context: DestinationContext,
}
fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
fn safari() -> RunningApp {
    app(33, "com.apple.Safari", "Safari")
}
/// A dictation of ChatGPT whose guard accepted the dispatch (disposition `Insert`).
fn fixture(source: DestinationSource, fails: bool) -> Fixture {
    let clock = Arc::new(FakeClock::new());
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(chatgpt()),
        vec![chatgpt(), safari()],
    ));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace.clone(),
        clock.clone(),
        platform,
    ));
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.mark_processing();
    manager.freeze(id, generation).expect("frozen");
    manager
        .set_disposition(id, generation, RecoveryDisposition::Insert)
        .expect("insert");
    let keys = Arc::new(Keys {
        clock: clock.clone(),
        pressed: Mutex::new(Vec::new()),
        fails,
    });
    let target = (source != DestinationSource::FrontmostAtEnd).then(|| ResolvedTarget {
        pid: chatgpt().pid,
        identity: chatgpt().identity,
    });
    let context = DestinationContext {
        dictation_id: id,
        destination_generation: generation,
        source,
        target,
    };
    Fixture {
        clock,
        workspace,
        manager,
        keys,
        context,
    }
}
fn transaction(
    fixture: &Fixture,
    policy: AutomaticEnterPolicy,
    switch_to: Option<RunningApp>,
) -> EnterTransaction {
    let frozen = EnterSnapshot {
        dictation_id: fixture.context.dictation_id,
        destination_generation: fixture.context.destination_generation,
        policy,
        key: AutoSubmitKey::CtrlEnter,
    };
    let pause = Arc::new(SwitchingPause {
        clock: fixture.clock.clone(),
        workspace: fixture.workspace.clone(),
        switch_to,
    });
    let ports = EnterPorts {
        claims: fixture.manager.clone(),
        workspace: fixture.workspace.clone(),
        keys: fixture.keys.clone(),
        clock: fixture.clock.clone(),
        pause,
    };
    EnterTransaction::new(frozen, ports)
}
fn presses(fixture: &Fixture) -> Vec<(AutoSubmitKey, Instant)> {
    fixture
        .keys
        .pressed
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}
const CLASSIC: Completion = Completion::Classic { succeeded: true };

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
    fixture.manager.freeze(id, generation).expect("frozen");
    fixture
        .manager
        .set_disposition(id, generation, RecoveryDisposition::Insert)
        .expect("insert");
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
        fixture
            .manager
            .set_disposition(id, generation, next)
            .expect("forward");
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
        };
        assert_eq!(
            enter.after_insertion(&report),
            Some(EnterResult::Suppressed(reason))
        );
        assert!(presses(&fixture).is_empty());
    }
}
