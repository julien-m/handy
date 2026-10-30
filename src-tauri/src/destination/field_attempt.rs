//! Return attempt of 011 at insertion (spec AC-002 steps 1 to 4), on 006's insertion worker
//! after the destination is confirmed in front: note the application's focused element, give
//! focus back to the remembered input, raise its window when it is not the focused one, then
//! restore the remembered range. Once, without retry: the note under `FIELD_NOTE_BUDGET` (a read,
//! so waiting for an application that answers late changes nothing), then the writes under one
//! `FIELD_RESTORE_BUDGET` started when the note answered; a step that fails ends the attempt. A
//! range write answered at once is read back within the same budget until it settles
//! (`field_range::settle`: Chromium applies it a moment later). The
//! facts the guard needs (noted element, window raised, a write left without an answer, the
//! stage and accessibility error where it stopped) are kept for its decision and its log;
//! nothing is read of the input's content.
use super::field::{
    FieldAccess, FieldNode, RememberedField, SelectionRange, FIELD_NOTE_BUDGET,
    FIELD_RESTORE_BUDGET, MIN_AX_TIMEOUT,
};
use super::field_range::{settle, RangeSeen};
use super::workspace::{Clock, Pause, RunningApp};
use crate::recovery::classifier::{AxQueryError, Deadline};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

/// Where the attempt stopped (logged, never with content or range).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStage {
    /// Focus given, window raised when needed, range restored.
    Restored,
    /// The range was not read at the start: no call was made (spec AC-004 b).
    RangeMissing,
    /// The application's focused element could not be noted: no attempt (spec AC-004 b).
    NotNoted,
    /// The focus write was refused at once (element gone, or not focusable).
    FocusRefused,
    /// A focus or range write got no answer in time; the target may still apply it.
    WriteUnanswered,
    /// The window could not be read or raised: refused at once.
    RaiseFailed,
    /// The window read or the raise got no answer in time.
    RaiseUnanswered,
    /// The range write was refused at once.
    RangeNotRestored,
    /// Less than `MIN_AX_TIMEOUT` was left before a call; nothing was sent for it.
    BudgetSpent,
}
/// Where a step stopped, with the accessibility error when one was returned (never content).
type Stop = (AttemptStage, Option<AxQueryError>);

/// One attempt and its facts (spec Key Entities, "during insertion").
pub struct FieldAttempt {
    pub field: RememberedField,
    /// The application's focused element noted before the attempt.
    pub noted: Option<Box<dyn FieldNode>>,
    /// A window raise was sent, whatever its result.
    pub raised: bool,
    /// A focus or range write got no answer before its deadline.
    pub unsettled: bool,
    pub stage: AttemptStage,
    /// The accessibility error of the step that stopped the attempt, if any (logged).
    pub error: Option<AxQueryError>,
    /// Time the note took to answer (logged).
    pub noted_in: Duration,
    /// The range read back after its write, and how many reads it took (logged, no value).
    pub settled: (RangeSeen, u8),
    pub elapsed: Duration,
}

/// The attempt handed to the guard with the ports its verification uses.
pub struct FieldReturn {
    pub attempt: FieldAttempt,
    pub access: Arc<dyn FieldAccess>,
    pub clock: Arc<dyn Clock>,
}

// @spec FR-002: Note then focus then raise then restore — .specs/features/011-dictation-field-return/spec.md#fr-002
// @spec AC-002: Note in 300 ms, then writes in 50 ms, once — .specs/features/011-dictation-field-return/spec.md#ac-002
/// Run the attempt for `field` once; see the module documentation.
pub fn attempt(
    access: &dyn FieldAccess,
    field: RememberedField,
    clock: &dyn Clock,
    pause: &dyn Pause,
) -> FieldAttempt {
    let started = clock.now();
    let mut attempt = FieldAttempt {
        field,
        noted: None,
        raised: false,
        unsettled: false,
        stage: AttemptStage::RangeMissing,
        error: None,
        noted_in: Duration::ZERO,
        settled: (RangeSeen::NotRead, 0),
        elapsed: Duration::ZERO,
    };
    if let Some(range) = attempt.field.range.clone() {
        (attempt.stage, attempt.error) = match run(access, &mut attempt, &range, (clock, pause)) {
            Ok(()) => (AttemptStage::Restored, None),
            Err(stop) => stop,
        };
    }
    attempt.elapsed = clock.now().saturating_duration_since(started);
    attempt
}

/// Step 1, then steps 2 to 4 under the writes' budget, which starts once the application has
/// answered the note (N18), then the read-back of a range write answered at once.
fn run(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    range: &SelectionRange,
    (clock, pause): (&dyn Clock, &dyn Pause),
) -> Result<(), Stop> {
    let started = clock.now();
    note(access, attempt, clock)?;
    attempt.noted_in = clock.now().saturating_duration_since(started);
    let deadline = Deadline::new(clock, FIELD_RESTORE_BUDGET, MIN_AX_TIMEOUT);
    restore(access, attempt, range, &deadline)?;
    attempt.settled = settle(attempt.field.element.as_mut(), range, &deadline, pause);
    Ok(())
}

