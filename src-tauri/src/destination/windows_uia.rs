//! UI Automation client of 011's Windows field adapter: Handy's own `IUIAutomation` on a
//! dedicated COM MTA thread, the elements and text ranges it keeps by opaque id, and the bounded
//! reads of the focused element (process, control type, password flag, patterns, runtime id)
//! and of its first selection range. Never `GetText`, never a value. Lives on the thread only.
use super::bounded_worker::{BoundedWorker, CallError};
use super::uia_mapping::UiaFacts;
use crate::recovery::classifier::AxQueryError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use windows::core::Interface;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, SAFEARRAY,
};
use windows::Win32::System::Ole::{
    SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement, IUIAutomationTextPattern,
    IUIAutomationTextRange, IUIAutomationValuePattern, TextPatternRangeEndpoint_End,
    TextPatternRangeEndpoint_Start, UIA_TextPatternId, UIA_ValuePatternId,
};

/// Longest runtime id read (runtime ids are a handful of integers).
const MAX_RUNTIME_ID: i32 = 64;

/// The client and the objects it obtained, alive on the UI Automation thread only.
pub(super) struct Uia {
    automation: IUIAutomation,
    pub(super) elements: HashMap<u64, IUIAutomationElement>,
    pub(super) ranges: HashMap<u64, IUIAutomationTextRange>,
    next: u64,
}
impl Uia {
    /// Keep `element` on the thread; returns its opaque id.
    fn keep_element(&mut self, element: IUIAutomationElement) -> u64 {
        self.next += 1;
        self.elements.insert(self.next, element);
        self.next
    }
    /// Keep `range` on the thread; returns its opaque id.
    pub(super) fn keep_range(&mut self, range: IUIAutomationTextRange) -> u64 {
        self.next += 1;
        self.ranges.insert(self.next, range);
        self.next
    }
    /// Business rule (plan): the client's own timeouts follow the call's budget, never 0.
    pub(super) fn bound(&self, timeout: Duration) {
        let millis = u32::try_from(timeout.as_millis())
            .unwrap_or(u32::MAX)
            .max(1);
        if let Ok(client) = self.automation.cast::<IUIAutomation2>() {
            // SAFETY: plain setters on Handy's own client object; failures keep the defaults.
            unsafe {
                let _ = client.SetConnectionTimeout(millis);
                let _ = client.SetTransactionTimeout(millis);
            }
        }
    }
}

/// Create the client on the UI Automation thread (`None`: every call is unsupported).
pub(super) fn create_client() -> Option<Uia> {
    // SAFETY: initializes COM for this dedicated thread only, in the multithreaded apartment.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .ok()?;
    // SAFETY: creates Handy's in-process UI Automation client.
    let automation: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }.ok()?;
    Some(Uia {
        automation,
        elements: HashMap::new(),
        ranges: HashMap::new(),
        next: 0,
    })
}

pub(super) type Worker = Arc<BoundedWorker<Uia>>;

/// A failed call: refused while busy changed nothing; expired may still apply.
pub(super) fn call_error(error: CallError) -> AxQueryError {
    match error {
        CallError::Busy => AxQueryError::Failed,
        CallError::Expired => AxQueryError::Timeout,
        CallError::Stopped => AxQueryError::Unsupported,
    }
}

/// Undo of an abandoned `focused` call: the element it kept is released on the thread.
pub(super) fn forget_seen(uia: &mut Uia, seen: Result<Seen, AxQueryError>) {
    if let Ok(seen) = seen {
        uia.elements.remove(&seen.id);
    }
}

/// Undo of an abandoned range read: the range it kept is released on the thread.
pub(super) fn forget_range(uia: &mut Uia, kept: Result<u64, AxQueryError>) {
    if let Ok(id) = kept {
        uia.ranges.remove(&id);
    }
}

/// What the start read and the verification learn of the focused element, in one call.
pub(super) struct Seen {
    pub(super) id: u64,
    pub(super) pid: u32,
    pub(super) facts: UiaFacts,
    pub(super) runtime_id: Vec<i32>,
}

