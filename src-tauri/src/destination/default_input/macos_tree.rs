//! macOS accessibility adapter of feature 013's tree port. It reads only one application's
//! element (`AXUIElementCreateApplication(pid)`) and the elements obtained from it, never the
//! system-wide element, and sets the messaging timeout on each element right before messaging
//! it. One `AXUIElementCopyMultipleAttributeValues` per element returns its role, subrole,
//! title, description, placeholder, DOM id and children (a missing attribute comes back as an
//! error value and reads as absent). It never reads `AXValue`, `AXSelectedText` or DOM classes,
//! writes only `AXFocused`, and logs nothing.
use super::fingerprint::NodeFacts;
use super::model::MAX_TEXT_CHARS;
use super::tree::{NodeRead, TreeAccess, TreeNode};
use crate::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use crate::recovery::platform::macos_ax::{
    attribute, ax_element, ax_result, copy_attribute, set_timeout, MacFocusedElement,
};
use crate::smart_paste::macos_cf::{string_units, type_id, CfIndex, CfOwned, CfTypeRef};
use std::any::Any;
use std::ffi::c_void;
use std::ptr;
use std::time::Duration;

/// Attributes of one read, in the order of `NodeRead`; `CFSTR` macros in the SDK, so not linkable.
const READ_ATTRIBUTES: [&str; 7] = [
    "AXRole",
    "AXSubrole",
    "AXTitle",
    "AXDescription",
    "AXPlaceholderValue",
    "AXDOMIdentifier",
    "AXChildren",
];
/// Most children taken from one element: far above any real container, a guard against a
/// pathological list (the search's element bound applies anyway).
const MAX_CHILDREN: CfIndex = 4096;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> CfTypeRef;
    fn AXUIElementCopyMultipleAttributeValues(
        element: CfTypeRef,
        attributes: CfTypeRef,
        options: u32,
        values: *mut CfTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: CfTypeRef,
        attribute: CfTypeRef,
        value: CfTypeRef,
    ) -> i32;
    fn AXIsProcessTrusted() -> u8;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: CfTypeRef;
    // Only its address is passed to CFArrayCreate; the struct's layout is never read here.
    static kCFTypeArrayCallBacks: u8;
    fn CFArrayCreate(
        allocator: *const c_void,
        values: *const CfTypeRef,
        count: CfIndex,
        callbacks: *const c_void,
    ) -> CfTypeRef;
    fn CFArrayGetCount(array: CfTypeRef) -> CfIndex;
    fn CFArrayGetValueAtIndex(array: CfTypeRef, index: CfIndex) -> CfTypeRef;
    fn CFArrayGetTypeID() -> usize;
    fn CFRetain(value: CfTypeRef) -> CfTypeRef;
    fn CFEqual(left: CfTypeRef, right: CfTypeRef) -> u8;
}

/// Production [`TreeAccess`] of macOS.
pub struct MacTreeAccess;

impl MacTreeAccess {
    fn application_element(
        pid: u32,
        name: &str,
        timeout: Duration,
    ) -> Result<Box<dyn TreeNode>, AxQueryError> {
        let pid = i32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
            .ok_or(AxQueryError::Failed)?;
        let element = ax_element(copy_attribute(&application, name, timeout)?)?;
        Ok(Box::new(MacTreeNode::new(element)))
    }
}

