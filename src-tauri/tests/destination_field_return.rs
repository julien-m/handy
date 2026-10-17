//! 011 return attempt and guard decision (plan Step 4, spec AC-002, AC-004, SC-002) through
//! 006's real orchestrator and guard with the scripted accessibility world: the verified input
//! receives the paste at its restored caret; an attempt that changed nothing continues on the
//! existing path; every other failure (focus moved, window raised, a write without an answer,
//! range missing, not restored or not verified, focus not noted or unreadable) pastes nothing
//! and fails with `field_return_failed`, even without 010; 006's own refusals keep precedence.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use destination::notes;
use fake::{Element, Write};
use handy_app_lib::destination::context::RecoveryDisposition;
use handy_app_lib::destination::field::MIN_AX_TIMEOUT;
use handy_app_lib::destination::field_attempt::AttemptStage;
use handy_app_lib::destination::field_check::FieldCheck;
use handy_app_lib::destination::insertion::InsertionReport;
use handy_app_lib::destination::outcome::{DestinationFailure, InsertionStatus};
use handy_app_lib::recovery::classifier::AxQueryError;
use rig::{FieldRig, PasteSpy, CARET, INPUT, OTHER, WINDOW};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const PID: u32 = 22;
const TAB: u32 = 3;
const SECOND_WINDOW: u32 = 101;

/// Comet's input remembered at the start, then Notes brought forward by the user.
fn remembered() -> FieldRig {
    let field = FieldRig::comet();
    field
        .world
        .element(TAB, Element::input(PID, WINDOW, (0, 0)));
    field
        .world
        .element(SECOND_WINDOW, Element::control(PID, "AXWindow"));
    field
}
/// Start, switch to Notes, apply `before` to the world, then deliver with `spy` (no 010).
fn dictate(field: &FieldRig, before: impl FnOnce(&FieldRig), spy: &PasteSpy) -> InsertionReport {
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    before(field);
    field.deliver_plain(&started, spy)
}
fn check_of(spy: &PasteSpy) -> Option<FieldCheck> {
    spy.contexts().first().map(|context| context.field)
}
/// Nothing pasted, the dictation failed with 011's reason and its disposition is `Recover`.
fn assert_diverted(field: &FieldRig, report: &InsertionReport, spy: &PasteSpy) {
    assert!(spy.pasted().is_empty(), "nothing pasted");
    assert_eq!(
        report.status,
        InsertionStatus::Failed,
        "transcript kept by 006"
    );
    assert_eq!(report.reason, Some(DestinationFailure::FieldReturnFailed));
    assert_eq!(field.rig.disposition(), Some(RecoveryDisposition::Recover));
    assert_eq!(
        field.world.alive(),
        0,
        "references released after the decision"
    );
}

// @spec AC-002: Focus back and range restored then verified — .specs/features/011-dictation-field-return/spec.md#ac-002
// @spec FR-002: Return to the input at insertion — .specs/features/011-dictation-field-return/spec.md#fr-002
#[test]
fn the_verified_input_receives_the_paste_at_its_restored_caret() {
    for clicked_elsewhere in [false, true] {
        let field = remembered();
        let spy = PasteSpy::new(&field.world);
        let report = dictate(
            &field,
            |field| {
                field.world.edit(INPUT, |input| input.range = Ok((0, 3)));
                if clicked_elsewhere {
                    field.world.focus(PID, Ok(OTHER));
                }
            },
            &spy,
        );
        assert_eq!(report.status, InsertionStatus::InsertionAttempted);
        assert_eq!(spy.pasted(), vec![Some(INPUT)], "the remembered input wins");
        assert_eq!(field.world.range(INPUT), Some(CARET), "caret of the start");
        assert_eq!(check_of(&spy), Some(FieldCheck::Verified));
        assert_eq!(field.calls("raise"), 0, "its window was already focused");
    }
}
#[test]
fn an_input_in_another_window_has_its_window_raised_then_verified() {
    let field = remembered();
    field
        .world
        .edit(INPUT, |input| input.window = Some(SECOND_WINDOW));
    let spy = PasteSpy::new(&field.world);
    dictate(&field, |_| {}, &spy);
    assert_eq!(field.calls("raise"), 1);
    assert_eq!(spy.pasted(), vec![Some(INPUT)]);
}
#[test]
fn the_destination_already_in_front_still_returns_to_its_input() {
    let field = remembered();
    let started = field.start(true);
    field.world.focus(PID, Ok(OTHER));
    let spy = PasteSpy::new(&field.world);
    field.deliver_plain(&started, &spy);
    assert_eq!(field.rig.workspace.activations().len(), 0);
    assert_eq!(spy.pasted(), vec![Some(INPUT)]);
}

