//! Enter decisions over the real insertion boundary (009 AC-005, SC-001): smart paste as 006's
//! insertion method with every terminal outcome injected through its delivered submitter port
//! and the production follow-up wiring to 009's transaction; the classic method decided after
//! the orchestrator returns; the reliable rendezvous in both signal orders, with a chord failure
//! after an early read and with duplicate reads; and a 010 `Recover` verdict before insertion.
#[path = "pin_enter/support.rs"]
mod support;

use handy_app_lib::automatic_enter::policy::{Completion, Suppression};
use handy_app_lib::automatic_enter::rendezvous::{ReceiptSink, ReliableRendezvous};
use handy_app_lib::automatic_enter::transaction::{EnterResult, EnterTransaction};
use handy_app_lib::automatic_enter::AutomaticEnterPolicy as Policy;
use handy_app_lib::destination::context::{
    DestinationContext, PreInsertionVerdict, RecoveryPort, RecoveryReason,
};
use handy_app_lib::destination::insertion::{InsertionRequest, Orchestrator};
use handy_app_lib::destination::outcome::{InsertionStatus, MethodResult};
use handy_app_lib::smart_paste::actor::ActorStopped;
use handy_app_lib::smart_paste::destination_bridge::DestinationChecks;
use handy_app_lib::smart_paste::dispatch::{InsertionInputs, InsertionSubmitter, SmartPasteMethod};
use handy_app_lib::smart_paste::ids::{TransactionId, Transcript};
use handy_app_lib::smart_paste::outcome::{InsertionPath, PasteOutcome, PasteOutcomeKind};
use handy_app_lib::smart_paste::request::InsertionRequest as PasteRequest;
use handy_app_lib::smart_paste::settlement::ClipboardPolicy;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};
use support::{chatgpt, safari, Method, Rig, TestResult};

/// 007's actor replaced by a submitter answering one scripted terminal outcome.
struct Answers(PasteOutcomeKind);
impl InsertionSubmitter for Answers {
    fn submit(&self, request: PasteRequest) -> Result<Receiver<PasteOutcome>, ActorStopped> {
        let (sender, receiver) = channel();
        let outcome = PasteOutcome {
            dictation_id: request.dictation_id,
            transaction_id: Some(TransactionId(1)),
            destination_generation: request.destination.generation,
            path: InsertionPath::ClipboardPaste,
            kind: self.0,
        };
        sender.send(outcome).map_err(|_| ActorStopped)?;
        Ok(receiver)
    }
}
/// Smart paste wired as `smart_paste::app::session_method` wires it: the terminal outcome and
/// 006's context go to the transaction when the outcome is published.
fn smart_paste(kind: PasteOutcomeKind, enter: Arc<EnterTransaction>) -> SmartPasteMethod {
    SmartPasteMethod {
        submitter: Arc::new(Answers(kind)),
        checks: Arc::new(DestinationChecks::default()),
        inputs: InsertionInputs {
            transcript: Transcript::new("safe test phrase"),
            clipboard_policy: ClipboardPolicy::DontModify,
            paste_delay: Duration::from_millis(10),
        },
        follow_up: Box::new(move |outcome, context| {
            let completion = Completion::SmartPaste(outcome.kind.method_outcome());
            enter.complete(completion, Some(context), Instant::now());
        }),
    }
}
const KINDS: [(PasteOutcomeKind, usize); 8] = [
    (PasteOutcomeKind::InsertedVerified, 1),
    (PasteOutcomeKind::ReadObserved, 1),
    (PasteOutcomeKind::Typed, 1),
    (PasteOutcomeKind::NotRead, 0),
    (PasteOutcomeKind::PermissionMissing, 0),
    (PasteOutcomeKind::Uncertain, 0),
    (PasteOutcomeKind::ClipboardHandedOver, 0),
    (PasteOutcomeKind::InsertionFailed, 0),
];

