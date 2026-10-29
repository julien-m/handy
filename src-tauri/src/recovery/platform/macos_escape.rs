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
//! An Escape a monitor sees also turns off the session's slide back at once (N26): AppKit would
//! otherwise animate the drag image back to the bubble before reporting the cancellation. An
//! Escape seen only as the ending event comes too late for that: AppKit has already decided
//! (and played) the slide back when it reports the end, so that path keeps the animation and
//! the native proof (N26 in the checklist) records which path each Escape took.
//!
//! The monitors are removed when the watch is finished or dropped, whatever ended the session.
use crate::recovery::types::drag_image_slides_back;
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, Message};
use objc2_app_kit::{NSApplication, NSDraggingSession, NSEvent, NSEventMask, NSEventType};
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
    /// Install the global and local key-down monitors for `session`, whose slide back they
    /// turn off when they see Escape.
    pub(super) fn start(_mtm: MainThreadMarker, session: &NSDraggingSession) -> Self {
        let pressed = Arc::new(AtomicBool::new(false));
        let seen = EscapeSeen {
            pressed: Arc::clone(&pressed),
            session: session.retain(),
        };
        // A missing monitor only loses that source: an Escape it would have seen leaves the
        // bubble for another attempt, the behavior before N25.
        let monitors = [global_monitor(&seen), local_monitor(&seen)]
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

/// What a monitor updates when it sees Escape: the session's flag and its slide back.
#[derive(Clone)]
struct EscapeSeen {
    pressed: Arc<AtomicBool>,
    session: Retained<NSDraggingSession>,
}

impl EscapeSeen {
    // @spec AC-004: Escape drops the slide back at once — .specs/features/010-recovery-bubble/spec.md#ac-004
    /// Note an Escape key press. The slide back is turned off on the main thread only (AppKit
    /// calls monitor handlers there), before the cancellation ends the session.
    fn note(&self, event: &NSEvent) {
        if !is_escape_down(event) {
            return;
        }
        self.pressed.store(true, Ordering::SeqCst);
        if MainThreadMarker::new().is_some() {
            self.session
                .setAnimatesToStartingPositionsOnCancelOrFail(drag_image_slides_back(true));
        }
    }
}

/// Escape presses routed to other applications (Handy's panel is never key).
fn global_monitor(seen: &EscapeSeen) -> Option<Retained<AnyObject>> {
    let seen = seen.clone();
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| {
        // SAFETY: AppKit hands the handler a valid event for the duration of the call.
        seen.note(unsafe { event.as_ref() });
    });
    let monitor =
        NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler);
    if monitor.is_none() {
        log::warn!("Recovery drag: no global Escape monitor");
    }
    monitor
}

/// Escape presses dispatched to Handy itself, passed on unchanged.
fn local_monitor(seen: &EscapeSeen) -> Option<Retained<AnyObject>> {
    let seen = seen.clone();
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: as above; the event is returned unchanged so its dispatch goes on.
        seen.note(unsafe { event.as_ref() });
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

/// An Escape key press. The type is checked first: `keyCode` raises on a non-key event.
fn is_escape_down(event: &NSEvent) -> bool {
    event.r#type() == NSEventType::KeyDown && event.keyCode() == ESCAPE_KEY_CODE
}
