//! macOS Text Input Sources and key-event adapters (AC-010, AC-011).
//!
//! Text Input Sources must run on the main thread, so each call is one short
//! bounded main-thread dispatch. Key events are created and posted with
//! CoreGraphics from the calling worker thread; posting success never means
//! the destination received the text.

use std::ffi::c_void;
use std::time::Duration;

use super::key_events::{unicode_chunks, KeyEventPort, PasteKey, MAX_UNICODE_UNITS_PER_EVENT};
use super::keyboard::{InputSourceInfo, InputSourceKind, InputSourcePort, KeyboardError};
use super::macos_cf::{string_units, CfOwned, CfTypeRef};
use super::main_thread::run_on_main;

// @spec FR-007: Keyboard and input-source adapters — .specs/features/007-smart-paste/spec.md#fr-007

/// Main-thread Text Input Sources calls are short; one second only trips
/// when the UI thread is stuck.
const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(1);
/// Longest accepted input-source identifier, far above real ones.
const MAX_SOURCE_ID_UNITS: usize = 512;
/// kVK_Command from HIToolbox/Events.h.
const COMMAND_KEY_CODE: u16 = 0x37;
/// kVK_ANSI_V: Handy's historical fallback when the layout cannot be read.
const ANSI_V_KEY_CODE: u16 = 9;
/// kCGEventFlagMaskCommand from CGEventTypes.h.
const COMMAND_FLAG: u64 = 0x0010_0000;
/// kCGHIDEventTap: events enter where hardware events do.
const HID_EVENT_TAP: u32 = 0;

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceID: CfTypeRef;
    static kTISPropertyInputSourceType: CfTypeRef;
    static kTISTypeKeyboardLayout: CfTypeRef;
    fn TISCopyCurrentKeyboardInputSource() -> CfTypeRef;
    fn TISGetInputSourceProperty(source: CfTypeRef, key: CfTypeRef) -> CfTypeRef;
    fn TISCreateInputSourceList(properties: CfTypeRef, include_all_installed: u8) -> CfTypeRef;
    fn TISSelectInputSource(source: CfTypeRef) -> i32;
}

/// Opaque CFDictionary callback tables, only ever passed by address.
#[repr(C)]
struct CfDictionaryCallBacks {
    _opaque: [u8; 0],
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeDictionaryKeyCallBacks: CfDictionaryCallBacks;
    static kCFTypeDictionaryValueCallBacks: CfDictionaryCallBacks;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const CfTypeRef,
        values: *const CfTypeRef,
        count: isize,
        key_callbacks: *const CfDictionaryCallBacks,
        value_callbacks: *const CfDictionaryCallBacks,
    ) -> CfTypeRef;
    fn CFArrayGetCount(array: CfTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CfTypeRef, index: isize) -> CfTypeRef;
    fn CFEqual(left: CfTypeRef, right: CfTypeRef) -> u8;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreateKeyboardEvent(source: *const c_void, key: u16, key_down: bool) -> CfTypeRef;
    fn CGEventSetFlags(event: CfTypeRef, flags: u64);
    fn CGEventKeyboardSetUnicodeString(event: CfTypeRef, length: usize, string: *const u16);
    fn CGEventPost(tap: u32, event: CfTypeRef);
}

fn current_source_on_main() -> Result<InputSourceInfo, KeyboardError> {
    // SAFETY: runs on the main thread; the source follows the Create Rule.
    let source = CfOwned::from_create_rule(unsafe { TISCopyCurrentKeyboardInputSource() })
        .ok_or(KeyboardError::SourceUnavailable)?;
    // SAFETY: Get Rule values owned by the retained source; constants are
    // Carbon statics.
    let (id, kind) = unsafe {
        (
            TISGetInputSourceProperty(source.as_ptr(), kTISPropertyInputSourceID),
            TISGetInputSourceProperty(source.as_ptr(), kTISPropertyInputSourceType),
        )
    };
    let id = string_units(id, MAX_SOURCE_ID_UNITS).map_err(|_| KeyboardError::SourceUnavailable)?;
    // SAFETY: both are live CFStrings (or null, which CFEqual must not get).
    let is_layout = !kind.is_null() && unsafe { CFEqual(kind, kTISTypeKeyboardLayout) != 0 };
    Ok(InputSourceInfo {
        id: String::from_utf16_lossy(&id),
        kind: if is_layout {
            InputSourceKind::KeyboardLayout
        } else {
            InputSourceKind::InputMethod
        },
    })
}

