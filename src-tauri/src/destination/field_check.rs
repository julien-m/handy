//! Verification of 011 at 006's pre-dispatch guard (spec AC-002 step 5, AC-004): within
//! `FIELD_VERIFY_BUDGET`, read the application's focused element and, when it is the remembered
//! input, its selection range; then decide once, through the pure table `decide_field_check`,
//! whether the method may paste (`Verified`), insertion continues as without 011 because the
//! attempt changed nothing (`Unchanged`), or nothing may be pasted (`Diverted`).
use super::field::{FieldAccess, FIELD_VERIFY_BUDGET, MIN_AX_TIMEOUT};
use super::field_attempt::{AttemptStage, FieldAttempt, FieldReturn};
use super::field_range::{compare, RangeSeen};
use super::workspace::Clock;
use crate::recovery::classifier::Deadline;
use serde::Serialize;

/// Why the guard refuses to paste after a return attempt (logged, never with content).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "stage", rename_all = "snake_case")]
pub enum Divergence {
    /// The attempt was not made or stopped at this stage, and that alone forbids a paste.
    Attempt(AttemptStage),
    /// The focused element could not be read at the guard.
    FocusUnreadable,
    /// The remembered input is focused but its range is not the remembered one.
    RangeNotVerified,
    /// Another element than the noted one is focused, a window was raised or a write is
    /// unanswered: the attempt may have changed something (stage of the attempt).
    Changed(AttemptStage),
}

/// The guard's field decision, carried by `DestinationContext.field` (internal, never sent).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum FieldCheck {
    /// No input remembered for this insertion: 006 and 010 as delivered.
    #[default]
    Absent,
    /// The remembered input is focused with its remembered range: paste (010 counts it editable).
    Verified,
    /// The attempt changed nothing: insertion as without 011 (stage of the attempt).
    Unchanged(AttemptStage),
    /// Paste nothing: 010's bubble, or 006's `paste-error` when it cannot be shown.
    Diverted(Divergence),
}

/// What the guard observed in the application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusAtGuard {
    /// The remembered input is focused; whether its range is the remembered one.
    Remembered { range_verified: bool },
    /// The element noted before the attempt is focused (and it is not the remembered input).
    Noted,
    /// Another element is focused.
    Other,
    /// The focused element could not be read (error, deadline, permission).
    Unreadable,
}

// @spec FR-004: Paste only into the verified input or unchanged — .specs/features/011-dictation-field-return/spec.md#fr-004
// @spec AC-004: Branches (a) and (b) decided at the guard — .specs/features/011-dictation-field-return/spec.md#ac-004
/// Business rule (spec AC-004, plan guard table, first match): an attempt not made forbids a
/// paste; an unreadable focus forbids it; the remembered input focused with its range verified
/// at the guard pastes, wherever the attempt stopped (decision `verified-at-guard`, N18: what
/// is checked is the state at dispatch; every write the attempt sent targets that input and that
/// range, so one answered late converges on the verified state, and a refused one changed
/// nothing); the remembered input focused otherwise never pastes, even when it was already
/// focused when noted; the noted element still focused with no window raised and no write left
/// unanswered is branch (a); everything else is branch (b).
pub(crate) fn decide_field_check(
    stage: AttemptStage,
    raised: bool,
    unsettled: bool,
    seen: Option<FocusAtGuard>,
) -> FieldCheck {
    if matches!(stage, AttemptStage::RangeMissing | AttemptStage::NotNoted) {
        return FieldCheck::Diverted(Divergence::Attempt(stage));
    }
    match seen.unwrap_or(FocusAtGuard::Unreadable) {
        FocusAtGuard::Unreadable => FieldCheck::Diverted(Divergence::FocusUnreadable),
        FocusAtGuard::Remembered {
            range_verified: true,
        } => FieldCheck::Verified,
        FocusAtGuard::Remembered { .. } => FieldCheck::Diverted(Divergence::RangeNotVerified),
        FocusAtGuard::Noted if !raised && !unsettled => FieldCheck::Unchanged(stage),
        FocusAtGuard::Noted | FocusAtGuard::Other => {
            FieldCheck::Diverted(Divergence::Changed(stage))
        }
    }
}

// @spec AC-002: Verify input and range at the guard within 10 ms — .specs/features/011-dictation-field-return/spec.md#ac-002
/// Verify `field`'s attempt and decide. One debug line: decision, stage, accessibility error,
/// how the range compared after its write and at the guard (which parts differ, never a
/// position, a length or content, spec AC-008) and times.
pub fn verify(field: FieldReturn) -> FieldCheck {
    let FieldReturn {
        mut attempt,
        access,
        clock,
    } = field;
    let started = clock.now();
    let (seen, range) = match attempt.stage {
        AttemptStage::RangeMissing | AttemptStage::NotNoted => (None, RangeSeen::NotRead),
        _ => {
            let (seen, range) = observe(access.as_ref(), &mut attempt, clock.as_ref());
            (Some(seen), range)
        }
    };
    let check = decide_field_check(attempt.stage, attempt.raised, attempt.unsettled, seen);
    let verified_in = clock.now().saturating_duration_since(started);
    let (settled, reads) = attempt.settled;
    log::debug!(
        "field_return {check:?} (stage {:?}, ax_error {:?}, settled {settled:?} in {reads} reads, guard_range {range:?}, note {:?}, attempt {:?}, verify {verified_in:?})",
        attempt.stage,
        attempt.error,
        attempt.noted_in,
        attempt.elapsed
    );
    check
}

/// The focused element now, compared with the remembered input and the noted element, and the
/// remembered input's range when it is focused.
fn observe(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    clock: &dyn Clock,
) -> (FocusAtGuard, RangeSeen) {
    let deadline = Deadline::new(clock, FIELD_VERIFY_BUDGET, MIN_AX_TIMEOUT);
    let Ok(timeout) = deadline.remaining() else {
        return (FocusAtGuard::Unreadable, RangeSeen::NotRead);
    };
    let Ok(focused) = access.focused_element(attempt.field.pid, timeout) else {
        return (FocusAtGuard::Unreadable, RangeSeen::NotRead);
    };
    if focused.same_as(attempt.field.element.as_ref()) {
        let range = range_now(attempt, &deadline);
        let range_verified = range.accepts();
        return (FocusAtGuard::Remembered { range_verified }, range);
    }
    let noted = attempt.noted.as_deref();
    if noted.is_some_and(|noted| focused.same_as(noted)) {
        (FocusAtGuard::Noted, RangeSeen::NotRead)
    } else {
        (FocusAtGuard::Other, RangeSeen::NotRead)
    }
}

/// The remembered input's selection now against the remembered one (the guard runs this only
/// for an attempt made, so with a remembered range).
fn range_now(attempt: &mut FieldAttempt, deadline: &Deadline<'_>) -> RangeSeen {
    let Some(range) = attempt.field.range.as_ref() else {
        return RangeSeen::NotRead;
    };
    match deadline.remaining() {
        Ok(timeout) => compare(attempt.field.element.as_mut(), range, timeout),
        Err(_) => RangeSeen::NotRead,
    }
}

#[cfg(test)]
#[path = "field_check_tests.rs"]
mod tests;
