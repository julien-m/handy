//! Smart paste through 006's real orchestrator, manager and guard (fake
//! workspace and fake actor): the context handed to the actor, the guard
//! registered for every later destination check, each terminal kind mapped
//! to 006's status and disposition, refusals and lost replies, the insertion
//! without a session, the follow-ups and the route of each gate combination.
use super::*;
use crate::destination::context::{NoRecoveryConsumer, RecoveryDisposition};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::insertion::{InsertionRequest as Delivery, Orchestrator};
use crate::destination::manager::DestinationManager;
use crate::destination::outcome::InsertionStatus;
use crate::destination::workspace::{Foreground, RunningApp};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use crate::smart_paste::destination_bridge::RegisteredDestinations;
use crate::smart_paste::ids::TransactionId;
use crate::smart_paste::request::DestinationGuard;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

const ALL: [PasteOutcomeKind; 9] = [
    PasteOutcomeKind::InsertedVerified,
    PasteOutcomeKind::ReadObserved,
    PasteOutcomeKind::NotRead,
    PasteOutcomeKind::PermissionMissing,
    PasteOutcomeKind::Uncertain,
    PasteOutcomeKind::Typed,
    PasteOutcomeKind::ClipboardHandedOver,
    PasteOutcomeKind::InsertionFailed,
    PasteOutcomeKind::FieldUnchanged,
];

/// What the fake actor saw: the request and whether its destination was
/// current through the registered checks at submission time.
type Seen = (InsertionRequest, bool);

/// Fake actor answering `reply` (or refusing, or dropping the reply).
struct FakeActor {
    checks: Arc<DestinationChecks>,
    reply: Option<PasteOutcomeKind>,
    refuse: bool,
    seen: Mutex<Vec<Seen>>,
}
impl FakeActor {
    fn new(checks: &Arc<DestinationChecks>, reply: Option<PasteOutcomeKind>) -> Arc<Self> {
        let (checks, refuse, seen) = (Arc::clone(checks), false, Mutex::new(Vec::new()));
        Arc::new(Self {
            checks,
            reply,
            refuse,
            seen,
        })
    }
    fn seen(&self) -> Vec<Seen> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
impl InsertionSubmitter for FakeActor {
    fn submit(&self, request: InsertionRequest) -> Result<Receiver<PasteOutcome>, ActorStopped> {
        if self.refuse {
            return Err(ActorStopped);
        }
        let current =
            RegisteredDestinations(Arc::clone(&self.checks)).is_current(&request.destination);
        let (sender, receiver) = mpsc::channel();
        if let Some(kind) = self.reply {
            let outcome = PasteOutcome {
                dictation_id: request.dictation_id,
                transaction_id: Some(TransactionId(5)),
                destination_generation: request.destination.generation,
                path: InsertionPath::ClipboardPaste,
                kind,
            };
            let _ = sender.send(outcome);
        }
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((request, current));
        Ok(receiver)
    }
}

fn textedit() -> RunningApp {
    app(11, "com.apple.TextEdit", "TextEdit")
}
fn safari() -> RunningApp {
    app(22, "com.apple.Safari", "Safari")
}
fn inputs() -> InsertionInputs {
    InsertionInputs {
        transcript: Transcript::new("hello "),
        clipboard_policy: ClipboardPolicy::DontModify,
        paste_delay: Duration::from_millis(60),
        verifies_paste: false,
    }
}
/// A dictation started in TextEdit (`return_enabled`), Safari in front at the end.
fn session(return_enabled: bool) -> (Orchestrator, Delivery, Arc<FakeWorkspace>) {
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(textedit()),
        vec![textedit(), safari()],
    ));
    let clock = Arc::new(FakeClock::new());
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace.clone(),
        clock.clone(),
        platform,
    ));
    let started = manager.commit_start(manager.capture(return_enabled));
    workspace.set_foreground(Foreground::Application(safari()));
    manager.mark_processing();
    let delivery = Delivery {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(|| false),
    };
    let recovery = Arc::new(NoRecoveryConsumer);
    let orchestrator = Orchestrator {
        manager,
        clock: clock.clone(),
        pause: clock,
        recovery,
    };
    (orchestrator, delivery, workspace)
}
fn method(
    actor: &Arc<FakeActor>,
    checks: &Arc<DestinationChecks>,
    calls: &Arc<Mutex<u32>>,
) -> SmartPasteMethod {
    let calls = Arc::clone(calls);
    SmartPasteMethod {
        submitter: actor.clone(),
        checks: Arc::clone(checks),
        inputs: inputs(),
        follow_up: Box::new(move |_, _| *calls.lock().unwrap_or_else(PoisonError::into_inner) += 1),
    }
}