// @spec AC-004: Branch (a) when the attempt changed nothing — .specs/features/011-dictation-field-return/spec.md#ac-004
#[test]
fn an_input_gone_with_nothing_changed_inserts_as_without_011() {
    let field = remembered();
    field.world.edit(INPUT, |input| input.focus = Write::Refuse);
    let spy = PasteSpy::new(&field.world);
    let report = dictate(&field, |field| field.world.focus(PID, Ok(OTHER)), &spy);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(spy.pasted(), vec![Some(OTHER)], "006's target, as today");
    assert_eq!(
        check_of(&spy),
        Some(FieldCheck::Unchanged(AttemptStage::FocusRefused))
    );
}

/// Where Comet's focus is when the attempt notes it.
#[derive(Clone, Copy)]
enum Noted {
    Input,
    Other,
    Missing,
}
/// One fault of SC-002: the focus noted, the input's scripted answers, a refused raise.
struct Fault {
    name: &'static str,
    noted: Noted,
    input: fn(&mut Element),
    raise_refused: bool,
}
const fn fault(name: &'static str, noted: Noted, input: fn(&mut Element)) -> Fault {
    Fault {
        name,
        noted,
        input,
        raise_refused: false,
    }
}
const FAULTS: [Fault; 9] = [
    fault("focus moved to a background tab", Noted::Other, |input| {
        input.focus_lands_on = Some(TAB);
    }),
    fault(
        "window raised, focus not on the input",
        Noted::Other,
        |input| {
            input.window = Some(SECOND_WINDOW);
            input.focus_lands_on = Some(OTHER);
        },
    ),
    fault("focus write unanswered", Noted::Other, |input| {
        input.focus = Write::Unanswered;
    }),
    fault(
        "focus write unanswered but applied",
        Noted::Other,
        |input| {
            input.focus = Write::UnansweredApplied;
        },
    ),
    fault("range write unanswered", Noted::Input, |input| {
        input.select = Write::Unanswered;
    }),
    fault(
        "input already focused when noted, range refused",
        Noted::Input,
        |input| {
            input.select = Write::Refuse;
        },
    ),
    fault("window raised then range refused", Noted::Input, |input| {
        input.window = Some(SECOND_WINDOW);
        input.select = Write::Refuse;
    }),
    Fault {
        raise_refused: true,
        ..fault("raise refused", Noted::Input, |input| {
            input.window = Some(SECOND_WINDOW);
        })
    },
    fault("focused element not noted", Noted::Missing, |_| {}),
];
fn inject(field: &FieldRig, fault: &Fault) {
    match fault.noted {
        Noted::Input => field.world.focus(PID, Ok(INPUT)),
        Noted::Other => field.world.focus(PID, Ok(OTHER)),
        Noted::Missing => field.world.focus(PID, Err(AxQueryError::NoValue)),
    }
    field.world.edit(INPUT, fault.input);
    if fault.raise_refused {
        field
            .world
            .edit(SECOND_WINDOW, |window| window.raise_works = false);
    }
}

