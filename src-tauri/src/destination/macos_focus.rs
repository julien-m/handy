//! Live read of the focused application on macOS through the Accessibility system-wide element
//! (`AXFocusedApplication`). Unlike `NSWorkspace.frontmostApplication`, which this process only
//! refreshes when its main run loop turns, the answer comes from the system at the time of the
//! call, so the pre-dispatch guard sees an application switch made during the configured paste
//! delay even though it runs on the main thread. It needs the Accessibility permission Handy
//! already needs to send keys; any error is reported, never replaced by a cached value.
use objc2::rc::Retained;
use objc2_foundation::NSString;
use std::ffi::c_void;

type AxElementRef = *const c_void;
type CfTypeRef = *const c_void;

/// `kAXErrorSuccess`.
const AX_SUCCESS: i32 = 0;
/// `kAXFocusedApplicationAttribute` (a `CFSTR` macro in the SDK, so not a linkable symbol).
const FOCUSED_APPLICATION: &str = "AXFocusedApplication";

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateSystemWide() -> AxElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AxElementRef,
        attribute: CfTypeRef,
        value: *mut CfTypeRef,
    ) -> i32;
    fn AXUIElementGetPid(element: AxElementRef, pid: *mut i32) -> i32;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
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