#[test]
fn every_kind_reaches_006_as_its_method_outcome_status_and_disposition() {
    for kind in ALL {
        let (orchestrator, delivery, _) = session(false);
        let checks = Arc::new(DestinationChecks::default());
        let actor = FakeActor::new(&checks, Some(kind));
        let calls = Arc::new(Mutex::new(0));
        let report = orchestrator.deliver(delivery, &method(&actor, &checks, &calls));
        let result = report.method_result.clone();
        let outcome = result.as_ref().and_then(MethodResult::method_outcome);
        assert_eq!(outcome, Some(kind.method_outcome()), "{kind:?}");
        let transaction = result.as_ref().and_then(MethodResult::transaction_id);
        assert_eq!(transaction.as_deref(), Some("5"));
        let failed = report.status == InsertionStatus::Failed;
        assert_eq!(failed, kind.requires_recovery(), "{kind:?}");
        assert_eq!(disposition(&orchestrator), Some(expected_disposition(kind)));
        assert_eq!(*calls.lock().unwrap_or_else(PoisonError::into_inner), 1);
    }
}

/// Business rule (006 outcome table): failures recover, everything else keeps `Insert`.
fn expected_disposition(kind: PasteOutcomeKind) -> RecoveryDisposition {
    if kind.requires_recovery() {
        RecoveryDisposition::Recover
    } else {
        RecoveryDisposition::Insert
    }
}

fn disposition(orchestrator: &Orchestrator) -> Option<RecoveryDisposition> {
    let snapshot = orchestrator.manager.snapshot();
    snapshot.map(|snapshot| snapshot.disposition)
}

#[test]
fn the_actor_gets_006s_target_and_the_guard_answers_only_while_it_runs() {
    let (orchestrator, delivery, _) = session(false);
    let generation = delivery.destination_generation;
    let checks = Arc::new(DestinationChecks::default());
    let actor = FakeActor::new(&checks, Some(PasteOutcomeKind::ReadObserved));
    let calls = Arc::new(Mutex::new(0));
    orchestrator.deliver(delivery, &method(&actor, &checks, &calls));
    let seen = actor.seen();
    let (request, current) = &seen[0];
    assert_eq!(request.destination.pid, Some(22));
    assert_eq!(
        request.destination.bundle_id.as_deref(),
        Some("com.apple.Safari")
    );
    assert_eq!(
        request.destination.generation,
        DestinationGeneration(generation)
    );
    assert_eq!((request.transcript.as_str(), *current), ("hello ", true));
    assert!(!RegisteredDestinations(checks).is_current(&request.destination));
}

#[test]
fn with_destination_return_the_activated_application_is_the_target() {
    let (orchestrator, delivery, workspace) = session(true);
    let checks = Arc::new(DestinationChecks::default());
    let actor = FakeActor::new(&checks, Some(PasteOutcomeKind::InsertedVerified));
    let report = orchestrator.deliver(delivery, &method(&actor, &checks, &Arc::new(Mutex::new(0))));
    assert_eq!(workspace.activation_count(), 1);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(actor.seen()[0].0.destination.pid, Some(11));
}

#[test]
fn a_cancelled_dictation_never_reaches_the_actor() {
    let (orchestrator, mut delivery, _) = session(false);
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    delivery.cancelled = Box::new(move || flag.load(Ordering::SeqCst));
    let checks = Arc::new(DestinationChecks::default());
    let actor = FakeActor::new(&checks, Some(PasteOutcomeKind::ReadObserved));
    let calls = Arc::new(Mutex::new(0));
    let port = method(&actor, &checks, &calls);
    cancelled.store(true, Ordering::SeqCst);
    let report = orchestrator.deliver(delivery, &port);
    assert_eq!(report.status, InsertionStatus::Cancelled);
    assert!(actor.seen().is_empty());
    assert_eq!(*calls.lock().unwrap_or_else(PoisonError::into_inner), 0);
}

fn request() -> InsertionRequest {
    inputs().request(
        DictationId(3),
        DestinationContext::from_foreground(&Foreground::Unreadable, DestinationGeneration(8)),
    )
}
struct Always;
impl RequestCheck for Always {
    fn is_current(&self, _destination: &DestinationContext) -> bool {
        true
    }
}

