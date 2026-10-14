//! Records how this test process empties pasteboards before writing.
//!
//! AppKit has no getter for the contents options applied to a pasteboard,
//! so the options are observed where they are applied: the
//! `prepareForNewContentsWithOptions:` and `clearContents` calls are
//! replaced, in this test process only, by recorders that forward to the
//! original AppKit implementations. Calls are recorded per pasteboard name.

use std::sync::{Mutex, OnceLock, PoisonError};

use objc2::runtime::{AnyClass, Imp, Sel};
use objc2::sel;
use objc2_app_kit::{NSPasteboard, NSPasteboardContentsOptions};

type PrepareFn = extern "C-unwind" fn(&NSPasteboard, Sel, NSPasteboardContentsOptions) -> isize;
type ClearFn = extern "C-unwind" fn(&NSPasteboard, Sel) -> isize;

/// One recorded call: pasteboard name and the call with its options.
pub type ContentsCall = (String, String);

static ORIGINALS: OnceLock<Option<(Imp, Imp)>> = OnceLock::new();
static CALLS: Mutex<Vec<ContentsCall>> = Mutex::new(Vec::new());

fn record(pasteboard: &NSPasteboard, call: String) {
    let name = pasteboard.name().to_string();
    CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((name, call));
}

extern "C-unwind" fn recording_prepare(
    pasteboard: &NSPasteboard,
    selector: Sel,
    options: NSPasteboardContentsOptions,
) -> isize {
    record(pasteboard, format!("prepare({})", options.0));
    let Some(Some((original, _))) = ORIGINALS.get() else {
        return 0;
    };
    // SAFETY: `original` is AppKit's implementation of this very selector,
    // whose signature is `- (NSInteger)prepareForNewContentsWithOptions:`.
    let original: PrepareFn = unsafe { std::mem::transmute::<Imp, PrepareFn>(*original) };
    original(pasteboard, selector, options)
}

extern "C-unwind" fn recording_clear(pasteboard: &NSPasteboard, selector: Sel) -> isize {
    record(pasteboard, "clear".to_string());
    let Some(Some((_, original))) = ORIGINALS.get() else {
        return 0;
    };
    // SAFETY: AppKit's implementation of `- (NSInteger)clearContents`.
    let original: ClearFn = unsafe { std::mem::transmute::<Imp, ClearFn>(*original) };
    original(pasteboard, selector)
}

/// Installs the recorders once for this process.
///
/// # Errors
/// Returns a message if NSPasteboard or one of its methods is missing.
pub fn install() -> Result<(), String> {
    let installed = ORIGINALS.get_or_init(|| {
        let class = AnyClass::get(c"NSPasteboard")?;
        let prepare = class.instance_method(sel!(prepareForNewContentsWithOptions:))?;
        let clear = class.instance_method(sel!(clearContents))?;
        // SAFETY: each replacement has the exact signature of the method it
        // replaces and forwards to the original implementation.
        unsafe {
            let prepare_imp = std::mem::transmute::<PrepareFn, Imp>(recording_prepare);
            let clear_imp = std::mem::transmute::<ClearFn, Imp>(recording_clear);
            Some((
                prepare.set_implementation(prepare_imp),
                clear.set_implementation(clear_imp),
            ))
        }
    });
    installed
        .as_ref()
        .map(|_| ())
        .ok_or_else(|| "NSPasteboard recorders could not be installed".to_string())
}

/// Takes the calls recorded for the pasteboard `name` since the last take.
pub fn take_calls(name: &str) -> Vec<String> {
    let mut calls = CALLS.lock().unwrap_or_else(PoisonError::into_inner);
    let (taken, kept): (Vec<_>, Vec<_>) = calls.drain(..).partition(|(board, _)| board == name);
    *calls = kept;
    taken.into_iter().map(|(_, call)| call).collect()
}