fn select_source_on_main(id: &str) -> Result<(), KeyboardError> {
    let value = CfOwned::from_text(id).ok_or(KeyboardError::SelectFailed)?;
    // SAFETY: keys/values are live CF objects for the duration of the call;
    // the CFType callbacks retain them; the dictionary follows the Create Rule.
    let filter = CfOwned::from_create_rule(unsafe {
        let key = kTISPropertyInputSourceID;
        let value = value.as_ptr();
        CFDictionaryCreate(
            std::ptr::null(),
            &key,
            &value,
            1,
            &raw const kCFTypeDictionaryKeyCallBacks,
            &raw const kCFTypeDictionaryValueCallBacks,
        )
    })
    .ok_or(KeyboardError::SelectFailed)?;
    // Only enabled sources are listed (include_all_installed = false):
    // selecting a disabled layout would require changing system settings.
    // SAFETY: runs on the main thread; the list follows the Create Rule.
    let list = CfOwned::from_create_rule(unsafe { TISCreateInputSourceList(filter.as_ptr(), 0) })
        .ok_or(KeyboardError::SelectFailed)?;
    // SAFETY: `list` is a live CFArray; index 0 exists when count > 0 and
    // its element is owned by the array.
    let status = unsafe {
        if CFArrayGetCount(list.as_ptr()) == 0 {
            return Err(KeyboardError::SelectFailed);
        }
        TISSelectInputSource(CFArrayGetValueAtIndex(list.as_ptr(), 0))
    };
    (status == 0)
        .then_some(())
        .ok_or(KeyboardError::SelectFailed)
}

/// Production [`InputSourcePort`] backed by Text Input Sources.
#[derive(Debug, Default)]
pub struct MacInputSources;

impl InputSourcePort for MacInputSources {
    fn current_source(&mut self) -> Result<InputSourceInfo, KeyboardError> {
        run_on_main(current_source_on_main, MAIN_THREAD_TIMEOUT)
            .map_err(|_| KeyboardError::MainThreadUnavailable)?
    }

    fn select_source(&mut self, id: &str) -> Result<(), KeyboardError> {
        let id = id.to_string();
        run_on_main(move || select_source_on_main(&id), MAIN_THREAD_TIMEOUT)
            .map_err(|_| KeyboardError::MainThreadUnavailable)?
    }
}

/// Owned CoreGraphics keyboard event.
fn key_event(key: u16, is_down: bool, flags: u64) -> Result<CfOwned, KeyboardError> {
    // SAFETY: a null source is allowed; the event follows the Create Rule.
    let event = CfOwned::from_create_rule(unsafe {
        CGEventCreateKeyboardEvent(std::ptr::null(), key, is_down)
    })
    .ok_or(KeyboardError::PostFailed)?;
    // SAFETY: `event` is a live CGEvent.
    unsafe { CGEventSetFlags(event.as_ptr(), flags) };
    Ok(event)
}

fn post(event: &CfOwned) {
    // SAFETY: `event` is a live CGEvent; posting is thread-safe.
    unsafe { CGEventPost(HID_EVENT_TAP, event.as_ptr()) };
}

/// Production [`KeyEventPort`] posting CoreGraphics events.
#[derive(Debug, Default)]
pub struct MacKeyEvents;

impl KeyEventPort for MacKeyEvents {
    fn resolve_paste_key(&mut self) -> Result<PasteKey, KeyboardError> {
        // Resolved for the layout active now, after any switch (AC-008);
        // an unreadable layout falls back to the ANSI V key code as before.
        let resolved = run_on_main(
            crate::input::resolve_macos_command_v_keycode,
            MAIN_THREAD_TIMEOUT,
        )
        .map_err(|_| KeyboardError::MainThreadUnavailable)?;
        Ok(PasteKey(resolved.unwrap_or(ANSI_V_KEY_CODE)))
    }

    fn post_paste_chord(&mut self, key: PasteKey, hold: Duration) -> Result<(), KeyboardError> {
        // All events are created first, so a creation failure posts nothing
        // and can never leave Command held down.
        let events = [
            key_event(COMMAND_KEY_CODE, true, COMMAND_FLAG)?,
            key_event(key.0, true, COMMAND_FLAG)?,
            key_event(key.0, false, COMMAND_FLAG)?,
        ];
        let release = key_event(COMMAND_KEY_CODE, false, 0)?;
        events.iter().for_each(post);
        std::thread::sleep(hold);
        post(&release);
        Ok(())
    }

    fn type_unicode(&mut self, text: &str) -> Result<(), KeyboardError> {
        let units: Vec<u16> = text.encode_utf16().collect();
        for chunk in unicode_chunks(&units, MAX_UNICODE_UNITS_PER_EVENT) {
            for is_down in [true, false] {
                let event = key_event(0, is_down, 0)?;
                // SAFETY: `chunk` is valid for `chunk.len()` units during the call.
                unsafe {
                    CGEventKeyboardSetUnicodeString(event.as_ptr(), chunk.len(), chunk.as_ptr())
                };
                post(&event);
            }
        }
        Ok(())
    }
}
