//! Escape watch of the bubble's native drag session (spec AC-004, native session N25). AppKit
//! cancels a drag on Escape but reports it to the source like any drop nobody accepted
//! (`NSDragOperationNone`), so the source cannot tell Escape from a release outside a target.
//! For the session's lifetime only, Escape key presses are watched in the three places where
//! AppKit can deliver one (which one does is a native proof, N25 in the checklist):
//! - a global monitor: Handy's panel never becomes key, so a key press routed to the frontmost
//!   application is seen there (key events need the accessibility trust Handy already holds
//!   for its shortcuts);
//! - a local monitor: the same key press when it is dispatched to Handy itself;
//! - AppKit's current event when the session ends: the key press the drag's own tracking loop
//!   consumed, which no monitor sees (monitors skip nested tracking loops).
//!
//! The monitors are removed when the watch is finished or dropped, whatever ended the session.
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSEvent, NSEventMask, NSEventType};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Virtual key code of Escape (`kVK_Escape` = 0x35 in HIToolbox `Events.h`).
const ESCAPE_KEY_CODE: u16 = 0x35;

/// Escape key presses seen since the drag session started; main thread only.
pub(super) struct EscapeWatch {
    pressed: Arc<AtomicBool>,
    monitors: Vec<Retained<AnyObject>>,
}

impl EscapeWatch {
    /// Install the global and local key-down monitors for one drag session.
    pub(super) fn start(_mtm: MainThreadMarker) -> Self {
        let pressed = Arc::new(AtomicBool::new(false));
        // A missing monitor only loses that source: an Escape it would have seen leaves the
        // bubble for another attempt, the behavior before N25.
        let monitors = [global_monitor(&pressed), local_monitor(&pressed)]
            .into_iter()
            .flatten()
            .collect();
        Self { pressed, monitors }
    }

    /// End the watch (removing its monitors) and tell whether Escape was pressed during the
    /// session, including the key press that ends it.
    pub(super) fn finish(self, mtm: MainThreadMarker) -> bool {
        let ending = NSApplication::sharedApplication(mtm).currentEvent();
        self.pressed.load(Ordering::SeqCst) || ending.as_deref().is_some_and(is_escape_down)
    }
}

impl Drop for EscapeWatch {
    fn drop(&mut self) {
        for monitor in self.monitors.drain(..) {
            // SAFETY: each object was returned by `addGlobal`/`addLocalMonitor…` and is removed
            // once.
            unsafe { NSEvent::removeMonitor(&monitor) };
        }
    }
}

/// Escape presses routed to other applications (Handy's panel is never key).
fn global_monitor(pressed: &Arc<AtomicBool>) -> Option<Retained<AnyObject>> {
    let seen = Arc::clone(pressed);
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| {
        // SAFETY: AppKit hands the handler a valid event for the duration of the call.
        note_escape(unsafe { event.as_ref() }, &seen);
    });
    let monitor =
        NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler);
    if monitor.is_none() {
        log::warn!("Recovery drag: no global Escape monitor");
    }
    monitor
}

/// Escape presses dispatched to Handy itself, passed on unchanged.
fn local_monitor(pressed: &Arc<AtomicBool>) -> Option<Retained<AnyObject>> {
    let seen = Arc::clone(pressed);
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: as above; the event is returned unchanged so its dispatch goes on.
        note_escape(unsafe { event.as_ref() }, &seen);
        event.as_ptr()
    });
    // SAFETY: the handler returns the valid event it received, never a dangling pointer.
    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler)
    };
    if monitor.is_none() {
        log::warn!("Recovery drag: no local Escape monitor");
    }
    monitor
}

fn note_escape(event: &NSEvent, pressed: &AtomicBool) {
    if is_escape_down(event) {
        pressed.store(true, Ordering::SeqCst);
    }
}

/// An Escape key press. The type is checked first: `keyCode` raises on a non-key event.
fn is_escape_down(event: &NSEvent) -> bool {
    event.r#type() == NSEventType::KeyDown && event.keyCode() == ESCAPE_KEY_CODE
}
