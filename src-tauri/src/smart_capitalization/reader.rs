//! Bounded read of the text around the selection (feature 015, spec AC-005 to AC-007; the side
//! after the selection is smart spacing's, 017 AC-009), behind ports so the rules are tested
//! without accessibility. One deadline of `CONTEXT_BUDGET` covers every query; each query's
//! timeout is the remaining budget. The focused element must be editable by 010's rule (which
//! already refuses password fields, terminals' read-only text and unknown roles) before any
//! text is read, and at most `CONTEXT_WINDOW_UNITS` UTF-16 units before the selection and
//! `AFTER_WINDOW_UNITS` after it are read. The text is never logged: errors carry no value.
use super::context::{AfterContext, InsertionContext};
use super::decision::CursorContext;
use crate::destination::field::{FieldNode, MIN_AX_TIMEOUT};
use crate::destination::workspace::Clock;
use crate::recovery::classifier::{
    classify_focused, AxQueryError, Deadline, FocusClassification, FocusedElement, Undetermined,
};
use std::time::Duration;

/// Whole budget of one context read: 010's classification budget (spec decision
/// `budget_50ms_64_units`).
pub const CONTEXT_BUDGET: Duration = Duration::from_millis(50);
/// Most UTF-16 units read before the caret: enough for a sentence end, closing marks, a list
/// marker and its indentation (spec decision `budget_50ms_64_units`).
pub const CONTEXT_WINDOW_UNITS: usize = 64;
/// Most UTF-16 units read after the selection (017 decision `after_window_16`): the junction
/// rule needs the first character and the one after it; 16 units hold them with room for a
/// surrogate pair and combining marks while keeping the read short.
pub const AFTER_WINDOW_UNITS: usize = 16;

/// The text of an editable field, as the context read needs it.
pub trait FieldText {
    /// The selection (the caret when its length is 0), as start and length in UTF-16 units.
    fn selection(&mut self, timeout: Duration) -> Result<(usize, usize), AxQueryError>;
    /// The field's length in UTF-16 units (`AXNumberOfCharacters`), for the read after (017).
    fn character_count(&mut self, timeout: Duration) -> Result<usize, AxQueryError>;
    /// The text of the `length` units starting at `location` (`AXStringForRange`).
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError>;
}

/// The focused element of the destination, as the context read needs it.
pub trait CaretElement: FieldText + Send {
    /// 010's facts (process, role, subrole, edit affordances).
    fn facts(&mut self) -> &mut dyn FocusedElement;
}

