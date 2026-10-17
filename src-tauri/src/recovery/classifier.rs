//! Bounded classification of the destination's focused element (spec AC-002, plan "Bounded AX
//! classification and threading"). Only the destination's application element is read, never
//! the system-wide one. One monotonic deadline of `CLASSIFY_BUDGET` covers every query; each
//! query's messaging timeout is the remaining budget, set per element and never 0. Any doubt
//! (error, timeout, spent budget, focus in another process, unknown or secure role) is
//! undetermined, which inserts as usual. The native adapter lives in `platform::macos_ax`.
use crate::destination::workspace::Clock;
use std::time::{Duration, Instant};

/// Whole AX budget of one classification.
pub const CLASSIFY_BUDGET: Duration = Duration::from_millis(50);

/// Roles of a field that takes text when one of its edit attributes is settable.
const TEXT_ROLES: [&str; 4] = ["AXTextField", "AXTextArea", "AXComboBox", "AXSearchField"];
/// Roles that never take typed text in the destination's own process: controls and lists only
/// (initial list, calibrated by the native matrix of Step 1). Containers that Chromium and
/// Electron report while a web or custom field has the focus (`AXWebArea`, `AXWindow`,
/// `AXGroup`, `AXApplication`, `AXScrollArea`) are deliberately absent: they stay undetermined.
const NON_TEXT_ROLES: [&str; 15] = [
    "AXList",
    "AXOutline",
    "AXTable",
    "AXBrowser",
    "AXRow",
    "AXCell",
    "AXImage",
    "AXButton",
    "AXCheckBox",
    "AXRadioButton",
    "AXPopUpButton",
    "AXMenuButton",
    "AXSlider",
    "AXTabGroup",
    "AXLink",
];
/// Password fields, as a role or a subrole.
const SECURE_TEXT_FIELD: &str = "AXSecureTextField";

/// Failure of one accessibility query (never carries a value).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxQueryError {
    Timeout,
    NoValue,
    Unsupported,
    Failed,
}

/// Attributes whose settability shows an edit affordance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditAttribute {
    Value,
    SelectedText,
    SelectedTextRange,
}
const EDIT_ATTRIBUTES: [EditAttribute; 3] = [
    EditAttribute::Value,
    EditAttribute::SelectedText,
    EditAttribute::SelectedTextRange,
];

/// The destination's focused element, read with a timeout given per query.
pub trait FocusedElement {
    /// Process owning the element (a local read, no messaging).
    fn pid(&mut self) -> Result<u32, AxQueryError>;
    /// `AXRole`, bounded.
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError>;
    /// `AXSubrole`, `None` when the element has none.
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError>;
    /// Whether `attribute` is settable.
    fn is_settable(
        &mut self,
        attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError>;
}

/// Accessibility of this process.
pub trait FocusedElementPort: Send {
    /// Accessibility permission, checked without prompting and before any query.
    fn is_trusted(&mut self) -> bool;
    /// `AXFocusedUIElement` of application `pid`, with `timeout` set on its application element.
    fn focused_element(
        &mut self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FocusedElement>, AxQueryError>;
}

/// Why a classification stayed undetermined (diagnostics only).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Undetermined {
    NoFocusedElement,
    ForeignProcess,
    AxError,
    Timeout,
    BudgetSpent,
    UnknownRole,
    SecureField,
    TextWithoutAffordance,
    UnexpectedAffordance,
}

/// Result of one classification (plan Key Entities "Focus classification").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusClassification {
    Editable,
    NotEditable,
    Undetermined(Undetermined),
    PermissionMissing,
}

/// The single deadline of one classification (also 011's capture, with its own floor).
pub(crate) struct Deadline<'a> {
    clock: &'a dyn Clock,
    end: Instant,
    /// Smallest timeout still worth sending: below it the budget counts as spent (0 for 010).
    floor: Duration,
}
impl<'a> Deadline<'a> {
    /// A deadline `budget` from now on `clock`, spent once less than `floor` remains.
    pub(crate) fn new(clock: &'a dyn Clock, budget: Duration, floor: Duration) -> Self {
        Self::ending(clock, clock.now() + budget, floor)
    }
    /// A deadline at `end` on `clock` (a budget that started earlier).
    pub(crate) fn ending(clock: &'a dyn Clock, end: Instant, floor: Duration) -> Self {
        Self { clock, end, floor }
    }
    /// Remaining budget, `None` once spent: a timeout of 0 would mean the default timeout.
    pub(crate) fn remaining(&self) -> Result<Duration, Undetermined> {
        let left = self.end.saturating_duration_since(self.clock.now());
        if left.is_zero() || left < self.floor {
            Err(Undetermined::BudgetSpent)
        } else {
            Ok(left)
        }
    }
}

