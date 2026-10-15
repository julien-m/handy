//! Start read of 011 (spec AC-001): on a short-lived worker spawned right after 006 commits a
//! dictation's session, read the focused element of the destination's running instance, keep
//! it only when 010's rule finds it editable in that process, then read its selection range.
//! One deadline of `FIELD_CAPTURE_BUDGET` covers every query; each query's timeout is the
//! remaining budget, never below `MIN_AX_TIMEOUT`. The recording never waits for this read.
use super::field::{
    FieldAccess, FieldNode, FieldTarget, RememberedField, SelectionRange, FIELD_CAPTURE_BUDGET,
    MIN_AX_TIMEOUT,
};
use super::manager::DestinationManager;
use super::workspace::{resolve_instance, Clock, Resolution};
use crate::recovery::classifier::{
    classify_focused, AxQueryError, Deadline, FocusClassification, Undetermined,
};
use std::sync::Arc;
use std::time::Instant;

/// Why nothing was remembered at the start (logged as a kind, never with content).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureMiss {
    PermissionMissing,
    /// Role or subrole `AXSecureTextField`.
    PasswordField,
    /// The focused element belongs to another process.
    OtherProcess,
    NoFocusedElement,
    NotEditable,
    Undetermined,
    AxError,
    /// The deadline expired before the classification.
    BudgetSpent,
}

/// The element remembered at the start, with its range when it could be read.
pub struct CapturedField {
    pub element: Box<dyn FieldNode>,
    pub range: Option<SelectionRange>,
}

/// Result of one start read.
pub enum FieldCapture {
    Remembered(CapturedField),
    NotRemembered(CaptureMiss),
}

// @spec FR-001: Remember the focused input and its range — .specs/features/011-dictation-field-return/spec.md#fr-001
// @spec AC-001: Editable input of the same process only — .specs/features/011-dictation-field-return/spec.md#ac-001
/// Read the focused input of application `pid` within `FIELD_CAPTURE_BUDGET`; see the module
/// documentation. A missing permission makes no query at all.
pub fn capture(access: &dyn FieldAccess, pid: u32, clock: &dyn Clock) -> FieldCapture {
    capture_since(access, pid, clock, clock.now())
}

/// [`capture`] for a read requested at `requested_at`: the budget runs from the request, so a
/// worker that starts late has less time, never more.
pub fn capture_since(
    access: &dyn FieldAccess,
    pid: u32,
    clock: &dyn Clock,
    requested_at: Instant,
) -> FieldCapture {
    if !access.is_trusted() {
        return FieldCapture::NotRemembered(CaptureMiss::PermissionMissing);
    }
    let end = requested_at + FIELD_CAPTURE_BUDGET;
    let deadline = Deadline::ending(clock, end, MIN_AX_TIMEOUT);
    match editable_element(access, pid, &deadline) {
        Ok(mut element) => {
            let range = deadline
                .remaining()
                .ok()
                .and_then(|timeout| element.read_range(timeout).ok());
            FieldCapture::Remembered(CapturedField { element, range })
        }
        Err(miss) => FieldCapture::NotRemembered(miss),
    }
}

/// The focused element of `pid` when 010's rule finds it editable in that process.
fn editable_element(
    access: &dyn FieldAccess,
    pid: u32,
    deadline: &Deadline<'_>,
) -> Result<Box<dyn FieldNode>, CaptureMiss> {
    let timeout = deadline.remaining().map_err(miss_of)?;
    let mut element = access
        .focused_element(pid, timeout)
        .map_err(|error| match error {
            AxQueryError::NoValue => CaptureMiss::NoFocusedElement,
            AxQueryError::Timeout => CaptureMiss::BudgetSpent,
            AxQueryError::Unsupported | AxQueryError::Failed => CaptureMiss::AxError,
        })?;
    match classify_focused(element.as_mut(), pid, deadline) {
        Ok(FocusClassification::Editable) => Ok(element),
        Ok(FocusClassification::NotEditable) => Err(CaptureMiss::NotEditable),
        Ok(FocusClassification::PermissionMissing) => Err(CaptureMiss::PermissionMissing),
        Ok(FocusClassification::Undetermined(reason)) | Err(reason) => Err(miss_of(reason)),
    }
}

