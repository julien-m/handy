//! Minimal CoreFoundation ownership and UTF-16 string helpers shared by the
//! macOS smart-paste adapters (accessibility and Text Input Sources).

use std::ffi::c_void;
use std::ptr;

/// Untyped CoreFoundation reference.
pub(crate) type CfTypeRef = *const c_void;
/// CoreFoundation signed index.
pub(crate) type CfIndex = isize;

/// CoreFoundation range, also used by AX selected-text ranges.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CfRange {
    pub(crate) location: CfIndex,
    pub(crate) length: CfIndex,
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: CfTypeRef);
    fn CFGetTypeID(value: CfTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringGetLength(string: CfTypeRef) -> CfIndex;
    fn CFStringGetCharacters(string: CfTypeRef, range: CfRange, buffer: *mut u16);
    fn CFStringCreateWithCharacters(
        allocator: *const c_void,
        characters: *const u16,
        count: CfIndex,
    ) -> CfTypeRef;
}

/// Owned (+1) CoreFoundation reference, released on drop.
pub(crate) struct CfOwned(CfTypeRef);

// SAFETY: CoreFoundation retain/release are thread-safe; AX elements may be
// messaged from any thread and other references are only used by their
// owner, on the thread the owning operation requires.
unsafe impl Send for CfOwned {}

impl CfOwned {
    /// Takes ownership of a reference obtained under the Create/Copy Rule.
    pub(crate) fn from_create_rule(reference: CfTypeRef) -> Option<Self> {
        (!reference.is_null()).then_some(Self(reference))
    }

    /// Creates a CFString from UTF-16 code units.
    pub(crate) fn string(units: &[u16]) -> Option<Self> {
        let count = CfIndex::try_from(units.len()).ok()?;
        // SAFETY: `units` is a valid buffer of `count` UTF-16 code units for
        // the duration of the call; the result follows the Create Rule.
        let string = unsafe { CFStringCreateWithCharacters(ptr::null(), units.as_ptr(), count) };
        Self::from_create_rule(string)
    }

    /// Creates a CFString from Rust text.
    pub(crate) fn from_text(text: &str) -> Option<Self> {
        Self::string(&text.encode_utf16().collect::<Vec<_>>())
    }

    /// Borrowed raw reference, valid while `self` lives.
    pub(crate) fn as_ptr(&self) -> CfTypeRef {
        self.0
    }
}

impl Drop for CfOwned {
    fn drop(&mut self) {
        // SAFETY: the reference was obtained under the Create/Copy Rule and
        // is released exactly once here.
        unsafe { CFRelease(self.0) };
    }
}

/// Why a CF value could not be read as a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CfStringError {
    /// The value is not a CFString.
    NotAString,
    /// The string is longer than the caller's limit (in UTF-16 units).
    TooLarge(usize),
}

/// Copies a CFString's UTF-16 code units, refusing strings longer than
/// `max_units` before copying anything.
pub(crate) fn string_units(value: CfTypeRef, max_units: usize) -> Result<Vec<u16>, CfStringError> {
    // SAFETY: `value` is a live CF object; its type is checked first.
    if value.is_null() || unsafe { CFGetTypeID(value) != CFStringGetTypeID() } {
        return Err(CfStringError::NotAString);
    }
    // SAFETY: `value` is a live CFString.
    let length = unsafe { CFStringGetLength(value) };
    let units = usize::try_from(length).map_err(|_| CfStringError::NotAString)?;
    if units > max_units {
        return Err(CfStringError::TooLarge(units));
    }
    let mut buffer = vec![0_u16; units];
    let range = CfRange {
        location: 0,
        length,
    };
    // SAFETY: `buffer` holds exactly `length` units, the full string range.
    unsafe { CFStringGetCharacters(value, range, buffer.as_mut_ptr()) };
    Ok(buffer)
}

/// Returns the CF type identifier of a live value.
pub(crate) fn type_id(value: CfTypeRef) -> usize {
    // SAFETY: callers pass live CF objects.
    unsafe { CFGetTypeID(value) }
}
