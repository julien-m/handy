//! macOS accessibility adapter of 011's field port. It reads only the destination's application
//! element (`AXUIElementCreateApplication(pid)`) and elements obtained from it, never the
//! system-wide element, and sets the messaging timeout on each element right before messaging
//! it, to the remaining budget handed in by the caller (never 0, refused below by 010's helper).
//! It reads the facts of 010's rule, `AXSelectedTextRange` (a `CFRange`: position and length),
//! `AXWindow` and `AXFocusedWindow`; it writes `AXFocused`, `AXSelectedTextRange` and performs
//! `AXRaise`. It never reads `AXValue`, `AXSelectedText` or a title, and logs nothing.
use super::field::{FieldAccess, FieldNode, SelectionRange};
use crate::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use crate::recovery::platform::macos_ax::{
    attribute, ax_element, ax_result, copy_attribute, set_timeout, MacFocusedElement, AX_SUCCESS,
};
use crate::smart_paste::macos_cf::{type_id, CfIndex, CfOwned, CfRange, CfTypeRef};
use std::any::Any;
use std::ffi::c_void;
use std::time::Duration;

/// `kAXValueCFRangeType` (AXValue.h).
const AX_VALUE_CF_RANGE: u32 = 4;
/// `kAXFocusedAttribute`, `kAXSelectedTextRangeAttribute`, `kAXWindowAttribute`,
/// `kAXFocusedWindowAttribute`, `kAXFocusedUIElementAttribute` and `kAXRaiseAction` are `CFSTR`
/// macros in the SDK, so not linkable symbols.
const FOCUSED: &str = "AXFocused";
const SELECTED_TEXT_RANGE: &str = "AXSelectedTextRange";
const WINDOW: &str = "AXWindow";
const FOCUSED_WINDOW: &str = "AXFocusedWindow";
const FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
const RAISE: &str = "AXRaise";

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> CfTypeRef;
    fn AXUIElementSetAttributeValue(
        element: CfTypeRef,
        attribute: CfTypeRef,
        value: CfTypeRef,
    ) -> i32;
    fn AXUIElementPerformAction(element: CfTypeRef, action: CfTypeRef) -> i32;
    fn AXUIElementGetPid(element: CfTypeRef, pid: *mut i32) -> i32;
    fn AXValueCreate(value_type: u32, value: *const c_void) -> CfTypeRef;
    fn AXValueGetType(value: CfTypeRef) -> u32;
    fn AXValueGetValue(value: CfTypeRef, value_type: u32, out: *mut c_void) -> u8;
    fn AXValueGetTypeID() -> usize;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: CfTypeRef;
    fn CFEqual(left: CfTypeRef, right: CfTypeRef) -> u8;
}

/// Production [`FieldAccess`] of macOS.
#[derive(Debug, Default)]
pub struct MacFieldAccess;

impl MacFieldAccess {
    /// `attribute` (an element) of application `pid`, with `timeout` on its application element.
    fn application_element(
        pid: u32,
        name: &str,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        let pid = i32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
            .ok_or(AxQueryError::Failed)?;
        let element = ax_element(copy_attribute(&application, name, timeout)?)?;
        Ok(Box::new(MacFieldNode::new(element)))
    }
}

impl FieldAccess for MacFieldAccess {
    fn is_trusted(&self) -> bool {
        // SAFETY: AXIsProcessTrusted takes no arguments and never prompts.
        unsafe { AXIsProcessTrusted() != 0 }
    }
    fn focused_element(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        Self::application_element(pid, FOCUSED_UI_ELEMENT, timeout)
    }
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        Self::application_element(pid, FOCUSED_WINDOW, timeout)
    }
}

/// One `AXUIElement` held by 011 (released once on drop by `CfOwned`).
pub struct MacFieldNode {
    inner: MacFocusedElement,
}

impl MacFieldNode {
    fn new(element: CfOwned) -> Self {
        Self {
            inner: MacFocusedElement { element },
        }
    }
    fn element(&self) -> &CfOwned {
        &self.inner.element
    }
    /// The owning process, a local read without messaging.
    fn process(&self) -> Option<i32> {
        let mut pid = 0;
        // SAFETY: the element is live; `pid` is a valid out pointer.
        let status = unsafe { AXUIElementGetPid(self.element().as_ptr(), &mut pid) };
        (status == AX_SUCCESS).then_some(pid)
    }
    fn set(&self, name: &str, value: CfTypeRef, timeout: Duration) -> Result<(), AxQueryError> {
        set_timeout(self.element(), timeout)?;
        let name = attribute(name)?;
        // SAFETY: element, attribute and value are live CF objects for the call.
        ax_result(unsafe {
            AXUIElementSetAttributeValue(self.element().as_ptr(), name.as_ptr(), value)
        })
    }
}