/// Business rule (spec AC-001): only an editable element is remembered; every doubt is a miss.
fn miss_of(reason: Undetermined) -> CaptureMiss {
    match reason {
        Undetermined::SecureField => CaptureMiss::PasswordField,
        Undetermined::ForeignProcess => CaptureMiss::OtherProcess,
        Undetermined::NoFocusedElement => CaptureMiss::NoFocusedElement,
        Undetermined::BudgetSpent | Undetermined::Timeout => CaptureMiss::BudgetSpent,
        Undetermined::AxError => CaptureMiss::AxError,
        Undetermined::UnknownRole
        | Undetermined::TextWithoutAffordance
        | Undetermined::UnexpectedAffordance => CaptureMiss::Undetermined,
    }
}

// @spec FR-003: Read the pinned instance's input at the start — .specs/features/011-dictation-field-return/spec.md#fr-003
// @spec AC-006: No read without a destination — .specs/features/011-dictation-field-return/spec.md#ac-006
/// Spawn the start read of dictation `target` (006's captured application or 009's pin), off
/// the recording start path. Returns the worker so tests can join it; `None` when nothing is
/// read: rollout gate closed, no application destination (return off without pin, Handy in
/// front, unreadable foreground, Linux).
pub fn spawn_start_read(
    manager: &Arc<DestinationManager>,
    target: Option<FieldTarget>,
) -> Option<std::thread::JoinHandle<()>> {
    if !super::rollout::Rollout::CURRENT.opens_sessions() {
        return None;
    }
    let target = target?;
    let manager = Arc::clone(manager);
    let requested_at = manager.clock.now();
    Some(std::thread::spawn(move || {
        read_and_attach(&manager, target, requested_at);
    }))
}

/// The worker's body: resolve the running instance of `target`, read its focused input, then
/// attach it while the session still wants it. Blocking (bounded by the capture budget); call it
/// off the start path, as `spawn_start_read` does.
pub fn read_and_attach(manager: &DestinationManager, target: FieldTarget, requested_at: Instant) {
    let instances = manager.workspace.instances(&target.identity);
    let Resolution::Running(app) = resolve_instance(&target.identity, target.route_pid, &instances)
    else {
        // A pinned application not running (009's failure applies) or an ambiguous instance.
        log::debug!("field_return not_remembered(no_running_instance)");
        return;
    };
    let access = manager.field_access();
    let clock = manager.clock.as_ref();
    let captured = capture_since(access.as_ref(), app.pid, clock, requested_at);
    let elapsed = clock.now().saturating_duration_since(requested_at);
    let captured = match captured {
        FieldCapture::Remembered(field) => field,
        FieldCapture::NotRemembered(miss) => {
            log::debug!("field_return not_remembered({miss:?}) in {elapsed:?}");
            return;
        }
    };
    let ranged = captured.range.is_some();
    let field = remembered(target, app.pid, captured, requested_at);
    let attached = manager.attach_field(field);
    log::debug!(
        "field_return remembered(range_read: {ranged}, attached: {attached}) in {elapsed:?}"
    );
}

/// The remembered input of `target`'s instance `pid`, read at `requested_at`.
fn remembered(
    target: FieldTarget,
    pid: u32,
    captured: CapturedField,
    requested_at: Instant,
) -> RememberedField {
    RememberedField {
        dictation_id: target.dictation_id,
        destination_generation: target.destination_generation,
        identity: target.identity,
        pid,
        route_pid: target.route_pid,
        element: captured.element,
        range: captured.range,
        captured_at: requested_at,
    }
}
