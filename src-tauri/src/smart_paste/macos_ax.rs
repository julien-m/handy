//! macOS accessibility adapter for verified direct insertion.
//!
//! Thin FFI over ApplicationServices: every request first sets a messaging
//! timeout on the element it messages, values are exchanged as UTF-16 so the
//! direct-insertion arithmetic matches AX ranges, and every CoreFoundation
//! object is released by RAII. No value read here is ever logged.

use std::ffi::c_void;
use std::ptr;
use std::time::Duration;

use super::ax_port::{AxError, AxField, AxTextPort, Utf16Range};
use super::macos_cf::{string_units, type_id, CfOwned, CfRange, CfStringError, CfTypeRef};

// AXError codes from HIServices/AXError.h.
const AX_SUCCESS: i32 = 0;
const AX_ERROR_CANNOT_COMPLETE: i32 = -25204;
const AX_ERROR_ATTRIBUTE_UNSUPPORTED: i32 = -25205;
const AX_ERROR_NOT_IMPLEMENTED: i32 = -25208;
const AX_ERROR_API_DISABLED: i32 = -25211;
const AX_ERROR_NO_VALUE: i32 = -25212;
// kAXValueTypeCFRange from HIServices/AXValue.h.
const AX_VALUE_CF_RANGE_TYPE: u32 = 4;
const SECURE_TEXT_FIELD: &str = "AXSecureTextField";

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
    fn AXUIElementSetAttributeValue(
        element: CfTypeRef,
        attribute: CfTypeRef,
        value: CfTypeRef,
    ) -> i32;
    fn AXUIElementGetPid(element: CfTypeRef, pid: *mut i32) -> i32;
    fn AXValueGetTypeID() -> usize;
    fn AXValueGetValue(value: CfTypeRef, value_type: u32, value_ptr: *mut c_void) -> u8;
}

fn attribute_name(name: &str) -> Result<CfOwned, AxError> {
    CfOwned::from_text(name).ok_or(AxError::Failed(AX_ERROR_CANNOT_COMPLETE))
}

fn ax_result(code: i32) -> Result<(), AxError> {
    match code {
        AX_SUCCESS => Ok(()),
        AX_ERROR_CANNOT_COMPLETE => Err(AxError::Timeout),
        AX_ERROR_NO_VALUE => Err(AxError::NoValue),
        AX_ERROR_ATTRIBUTE_UNSUPPORTED | AX_ERROR_NOT_IMPLEMENTED => Err(AxError::Unsupported),
        AX_ERROR_API_DISABLED => Err(AxError::NotTrusted),
        other => Err(AxError::Failed(other)),
    }
}

fn set_messaging_timeout(element: &CfOwned, timeout: Duration) -> Result<(), AxError> {
    // SAFETY: `element` is a live AXUIElement owned by the caller.
    ax_result(unsafe { AXUIElementSetMessagingTimeout(element.as_ptr(), timeout.as_secs_f32()) })
}

fn copy_attribute(element: &CfOwned, name: &str, timeout: Duration) -> Result<CfOwned, AxError> {
    set_messaging_timeout(element, timeout)?;
    let attribute = attribute_name(name)?;
    let mut value: CfTypeRef = ptr::null();
    // SAFETY: element and attribute are live CF objects; `value` receives a
    // +1 reference (Copy Rule) owned by the returned CfOwned.
    ax_result(unsafe {
        AXUIElementCopyAttributeValue(element.as_ptr(), attribute.as_ptr(), &mut value)
    })?;
    CfOwned::from_create_rule(value).ok_or(AxError::NoValue)
}

fn string_utf16(value: &CfOwned, max_units: usize) -> Result<Vec<u16>, AxError> {
    string_units(value.as_ptr(), max_units).map_err(|error| match error {
        CfStringError::TooLarge(utf16_units) => AxError::TooLarge { utf16_units },
        CfStringError::NotAString => AxError::UnexpectedType,
    })
}

