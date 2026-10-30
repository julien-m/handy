//! 011 return attempt against a destination that answers late right after its activation
//! (native finding N18: Codex, an Electron application, and a Chrome page answered their first
//! accessibility requests after more than 50 ms). The note waits for the application within its
//! own budget, the writes' budget starts once it answered, and a write answered late is verified
//! at the guard: the remembered input receives the paste when it is focused with its remembered
//! range, and nothing is pasted when either cannot be verified (spec AC-002, AC-004). A range
//! write answered at once but applied later (Chromium) is read back until it settles; a caret
//! normalized elsewhere still receives the paste, a selection never unless exact.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use destination::notes;
use handy_app_lib::destination::field::{FIELD_NOTE_BUDGET, FIELD_RESTORE_BUDGET};
use handy_app_lib::destination::field_check::FieldCheck;
use handy_app_lib::destination::insertion::InsertionReport;
use handy_app_lib::destination::outcome::{DestinationFailure, InsertionStatus};
use rig::{FieldRig, PasteSpy, CARET, INPUT, OTHER};
use std::sync::Arc;
use std::time::Duration;

const PID: u32 = 22;
/// The configured paste delay, spent by the method before the guard.
const PASTE_DELAY: Duration = Duration::from_millis(60);

/// Start in Comet's input, switch to Notes, apply `before`, then deliver: the method spends the
/// paste delay on the manual clock before the guard, as the real methods do.
fn dictate(field: &FieldRig, before: impl FnOnce(&FieldRig)) -> (InsertionReport, PasteSpy) {
    dictate_with_delay(field, PASTE_DELAY, before)
}
fn dictate_with_delay(
    field: &FieldRig,
    delay: Duration,
    before: impl FnOnce(&FieldRig),
) -> (InsertionReport, PasteSpy) {
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    before(field);
    let clock = Arc::clone(&field.rig.clock);
    let spy = PasteSpy::new(&field.world).during_delay(move || clock.advance(delay));
    let report = field.deliver_plain(&started, &spy);
    (report, spy)
}
fn check_of(spy: &PasteSpy) -> Option<FieldCheck> {
    spy.contexts().first().map(|context| context.field)
}
fn assert_pasted_at_the_caret(field: &FieldRig, report: &InsertionReport, spy: &PasteSpy) {
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(spy.pasted(), vec![Some(INPUT)], "the remembered input");
    assert_eq!(field.world.range(INPUT), Some(CARET), "caret of the start");
    assert_eq!(check_of(spy), Some(FieldCheck::Verified));
    assert_eq!(field.world.alive(), 0, "references released");
}
fn assert_nothing_pasted(field: &FieldRig, report: &InsertionReport, spy: &PasteSpy) {
    assert!(spy.pasted().is_empty(), "nothing pasted");
    assert_eq!(report.status, InsertionStatus::Failed);
    assert_eq!(report.reason, Some(DestinationFailure::FieldReturnFailed));
    assert_eq!(field.world.alive(), 0, "references released");
}
/// From the call right after `op`, the application answers nothing for `delay`.
fn late_after(field: &FieldRig, op: &'static str, delay: Duration) {
    let world = Arc::clone(&field.world);
    field.world.after(op, move || world.answers_late(delay));
}

// @spec AC-002: The note waits for a late application — .specs/features/011-dictation-field-return/spec.md#ac-002
#[test]
fn a_destination_answering_late_after_its_activation_still_receives_the_paste() {
    for late in [Duration::from_millis(55), Duration::from_millis(120)] {
        let field = FieldRig::comet();
        let (report, spy) = dictate(&field, |field| {
            field.world.focus(PID, Ok(OTHER));
            field.world.answers_late(late);
        });
        assert_pasted_at_the_caret(&field, &report, &spy);
        // The start read's first call, then the note.
        let reads = field.world.calls().into_iter();
        let note = reads.filter(|call| call.op == "focused_element").nth(1);
        let note = note.map(|call| call.timeout);
        assert_eq!(
            note,
            Some(FIELD_NOTE_BUDGET),
            "the note carries its own budget"
        );
        let focus = field
            .world
            .calls()
            .into_iter()
            .find(|call| call.op == "focus");
        let focus = focus.map(|call| call.timeout);
        assert_eq!(
            focus,
            Some(FIELD_RESTORE_BUDGET),
            "writes budget starts after the note"
        );
    }
}

#[test]
fn a_destination_silent_beyond_the_note_budget_gets_no_focus_change() {
    let field = FieldRig::comet();
    let silent = FIELD_NOTE_BUDGET + Duration::from_millis(100);
    let (report, spy) = dictate(&field, |field| {
        field.world.focus(PID, Ok(OTHER));
        field.world.answers_late(silent);
    });
    assert_nothing_pasted(&field, &report, &spy);
    assert_eq!(field.calls("focus"), 0, "no attempt without a note");
    assert_eq!(field.world.focused(PID), Some(OTHER), "focus untouched");
}

