//! 011 with 010 and 009 at 006's real insertion boundary (plan Step 4, spec AC-004, AC-005,
//! SC-002): the production `RecoveryService` answers `Insert` for a verified input and
//! `Recover(field_return_failed)` for a diverted return, both without any accessibility query,
//! and classifies as delivered for an unchanged attempt or no remembered input; the bubble says
//! "Insertion failed" with "nothing was sent", on every platform that offers it, or 006's
//! `paste-error` once when it cannot be presented; 009's Enter is unchanged (none after a
//! recovery, none when another application comes forward before the press).
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;
#[path = "recovery/support.rs"]
mod support;

use destination::notes;
use fake::Write;
use handy_app_lib::automatic_enter::AutomaticEnterPolicy as Policy;
use handy_app_lib::destination::context::{DestinationContext, DictationId, RecoveryPort};
use handy_app_lib::destination::field_check::FieldCheck;
use handy_app_lib::destination::insertion::InsertionReport;
use handy_app_lib::destination::outcome::{DestinationFailure, HistoryStatus, InsertionStatus};
use handy_app_lib::destination::recovery::{ConsumerAck, OutcomeConsumer};
use handy_app_lib::recovery::types::{BubbleReason, ReasonDetail, RecoveryPlatform};
use rig::{FieldRig, PasteSpy, INPUT, OTHER};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use support::{Bench, Focus, TestResult};

const PID: u32 = 22;

/// One dictation from Comet's input, Notes in front at the end, delivered with 010's check and
/// 009's Enter (Always); `script` changes the world before the insertion.
fn dictate(
    field: &FieldRig,
    bench: &Bench,
    spy: &PasteSpy,
    script: impl FnOnce(&FieldRig),
) -> InsertionReport {
    bench.service.dictation_starting(true);
    let started = field.start(true);
    bench.service.dictation_started(started.dictation_id);
    let enter = field.rig.enter(&started, Policy::Always);
    field.rig.workspace.bring(notes());
    script(field);
    let recovery = bench.service.clone();
    let report = field.deliver(&started, spy, recovery, &Arc::new(AtomicBool::new(false)));
    enter.after_insertion(&report);
    report
}