fn range_value(value: &CfOwned) -> Result<Utf16Range, AxError> {
    // SAFETY: `value` is a live CF object; its type is checked first.
    if type_id(value.as_ptr()) != unsafe { AXValueGetTypeID() } {
        return Err(AxError::UnexpectedType);
    }
    let mut range = CfRange::default();
    // SAFETY: `range` is a CFRange, the type requested from the AXValue.
    let is_range = unsafe {
        AXValueGetValue(
            value.as_ptr(),
            AX_VALUE_CF_RANGE_TYPE,
            (&mut range as *mut CfRange).cast(),
        )
    };
    if is_range == 0 {
        return Err(AxError::UnexpectedType);
    }
    Ok(Utf16Range {
        location: usize::try_from(range.location).map_err(|_| AxError::UnexpectedType)?,
        length: usize::try_from(range.length).map_err(|_| AxError::UnexpectedType)?,
    })
}

/// Production [`AxTextPort`] backed by the macOS accessibility API.
#[derive(Debug, Default)]
pub struct MacAxTextPort;

impl AxTextPort for MacAxTextPort {
    fn is_trusted(&mut self) -> bool {
        // SAFETY: AXIsProcessTrusted takes no arguments and has no side effect
        // (it never prompts, unlike AXIsProcessTrustedWithOptions).
        unsafe { AXIsProcessTrusted() != 0 }
    }

    fn focused_field(&mut self, pid: i32, timeout: Duration) -> Result<Box<dyn AxField>, AxError> {
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
            .ok_or(AxError::Failed(AX_ERROR_CANNOT_COMPLETE))?;
        let element = copy_attribute(&application, "AXFocusedUIElement", timeout)?;
        Ok(Box::new(MacAxField { element }))
    }
}

/// One AX element of the destination application.
pub struct MacAxField {
    element: CfOwned,
}

impl MacAxField {
    fn attribute_is(&self, name: &str, expected: &str, timeout: Duration) -> Result<bool, AxError> {
        let value = copy_attribute(&self.element, name, timeout)?;
        let text = string_utf16(&value, expected.len())
            .map(|units| String::from_utf16_lossy(&units))
            .unwrap_or_default();
        Ok(text == expected)
    }
}

impl AxField for MacAxField {
    fn pid(&mut self) -> Result<i32, AxError> {
        let mut pid = 0;
        // SAFETY: element is live; `pid` is a valid out pointer.
        ax_result(unsafe { AXUIElementGetPid(self.element.as_ptr(), &mut pid) })?;
        Ok(pid)
    }

    fn is_secure(&mut self, timeout: Duration) -> Result<bool, AxError> {
        // AppKit exposes password fields as role AXTextField with subrole
        // AXSecureTextField; some toolkits use the role itself.
        if self.attribute_is("AXRole", SECURE_TEXT_FIELD, timeout)? {
            return Ok(true);
        }
        match self.attribute_is("AXSubrole", SECURE_TEXT_FIELD, timeout) {
            Ok(is_secure) => Ok(is_secure),
            Err(AxError::NoValue | AxError::Unsupported) => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn value_utf16(&mut self, timeout: Duration, max_units: usize) -> Result<Vec<u16>, AxError> {
        string_utf16(
            &copy_attribute(&self.element, "AXValue", timeout)?,
            max_units,
        )
    }

    fn selected_range(&mut self, timeout: Duration) -> Result<Utf16Range, AxError> {
        range_value(&copy_attribute(
            &self.element,
            "AXSelectedTextRange",
            timeout,
        )?)
    }

    fn set_selected_text(&mut self, text: &[u16], timeout: Duration) -> Result<(), AxError> {
        set_messaging_timeout(&self.element, timeout)?;
        let attribute = attribute_name("AXSelectedText")?;
        let value = CfOwned::string(text).ok_or(AxError::TooLarge {
            utf16_units: text.len(),
        })?;
        // SAFETY: element, attribute and value are live CF objects.
        ax_result(unsafe {
            AXUIElementSetAttributeValue(self.element.as_ptr(), attribute.as_ptr(), value.as_ptr())
        })
    }
}
