//! Field return (feature 011): the input focused in the destination application when a
//! dictation starts is remembered as an opaque accessibility reference with its selection
//! range, never its content, then given focus back, its range restored and both verified at
//! 006's pre-dispatch guard before the unchanged insertion method pastes. This module holds the
//! vocabulary and the platform port; `field_capture`, `field_attempt` and `field_check` hold
//! the bounded sequences and the decision table, `field_lifetime` the session rules.
use super::context::{ApplicationIdentity, DictationId};
use super::session::Selection;
use crate::recovery::classifier::{AxQueryError, FocusedElement};
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// One deadline for every read made at the start (spec AC-001: 010's classification budget).
pub const FIELD_CAPTURE_BUDGET: Duration = Duration::from_millis(50);
/// Deadline of the note at insertion (spec AC-002 step 1), a read that changes nothing: a
/// Chromium or Electron application brought forward a moment ago may take longer than 50 ms to
/// answer its first accessibility request, so the note waits for it up to this bound (N18).
pub const FIELD_NOTE_BUDGET: Duration = Duration::from_millis(300);
/// One deadline for the focus, window and range steps at insertion, started once the note has
/// answered (spec AC-002 steps 2 to 4).
pub const FIELD_RESTORE_BUDGET: Duration = Duration::from_millis(50);
/// One deadline for the verification inside the pre-dispatch guard (spec AC-002 step 5).
pub const FIELD_VERIFY_BUDGET: Duration = Duration::from_millis(10);
/// Smallest messaging timeout worth sending: below it the operation stops as spent. Same value
/// as 007's `MIN_READ_TIMEOUT` (plan "Budgets and Accessibility timeouts"); never 0, which
/// would mean the default timeout.
pub const MIN_AX_TIMEOUT: Duration = Duration::from_millis(5);

/// The remembered selection: position and length on macOS, an opaque copy of the selection's
/// endpoints on Windows. Deliberately without `Debug`, `Display` or `Serialize`, so it can
/// never reach a log, an event or the CLI (spec AC-008).
#[derive(Clone)]
pub struct SelectionRange(RangeRepr);
#[derive(Clone)]
enum RangeRepr {
    Offsets { location: usize, length: usize },
    Endpoints(Arc<dyn Any + Send + Sync>),
}
impl SelectionRange {
    /// A range of `length` UTF-16 units starting at `location` (macOS `CFRange`).
    pub fn offsets(location: usize, length: usize) -> Self {
        Self(RangeRepr::Offsets { location, length })
    }
    /// An opaque platform copy of the selection's endpoints (Windows text range).
    pub fn endpoints(handle: Arc<dyn Any + Send + Sync>) -> Self {
        Self(RangeRepr::Endpoints(handle))
    }
    /// Position and length, for an offsets range.
    pub fn as_offsets(&self) -> Option<(usize, usize)> {
        match &self.0 {
            RangeRepr::Offsets { location, length } => Some((*location, *length)),
            RangeRepr::Endpoints(_) => None,
        }
    }
    /// The platform copy, for an endpoints range.
    pub fn as_endpoints(&self) -> Option<&(dyn Any + Send + Sync)> {
        match &self.0 {
            RangeRepr::Offsets { .. } => None,
            RangeRepr::Endpoints(handle) => Some(handle.as_ref()),
        }
    }
}