/// Step 1 (spec AC-002): note the application's focused element, under `FIELD_NOTE_BUDGET`.
/// Without a noted element nothing can later prove the attempt changed nothing: no attempt.
fn note(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    clock: &dyn Clock,
) -> Result<(), Stop> {
    let deadline = Deadline::new(clock, FIELD_NOTE_BUDGET, MIN_AX_TIMEOUT);
    let timeout = deadline
        .remaining()
        .map_err(|_| (AttemptStage::NotNoted, None))?;
    let noted = access
        .focused_element(attempt.field.pid, timeout)
        .map_err(|error| (AttemptStage::NotNoted, Some(error)))?;
    attempt.noted = Some(noted);
    Ok(())
}

/// Steps 2 to 4 in order (spec AC-002): the note precedes the focus write so the guard can tell
/// whether the attempt changed the focus; the range is restored last because giving focus or
/// raising a window may move the caret.
fn restore(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    range: &SelectionRange,
    deadline: &Deadline<'_>,
) -> Result<(), Stop> {
    let focused = attempt.field.element.focus(remaining(deadline)?);
    write_result(focused, AttemptStage::FocusRefused, &mut attempt.unsettled)?;
    raise_if_needed(access, attempt, deadline)?;
    let selected = attempt.field.element.select(range, remaining(deadline)?);
    write_result(
        selected,
        AttemptStage::RangeNotRestored,
        &mut attempt.unsettled,
    )
}

/// Business rule (spec AC-002): a write refused at once changed nothing; a write without an
/// answer in time (`Timeout`, the AX messaging timeout set to the remaining budget) may still be
/// applied, so the attempt has changed something.
fn write_result(
    result: Result<(), AxQueryError>,
    refused: AttemptStage,
    unsettled: &mut bool,
) -> Result<(), Stop> {
    match result {
        Ok(()) => Ok(()),
        Err(AxQueryError::Timeout) => {
            *unsettled = true;
            Err((AttemptStage::WriteUnanswered, Some(AxQueryError::Timeout)))
        }
        Err(error) => Err((refused, Some(error))),
    }
}

/// A window step that failed: unanswered in time, or refused at once.
fn raise_error(error: AxQueryError) -> Stop {
    let stage = match error {
        AxQueryError::Timeout => AttemptStage::RaiseUnanswered,
        _ => AttemptStage::RaiseFailed,
    };
    (stage, Some(error))
}

/// Step 3: raise the input's window when it is not the application's focused window. `raised`
/// is set as soon as the action is sent, whatever its result (spec AC-002).
fn raise_if_needed(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    deadline: &Deadline<'_>,
) -> Result<(), Stop> {
    let window = attempt.field.element.window(remaining(deadline)?);
    let Some(mut window) = window.map_err(raise_error)? else {
        // UI Automation: `SetFocus` already brings the element's window.
        return Ok(());
    };
    let focused = access
        .focused_window(attempt.field.pid, remaining(deadline)?)
        .map_err(raise_error)?;
    if window.same_as(focused.as_ref()) {
        return Ok(());
    }
    let timeout = remaining(deadline)?;
    attempt.raised = true;
    window.raise(timeout).map_err(raise_error)
}

fn remaining(deadline: &Deadline<'_>) -> Result<Duration, Stop> {
    deadline
        .remaining()
        .map_err(|_| (AttemptStage::BudgetSpent, None))
}

/// The attempt of `field` when 006 brought its very instance forward (`target`) and the
/// dictation is not `cancelled`; `None` drops the field unused: no activation route (paste
/// method "none", frontmost at the end, return off without pin), another instance than the one
/// read, or a cancellation (spec AC-002, AC-006).
pub fn begin(
    access: Arc<dyn FieldAccess>,
    (clock, pause): (Arc<dyn Clock>, &dyn Pause),
    field: Option<RememberedField>,
    (target, cancelled): (Option<&RunningApp>, &dyn Fn() -> bool),
) -> Option<FieldReturn> {
    let field = field?;
    // A dictation cancelled since its activation never gets a focus change (spec AC-006).
    if cancelled() {
        log::debug!("field_return dropped(cancelled)");
        return None;
    }
    if !target.is_some_and(|app| app.pid == field.pid && app.identity == field.identity) {
        log::debug!("field_return dropped(no_matching_activation)");
        return None;
    }
    let attempt = attempt(access.as_ref(), field, clock.as_ref(), pause);
    Some(FieldReturn {
        attempt,
        access,
        clock,
    })
}
