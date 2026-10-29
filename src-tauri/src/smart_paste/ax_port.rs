//! Accessibility port used by verified direct insertion (AC-017).
//!
//! The production adapter talks to the macOS AX API; tests use scripted
//! fields. Every call takes an explicit per-call timeout because a blocked
//! AX request cannot be cancelled from outside.

use std::fmt;
use std::time::Duration;

/// Errors reported by accessibility calls. They never carry field content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxError {
    /// The process is not trusted for accessibility.
    NotTrusted,
    /// The request did not complete within its messaging timeout.
    Timeout,
    /// The element has no value for the attribute (for example no focus).
    NoValue,
    /// The attribute is unsupported or not settable by this element.
    Unsupported,
    /// The value exceeds the conversion guard; it was not copied.
    TooLarge {
        /// Size of the value in UTF-16 code units.
        utf16_units: usize,
    },
    /// The value had an unexpected type.
    UnexpectedType,
    /// Any other AX error code.
    Failed(i32),
}

impl fmt::Display for AxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AxError {}

/// Selected range of a text field, in UTF-16 code units (default: caret at
/// the start).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Utf16Range {
    /// Start of the selection (caret position when empty).
    pub location: usize,
    /// Length of the selection.
    pub length: usize,
}

/// One focused text element of the destination application.
///
/// Implementations hold the native element; values are returned as UTF-16
/// code units so range arithmetic matches the AX API exactly.
pub trait AxField: Send {
    /// Process identifier owning the element.
    fn pid(&mut self) -> Result<i32, AxError>;
    /// Whether the element is a password field (role or subrole
    /// `AXSecureTextField`).
    fn is_secure(&mut self, timeout: Duration) -> Result<bool, AxError>;
    /// Reads the element value; refuses values longer than `max_units`
    /// without copying them.
    fn value_utf16(&mut self, timeout: Duration, max_units: usize) -> Result<Vec<u16>, AxError>;
    /// Reads the selected range.
    fn selected_range(&mut self, timeout: Duration) -> Result<Utf16Range, AxError>;
    /// Sets `AXSelectedText`, replacing the selection or inserting at the caret.
    fn set_selected_text(&mut self, text: &[u16], timeout: Duration) -> Result<(), AxError>;
    /// Whether the element or one of its ancestors has the role
    /// [`WEB_AREA_ROLE`], searched within `limits` (see [`search_web_area`]).
    fn is_in_web_area(&mut self, limits: AncestryLimits) -> Result<bool, AxError>;
}

/// Role of the root of a rendered web document (Chrome, Safari, Electron
/// apps): a text field below it is web text whose `AXValue` follows edits.
pub const WEB_AREA_ROLE: &str = "AXWebArea";

/// Bounds of an ancestry search: each AX request has its own messaging
/// timeout, the walk stops after `max_depth` parents and once `budget` is
/// spent, so a deep or slow tree never delays the paste for long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AncestryLimits {
    /// Messaging timeout of each AX request.
    pub per_call: Duration,
    /// Parents examined at most above the element itself.
    pub max_depth: usize,
    /// Total time the walk may take before it gives up.
    pub budget: Duration,
}

/// Walks from the element (depth 0) to its ancestors. `visit(depth)` reads
/// the element at `depth`: `Some(true)` when its role is the web area,
/// `Some(false)` for another role, `None` once there is no parent left; an
/// error (timeout, exhausted budget) aborts the walk. Reaching `max_depth`
/// without a web area answers `false`: the element has not shown it is web
/// text. Only roles are read, never a value.
pub fn search_web_area(
    max_depth: usize,
    mut visit: impl FnMut(usize) -> Result<Option<bool>, AxError>,
) -> Result<bool, AxError> {
    for depth in 0..=max_depth {
        match visit(depth)? {
            Some(true) => return Ok(true),
            Some(false) => {}
            None => return Ok(false),
        }
    }
    Ok(false)
}

/// Access to the accessibility API of the current process.
pub trait AxTextPort: Send {
    /// Whether Handy has accessibility permission.
    fn is_trusted(&mut self) -> bool;
    /// Returns the focused element of the application `pid`.
    fn focused_field(&mut self, pid: i32, timeout: Duration) -> Result<Box<dyn AxField>, AxError>;
}