// @spec FR-004: Never paste where the attempt led — .specs/features/011-dictation-field-return/spec.md#fr-004
// @spec SC-002: Fault injection with zero wrong insertions — .specs/features/011-dictation-field-return/spec.md#sc-002
#[test]
fn every_attempt_that_may_have_changed_something_pastes_nothing() {
    // The exact divergence of each row is the decision table's (`field_check_tests`).
    for fault in &FAULTS {
        let field = remembered();
        let spy = PasteSpy::new(&field.world);
        let report = dictate(&field, |field| inject(field, fault), &spy);
        assert_diverted(&field, &report, &spy);
        let name = fault.name;
        assert_eq!(
            field.rig.disposition(),
            Some(RecoveryDisposition::Recover),
            "{name}"
        );
    }
}
#[test]
fn an_attempt_out_of_budget_stops_without_sending_and_pastes_nothing() {
    let field = remembered();
    field
        .world
        .edit(INPUT, |input| input.window = Some(SECOND_WINDOW));
    let spy = PasteSpy::new(&field.world);
    // 20 ms per call: note, focus and window fit the 50 ms budget, the focused window does not.
    let report = dictate(
        &field,
        |field| field.world.cost(Duration::from_millis(20)),
        &spy,
    );
    assert_diverted(&field, &report, &spy);
    assert_eq!(field.calls("focused_window"), 0, "not sent below 5 ms");
    assert_eq!(field.calls("select"), 0);
    let late = field
        .world
        .calls()
        .into_iter()
        .filter(|call| call.op != "focused_element");
    assert!(late.into_iter().all(|call| call.timeout >= MIN_AX_TIMEOUT));
}
#[test]
fn a_range_missing_at_the_start_makes_no_call_at_insertion_and_pastes_nothing() {
    let field = remembered();
    field
        .world
        .edit(INPUT, |input| input.range = Err(AxQueryError::Unsupported));
    let spy = PasteSpy::new(&field.world);
    let report = dictate(&field, |_| {}, &spy);
    assert_diverted(&field, &report, &spy);
    assert_eq!(field.calls("focused_element"), 1, "the start read only");
    for op in ["focus", "window", "raise", "select", "range_is"] {
        assert_eq!(field.calls(op), 0, "{op}");
    }
}
#[test]
fn a_range_or_focus_changed_during_the_paste_delay_pastes_nothing() {
    type Effect = fn(&Arc<fake::World>);
    let effects: [(&str, Effect); 3] = [
        ("range normalized", |world| {
            world.edit(INPUT, |input| input.range = Ok((0, 0)))
        }),
        ("focus unreadable", |world| {
            world.focus(PID, Err(AxQueryError::Failed))
        }),
        ("focus moved", |world| world.focus(PID, Ok(TAB))),
    ];
    for (name, effect) in effects {
        let field = remembered();
        let world = Arc::clone(&field.world);
        let spy = PasteSpy::new(&field.world).during_delay(move || effect(&world));
        let report = dictate(&field, |_| {}, &spy);
        assert_diverted(&field, &report, &spy);
        assert!(
            matches!(field.rig.disposition(), Some(RecoveryDisposition::Recover)),
            "{name}"
        );
    }
}

#[test]
fn a_cancellation_during_the_attempt_and_006_refusals_keep_their_precedence() {
    let field = remembered();
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    field
        .world
        .after("focus", move || flag.store(true, Ordering::SeqCst));
    let spy = PasteSpy::new(&field.world);
    let plain = Arc::new(handy_app_lib::destination::context::NoRecoveryConsumer);
    let report = field.deliver(&started, &spy, plain, &cancelled);
    assert_eq!(report.status, InsertionStatus::Cancelled);
    assert!(spy.pasted().is_empty());
    // The destination lost during the paste delay: 006's activation_failed, no field check.
    let field = remembered();
    let rig = Arc::clone(&field.rig.workspace);
    let spy = PasteSpy::new(&field.world).during_delay(move || rig.bring(notes()));
    let report = dictate(&field, |_| {}, &spy);
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    assert_eq!(field.calls("range_is"), 0);
    assert_eq!(field.world.alive(), 0);
}
#[test]
fn a_cancellation_after_the_activation_never_moves_the_focus() {
    let field = remembered();
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    // Read once at the freeze, once in the activation wait, then cancelled before the attempt.
    let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let spy = PasteSpy::new(&field.world);
    field.rig.manager.mark_processing();
    let counter = Arc::clone(&reads);
    let request = handy_app_lib::destination::insertion::InsertionRequest {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(move || counter.fetch_add(1, Ordering::SeqCst) >= 2),
    };
    let report = field.rig.orchestrator().deliver(request, &spy);
    assert_eq!(report.status, InsertionStatus::Cancelled);
    let seen = reads.load(Ordering::SeqCst);
    assert!(
        seen >= 4,
        "cancelled at the attempt, then seen by the guard ({seen})"
    );
    assert_eq!(
        field.calls("focus"),
        0,
        "no focus change for a cancelled dictation"
    );
    assert_eq!(field.world.alive(), 0);
}
