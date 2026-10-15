//! macOS accessibility adapter of the field classifier (spec AC-002). It reads only the
//! destination's application element (`AXUIElementCreateApplication(pid)`) and the elements
//! obtained from it, never the system-wide element, and sets the messaging timeout on each
//! element right before messaging it, to the remaining budget handed in by the classifier
//! (never 0). No value read here is ever logged; CoreFoundation objects are released by RAII.
use crate::recovery::classifier::{
    AxQueryError, EditAttribute, FocusedElement, FocusedElementPort,
};
use crate::smart_paste::macos_cf::{string_units, type_id, CfOwned, CfStringError, CfTypeRef};
use std::ptr;
use std::time::Duration;

// AXError codes from HIServices/AXError.h.
pub(crate) const AX_SUCCESS: i32 = 0;
const AX_ERROR_CANNOT_COMPLETE: i32 = -25204;
const AX_ERROR_ATTRIBUTE_UNSUPPORTED: i32 = -25205;
const AX_ERROR_NOT_IMPLEMENTED: i32 = -25208;
const AX_ERROR_NO_VALUE: i32 = -25212;
/// Longest role or subrole read (roles are short identifiers).
const MAX_ROLE_UNITS: usize = 64;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> CfTypeRef;
    fn AXUIElementSetMessagingTimeout(element: CfTypeRef, timeout_seconds: f32) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: CfTypeRef,
        attribute: CfTypeRef,
        value: *mut CfTypeRef,
    ) -> i32;
    fn AXUIElementIsAttributeSettable(
        element: CfTypeRef,
        attribute: CfTypeRef,
        settable: *mut u8,
    ) -> i32;
    fn AXUIElementGetPid(element: CfTypeRef, pid: *mut i32) -> i32;
    fn AXUIElementGetTypeID() -> usize;
}

/// The `AXError` `code` as a query result (never carrying a value).
pub(crate) fn ax_result(code: i32) -> Result<(), AxQueryError> {
    match code {
        AX_SUCCESS => Ok(()),
        AX_ERROR_CANNOT_COMPLETE => Err(AxQueryError::Timeout),
        AX_ERROR_NO_VALUE => Err(AxQueryError::NoValue),
        AX_ERROR_ATTRIBUTE_UNSUPPORTED | AX_ERROR_NOT_IMPLEMENTED => Err(AxQueryError::Unsupported),
        _ => Err(AxQueryError::Failed),
    }
}

/// Business rule (plan): the timeout is per element and never 0, which would mean the default.
pub(crate) fn set_timeout(element: &CfOwned, timeout: Duration) -> Result<(), AxQueryError> {
    if timeout.is_zero() {
        return Err(AxQueryError::Timeout);
    }
    // SAFETY: `element` is a live AXUIElement owned by the caller (never the system-wide one).
    ax_result(unsafe { AXUIElementSetMessagingTimeout(element.as_ptr(), timeout.as_secs_f32()) })
}

/// The attribute or action name `name` as a `CFString`.
pub(crate) fn attribute(name: &str) -> Result<CfOwned, AxQueryError> {
    CfOwned::from_text(name).ok_or(AxQueryError::Failed)
}

/// `name` of `element` (Copy Rule), with `timeout` set on `element` first.
pub(crate) fn copy_attribute(
    element: &CfOwned,
    name: &str,
    timeout: Duration,
) -> Result<CfOwned, AxQueryError> {
    set_timeout(element, timeout)?;
    let attribute = attribute(name)?;
    let mut value: CfTypeRef = ptr::null();
    // SAFETY: element and attribute are live CF objects; `value` receives a +1 reference
    // (Copy Rule) owned by the returned CfOwned.
    ax_result(unsafe {
        AXUIElementCopyAttributeValue(element.as_ptr(), attribute.as_ptr(), &mut value)
    })?;
    CfOwned::from_create_rule(value).ok_or(AxQueryError::NoValue)
}

/// A short string attribute (a role or subrole), at most `MAX_ROLE_UNITS` long.
pub(crate) fn short_string(
    element: &CfOwned,
    name: &str,
    timeout: Duration,
) -> Result<String, AxQueryError> {
    let value = copy_attribute(element, name, timeout)?;
    string_units(value.as_ptr(), MAX_ROLE_UNITS)
        .map(|units| String::from_utf16_lossy(&units))
        .map_err(|error| match error {
            CfStringError::NotAString | CfStringError::TooLarge(_) => AxQueryError::Failed,
        })
}

/// `value` when it is an `AXUIElement`, checked before any accessibility call is made on it.
pub(crate) fn ax_element(value: CfOwned) -> Result<CfOwned, AxQueryError> {
    // SAFETY: AXUIElementGetTypeID takes no arguments; `value` is a live CF object.
    if type_id(value.as_ptr()) != unsafe { AXUIElementGetTypeID() } {
        return Err(AxQueryError::Failed);
    }
    Ok(value)
}

/// Production [`FocusedElementPort`].
#[derive(Debug, Default)]
pub struct MacFocusedElementPort;

impl FocusedElementPort for MacFocusedElementPort {
    fn is_trusted(&mut self) -> bool {
        // SAFETY: AXIsProcessTrusted takes no arguments and never prompts.
        unsafe { AXIsProcessTrusted() != 0 }
    }
    fn focused_element(
        &mut self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FocusedElement>, AxQueryError> {
        let pid = i32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
            .ok_or(AxQueryError::Failed)?;
        let element = copy_attribute(&application, "AXFocusedUIElement", timeout)?;
        // SAFETY: AXUIElementGetTypeID takes no arguments; `element` is a live CF object.
        Ok(Box::new(MacFocusedElement {
            element: ax_element(element)?,
        }))
    }
}

/// The destination's focused element (also the base of 011's remembered field element).
pub(crate) struct MacFocusedElement {
    pub(crate) element: CfOwned,
}

impl FocusedElement for MacFocusedElement {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        let mut pid = 0;
        // SAFETY: element is live; `pid` is a valid out pointer. A local read, no messaging.
        ax_result(unsafe { AXUIElementGetPid(self.element.as_ptr(), &mut pid) })?;
        u32::try_from(pid).map_err(|_| AxQueryError::Failed)
    }
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError> {
        short_string(&self.element, "AXRole", timeout)
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        match short_string(&self.element, "AXSubrole", timeout) {
            Ok(subrole) => Ok(Some(subrole)),
            Err(AxQueryError::NoValue | AxQueryError::Unsupported) => Ok(None),
            Err(error) => Err(error),
        }
    }
    fn is_settable(
        &mut self,
        attribute_kind: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        set_timeout(&self.element, timeout)?;
        let name = match attribute_kind {
            EditAttribute::Value => "AXValue",
            EditAttribute::SelectedText => "AXSelectedText",
            EditAttribute::SelectedTextRange => "AXSelectedTextRange",
        };
        let attribute = attribute(name)?;
        let mut settable = 0_u8;
        // SAFETY: element and attribute are live CF objects; `settable` is a valid out pointer.
        ax_result(unsafe {
            AXUIElementIsAttributeSettable(self.element.as_ptr(), attribute.as_ptr(), &mut settable)
        })?;
        Ok(settable != 0)
    }
}
