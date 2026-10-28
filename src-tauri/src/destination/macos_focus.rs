//! Live read of the focused application on macOS through the Accessibility system-wide element
//! (`AXFocusedApplication`). Unlike `NSWorkspace.frontmostApplication`, which this process only
//! refreshes when its main run loop turns, the answer comes from the system at the time of the
//! call, so the pre-dispatch guard sees an application switch made during the configured paste
//! delay even though it runs on the main thread. It needs the Accessibility permission Handy
//! already needs to send keys; any error is reported, never replaced by a cached value.
//! It also sends the accessibility activation request of the activation chain (N16): setting
//! `AXFrontmost` on one application's element, with a short messaging timeout.
use crate::recovery::classifier::AxQueryError;
use crate::recovery::platform::macos_ax::{attribute, ax_result, set_timeout};
use crate::smart_paste::macos_cf::CfOwned;
use objc2::rc::Retained;
use objc2_foundation::NSString;
use std::ffi::c_void;
use std::time::Duration;

type AxElementRef = *const c_void;
type CfTypeRef = *const c_void;

/// `kAXErrorSuccess`.
const AX_SUCCESS: i32 = 0;
/// `kAXFocusedApplicationAttribute` (a `CFSTR` macro in the SDK, so not a linkable symbol).
const FOCUSED_APPLICATION: &str = "AXFocusedApplication";
/// `kAXFrontmostAttribute` (a `CFSTR` macro in the SDK).
const FRONTMOST: &str = "AXFrontmost";
// Messaging timeout of the `AXFrontmost` request: a responsive application answers within a few
// milliseconds (N16 diagnostic), and an unresponsive one must not use up the 1 s activation
// deadline, which still has at least 700 ms left when this request is sent.
const FRONTMOST_REQUEST_TIMEOUT: Duration = Duration::from_millis(250);

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateSystemWide() -> AxElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AxElementRef,
        attribute: CfTypeRef,
        value: *mut CfTypeRef,
    ) -> i32;
    fn AXUIElementGetPid(element: AxElementRef, pid: *mut i32) -> i32;
    fn AXUIElementCreateApplication(pid: i32) -> AxElementRef;
    fn AXUIElementSetAttributeValue(
        element: AxElementRef,
        attribute: CfTypeRef,
        value: CfTypeRef,
    ) -> i32;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: CfTypeRef;
    fn CFRelease(value: CfTypeRef);
}

/// A Core Foundation object this module owns (Create/Copy rule), released once on drop.
struct Owned(CfTypeRef);
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer came from a Create/Copy call and is released exactly once.
            unsafe { CFRelease(self.0) };
        }
    }
}

/// Why the focused application could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusReadError {
    /// The Accessibility call failed with this `AXError` (permission missing, no focused
    /// application, or the system could not answer).
    Ax(i32),
    /// The call succeeded without a usable process id.
    NoProcess,
}

/// Process id of the focused application, read live from the system.
///
/// # Errors
/// The `AXError` of a failed call, or `NoProcess`.
pub fn focused_application_pid() -> Result<u32, FocusReadError> {
    let attribute: Retained<NSString> = NSString::from_str(FOCUSED_APPLICATION);
    // SAFETY: Create rule; the system-wide element is released by `Owned`.
    let system = Owned(unsafe { AXUIElementCreateSystemWide() });
    let mut value: CfTypeRef = std::ptr::null();
    // SAFETY: `NSString` is toll-free bridged to `CFString`; `value` receives a +1 reference
    // (Copy rule) on success, released by `Owned` below.
    let status = unsafe {
        AXUIElementCopyAttributeValue(system.0, Retained::as_ptr(&attribute).cast(), &mut value)
    };
    let application = Owned(value);
    if status != AX_SUCCESS {
        return Err(FocusReadError::Ax(status));
    }
    let mut pid = 0_i32;
    // SAFETY: `application.0` is the AXUIElement returned above; `pid` outlives the call.
    let status = unsafe { AXUIElementGetPid(application.0, &mut pid) };
    pid_of(status, pid)
}

// @spec FR-002: Accessibility activation request — .specs/features/006-destination-application/spec.md#fr-002
/// Ask application `pid` to come forward by setting `AXFrontmost` on its application element
/// (the system-wide element is never written). Success means the application accepted the
/// request, not that it is in front: the activation wait still observes the foreground.
///
/// # Errors
/// The `AXError` of the request, as a query error (`Failed` for an invalid process id).
pub fn request_frontmost(pid: u32) -> Result<(), AxQueryError> {
    let pid = i32::try_from(pid).map_err(|_| AxQueryError::Failed)?;
    // SAFETY: AXUIElementCreateApplication returns a +1 reference, released by `CfOwned`.
    let application = CfOwned::from_create_rule(unsafe { AXUIElementCreateApplication(pid) })
        .ok_or(AxQueryError::Failed)?;
    set_timeout(&application, FRONTMOST_REQUEST_TIMEOUT)?;
    let name = attribute(FRONTMOST)?;
    // SAFETY: element, attribute and the immortal `kCFBooleanTrue` are live for the call.
    ax_result(unsafe {
        AXUIElementSetAttributeValue(application.as_ptr(), name.as_ptr(), kCFBooleanTrue)
    })
}

/// Business rule: only a successful call with a positive process id names an application.
fn pid_of(status: i32, pid: i32) -> Result<u32, FocusReadError> {
    if status != AX_SUCCESS {
        return Err(FocusReadError::Ax(status));
    }
    u32::try_from(pid)
        .ok()
        .filter(|pid| *pid > 0)
        .ok_or(FocusReadError::NoProcess)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_successful_call_with_a_positive_pid_names_an_application() {
        assert_eq!(pid_of(AX_SUCCESS, 4242), Ok(4242));
        assert_eq!(pid_of(AX_SUCCESS, 0), Err(FocusReadError::NoProcess));
        assert_eq!(pid_of(AX_SUCCESS, -1), Err(FocusReadError::NoProcess));
        // kAXErrorAPIDisabled: the Accessibility permission is missing.
        assert_eq!(pid_of(-25211, 4242), Err(FocusReadError::Ax(-25211)));
    }
}