// @spec AC-005: Enter only after a successful insertion — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
#[test]
fn all_eight_smart_paste_outcomes_give_the_expected_enter_count() -> TestResult {
    for (kind, expected) in KINDS {
        let rig = Rig::new(chatgpt(), vec![chatgpt()]);
        let started = rig.start(true);
        let enter = rig.enter(&started, Policy::Always);
        let method = smart_paste(kind, Arc::clone(&enter));
        let report = rig.deliver(&started, &method, false);
        // 006's worker then runs the legacy decision: smart paste already decided.
        assert_eq!(enter.after_insertion(&report), None, "{kind:?}");
        assert_eq!(rig.keys.count(), expected, "{kind:?}");
    }
    Ok(())
}
#[test]
fn a_classic_success_is_decided_after_the_orchestrator_and_a_failure_is_not() -> TestResult {
    for (result, expected) in [
        (MethodResult::Legacy(Ok(())), 1),
        (MethodResult::Legacy(Err("paste failed".to_owned())), 0),
    ] {
        let rig = Rig::new(chatgpt(), vec![chatgpt()]);
        let started = rig.start(true);
        let enter = rig.enter(&started, Policy::Always);
        let report = rig.deliver(&started, &Method::returning(result), false);
        enter.after_insertion(&report);
        assert_eq!(rig.keys.count(), expected);
    }
    Ok(())
}
#[test]
fn paste_method_none_and_a_failed_destination_never_send_enter() -> TestResult {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let started = rig.start(true);
    let enter = rig.enter(&started, Policy::Always);
    let report = rig.deliver(&started, &Method::succeeding(), true);
    assert_eq!(report.status, InsertionStatus::Skipped);
    assert_eq!(
        enter.after_insertion(&report),
        Some(EnterResult::Suppressed(Suppression::PasteMethodNone))
    );
    let closed = Rig::new(safari(), vec![safari()]);
    closed.manager.pin_identifier("com.apple.Safari")?;
    closed.workspace.set_running(vec![]);
    let started = closed.start(true);
    let enter = closed.enter(&started, Policy::Always);
    let report = closed.deliver(&started, &Method::succeeding(), false);
    enter.after_insertion(&report);
    assert_eq!(closed.keys.count(), 0);
    Ok(())
}

/// The 010 pre-insertion check asking to recover.
struct RecoverInstead;
impl RecoveryPort for RecoverInstead {
    fn before_insertion(&self, _context: &DestinationContext) -> PreInsertionVerdict {
        PreInsertionVerdict::Recover(RecoveryReason::NoTextField)
    }
}
#[test]
fn a_recover_verdict_of_010_suppresses_enter() -> TestResult {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let started = rig.start(true);
    let enter = rig.enter(&started, Policy::Always);
    rig.manager.mark_processing();
    let orchestrator = Orchestrator {
        recovery: Arc::new(RecoverInstead),
        ..rig.orchestrator()
    };
    let request = InsertionRequest {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(|| false),
    };
    let report = orchestrator.deliver(request, &Method::succeeding());
    enter.after_insertion(&report);
    assert_eq!(rig.keys.count(), 0);
    Ok(())
}

/// A reliable transfer of a dispatched dictation, its rendezvous and transaction.
fn reliable(
    rig: &Rig,
) -> Result<(Arc<EnterTransaction>, ReliableRendezvous), Box<dyn std::error::Error>> {
    let started = rig.start(true);
    let enter = rig.enter(&started, Policy::Always);
    let report = rig.deliver(&started, &Method::succeeding(), false);
    let context = report.context.as_deref().cloned().ok_or("not dispatched")?;
    let fire = Arc::clone(&enter);
    let rendezvous = ReliableRendezvous::new(Box::new(move |t0| {
        fire.complete(Completion::ReliableConfirmed, Some(&context), t0);
    }));
    let start = rig.clock_now();
    rendezvous.injection_started(start);
    assert!(enter.await_receipt());
    assert_eq!(
        enter.after_insertion(&report),
        None,
        "the method return is not a success"
    );
    Ok((enter, rendezvous))
}
#[test]
fn reliable_paste_sends_one_enter_in_either_order_and_none_without_a_read() -> TestResult {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let (_, read_first) = reliable(&rig)?;
    let t = rig.clock_now();
    read_first.target_read(t + Duration::from_millis(30));
    assert_eq!(rig.keys.count(), 0, "a read alone is not enough");
    read_first.chord_completed(true, t + Duration::from_millis(100));
    assert_eq!(rig.keys.count(), 1);
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let (_, success_first) = reliable(&rig)?;
    success_first.chord_completed(true, rig.clock_now());
    assert_eq!(rig.keys.count(), 0);
    for offset in [10, 20, 30] {
        success_first.target_read(rig.clock_now() + Duration::from_millis(offset));
    }
    assert_eq!(rig.keys.count(), 1, "duplicate reads press once");
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let (enter, no_read) = reliable(&rig)?;
    no_read.chord_completed(true, rig.clock_now());
    assert_eq!((rig.keys.count(), enter.result()), (0, None));
    Ok(())
}
#[test]
fn a_chord_failure_after_an_early_read_suppresses_enter_despite_the_outer_ok() -> TestResult {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let (enter, rendezvous) = reliable(&rig)?;
    let t = rig.clock_now();
    rendezvous.target_read(t + Duration::from_millis(20));
    rendezvous.chord_completed(false, t + Duration::from_millis(100));
    rendezvous.target_read(t + Duration::from_millis(150));
    assert_eq!((rig.keys.count(), enter.result()), (0, None));
    Ok(())
}