#[test]
fn a_refused_submission_inserted_nothing_and_a_lost_reply_is_uncertain() {
    let checks = Arc::new(DestinationChecks::default());
    let refusing = FakeActor {
        checks: Arc::clone(&checks),
        reply: None,
        refuse: true,
        seen: Mutex::new(Vec::new()),
    };
    let refused = run_insertion(&refusing, &checks, request(), Arc::new(Always));
    assert_eq!(
        (refused.kind, refused.transaction_id),
        (PasteOutcomeKind::InsertionFailed, None)
    );
    let silent = FakeActor::new(&checks, None);
    let lost = run_insertion(silent.as_ref(), &checks, request(), Arc::new(Always));
    assert_eq!(lost.kind, PasteOutcomeKind::Uncertain);
    assert_eq!(lost.destination_generation, DestinationGeneration(8));
}

#[test]
fn without_a_session_the_application_in_front_is_the_destination() {
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(safari()),
        vec![safari()],
    ));
    let checks = Arc::new(DestinationChecks::default());
    let actor = FakeActor::new(&checks, Some(PasteOutcomeKind::Typed));
    let insertion = ForegroundInsertion {
        submitter: actor.clone(),
        checks: Arc::clone(&checks),
        workspace: workspace.clone(),
        inputs: inputs(),
    };
    let outcome = insertion.insert(Box::new(|| false));
    assert_eq!(
        outcome.map(|outcome| outcome.kind),
        Some(PasteOutcomeKind::Typed)
    );
    let (request, current) = actor.seen()[0].clone();
    assert_eq!((request.destination.pid, current), (Some(22), true));
    assert!(request.destination.generation.0 > LOCAL_GENERATION_BASE);
    assert_eq!(insertion.insert(Box::new(|| true)), None);
    assert_eq!(actor.seen().len(), 1);
}

#[test]
fn only_the_enter_eligible_kinds_submit_and_only_the_hand_over_shows_its_message() {
    for kind in ALL {
        for enabled in [false, true] {
            let follow = follow_ups(kind, enabled);
            assert_eq!(
                follow.auto_submit,
                enabled && kind.is_auto_submit_eligible()
            );
            assert_eq!(
                follow.hand_over_message,
                kind == PasteOutcomeKind::ClipboardHandedOver
            );
            assert_eq!(follow.failure_notice, kind.requires_recovery());
        }
    }
    let hand_over = follow_ups(PasteOutcomeKind::ClipboardHandedOver, true);
    assert!(!hand_over.auto_submit && !hand_over.failure_notice);
}

#[test]
fn each_gate_combination_has_one_route() {
    use InsertionRoute::*;
    for macos in [false, true] {
        for dispatch_wired in [false, true] {
            let rollout = Rollout {
                macos,
                dispatch_wired,
            };
            for has_session in [false, true] {
                let route = insertion_route(rollout, has_session, PasteMethod::SmartPaste);
                let expected = match (macos && dispatch_wired, has_session) {
                    (false, _) => Classic,
                    (true, true) => SmartPasteInSession,
                    (true, false) => SmartPasteAtForeground,
                };
                assert_eq!(route, expected, "{rollout:?} session={has_session}");
                assert_eq!(
                    insertion_route(rollout, has_session, PasteMethod::CtrlV),
                    Classic
                );
            }
        }
    }
}

#[test]
fn this_build_reports_both_gates() {
    // Printed so a proof run shows which gate state it exercised.
    println!(
        "smart_paste_dispatch={} destination_return={} smart_paste_active={}",
        Rollout::CURRENT.dispatch_wired,
        crate::destination::rollout::Rollout::CURRENT.wired,
        Rollout::CURRENT.is_active()
    );
    assert_eq!(
        Rollout::CURRENT.dispatch_wired,
        cfg!(feature = "smart-paste-dispatch")
    );
}

#[test]
fn inputs_follow_the_clipboard_handling_and_paste_delay_settings() {
    let mut settings = crate::settings::get_default_settings();
    settings.clipboard_handling = ClipboardHandling::CopyToClipboard;
    settings.paste_delay_ms = 75;
    let inputs = InsertionInputs::new("text ".into(), &settings);
    assert_eq!(inputs.clipboard_policy, ClipboardPolicy::CopyToClipboard);
    assert_eq!(inputs.paste_delay, Duration::from_millis(75));
    assert_eq!(inputs.transcript.as_str(), "text ");
}