// @spec AC-005: Verified counts editable; Enter rule unchanged — .specs/features/011-dictation-field-return/spec.md#ac-005
#[test]
fn a_verified_input_inserts_without_any_010_query_and_enter_follows_009() {
    let field = FieldRig::comet();
    let focus = Focus::role("AXList", PID);
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus.clone());
    let spy = PasteSpy::new(&field.world);
    let report = dictate(&field, &bench, &spy, |_| {});
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(spy.pasted(), vec![Some(INPUT)]);
    assert_eq!(
        focus.queries(),
        0,
        "no 'No text field is focused' bubble, no AX query"
    );
    assert_eq!(
        field.rig.keys.count(),
        1,
        "Always: Enter sent with the destination in front"
    );
}
#[test]
fn another_application_in_front_before_enter_still_gets_no_enter() {
    let field = FieldRig::comet();
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, Focus::role("AXTextArea", PID));
    let workspace = Arc::clone(&field.rig.workspace);
    let spy = PasteSpy::new(&field.world);
    bench.service.dictation_starting(true);
    let started = field.start(true);
    bench.service.dictation_started(started.dictation_id);
    let enter = field.rig.enter(&started, Policy::Always);
    let report = field.deliver(
        &started,
        &spy,
        bench.service.clone(),
        &Arc::new(AtomicBool::new(false)),
    );
    workspace.bring(notes());
    enter.after_insertion(&report);
    assert_eq!(spy.pasted(), vec![Some(INPUT)]);
    assert_eq!(
        field.rig.keys.count(),
        0,
        "009's foreground check before the press"
    );
}
// @spec AC-004: Diverted return shows the 010 bubble — .specs/features/011-dictation-field-return/spec.md#ac-004
#[test]
fn a_diverted_return_recovers_without_any_010_query_and_sends_no_enter() {
    let field = FieldRig::comet();
    field
        .world
        .edit(INPUT, |input| input.select = Write::Refuse);
    let focus = Focus::role("AXTextArea", PID);
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus.clone());
    let spy = PasteSpy::new(&field.world);
    let report = dictate(&field, &bench, &spy, |_| {});
    assert!(spy.pasted().is_empty());
    assert_eq!(report.reason, Some(DestinationFailure::FieldReturnFailed));
    assert_eq!(focus.queries(), 0);
    assert_eq!(field.rig.keys.count(), 0, "no Enter after (b)");
}
#[test]
fn an_unchanged_attempt_or_no_remembered_input_is_classified_by_010_as_delivered() {
    for unchanged in [true, false] {
        let field = FieldRig::comet();
        if unchanged {
            field.world.edit(INPUT, |input| input.focus = Write::Refuse);
        } else {
            field.world.edit(INPUT, |input| input.settable = false);
        }
        let focus = Focus::role("AXList", PID);
        let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus.clone());
        let spy = PasteSpy::new(&field.world);
        let report = dictate(&field, &bench, &spy, |field| {
            field.world.focus(PID, Ok(OTHER))
        });
        assert!(focus.queries() > 0, "010 classifies as delivered");
        assert_eq!(
            report.reason,
            Some(DestinationFailure::NoTextField),
            "010's own verdict"
        );
        assert!(spy.pasted().is_empty());
    }
}
/// 010's port answers the field decision alone, whatever its setting and platform.
#[test]
fn the_010_port_answers_the_field_decision_without_classifying() {
    use handy_app_lib::destination::context::{PreInsertionVerdict, RecoveryReason};
    use handy_app_lib::destination::field_check::Divergence;
    let focus = Focus::role("AXList", PID);
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus.clone());
    bench.service.dictation_starting(true);
    bench.service.dictation_started(DictationId(1));
    let context = |field| DestinationContext {
        dictation_id: DictationId(1),
        destination_generation: 1,
        source: handy_app_lib::destination::context::DestinationSource::Captured,
        target: None,
        field,
    };
    assert_eq!(
        bench
            .service
            .before_insertion(&context(FieldCheck::Verified)),
        PreInsertionVerdict::Insert
    );
    assert_eq!(
        bench
            .service
            .before_insertion(&context(FieldCheck::Diverted(Divergence::FocusUnreadable))),
        PreInsertionVerdict::Recover(RecoveryReason::FieldReturnFailed)
    );
    assert_eq!(focus.queries(), 0);
}
#[test]
fn the_bubble_says_insertion_failed_nothing_sent_on_every_platform() -> TestResult {
    for platform in [
        RecoveryPlatform::MacOs,
        RecoveryPlatform::Windows,
        RecoveryPlatform::Linux,
    ] {
        let bench = Bench::new(platform);
        let outcome = bench.failed(1, Some(DestinationFailure::FieldReturnFailed), None);
        let ConsumerAck::PresentationPending(_receipt) = bench.service.consume(&outcome) else {
            return Err(format!("{platform:?}: no bubble").into());
        };
        let view = bench.surface.presented().pop().ok_or("presented")?;
        assert_eq!(
            (view.reason, view.detail),
            (BubbleReason::InsertionFailed, ReasonDetail::NothingSent)
        );
        assert!(!view.preview.is_empty(), "the transcript is kept for Copy");
    }
    Ok(())
}
#[test]
fn a_bubble_that_cannot_be_presented_gives_one_paste_error() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let outcome = bench.failed(1, Some(DestinationFailure::FieldReturnFailed), None);
    let ConsumerAck::PresentationPending(receipt) = bench.service.consume(&outcome) else {
        return Err("no bubble".into());
    };
    drop(receipt);
    let view = bench.surface.presented().pop().ok_or("presented")?;
    bench.service.presentation_failed(view.bubble_id);
    assert_eq!(
        bench.toasts(),
        vec![HistoryStatus::Saved],
        "006's paste-error, once"
    );
    Ok(())
}
