//! Return attempt of 011 at insertion (spec AC-002 steps 1 to 4), on 006's insertion worker
//! after the destination is confirmed in front: note the application's focused element, give
//! focus back to the remembered input, raise its window when it is not the focused one, then
//! restore the remembered range. Once, without retry, under one `FIELD_RESTORE_BUDGET`; a step
//! that fails ends the attempt. The facts the guard needs (noted element, window raised, a write
//! left without an answer) are kept for its decision; nothing is read of the input's content.
use super::field::{
    FieldAccess, FieldNode, RememberedField, SelectionRange, FIELD_RESTORE_BUDGET, MIN_AX_TIMEOUT,
};
use super::workspace::{Clock, RunningApp};
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
    /// The window could not be read or raised.
    RaiseFailed,
    /// The range write was refused at once.
    RangeNotRestored,
    /// Less than `MIN_AX_TIMEOUT` was left before a call; nothing was sent for it.
    BudgetSpent,
}

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
    pub elapsed: Duration,
}

/// The attempt handed to the guard with the ports its verification uses.
pub struct FieldReturn {
    pub attempt: FieldAttempt,
    pub access: Arc<dyn FieldAccess>,
    pub clock: Arc<dyn Clock>,
}

// @spec FR-002: Note then focus then raise then restore — .specs/features/011-dictation-field-return/spec.md#fr-002
// @spec AC-002: One attempt within 50 ms without retry — .specs/features/011-dictation-field-return/spec.md#ac-002
/// Run the attempt for `field` once; see the module documentation.
pub fn attempt(
    access: &dyn FieldAccess,
    field: RememberedField,
    clock: &dyn Clock,
) -> FieldAttempt {
    let started = clock.now();
    let mut attempt = FieldAttempt {
        field,
        noted: None,
        raised: false,
        unsettled: false,
        stage: AttemptStage::RangeMissing,
        elapsed: Duration::ZERO,
    };
    if let Some(range) = attempt.field.range.clone() {
        let deadline = Deadline::new(clock, FIELD_RESTORE_BUDGET, MIN_AX_TIMEOUT);
        attempt.stage = match run_steps(access, &mut attempt, &range, &deadline) {
            Ok(()) => AttemptStage::Restored,
            Err(stage) => stage,
        };
    }
    attempt.elapsed = clock.now().saturating_duration_since(started);
    attempt
}

/// Steps 1 to 4 in order (spec AC-002): the note must precede the focus write so the guard can
/// tell whether the attempt changed the focus; the range is restored last because giving focus
/// or raising a window may move the caret.
fn run_steps(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    range: &SelectionRange,
    deadline: &Deadline<'_>,
) -> Result<(), AttemptStage> {
    let pid = attempt.field.pid;
    // Without a noted element nothing can later prove the attempt changed nothing: no attempt.
    let timeout = deadline.remaining().map_err(|_| AttemptStage::NotNoted)?;
    let noted = access
        .focused_element(pid, timeout)
        .map_err(|_| AttemptStage::NotNoted)?;
    attempt.noted = Some(noted);
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
) -> Result<(), AttemptStage> {
    match result {
        Ok(()) => Ok(()),
        Err(AxQueryError::Timeout) => {
            *unsettled = true;
            Err(AttemptStage::WriteUnanswered)
        }
        Err(_) => Err(refused),
    }
}

/// Step 3: raise the input's window when it is not the application's focused window. `raised`
/// is set as soon as the action is sent, whatever its result (spec AC-002).
fn raise_if_needed(
    access: &dyn FieldAccess,
    attempt: &mut FieldAttempt,
    deadline: &Deadline<'_>,
) -> Result<(), AttemptStage> {
    let window = attempt.field.element.window(remaining(deadline)?);
    let Some(mut window) = window.map_err(|_| AttemptStage::RaiseFailed)? else {
        // UI Automation: `SetFocus` already brings the element's window.
        return Ok(());
    };
    let focused = access
        .focused_window(attempt.field.pid, remaining(deadline)?)
        .map_err(|_| AttemptStage::RaiseFailed)?;
    if window.same_as(focused.as_ref()) {
        return Ok(());
    }
    let timeout = remaining(deadline)?;
    attempt.raised = true;
    window.raise(timeout).map_err(|_| AttemptStage::RaiseFailed)
}

fn remaining(deadline: &Deadline<'_>) -> Result<Duration, AttemptStage> {
    deadline.remaining().map_err(|_| AttemptStage::BudgetSpent)
}

/// The attempt of `field` when 006 brought its very instance forward (`target`) and the
/// dictation is not `cancelled`; `None` drops the field unused: no activation route (paste
/// method "none", frontmost at the end, return off without pin), another instance than the one
/// read, or a cancellation (spec AC-002, AC-006).
pub fn begin(
    access: Arc<dyn FieldAccess>,
    clock: Arc<dyn Clock>,
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
    let attempt = attempt(access.as_ref(), field, clock.as_ref());
    Some(FieldReturn {
        attempt,
        access,
        clock,
    })
}