// @spec AC-004: A late write verified at the guard — .specs/features/011-dictation-field-return/spec.md#ac-004
#[test]
fn a_focus_write_answered_late_is_verified_at_the_guard_and_pastes() {
    let field = FieldRig::comet();
    let (report, spy) = dictate(&field, |field| {
        field.world.focus(PID, Ok(OTHER));
        late_after(field, "focused_element", Duration::from_millis(80));
    });
    assert_pasted_at_the_caret(&field, &report, &spy);
    assert_eq!(
        field.calls("select"),
        0,
        "stopped at the unanswered focus write"
    );
}

#[test]
fn a_range_write_answered_late_is_verified_at_the_guard_and_pastes() {
    let field = FieldRig::comet();
    let (report, spy) = dictate(&field, |field| {
        field.world.edit(INPUT, |input| input.range = Ok((0, 3)));
        late_after(field, "focused_window", Duration::from_millis(80));
    });
    assert_pasted_at_the_caret(&field, &report, &spy);
}

// @spec FR-004: Never paste over an unverified range — .specs/features/011-dictation-field-return/spec.md#fr-004
#[test]
fn a_late_write_never_pastes_over_a_range_it_could_not_restore() {
    let field = FieldRig::comet();
    let (report, spy) = dictate(&field, |field| {
        field.world.focus(PID, Ok(OTHER));
        field.world.edit(INPUT, |input| input.range = Ok((0, 3)));
        late_after(field, "focused_element", Duration::from_millis(80));
    });
    assert_nothing_pasted(&field, &report, &spy);
    assert_eq!(
        field.world.focused(PID),
        Some(INPUT),
        "the late focus landed"
    );
    assert_eq!(
        field.world.range(INPUT),
        Some((0, 3)),
        "the draft's selection kept"
    );
}

#[test]
fn a_late_focus_landing_elsewhere_never_pastes() {
    let field = FieldRig::comet();
    let (report, spy) = dictate(&field, |field| {
        field.world.focus(PID, Ok(OTHER));
        field
            .world
            .edit(INPUT, |input| input.focus_lands_on = Some(OTHER + 10));
        field
            .world
            .element(OTHER + 10, fake::Element::input(PID, rig::WINDOW, (0, 0)));
        late_after(field, "focused_element", Duration::from_millis(80));
    });
    assert_nothing_pasted(&field, &report, &spy);
    assert_eq!(field.world.focused(PID), Some(OTHER + 10));
}

// @spec AC-002: A range applied later is read back until set — .specs/features/011-dictation-field-return/spec.md#ac-002
#[test]
fn a_range_write_applied_a_moment_later_is_read_back_until_it_settles() {
    let field = FieldRig::comet();
    let (report, spy) = dictate_with_delay(&field, Duration::ZERO, |field| {
        field.world.edit(INPUT, |input| {
            input.range = Ok((0, 3));
            input.select_lag = Duration::from_millis(20);
        });
    });
    assert_pasted_at_the_caret(&field, &report, &spy);
    // The start read, then at least two read-backs before the range shows, then the guard.
    let reads = field.calls("read_range");
    assert!(reads >= 4, "{reads}");
}

#[test]
fn a_range_write_never_applied_in_time_never_pastes_over_the_selection() {
    let field = FieldRig::comet();
    let (report, spy) = dictate_with_delay(&field, Duration::ZERO, |field| {
        field.world.edit(INPUT, |input| {
            input.range = Ok((0, 3));
            input.select_lag = Duration::from_secs(1);
        });
    });
    assert_nothing_pasted(&field, &report, &spy);
    let reads = field.calls("read_range");
    // The start read, the read-backs within the writes' 50 ms, the guard's read.
    assert!(reads <= 1 + 10 + 1, "read-backs bounded ({reads})");
}

// @spec AC-004: A moved caret pastes, a selection never — .specs/features/011-dictation-field-return/spec.md#ac-004
#[test]
fn a_caret_normalized_elsewhere_in_the_input_still_receives_the_paste() {
    let field = FieldRig::comet();
    let elsewhere = (CARET.0 + 3, 0);
    let (report, spy) = dictate(&field, |field| {
        field
            .world
            .edit(INPUT, |input| input.select_lands_at = Some(elsewhere));
    });
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(spy.pasted(), vec![Some(INPUT)]);
    assert_eq!(check_of(&spy), Some(FieldCheck::Verified));
    assert_eq!(field.world.range(INPUT), Some(elsewhere));
}

#[test]
fn a_range_normalized_into_a_selection_never_pastes() {
    for lands in [(CARET.0, 4), (0, CARET.0)] {
        let field = FieldRig::comet();
        let (report, spy) = dictate(&field, |field| {
            field
                .world
                .edit(INPUT, |input| input.select_lands_at = Some(lands));
        });
        assert_nothing_pasted(&field, &report, &spy);
        assert_eq!(field.world.range(INPUT), Some(lands), "selection kept");
    }
}

#[test]
fn an_input_already_focused_whose_focus_write_is_refused_pastes_at_its_caret() {
    let field = FieldRig::comet();
    let (report, spy) = dictate(&field, |field| {
        field
            .world
            .edit(INPUT, |input| input.focus = fake::Write::Refuse);
    });
    assert_pasted_at_the_caret(&field, &report, &spy);
    assert_eq!(
        field.calls("select"),
        0,
        "stopped at the refused focus write"
    );
}
