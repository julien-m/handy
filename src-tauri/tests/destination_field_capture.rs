//! 011 start read (plan Step 3, spec AC-001, AC-003, AC-006, SC-002 capture cases, SC-005): the
//! production capture over a scripted accessibility world remembers only an editable input of
//! the destination's process, with its range or without it; a password field, another process,
//! a non-editable or undetermined element, a missing permission, an error and a spent deadline
//! remember nothing; every query carries the remaining budget and none is sent below 5 ms; the
//! pinned instance is read whatever is in front; no destination reads nothing; the start path
//! never waits for the read and a late answer is never attached.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use destination::{comet, notes, textedit, ManualClock};
use fake::{Access, Element, World};
use handy_app_lib::destination::field::{FIELD_CAPTURE_BUDGET, MIN_AX_TIMEOUT};
use handy_app_lib::destination::field_capture::{
    capture, read_and_attach, CaptureMiss, FieldCapture,
};
use handy_app_lib::destination::snapshot::{DestinationSnapshot, MenuChoice};
use handy_app_lib::recovery::classifier::AxQueryError;
use rig::{FieldRig, CARET, INPUT, OTHER, WINDOW};
use std::sync::Arc;
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Capture Comet's focused element in `world` and return the miss, if any.
fn miss(world: &Arc<World>, clock: &Arc<ManualClock>) -> Option<CaptureMiss> {
    match capture(&Access(Arc::clone(world)), comet().pid, clock.as_ref()) {
        FieldCapture::Remembered(_) => None,
        FieldCapture::NotRemembered(miss) => Some(miss),
    }
}
fn world_with(element: Element) -> (Arc<World>, Arc<ManualClock>) {
    let clock = ManualClock::new();
    let world = World::new(&clock);
    world.element(INPUT, element);
    world.focus(comet().pid, Ok(INPUT));
    (world, clock)
}

// @spec AC-001: Editable input remembered with its range — .specs/features/011-dictation-field-return/spec.md#ac-001
#[test]
fn an_editable_input_is_remembered_with_its_range_and_nothing_else_is_read() -> TestResult {
    let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
    let FieldCapture::Remembered(field) =
        capture(&Access(world.clone()), comet().pid, clock.as_ref())
    else {
        return Err("not remembered".into());
    };
    assert_eq!(
        field.range.and_then(|range| range.as_offsets()),
        Some(CARET)
    );
    let ops: Vec<_> = world.calls().into_iter().map(|call| call.op).collect();
    assert_eq!(
        ops,
        vec![
            "focused_element",
            "role",
            "subrole",
            "is_settable",
            "read_range"
        ],
        "facts and range only: no value, selected text or title (the port has none)"
    );
    Ok(())
}
#[test]
fn an_editable_input_whose_range_cannot_be_read_is_remembered_without_it() -> TestResult {
    let mut element = Element::input(comet().pid, WINDOW, CARET);
    element.range = Err(AxQueryError::Unsupported);
    let (world, clock) = world_with(element);
    let FieldCapture::Remembered(field) = capture(&Access(world), comet().pid, clock.as_ref())
    else {
        return Err("not remembered".into());
    };
    assert!(field.range.is_none(), "AC-004 (b) applies at insertion");
    Ok(())
}
// @spec SC-002: Capture fault cases remember nothing — .specs/features/011-dictation-field-return/spec.md#sc-002
#[test]
fn a_password_field_another_process_or_a_doubt_remembers_nothing() {
    for (element, expected) in doubts() {
        let (world, clock) = world_with(element.clone());
        assert_eq!(miss(&world, &clock), Some(expected), "{element:?}");
        assert_eq!(world.count("read_range"), 0, "no range read for a miss");
        assert_eq!(world.alive(), 0, "the element is released at once");
    }
}
/// Elements that are never remembered, and why (spec AC-001).
fn doubts() -> [(Element, CaptureMiss); 7] {
    let pid = comet().pid;
    let read_only = Element {
        settable: false,
        ..Element::input(pid, WINDOW, CARET)
    };
    [
        (
            secure("AXSecureTextField", None),
            CaptureMiss::PasswordField,
        ),
        (
            secure("AXTextField", Some("AXSecureTextField")),
            CaptureMiss::PasswordField,
        ),
        (
            Element::input(notes().pid, WINDOW, CARET),
            CaptureMiss::OtherProcess,
        ),
        (Element::control(pid, "AXList"), CaptureMiss::NotEditable),
        (Element::control(pid, "AXGroup"), CaptureMiss::Undetermined),
        (
            Element::control(pid, "AXWebArea"),
            CaptureMiss::Undetermined,
        ),
        (read_only, CaptureMiss::Undetermined),
    ]
}
/// A password field of Comet, by role or subrole.
fn secure(role: &'static str, subrole: Option<&'static str>) -> Element {
    Element {
        role,
        subrole,
        ..Element::input(comet().pid, WINDOW, CARET)
    }
}
#[test]
fn a_missing_permission_makes_no_query_and_an_error_remembers_nothing() {
    let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
    world.trusted(false);
    assert_eq!(miss(&world, &clock), Some(CaptureMiss::PermissionMissing));
    assert!(world.calls().is_empty(), "no accessibility query");
    for (error, expected) in [
        (AxQueryError::Failed, CaptureMiss::AxError),
        (AxQueryError::NoValue, CaptureMiss::NoFocusedElement),
        (AxQueryError::Timeout, CaptureMiss::BudgetSpent),
    ] {
        let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
        world.focus(comet().pid, Err(error));
        assert_eq!(miss(&world, &clock), Some(expected));
    }
}
#[test]
fn each_query_carries_the_remaining_budget_and_none_is_sent_below_5_ms() {
    let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
    let _ = miss(&world, &clock);
    for (index, call) in world.calls().iter().enumerate() {
        let spent = Duration::from_millis(u64::try_from(index).unwrap_or(0));
        assert_eq!(call.timeout, FIELD_CAPTURE_BUDGET - spent, "{call:?}");
    }
    for cost in [9, 10, 20, 49] {
        let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
        world.cost(Duration::from_millis(cost));
        let _ = miss(&world, &clock);
        assert!(
            world
                .calls()
                .iter()
                .all(|call| call.timeout >= MIN_AX_TIMEOUT),
            "cost {cost} ms: {:?}",
            world.calls()
        );
    }
}
#[test]
fn a_deadline_before_the_verdict_misses_and_during_the_range_read_keeps_no_range() -> TestResult {
    // 20 ms per call: the fourth query would have 10 ms - 20 ms left, so no verdict.
    let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
    world.cost(Duration::from_millis(20));
    assert_eq!(miss(&world, &clock), Some(CaptureMiss::BudgetSpent));
    // 12 ms per call: the verdict arrives at 48 ms, leaving 2 ms for the range read.
    let (world, clock) = world_with(Element::input(comet().pid, WINDOW, CARET));
    world.cost(Duration::from_millis(12));
    let FieldCapture::Remembered(field) =
        capture(&Access(world.clone()), comet().pid, clock.as_ref())
    else {
        return Err("classified within the budget".into());
    };
    assert!(field.range.is_none(), "deadline during the range read");
    assert_eq!(world.count("read_range"), 0, "not sent below 5 ms");
    Ok(())
}