fn undetermined(error: AxQueryError) -> Undetermined {
    match error {
        AxQueryError::Timeout => Undetermined::Timeout,
        AxQueryError::NoValue | AxQueryError::Unsupported | AxQueryError::Failed => {
            Undetermined::AxError
        }
    }
}

// @spec FR-001: Classify the focused element, bounded — .specs/features/010-recovery-bubble/spec.md#fr-001
/// Classify the focused element of the destination `pid` within `CLASSIFY_BUDGET`.
pub fn classify(
    port: &mut dyn FocusedElementPort,
    pid: u32,
    clock: &dyn Clock,
) -> FocusClassification {
    if !port.is_trusted() {
        return FocusClassification::PermissionMissing;
    }
    let deadline = Deadline::new(clock, CLASSIFY_BUDGET, Duration::ZERO);
    classify_within(port, pid, &deadline).unwrap_or_else(FocusClassification::Undetermined)
}

fn classify_within(
    port: &mut dyn FocusedElementPort,
    pid: u32,
    deadline: &Deadline<'_>,
) -> Result<FocusClassification, Undetermined> {
    let mut element =
        port.focused_element(pid, deadline.remaining()?)
            .map_err(|error| match error {
                AxQueryError::NoValue => Undetermined::NoFocusedElement,
                other => undetermined(other),
            })?;
    classify_focused(element.as_mut(), pid, deadline)
}

// @spec AC-001: 010's editable rule decides what 011 remembers — .specs/features/011-dictation-field-return/spec.md#ac-001
/// 010's editable rule over an element already obtained: shared with 011's capture, which keeps
/// a reference to the element it classifies (spec 011 AC-001).
pub(crate) fn classify_focused(
    element: &mut dyn FocusedElement,
    pid: u32,
    deadline: &Deadline<'_>,
) -> Result<FocusClassification, Undetermined> {
    // Business rule (spec AC-002): focus owned by another process is undetermined, before any
    // role check, whatever its role.
    if element.pid().map_err(undetermined)? != pid {
        return Err(Undetermined::ForeignProcess);
    }
    let role = element.role(deadline.remaining()?).map_err(undetermined)?;
    if role == SECURE_TEXT_FIELD {
        return Err(Undetermined::SecureField);
    }
    if TEXT_ROLES.contains(&role.as_str()) {
        return text_field(element, deadline);
    }
    if NON_TEXT_ROLES.contains(&role.as_str()) {
        return non_text(element, deadline);
    }
    Err(Undetermined::UnknownRole)
}

/// A text role is editable when one edit attribute is settable; a password field or a
/// read-only text role stays undetermined.
fn text_field(
    element: &mut dyn FocusedElement,
    deadline: &Deadline<'_>,
) -> Result<FocusClassification, Undetermined> {
    let subrole = element
        .subrole(deadline.remaining()?)
        .map_err(undetermined)?;
    if subrole.as_deref() == Some(SECURE_TEXT_FIELD) {
        return Err(Undetermined::SecureField);
    }
    if any_settable(element, deadline)? {
        Ok(FocusClassification::Editable)
    } else {
        Err(Undetermined::TextWithoutAffordance)
    }
}

// @spec AC-002: Not editable only on positive proof — .specs/features/010-recovery-bubble/spec.md#ac-002
/// Business rule (delivery review 2026-10-15): "not editable" needs positive proof, a known
/// non-text role whose every edit attribute was read and found not settable. An attribute the
/// element does not support, has no value for, or that could not be read is a doubt.
fn non_text(
    element: &mut dyn FocusedElement,
    deadline: &Deadline<'_>,
) -> Result<FocusClassification, Undetermined> {
    if any_settable(element, deadline)? {
        return Err(Undetermined::UnexpectedAffordance);
    }
    Ok(FocusClassification::NotEditable)
}

/// True when one edit attribute is settable; any attribute that cannot be read (unsupported,
/// no value, timeout or another error) makes the classification undetermined.
fn any_settable(
    element: &mut dyn FocusedElement,
    deadline: &Deadline<'_>,
) -> Result<bool, Undetermined> {
    for attribute in EDIT_ATTRIBUTES {
        match element.is_settable(attribute, deadline.remaining()?) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(error) => return Err(undetermined(error)),
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "classifier_tests.rs"]
mod tests;
