//! Verified direct insertion (AC-017, AC-018) against scripted AX fields.

use std::time::Duration;

use handy_app_lib::smart_paste::ax_port::{AxError, Utf16Range};
use handy_app_lib::smart_paste::clock::{Clock, ManualClock};
use handy_app_lib::smart_paste::direct::{
    attempt_direct_insertion, DirectAttempt, GuardVerdict, SkipReason, UncertainReason,
    PER_CALL_TIMEOUT, TOTAL_DIRECT_BUDGET,
};

use super::fakes::{elapsed_since, utf16, FakeAx, ScriptedField, WriteBehavior};

// @spec FR-011: Verified direct insertion and fallback — .specs/features/007-smart-paste/spec.md#fr-011

const PID: i32 = 4242;

fn caret_at_end(text: &str) -> Utf16Range {
    Utf16Range {
        location: text.encode_utf16().count(),
        length: 0,
    }
}

fn field(clock: &ManualClock, text: &str) -> ScriptedField {
    ScriptedField::new(clock, PID, text, caret_at_end(text))
}

fn attempt(clock: &ManualClock, field: &ScriptedField, transcript: &str) -> DirectAttempt {
    let mut ax = FakeAx::with_field(field);
    attempt_direct_insertion(clock, &mut ax, PID, &utf16(transcript))
}

fn applied_after(clock: &ManualClock, millis: u64) -> (ScriptedField, DirectAttempt) {
    let field = field(clock, "note: ");
    field.state().behavior = WriteBehavior::ApplyAfter(Duration::from_millis(millis));
    let result = attempt(clock, &field, "hello world");
    (field, result)
}

#[test]
fn immediate_application_is_verified_with_one_write() {
    let clock = ManualClock::new();
    let field = field(&clock, "note: ");
    let result = attempt(&clock, &field, "hello world");
    assert!(matches!(result, DirectAttempt::Verified));
    assert_eq!(field.state().writes, 1);
    assert_eq!(field.value_text(), "note: hello world");
}

#[test]
fn selection_with_astral_characters_is_replaced_in_utf16() {
    let clock = ManualClock::new();
    let range = Utf16Range {
        location: 1,
        length: 2,
    };
    let field = ScriptedField::new(&clock, PID, "A😀Z", range);
    let result = attempt(&clock, &field, "é🦀");
    assert!(matches!(result, DirectAttempt::Verified));
    assert_eq!(field.value_text(), "Aé🦀Z");
}

#[test]
fn transcript_equal_to_the_selection_is_a_verified_no_op() {
    let clock = ManualClock::new();
    let range = Utf16Range {
        location: 0,
        length: 5,
    };
    let field = ScriptedField::new(&clock, PID, "hello", range);
    field.state().behavior = WriteBehavior::Ignore;
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Verified
    ));
}

#[test]
fn ignored_write_falls_back_after_the_full_window() {
    let clock = ManualClock::new();
    let start = clock.now();
    let field = field(&clock, "x");
    field.state().behavior = WriteBehavior::Ignore;
    let result = attempt(&clock, &field, "hello");
    assert!(matches!(result, DirectAttempt::Unchanged(_)));
    assert!(elapsed_since(&clock, start) <= TOTAL_DIRECT_BUDGET);
    assert_eq!(field.state().writes, 1);
}

#[test]
fn setter_status_is_ignored_in_both_directions() {
    let clock = ManualClock::new();
    let applied = field(&clock, "a");
    applied.state().setter_result = Err(AxError::Failed(-25200));
    assert!(matches!(
        attempt(&clock, &applied, "b"),
        DirectAttempt::Verified
    ));
    let ignored = field(&clock, "a");
    ignored.state().behavior = WriteBehavior::Ignore;
    assert!(matches!(
        attempt(&clock, &ignored, "b"),
        DirectAttempt::Unchanged(_)
    ));
}

#[test]
fn late_application_boundary_is_the_last_read_of_the_window() {
    let clock = ManualClock::new();
    for millis in [144, 145] {
        let (_, result) = applied_after(&clock, millis);
        assert!(matches!(result, DirectAttempt::Verified), "{millis} ms");
    }
    let (_, result) = applied_after(&clock, 146);
    assert!(matches!(result, DirectAttempt::Unchanged(_)));
}

#[test]
fn write_applied_after_the_window_is_caught_by_the_fallback_guard() -> Result<(), String> {
    let clock = ManualClock::new();
    let (_, result) = applied_after(&clock, 151);
    let DirectAttempt::Unchanged(mut guard) = result else {
        return Err("expected an unchanged attempt".into());
    };
    assert_eq!(guard.check(), GuardVerdict::StillUnchanged);
    clock.advance(Duration::from_millis(10));
    assert_eq!(guard.check(), GuardVerdict::AppliedLate);
    Ok(())
}