impl TreeAccess for MacTreeAccess {
    fn is_trusted(&self) -> bool {
        // SAFETY: AXIsProcessTrusted takes no arguments and never prompts.
        unsafe { AXIsProcessTrusted() != 0 }
    }
    fn focused_element(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn TreeNode>, AxQueryError> {
        Self::application_element(pid, "AXFocusedUIElement", timeout)
    }
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn TreeNode>, AxQueryError> {
        Self::application_element(pid, "AXFocusedWindow", timeout)
    }
}

/// One `AXUIElement` held by 013 (released once on drop by `CfOwned`).
pub struct MacTreeNode {
    inner: MacFocusedElement,
}

impl MacTreeNode {
    fn new(element: CfOwned) -> Self {
        Self {
            inner: MacFocusedElement { element },
        }
    }
    fn element(&self) -> &CfOwned {
        &self.inner.element
    }
}

/// The attribute names of one read as a `CFArray` of `CFString`s.
fn attribute_names() -> Result<CfOwned, AxQueryError> {
    let names = READ_ATTRIBUTES
        .iter()
        .map(|name| attribute(name))
        .collect::<Result<Vec<_>, _>>()?;
    let pointers: Vec<CfTypeRef> = names.iter().map(CfOwned::as_ptr).collect();
    let count = CfIndex::try_from(pointers.len()).map_err(|_| AxQueryError::Failed)?;
    // SAFETY: `pointers` holds `count` live CFStrings for the call; the array retains them
    // (type callbacks) and follows the Create Rule.
    let array = unsafe {
        CFArrayCreate(
            ptr::null(),
            pointers.as_ptr(),
            count,
            (&raw const kCFTypeArrayCallBacks).cast::<c_void>(),
        )
    };
    CfOwned::from_create_rule(array).ok_or(AxQueryError::Failed)
}

/// Entry `index` of a live `CFArray` (borrowed, not retained).
fn array_value(array: CfTypeRef, index: CfIndex) -> CfTypeRef {
    // SAFETY: callers pass a live CFArray and an index below its count.
    unsafe { CFArrayGetValueAtIndex(array, index) }
}

fn is_array(value: CfTypeRef) -> bool {
    // SAFETY: CFArrayGetTypeID takes no arguments; `value` is a live CF object.
    !value.is_null() && type_id(value) == unsafe { CFArrayGetTypeID() }
}

/// A string value, absent when it is not a string (an error value) or longer than the bound.
fn text(value: CfTypeRef) -> Option<String> {
    string_units(value, MAX_TEXT_CHARS)
        .ok()
        .map(|units| String::from_utf16_lossy(&units))
}

/// The children of a `CFArray` value, each retained into its own node.
fn children(value: CfTypeRef) -> Vec<Box<dyn TreeNode>> {
    if !is_array(value) {
        return Vec::new();
    }
    // SAFETY: `value` is a live CFArray.
    let count = unsafe { CFArrayGetCount(value) }.min(MAX_CHILDREN);
    (0..count)
        .filter_map(|index| {
            let child = array_value(value, index);
            // SAFETY: the child is live while the array lives; CFRetain gives a +1 reference
            // owned by the CfOwned (released once on drop).
            let owned = CfOwned::from_create_rule(unsafe { CFRetain(child) })?;
            ax_element(owned).ok()
        })
        .map(|element| Box::new(MacTreeNode::new(element)) as Box<dyn TreeNode>)
        .collect()
}

impl FocusedElement for MacTreeNode {
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

impl TreeNode for MacTreeNode {
    fn read(&mut self, timeout: Duration) -> Result<NodeRead, AxQueryError> {
        set_timeout(self.element(), timeout)?;
        let names = attribute_names()?;
        let mut values: CfTypeRef = ptr::null();
        // SAFETY: element and names are live CF objects; `values` receives a +1 CFArray.
        ax_result(unsafe {
            AXUIElementCopyMultipleAttributeValues(
                self.element().as_ptr(),
                names.as_ptr(),
                0,
                &mut values,
            )
        })?;
        let values = CfOwned::from_create_rule(values).ok_or(AxQueryError::NoValue)?;
        let array = values.as_ptr();
        // SAFETY: `array` is checked to be a CFArray before its count is read.
        if !is_array(array) || unsafe { CFArrayGetCount(array) } < 7 {
            return Err(AxQueryError::Failed);
        }
        let at = |index| array_value(array, index);
        let facts = NodeFacts {
            role: text(at(0)),
            subrole: text(at(1)),
            title: text(at(2)),
            description: text(at(3)),
            placeholder: text(at(4)),
            dom_id: text(at(5)),
        };
        Ok(NodeRead {
            facts,
            children: children(at(6)),
        })
    }
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        set_timeout(self.element(), timeout)?;
        let name = attribute("AXFocused")?;
        // SAFETY: element, attribute and the immortal kCFBooleanTrue are live for the call.
        ax_result(unsafe {
            AXUIElementSetAttributeValue(self.element().as_ptr(), name.as_ptr(), kCFBooleanTrue)
        })
    }
    fn same_as(&self, other: &dyn TreeNode) -> bool {
        let Some(other) = other.as_any().downcast_ref::<MacTreeNode>() else {
            return false;
        };
        // SAFETY: both elements are live CF objects; CFEqual is a local comparison.
        unsafe { CFEqual(self.element().as_ptr(), other.element().as_ptr()) != 0 }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