// @spec AC-003: The pinned instance is read whatever is in front — .specs/features/011-dictation-field-return/spec.md#ac-003
#[test]
fn a_pinned_application_in_the_background_has_its_input_read() -> TestResult {
    let field = FieldRig::comet();
    let first = field.rig.start(true);
    field.rig.manager.pin_current(first.dictation_id, 0)?;
    field.rig.workspace.bring(notes());
    field.world.focus(notes().pid, Ok(OTHER));
    let started = field.start(true);
    let (frozen, remembered) = field
        .rig
        .manager
        .freeze_with_field(started.dictation_id, started.destination_generation)?;
    let remembered = remembered.ok_or("pinned input remembered")?;
    assert_eq!(remembered.pid, comet().pid);
    assert_eq!(
        frozen.selection.source(),
        handy_app_lib::destination::context::DestinationSource::Pinned
    );
    Ok(())
}
#[test]
fn a_pinned_application_not_running_reads_nothing() -> TestResult {
    let field = FieldRig::comet();
    let first = field.rig.start(true);
    field.rig.manager.pin_current(first.dictation_id, 0)?;
    field.rig.workspace.set_running(vec![notes(), textedit()]);
    field.rig.workspace.bring(notes());
    let started = field.start(true);
    assert_eq!(
        field.calls("focused_element"),
        0,
        "009's failure, nothing read"
    );
    let (id, generation) = (started.dictation_id, started.destination_generation);
    assert!(field
        .rig
        .manager
        .freeze_with_field(id, generation)?
        .1
        .is_none());
    Ok(())
}
#[test]
fn return_off_without_pin_or_handy_in_front_reads_nothing() -> TestResult {
    for handy_in_front in [false, true] {
        let field = FieldRig::comet();
        if handy_in_front {
            *field.rig.workspace.foreground.lock().map_err(|_| "lock")? =
                handy_app_lib::destination::workspace::Foreground::Handy;
        }
        let started = field.start(handy_in_front);
        assert_eq!(
            field.calls("focused_element"),
            0,
            "Handy in front: {handy_in_front}"
        );
        let (id, generation) = (started.dictation_id, started.destination_generation);
        assert!(field.rig.manager.field_target(id, generation).is_none());
    }
    Ok(())
}