/// One accessibility element obtained by 011: the facts 010's rule needs, the selection range,
/// focus and range writes, its window, and identity with another element. No method returns
/// the element's value, selected text or title; only `text_in` reads a bounded slice, for 015.
/// Write timeouts answer `AxQueryError::Timeout` when the target did not answer in time (the
/// write may still be applied later).
pub trait FieldNode: FocusedElement + Send {
    /// `AXSelectedTextRange` (macOS) or the first `TextPattern` selection (Windows), bounded.
    fn read_range(&mut self, timeout: Duration) -> Result<SelectionRange, AxQueryError>;
    /// Whether the element's current selection is exactly `range`, bounded.
    fn range_is(&mut self, range: &SelectionRange, timeout: Duration)
        -> Result<bool, AxQueryError>;
    /// Give the element keyboard focus (`kAXFocusedAttribute` true, or `SetFocus`).
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError>;
    /// Restore `range` as the element's selection (`AXSelectedTextRange`, or `Select`).
    fn select(&mut self, range: &SelectionRange, timeout: Duration) -> Result<(), AxQueryError>;
    /// The element's window (`AXWindow`); `None` where focus already brings its window (UIA).
    fn window(&mut self, timeout: Duration) -> Result<Option<Box<dyn FieldNode>>, AxQueryError>;
    /// Raise this window (`AXRaise`).
    fn raise(&mut self, timeout: Duration) -> Result<(), AxQueryError>;
    /// Same element as `other` (`CFEqual` and process, or `CompareElements`); a local call.
    fn same_as(&self, other: &dyn FieldNode) -> bool;
    /// The platform object, for `same_as` between two elements of one adapter.
    fn as_any(&self) -> &dyn Any;
    /// Smart capitalization (015) only, with its option on: the `length` UTF-16 units starting
    /// at `location` (`AXStringForRange`), bounded by `timeout`. The one read of content this
    /// port allows; never logged or kept. Adapters without it answer `Unsupported`.
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError> {
        // Unused here: an adapter without a bounded text read has nothing to read.
        let _ = (location, length, timeout);
        Err(AxQueryError::Unsupported)
    }
}

/// Accessibility of this process for 011, per application element (never system-wide).
pub trait FieldAccess: Send + Sync {
    /// Accessibility permission, checked without prompting and before any query.
    fn is_trusted(&self) -> bool;
    /// The focused element of application `pid`, with `timeout` set on its application element.
    fn focused_element(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError>;
    /// The focused window of application `pid` (`AXFocusedWindow`).
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError>;
}

/// A platform without field return (Linux, and every build until an adapter is installed):
/// never trusted, so no query is ever made.
pub struct UnsupportedFieldAccess;
impl FieldAccess for UnsupportedFieldAccess {
    fn is_trusted(&self) -> bool {
        false
    }
    fn focused_element(&self, _pid: u32, _: Duration) -> Result<Box<dyn FieldNode>, AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
    fn focused_window(&self, _pid: u32, _: Duration) -> Result<Box<dyn FieldNode>, AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
}

/// The running instance whose focused element is read at the start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldTarget {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub identity: ApplicationIdentity,
    /// The process the selection names (006's captured PID or 009's `pid_hint`).
    pub route_pid: u32,
}

/// The input remembered for one dictation (spec Key Entities "Remembered input"): never its
/// content; the range is never logged. Released with the session, a cancellation, a
/// supersession, a destination change, or once the guard has decided.
pub struct RememberedField {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub identity: ApplicationIdentity,
    /// The running instance whose element this is.
    pub pid: u32,
    /// The process the selection named when it was read.
    pub route_pid: u32,
    pub element: Box<dyn FieldNode>,
    /// `None` when the range could not be read at the start (spec AC-004 b at insertion).
    pub range: Option<SelectionRange>,
    pub captured_at: Instant,
}
impl RememberedField {
    /// True while `selection` is still this field's application instance (spec AC-003): the same
    /// identity, and the process the route named or the one read.
    pub fn serves(&self, selection: &Selection) -> bool {
        match selection {
            Selection::Application(destination) => {
                destination.identity == self.identity
                    && (destination.captured_pid == self.route_pid
                        || destination.captured_pid == self.pid)
            }
            Selection::FrontmostAtEnd => false,
        }
    }
}
impl std::fmt::Debug for RememberedField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Element and range withheld (spec AC-008).
        formatter
            .debug_struct("RememberedField")
            .field("dictation_id", &self.dictation_id)
            .field("destination_generation", &self.destination_generation)
            .finish_non_exhaustive()
    }
}