/// The focused element, kept on the thread, with its facts and identity.
pub(super) fn focused(uia: &mut Uia, timeout: Duration) -> Result<Seen, AxQueryError> {
    uia.bound(timeout);
    // SAFETY: COM calls on Handy's client, on its own thread.
    unsafe {
        let element = uia
            .automation
            .GetFocusedElement()
            .map_err(|_| AxQueryError::NoValue)?;
        let pid = element
            .CurrentProcessId()
            .map_err(|_| AxQueryError::Failed)?;
        let facts = facts_of(&element)?;
        let runtime_id = runtime_id(&element);
        let pid = u32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
        Ok(Seen {
            id: uia.keep_element(element),
            pid,
            facts,
            runtime_id,
        })
    }
}

/// Control type, password flag and patterns (never a value or text).
pub(super) unsafe fn facts_of(element: &IUIAutomationElement) -> Result<UiaFacts, AxQueryError> {
    let control_type = element
        .CurrentControlType()
        .map_err(|_| AxQueryError::Failed)?;
    let is_password = element
        .CurrentIsPassword()
        .map_err(|_| AxQueryError::Failed)?;
    let value = element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId);
    let text = element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId);
    let read_only = match &value {
        Ok(pattern) => Some(
            pattern
                .CurrentIsReadOnly()
                .map(|flag| flag.as_bool())
                .unwrap_or(true),
        ),
        Err(_) => None,
    };
    Ok(UiaFacts {
        control_type: control_type.0,
        has_value_pattern: value.is_ok(),
        has_text_pattern: text.is_ok(),
        is_password: is_password.as_bool(),
        read_only,
    })
}

/// The element's runtime id (empty when unreadable, which never matches another element).
pub(super) unsafe fn runtime_id(element: &IUIAutomationElement) -> Vec<i32> {
    let Ok(array) = element.GetRuntimeId() else {
        return Vec::new();
    };
    if array.is_null() {
        return Vec::new();
    }
    let ids = read_ints(array).unwrap_or_default();
    let _ = SafeArrayDestroy(array);
    ids
}

/// The integers of a one-dimensional SAFEARRAY, at most `MAX_RUNTIME_ID`.
pub(super) unsafe fn read_ints(array: *const SAFEARRAY) -> windows::core::Result<Vec<i32>> {
    let low = SafeArrayGetLBound(array, 1)?;
    let high = SafeArrayGetUBound(array, 1)?.min(low.saturating_add(MAX_RUNTIME_ID - 1));
    let mut ids = Vec::new();
    for index in low..=high {
        let mut value = 0_i32;
        SafeArrayGetElement(array, &index, (&raw mut value).cast())?;
        ids.push(value);
    }
    Ok(ids)
}

/// The first range of the element's current selection.
pub(super) unsafe fn first_selection(
    element: &IUIAutomationElement,
) -> Result<IUIAutomationTextRange, AxQueryError> {
    let pattern = element
        .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
        .map_err(|_| AxQueryError::Unsupported)?;
    let selection = pattern.GetSelection().map_err(|_| AxQueryError::Failed)?;
    if selection.Length().map_err(|_| AxQueryError::Failed)? < 1 {
        return Err(AxQueryError::NoValue);
    }
    selection.GetElement(0).map_err(|_| AxQueryError::Failed)
}

/// Whether the element's current first selection has exactly the endpoints of `kept`.
pub(super) unsafe fn selection_is(
    element: &IUIAutomationElement,
    kept: &IUIAutomationTextRange,
) -> Result<bool, AxQueryError> {
    let current = first_selection(element)?;
    let same = |endpoint| {
        current
            .CompareEndpoints(endpoint, kept, endpoint)
            .map(|order| order == 0)
            .map_err(|_| AxQueryError::Failed)
    };
    Ok(same(TextPatternRangeEndpoint_Start)? && same(TextPatternRangeEndpoint_End)?)
}