// @spec SC-005: The start path never waits for the read — .specs/features/011-dictation-field-return/spec.md#sc-005
#[cfg(feature = "destination-return")]
#[test]
fn the_start_path_returns_while_the_port_never_answers_and_a_late_answer_is_dropped() -> TestResult
{
    use handy_app_lib::destination::field_capture::spawn_start_read;
    let field = FieldRig::comet();
    let (release, blocked) = std::sync::mpsc::channel::<()>();
    field.world.block_until(blocked);
    let started = field.rig.start(true);
    let target = field
        .rig
        .manager
        .field_target(started.dictation_id, started.destination_generation);
    let worker = spawn_start_read(&field.rig.manager, target).ok_or("worker spawned")?;
    // The start path returned with the port still blocked; 60 ms pass on the clock.
    field.rig.clock.advance(Duration::from_millis(60));
    drop(release);
    worker.join().map_err(|_| "worker panicked")?;
    let (_, remembered) = field
        .rig
        .manager
        .freeze_with_field(started.dictation_id, started.destination_generation)?;
    assert!(remembered.is_none(), "abandoned at its 50 ms deadline");
    assert_eq!(field.world.alive(), 0, "the late reference is released");
    Ok(())
}
// @spec SC-003: Gate closed means zero focused-element reads — .specs/features/011-dictation-field-return/spec.md#sc-003
#[cfg(not(feature = "destination-return"))]
#[test]
fn with_the_gate_closed_a_whole_dictation_reads_nothing_and_pastes_as_without_011() -> TestResult {
    use handy_app_lib::destination::field_capture::spawn_start_read;
    use handy_app_lib::destination::field_check::FieldCheck;
    let field = FieldRig::comet();
    let started = field.rig.start(true);
    let (id, generation) = (started.dictation_id, started.destination_generation);
    let target = field.rig.manager.field_target(id, generation);
    assert!(
        spawn_start_read(&field.rig.manager, target).is_none(),
        "no start read"
    );
    field.rig.workspace.bring(notes());
    let spy = rig::PasteSpy::new(&field.world);
    let report = field.deliver_plain(&started, &spy);
    assert!(
        field.world.calls().is_empty(),
        "no read, no attempt, no verification"
    );
    let context = spy.contexts().pop().ok_or("dispatched")?;
    assert_eq!(context.field, FieldCheck::Absent);
    // The same dictation without 011's port: the same report and the same target.
    let plain = destination::Rig::new(comet(), vec![comet(), notes()]);
    let without = plain.start(true);
    plain.workspace.bring(notes());
    let method = destination::Method::succeeding();
    let reference = plain.deliver(&without, &method, false);
    assert_eq!(
        (report.status, report.reason),
        (reference.status, reference.reason)
    );
    let reference_target = method.dispatched().pop().and_then(|context| context.target);
    assert_eq!(
        context.target.map(|target| target.pid),
        reference_target.map(|target| target.pid)
    );
    Ok(())
}
#[cfg(not(feature = "destination-return"))]
// @spec SC-003: Gate closed means zero focused-element reads — .specs/features/011-dictation-field-return/spec.md#sc-003
#[test]
fn with_the_gate_closed_no_start_read_is_ever_spawned() {
    use handy_app_lib::destination::field_capture::spawn_start_read;
    let field = FieldRig::comet();
    let started = field.rig.start(true);
    let target = field
        .rig
        .manager
        .field_target(started.dictation_id, started.destination_generation);
    assert!(target.is_some(), "a destination exists");
    assert!(spawn_start_read(&field.rig.manager, target).is_none());
    assert!(field.world.calls().is_empty(), "zero focused-element reads");
}
/// Change `started`'s session as `change` says, while its start read is still running.
fn change_session(field: &FieldRig, started: &DestinationSnapshot, change: &str) -> TestResult {
    let manager = &field.rig.manager;
    let (id, generation) = (started.dictation_id, started.destination_generation);
    match change {
        "badge" => {
            let menu = manager.menu(id, 0)?;
            let entry = menu
                .entries
                .iter()
                .find(|entry| entry.display_name == "TextEdit");
            let token = entry.map(|entry| entry.token).ok_or("TextEdit entry")?;
            manager.choose(id, 0, MenuChoice::Application(token))?;
        }
        "freeze" => {
            manager.mark_processing();
            manager.freeze(id, generation)?;
        }
        "cancel" => {
            manager.cancel();
        }
        _ => {
            field.rig.start(true);
        }
    }
    Ok(())
}
#[test]
fn a_read_answering_after_a_badge_choice_freeze_or_cancel_is_refused() -> TestResult {
    for change in ["badge", "freeze", "cancel", "new_dictation"] {
        let field = FieldRig::new(comet(), vec![comet(), textedit()]);
        let started = field.rig.start(true);
        let (id, generation) = (started.dictation_id, started.destination_generation);
        let target = field
            .rig
            .manager
            .field_target(id, generation)
            .ok_or("target")?;
        change_session(&field, &started, change)?;
        read_and_attach(&field.rig.manager, target, field.rig.clock_now());
        assert_eq!(
            field.world.alive(),
            0,
            "{change}: handle released, not attached"
        );
    }
    Ok(())
}