/// Accessibility of this process for the context read.
pub trait CaretPort: Send {
    /// Accessibility permission, checked without prompting and before any query.
    fn is_trusted(&mut self) -> bool;
    /// The focused element of application `pid`.
    fn focused_element(
        &mut self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn CaretElement>, AxQueryError>;
}

/// A platform without a context reader (Windows and Linux in this feature): never trusted.
pub struct UnsupportedCaretPort;
impl CaretPort for UnsupportedCaretPort {
    fn is_trusted(&mut self) -> bool {
        false
    }
    fn focused_element(
        &mut self,
        _pid: u32,
        _timeout: Duration,
    ) -> Result<Box<dyn CaretElement>, AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
}

/// The window read before `caret`: its start and its length.
pub fn window_before(caret: usize) -> (usize, usize) {
    let length = caret.min(CONTEXT_WINDOW_UNITS);
    (caret - length, length)
}

/// The window read after the selection `end` of a field of `count` units; none when the
/// selection ends beyond the field (an answer that cannot be trusted).
pub fn window_after(end: usize, count: usize) -> Option<(usize, usize)> {
    let rest = count.checked_sub(end)?;
    Some((end, rest.min(AFTER_WINDOW_UNITS)))
}

/// The context from the `units` read in the window `(start, length)`; more units than asked is
/// an answer that cannot be trusted (Absent).
pub fn context_from_units(units: &[u16], (start, length): (usize, usize)) -> CursorContext {
    if units.len() > length {
        return CursorContext::Absent;
    }
    CursorContext::Before {
        // An unpaired surrogate at the window start becomes U+FFFD, which never decides.
        text: String::from_utf16_lossy(units),
        reaches_start: start == 0,
    }
}

/// The context after the selection from the `units` read in the window `(start, length)` of
/// a field of `count` units; more units than asked is Absent.
pub fn after_from_units(
    units: &[u16],
    (start, length): (usize, usize),
    count: usize,
) -> AfterContext {
    if units.len() > length {
        return AfterContext::Absent;
    }
    AfterContext::After {
        // Lossy only at the window end (a cut surrogate pair): the junction reads the start.
        text: String::from_utf16_lossy(units),
        reaches_end: start + units.len() == count,
    }
}

// @spec AC-005: Absent unless an editable field answers, AC-006: Bounded read — .specs/features/015-smart-capitalization/spec.md#ac-005
/// The context of the focused element of the frontmost application `pid`, within
/// `CONTEXT_BUDGET`; the side after the selection only `with_after` (smart spacing on).
pub fn read_frontmost(
    port: &mut dyn CaretPort,
    pid: u32,
    clock: &dyn Clock,
    with_after: bool,
) -> InsertionContext {
    if !port.is_trusted() {
        return InsertionContext::Unavailable;
    }
    let deadline = Deadline::new(clock, CONTEXT_BUDGET, MIN_AX_TIMEOUT);
    let Some(mut element) = deadline
        .remaining()
        .ok()
        .and_then(|timeout| port.focused_element(pid, timeout).ok())
    else {
        return InsertionContext::Unavailable;
    };
    match classify_focused(element.facts(), pid, &deadline) {
        Ok(FocusClassification::Editable) => read_field(element.as_mut(), &deadline, with_after),
        Ok(FocusClassification::NotEditable) => InsertionContext::NotText,
        Ok(FocusClassification::Undetermined(reason)) | Err(reason) => unread(reason),
        Ok(FocusClassification::PermissionMissing) => InsertionContext::Unavailable,
    }
}

// @spec AC-010: Not a text field — .specs/features/017-smart-spacing/spec.md#ac-010
/// Business rule (017 decision `not_text_steps_aside`): 010's answers that prove the focus is
/// something else than an editable text field (a password field, a text without edit
/// affordance such as a terminal's output, an unknown role) are `NotText`; a failure to learn
/// anything (no element, another process, an error, a timeout, a spent budget) is
/// `Unavailable`.
fn unread(reason: Undetermined) -> InsertionContext {
    match reason {
        Undetermined::SecureField
        | Undetermined::TextWithoutAffordance
        | Undetermined::UnknownRole
        | Undetermined::UnexpectedAffordance => InsertionContext::NotText,
        Undetermined::NoFocusedElement
        | Undetermined::ForeignProcess
        | Undetermined::AxError
        | Undetermined::Timeout
        | Undetermined::BudgetSpent => InsertionContext::Unavailable,
    }
}

// @spec FR-003: Reader extended after the selection, AC-009: Bounded read after the selection — .specs/features/017-smart-spacing/spec.md#fr-003
/// The text around the selection of an editable `field`, within `deadline`. A failed read
/// before the selection is `Unavailable`; a failed read after it only leaves that side
/// `Absent` (partial context).
pub(crate) fn read_field<F: FieldText + ?Sized>(
    field: &mut F,
    deadline: &Deadline<'_>,
    with_after: bool,
) -> InsertionContext {
    let Some((start, length)) = deadline
        .remaining()
        .ok()
        .and_then(|timeout| field.selection(timeout).ok())
    else {
        return InsertionContext::Unavailable;
    };
    let Some(before) = read_before(field, start, deadline) else {
        return InsertionContext::Unavailable;
    };
    let after = match start.checked_add(length) {
        Some(end) if with_after => read_after(field, end, deadline),
        _ => AfterContext::Absent,
    };
    InsertionContext::Field { before, after }
}

/// The text before `caret`; `None` when it cannot be read or trusted.
fn read_before<F: FieldText + ?Sized>(
    field: &mut F,
    caret: usize,
    deadline: &Deadline<'_>,
) -> Option<CursorContext> {
    let window = window_before(caret);
    if window.1 == 0 {
        return Some(context_from_units(&[], window));
    }
    let units = field
        .text_in(window.0, window.1, deadline.remaining().ok()?)
        .ok()?;
    // The answer is anchored at the window start: fewer units than asked do not end at the
    // caret, so they cannot be taken for the text right before it (017 review finding W2).
    if units.len() < window.1 {
        return None;
    }
    match context_from_units(&units, window) {
        CursorContext::Absent => None,
        before => Some(before),
    }
}

/// The text after the selection `end`; `Absent` when any query fails or the budget is spent.
fn read_after<F: FieldText + ?Sized>(
    field: &mut F,
    end: usize,
    deadline: &Deadline<'_>,
) -> AfterContext {
    let answer = (|| {
        let count = field.character_count(deadline.remaining().ok()?).ok()?;
        let window = window_after(end, count)?;
        if window.1 == 0 {
            return Some(after_from_units(&[], window, count));
        }
        let units = field
            .text_in(window.0, window.1, deadline.remaining().ok()?)
            .ok()?;
        Some(after_from_units(&units, window, count))
    })();
    answer.unwrap_or(AfterContext::Absent)
}

/// 011's remembered input, read at its remembered selection (`range`): the selection is the
/// one the paste restores, so no query asks for it again.
struct RememberedText<'a> {
    node: &'a mut dyn FieldNode,
    range: (usize, usize),
}

impl FieldText for RememberedText<'_> {
    fn selection(&mut self, _timeout: Duration) -> Result<(usize, usize), AxQueryError> {
        // No query: the remembered range is where the paste returns (015 AC-007).
        Ok(self.range)
    }
    fn character_count(&mut self, timeout: Duration) -> Result<usize, AxQueryError> {
        self.node.character_count(timeout)
    }
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError> {
        self.node.text_in(location, length, timeout)
    }
}

// @spec AC-007: Context of 011's remembered caret — .specs/features/015-smart-capitalization/spec.md#ac-007
/// The context of 011's remembered input `node` at its remembered selection `range` (offsets
/// only; none is `Unavailable`), within `CONTEXT_BUDGET` on `clock`. 011 remembered it as
/// editable by 010's rule, so it is not classified again.
pub fn read_remembered(
    node: &mut dyn FieldNode,
    range: Option<(usize, usize)>,
    clock: &dyn Clock,
    with_after: bool,
) -> InsertionContext {
    let Some(range) = range else {
        return InsertionContext::Unavailable;
    };
    let deadline = Deadline::new(clock, CONTEXT_BUDGET, MIN_AX_TIMEOUT);
    read_field(&mut RememberedText { node, range }, &deadline, with_after)
}

#[cfg(test)]
#[path = "reader_tests.rs"]
mod tests;