#[test]
fn partial_change_is_uncertain_and_never_falls_back() {
    let clock = ManualClock::new();
    let field = field(&clock, "x");
    field.state().behavior = WriteBehavior::ApplyPartially;
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Uncertain(UncertainReason::ChangedUnexpectedly)
    ));
}

#[test]
fn unreadable_value_after_the_write_is_uncertain() {
    let clock = ManualClock::new();
    let field = field(&clock, "x");
    field.state().behavior = WriteBehavior::Ignore;
    field.state().fail_reads_after_write = true;
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Uncertain(UncertainReason::ReadbackUnavailable)
    ));
}

#[test]
fn a_read_timeout_after_the_write_does_not_hide_a_verified_insertion() {
    let clock = ManualClock::new();
    let field = field(&clock, "x");
    field.state().transient_read_errors = 2;
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Verified
    ));
    assert_eq!(field.state().writes, 1);
}

#[test]
fn an_unchanged_field_after_a_read_timeout_still_falls_back_with_the_guard() {
    let clock = ManualClock::new();
    let field = field(&clock, "x");
    field.state().behavior = WriteBehavior::Ignore;
    field.state().transient_read_errors = 1;
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Unchanged(_)
    ));
}

#[test]
fn secure_field_is_skipped_without_writing() {
    let clock = ManualClock::new();
    let field = field(&clock, "");
    field.state().is_secure = true;
    assert!(matches!(
        attempt(&clock, &field, "pw"),
        DirectAttempt::Skipped(SkipReason::SecureField)
    ));
    assert_eq!(field.state().writes, 0);
}

#[test]
fn element_of_another_process_is_skipped() {
    let clock = ManualClock::new();
    let field = field(&clock, "");
    field.state().pid = PID + 1;
    assert!(matches!(
        attempt(&clock, &field, "x"),
        DirectAttempt::Skipped(SkipReason::PidMismatch)
    ));
    assert_eq!(field.state().writes, 0);
}

#[test]
fn unreadable_or_oversized_values_are_skipped() {
    let clock = ManualClock::new();
    let unreadable = field(&clock, "");
    unreadable.state().value_error = Some(AxError::NoValue);
    assert!(matches!(
        attempt(&clock, &unreadable, "x"),
        DirectAttempt::Skipped(SkipReason::Unreadable)
    ));
    let oversized = field(&clock, "");
    oversized.state().value = vec![u16::from(b'x'); 1_000_001];
    assert!(matches!(
        attempt(&clock, &oversized, "x"),
        DirectAttempt::Skipped(SkipReason::TooLarge)
    ));
    assert_eq!(unreadable.state().writes + oversized.state().writes, 0);
}

#[test]
fn caret_inside_a_surrogate_pair_is_skipped() {
    let clock = ManualClock::new();
    let range = Utf16Range {
        location: 2,
        length: 0,
    };
    let field = ScriptedField::new(&clock, PID, "A😀Z", range);
    assert!(matches!(
        attempt(&clock, &field, "x"),
        DirectAttempt::Skipped(SkipReason::InvalidRange)
    ));
}

#[test]
fn missing_focus_is_skipped() {
    let clock = ManualClock::new();
    let mut ax = FakeAx::with_field(&field(&clock, ""));
    ax.field = None;
    let result = attempt_direct_insertion(&clock, &mut ax, PID, &utf16("x"));
    assert!(matches!(
        result,
        DirectAttempt::Skipped(SkipReason::FieldUnavailable)
    ));
    assert_eq!(ax.focus_request_count(), 1);
}

#[test]
fn slow_preparation_skips_before_writing() {
    let clock = ManualClock::new();
    let field = field(&clock, "x");
    field.state().call_cost = Duration::from_millis(30);
    assert!(matches!(
        attempt(&clock, &field, "hello"),
        DirectAttempt::Skipped(SkipReason::PreparationTooSlow)
    ));
    assert_eq!(field.state().writes, 0);
}

#[test]
fn every_call_is_bounded_and_the_attempt_fits_the_total_budget() {
    let clock = ManualClock::new();
    let start = clock.now();
    let field = field(&clock, "x");
    field.state().behavior = WriteBehavior::Ignore;
    field.state().call_cost = Duration::from_millis(3);
    let _ = attempt(&clock, &field, "hello");
    let state = field.state();
    assert!(state
        .timeouts
        .iter()
        .all(|timeout| !timeout.is_zero() && *timeout <= PER_CALL_TIMEOUT));
    assert!(elapsed_since(&clock, start) <= TOTAL_DIRECT_BUDGET);
}