/// The `CFRange` inside an `AXValue`, or `Failed` for any other value (also 015's caret).
pub(crate) fn cf_range(value: &CfOwned) -> Result<CfRange, AxQueryError> {
    // SAFETY: AXValueGetTypeID takes no arguments; the value is a live CF object.
    let is_ax_value = type_id(value.as_ptr()) == unsafe { AXValueGetTypeID() };
    // SAFETY: AXValueGetType is only called on an AXValue.
    if !is_ax_value || unsafe { AXValueGetType(value.as_ptr()) } != AX_VALUE_CF_RANGE {
        return Err(AxQueryError::Failed);
    }
    let mut range = CfRange::default();
    // SAFETY: the value holds a CFRange; `range` is a CFRange-sized out buffer.
    let read = unsafe {
        AXValueGetValue(
            value.as_ptr(),
            AX_VALUE_CF_RANGE,
            (&raw mut range).cast::<c_void>(),
        )
    };
    if read == 0 {
        return Err(AxQueryError::Failed);
    }
    Ok(range)
}

fn to_offsets(range: CfRange) -> Result<SelectionRange, AxQueryError> {
    let location = usize::try_from(range.location).map_err(|_| AxQueryError::Failed)?;
    let length = usize::try_from(range.length).map_err(|_| AxQueryError::Failed)?;
    Ok(SelectionRange::offsets(location, length))
}

impl FocusedElement for MacFieldNode {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        self.inner.pid()
    }
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError> {
        self.inner.role(timeout)
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        self.inner.subrole(timeout)
    }
    fn is_settable(
        &mut self,
        attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.inner.is_settable(attribute, timeout)
    }
}

impl FieldNode for MacFieldNode {
    fn read_range(&mut self, timeout: Duration) -> Result<SelectionRange, AxQueryError> {
        let value = copy_attribute(self.element(), SELECTED_TEXT_RANGE, timeout)?;
        to_offsets(cf_range(&value)?)
    }
    fn range_is(
        &mut self,
        range: &SelectionRange,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        let expected = range.as_offsets().ok_or(AxQueryError::Unsupported)?;
        let current = self.read_range(timeout)?;
        Ok(current.as_offsets() == Some(expected))
    }
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        // SAFETY: kCFBooleanTrue is an immortal CoreFoundation constant.
        self.set(FOCUSED, unsafe { kCFBooleanTrue }, timeout)
    }
    fn select(&mut self, range: &SelectionRange, timeout: Duration) -> Result<(), AxQueryError> {
        let (location, length) = range.as_offsets().ok_or(AxQueryError::Unsupported)?;
        let range = CfRange {
            location: CfIndex::try_from(location).map_err(|_| AxQueryError::Failed)?,
            length: CfIndex::try_from(length).map_err(|_| AxQueryError::Failed)?,
        };
        // SAFETY: `range` is a CFRange read for the call; the result follows the Create Rule.
        let value =
            unsafe { AXValueCreate(AX_VALUE_CF_RANGE, (&raw const range).cast::<c_void>()) };
        let value = CfOwned::from_create_rule(value).ok_or(AxQueryError::Failed)?;
        self.set(SELECTED_TEXT_RANGE, value.as_ptr(), timeout)
    }
    fn window(&mut self, timeout: Duration) -> Result<Option<Box<dyn FieldNode>>, AxQueryError> {
        let window = ax_element(copy_attribute(self.element(), WINDOW, timeout)?)?;
        Ok(Some(Box::new(MacFieldNode::new(window))))
    }
    fn raise(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        set_timeout(self.element(), timeout)?;
        let action = attribute(RAISE)?;
        // SAFETY: element and action name are live CF objects for the call.
        ax_result(unsafe { AXUIElementPerformAction(self.element().as_ptr(), action.as_ptr()) })
    }
    fn same_as(&self, other: &dyn FieldNode) -> bool {
        let Some(other) = other.as_any().downcast_ref::<MacFieldNode>() else {
            return false;
        };
        // SAFETY: both elements are live CF objects; CFEqual is a local comparison.
        let equal = unsafe { CFEqual(self.element().as_ptr(), other.element().as_ptr()) } != 0;
        equal && self.process().is_some() && self.process() == other.process()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        timeout: Duration,
    ) -> Result<Vec<u16>, AxQueryError> {
        crate::smart_capitalization::macos::string_for_range(
            self.element(),
            (location, length),
            timeout,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_negative_cf_range_is_never_a_selection() {
        let negative = CfRange {
            location: -1,
            length: 0,
        };
        assert!(to_offsets(negative).is_err());
        let range = to_offsets(CfRange {
            location: 3,
            length: 2,
        });
        assert_eq!(
            range.ok().and_then(|range| range.as_offsets()),
            Some((3, 2))
        );
    }
}
