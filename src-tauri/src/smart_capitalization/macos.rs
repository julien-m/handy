//! macOS accessibility adapter of the context read (feature 015). It reads only the destination
//! application's element (`AXUIElementCreateApplication(pid)`) and its focused element, never
//! the system-wide element; it sets the messaging timeout right before each request (never 0).
//! It reads `AXSelectedTextRange` and the bounded parameterized attribute `AXStringForRange`,
//! never `AXValue`; the caller has classified the element editable (not a password field) by
//! 010's rule before any text is read. Smart spacing (017) adds `AXNumberOfCharacters`, a
//! length, so the bounded read after the selection never asks beyond the field's end.
//! Nothing read here is logged.
use super::reader::{CaretElement, CaretPort, FieldText};
use crate::recovery::classifier::{AxQueryError, FocusedElement};
use crate::recovery::platform::macos_ax::{
    attribute, ax_element, ax_result, copy_attribute, set_timeout, MacFocusedElement,
};
use crate::smart_paste::macos_cf::{string_units, type_id, CfIndex, CfOwned, CfRange, CfTypeRef};
use std::ffi::c_void;
use std::ptr;
use std::time::Duration;

/// `kAXValueCFRangeType` (AXValue.h).
const AX_VALUE_CF_RANGE: u32 = 4;
/// `kCFNumberSInt64Type` (CFNumber.h).
const CF_NUMBER_SINT64: CfIndex = 4;
/// `kAXStringForRangeParameterizedAttribute`, `kAXSelectedTextRangeAttribute` and
/// `kAXNumberOfCharactersAttribute` are `CFSTR` macros in the SDK, so not linkable symbols.
const STRING_FOR_RANGE: &str = "AXStringForRange";
const SELECTED_TEXT_RANGE: &str = "AXSelectedTextRange";
const NUMBER_OF_CHARACTERS: &str = "AXNumberOfCharacters";
const FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFNumberGetTypeID() -> usize;
    fn CFNumberGetValue(number: CfTypeRef, number_type: CfIndex, value: *mut c_void) -> u8;
}

// @spec AC-009: Bounded read after the selection — .specs/features/017-smart-spacing/spec.md#ac-009
/// The length of `element`'s text in UTF-16 units (`AXNumberOfCharacters`), with `timeout`
/// set on `element` first; any other answer than a non-negative number is `Failed`.
pub(crate) fn character_count(element: &CfOwned, timeout: Duration) -> Result<usize, AxQueryError> {
    let value = copy_attribute(element, NUMBER_OF_CHARACTERS, timeout)?;
    // SAFETY: CFNumberGetTypeID takes no arguments; the value is a live CF object.
    if type_id(value.as_ptr()) != unsafe { CFNumberGetTypeID() } {
        return Err(AxQueryError::Failed);
    }
    let mut count: i64 = 0;
    // SAFETY: the value is a CFNumber; `count` is an i64-sized out buffer for SInt64.
    let read = unsafe {
        CFNumberGetValue(
            value.as_ptr(),
            CF_NUMBER_SINT64,
            (&raw mut count).cast::<c_void>(),
        )
    };
    if read == 0 {
        return Err(AxQueryError::Failed);
    }
    usize::try_from(count).map_err(|_| AxQueryError::Failed)
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> CfTypeRef;
    fn AXUIElementCopyParameterizedAttributeValue(
        element: CfTypeRef,
        attribute: CfTypeRef,
        parameter: CfTypeRef,
        value: *mut CfTypeRef,
    ) -> i32;
    fn AXValueCreate(value_type: u32, value: *const c_void) -> CfTypeRef;
}

/// The `length` UTF-16 units of `element` starting at `location` (`AXStringForRange`), with
/// `timeout` set on `element` first. A longer answer than asked is refused without copying it.
pub(crate) fn string_for_range(
    element: &CfOwned,
    (location, length): (usize, usize),
    timeout: Duration,
) -> Result<Vec<u16>, AxQueryError> {
    let range = CfRange {
        location: CfIndex::try_from(location).map_err(|_| AxQueryError::Failed)?,
        length: CfIndex::try_from(length).map_err(|_| AxQueryError::Failed)?,
    };
    // SAFETY: `range` is a CFRange read for the call; the result follows the Create Rule.
    let parameter =
        unsafe { AXValueCreate(AX_VALUE_CF_RANGE, (&raw const range).cast::<c_void>()) };
    let parameter = CfOwned::from_create_rule(parameter).ok_or(AxQueryError::Failed)?;
    set_timeout(element, timeout)?;
    let name = attribute(STRING_FOR_RANGE)?;
    let mut value: CfTypeRef = ptr::null();
    // SAFETY: element, attribute and parameter are live CF objects; `value` receives a +1
    // reference (Copy Rule) owned by the CfOwned below.
    ax_result(unsafe {
        AXUIElementCopyParameterizedAttributeValue(
            element.as_ptr(),
            name.as_ptr(),
            parameter.as_ptr(),
            &mut value,
        )
    })?;
    let value = CfOwned::from_create_rule(value).ok_or(AxQueryError::NoValue)?;
    string_units(value.as_ptr(), length).map_err(|_| AxQueryError::Failed)
}

/// Production [`CaretPort`] of macOS.
#[derive(Debug, Default)]
pub struct MacCaretPort;

impl CaretPort for MacCaretPort {
    fn is_trusted(&mut self) -> bool {
        // SAFETY: AXIsProcessTrusted takes no arguments and never prompts.
        unsafe { AXIsProcessTrusted() != 0 }
    }
    fn focused_element(
        &mut self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn CaretElement>, AxQueryError> {
        let pid = i32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
            .ok_or(AxQueryError::Failed)?;
        let element = ax_element(copy_attribute(&application, FOCUSED_UI_ELEMENT, timeout)?)?;
        Ok(Box::new(MacCaretElement {
            inner: MacFocusedElement { element },
        }))
    }
}

/// The focused element of the frontmost application.
struct MacCaretElement {
    inner: MacFocusedElement,
}

impl CaretElement for MacCaretElement {
    fn facts(&mut self) -> &mut dyn FocusedElement {
        &mut self.inner
    }
}

impl FieldText for MacCaretElement {
    fn selection(&mut self, timeout: Duration) -> Result<(usize, usize), AxQueryError> {
        let value = copy_attribute(&self.inner.element, SELECTED_TEXT_RANGE, timeout)?;
        let range = crate::destination::macos_field::cf_range(&value)?;
        let start = usize::try_from(range.location).map_err(|_| AxQueryError::Failed)?;
        let length = usize::try_from(range.length).map_err(|_| AxQueryError::Failed)?;
        Ok((start, length))
    }
    fn character_count(&mut self, timeout: Duration) -> Result<usize, AxQueryError> {
        character_count(&self.inner.element, timeout)
    }
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError> {
        string_for_range(&self.inner.element, (location, length), timeout)
    }
}
