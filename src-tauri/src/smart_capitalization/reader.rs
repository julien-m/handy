//! Bounded read of the text before the caret (feature 015, spec AC-005 to AC-007), behind a
//! port so the rules are tested without accessibility. One deadline of `CONTEXT_BUDGET` covers
//! every query; each query's timeout is the remaining budget. The focused element must be
//! editable by 010's rule (which already refuses password fields, terminals' read-only text and
//! unknown roles) before any text is read, and at most `CONTEXT_WINDOW_UNITS` UTF-16 units are
//! read. The text is never logged: errors carry no value.
use super::decision::CursorContext;
use crate::destination::field::MIN_AX_TIMEOUT;
use crate::destination::workspace::Clock;
use crate::recovery::classifier::{
    classify_focused, AxQueryError, Deadline, FocusClassification, FocusedElement,
};
use std::time::Duration;

/// Whole budget of one context read: 010's classification budget (spec decision
/// `budget_50ms_64_units`).
pub const CONTEXT_BUDGET: Duration = Duration::from_millis(50);
/// Most UTF-16 units read before the caret: enough for a sentence end, closing marks, a list
/// marker and its indentation (spec decision `budget_50ms_64_units`).
pub const CONTEXT_WINDOW_UNITS: usize = 64;

/// The focused element of the destination, as the context read needs it.
pub trait CaretElement: Send {
    /// 010's facts (process, role, subrole, edit affordances).
    fn facts(&mut self) -> &mut dyn FocusedElement;
    /// Start of the selection (the caret when nothing is selected), in UTF-16 units.
    fn selection_start(&mut self, timeout: Duration) -> Result<usize, AxQueryError>;
    /// The text of the `length` units starting at `location` (`AXStringForRange`).
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError>;
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

// @spec AC-005: Absent unless an editable field answers, AC-006: Bounded read — .specs/features/015-smart-capitalization/spec.md#ac-005
/// The context of the focused element of the frontmost application `pid`, within
/// `CONTEXT_BUDGET`.
pub fn read_frontmost(port: &mut dyn CaretPort, pid: u32, clock: &dyn Clock) -> CursorContext {
    if !port.is_trusted() {
        return CursorContext::Absent;
    }
    let deadline = Deadline::new(clock, CONTEXT_BUDGET, MIN_AX_TIMEOUT);
    read_within(port, pid, &deadline).unwrap_or(CursorContext::Absent)
}

/// Every failure (error, timeout, spent budget, non-editable element) is `None`: Absent.
fn read_within(
    port: &mut dyn CaretPort,
    pid: u32,
    deadline: &Deadline<'_>,
) -> Option<CursorContext> {
    let mut element = port.focused_element(pid, deadline.remaining().ok()?).ok()?;
    let editable = classify_focused(element.facts(), pid, deadline).ok()?;
    if editable != FocusClassification::Editable {
        return None;
    }
    let caret = element.selection_start(deadline.remaining().ok()?).ok()?;
    let window = window_before(caret);
    if window.1 == 0 {
        return Some(context_from_units(&[], window));
    }
    let units = element
        .text_in(window.0, window.1, deadline.remaining().ok()?)
        .ok()?;
    Some(context_from_units(&units, window))
}

#[cfg(test)]
#[path = "reader_tests.rs"]
mod tests;
