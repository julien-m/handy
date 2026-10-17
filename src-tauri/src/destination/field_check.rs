//! Verification of 011 at 006's pre-dispatch guard (spec AC-002 step 5, AC-004): within
//! `FIELD_VERIFY_BUDGET`, read the application's focused element and, when it is the remembered
//! input, its selection range; then decide once, through the pure table `decide_field_check`,
//! whether the method may paste (`Verified`), insertion continues as without 011 because the
//! attempt changed nothing (`Unchanged`), or nothing may be pasted (`Diverted`).
use super::field::{FieldAccess, FIELD_VERIFY_BUDGET, MIN_AX_TIMEOUT};
use super::field_attempt::{AttemptStage, FieldAttempt, FieldReturn};
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
/// paste; an unreadable focus forbids it; the remembered input focused with its range restored
/// and verified pastes; the remembered input focused otherwise never pastes, even when it was
/// already focused when noted; the noted element still focused with no window raised and no
/// write left unanswered is branch (a); everything else is branch (b).
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
        } if stage == AttemptStage::Restored && !unsettled => FieldCheck::Verified,
        FocusAtGuard::Remembered { .. } => FieldCheck::Diverted(Divergence::RangeNotVerified),
        FocusAtGuard::Noted if !raised && !unsettled => FieldCheck::Unchanged(stage),
        FocusAtGuard::Noted | FocusAtGuard::Other => {
            FieldCheck::Diverted(Divergence::Changed(stage))
        }
    }
}

// @spec AC-002: Verify input and range at the guard within 10 ms — .specs/features/011-dictation-field-return/spec.md#ac-002
/// Verify `field`'s attempt and decide (one debug line, stages and times only).
pub fn verify(field: FieldReturn) -> FieldCheck {
    let FieldReturn {
        mut attempt,
        access,
        clock,
    } = field;
    let started = clock.now();
    let seen = match attempt.stage {
        AttemptStage::RangeMissing | AttemptStage::NotNoted => None,
        _ => Some(observe(access.as_ref(), &mut attempt, clock.as_ref())),
    };
    let check = decide_field_check(attempt.stage, attempt.raised, attempt.unsettled, seen);
    let verified_in = clock.now().saturating_duration_since(started);
    log::debug!(
        "field_return {check:?} (attempt {:?}, verify {verified_in:?})",
        attempt.elapsed
    );
    check
}

/// The focused element now, compared with the remembered input and the noted element.
fn observe(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    clock: &dyn Clock,
) -> FocusAtGuard {
    let deadline = Deadline::new(clock, FIELD_VERIFY_BUDGET, MIN_AX_TIMEOUT);
    let Ok(timeout) = deadline.remaining() else {
        return FocusAtGuard::Unreadable;
    };
    let Ok(focused) = access.focused_element(attempt.field.pid, timeout) else {
        return FocusAtGuard::Unreadable;
    };
    if focused.same_as(attempt.field.element.as_ref()) {
        return FocusAtGuard::Remembered {
            range_verified: range_verified(attempt, &deadline),
        };
    }
    let noted = attempt.noted.as_deref();
    if noted.is_some_and(|noted| focused.same_as(noted)) {
        FocusAtGuard::Noted
    } else {
        FocusAtGuard::Other
    }
}

/// The remembered range is the input's selection now (only after a restored attempt).
fn range_verified(attempt: &mut FieldAttempt, deadline: &Deadline<'_>) -> bool {
    let (Some(range), AttemptStage::Restored) = (attempt.field.range.as_ref(), attempt.stage)
    else {
        return false;
    };
    let Ok(timeout) = deadline.remaining() else {
        return false;
    };
    attempt
        .field
        .element
        .range_is(range, timeout)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "field_check_tests.rs"]
mod tests;
