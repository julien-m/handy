//! Native macOS drag source of the bubble (spec AC-004): the full transcript as one plain-text
//! `NSPasteboardItem`, a separate preview image, copy as the only operation whatever the
//! modifier keys, the bubble hidden while dragging. The general pasteboard and 007's smart
//! paste restoration are never touched. The source object is retained until its session ends;
//! its end callback names the bubble, so the service ignores a closed bubble's late end, and
//! tells a release outside any target from an Escape cancellation (N25, `macos_escape`).
use super::macos::set_dragging;
use super::macos_escape::EscapeWatch;
use super::DragEnded;
use crate::recovery::types::{BubbleId, DragResult};
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession,
    NSDraggingSource, NSEventType, NSImage, NSPasteboardItem, NSPasteboardTypeString,
    NSPasteboardWriting, NSTextField,
};
use objc2_foundation::{NSArray, NSObject, NSPoint, NSRect, NSSize, NSString};
use std::cell::RefCell;
use tauri::{Manager, WebviewWindow};

/// Size of the preview image under the pointer (logical points).
const PREVIEW_SIZE: NSSize = NSSize {
    width: 280.0,
    height: 22.0,
};
/// Characters of the transcript drawn in the preview image (the payload stays complete).
const PREVIEW_CHARS: usize = 48;

struct SourceIvars {
    bubble_id: BubbleId,
    ended: DragEnded,
    /// Escape presses of this session, from its start to its end.
    escape: RefCell<Option<EscapeWatch>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements; the ivars are only touched on the main
    // thread, where AppKit calls a dragging source.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "HandyRecoveryDragSource"]
    #[ivars = SourceIvars]
    struct RecoveryDragSource;

    unsafe impl NSObjectProtocol for RecoveryDragSource {}

    unsafe impl NSDraggingSource for RecoveryDragSource {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn operation_mask(
            &self,
            _session: &NSDraggingSession,
            _context: NSDraggingContext,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(ignoreModifierKeysForDraggingSession:))]
        fn ignore_modifiers(&self, _session: &NSDraggingSession) -> bool {
            true
        }

        #[unsafe(method(draggingSession:endedAtPoint:operation:))]
        fn ended(&self, _session: &NSDraggingSession, _point: NSPoint, operation: NSDragOperation) {
            let ivars = self.ivars();
            // Taking the watch removes its monitors, whatever the result.
            let watch = ivars.escape.borrow_mut().take();
            let escape = watch.is_some_and(|watch| watch.finish(self.mtm()));
            (ivars.ended)(
                ivars.bubble_id,
                DragResult::of_session(!operation.is_empty(), escape),
            );
            // Released at the end of this run-loop turn, never while AppKit ends the session.
            if let Some(source) = ACTIVE_SOURCE.with(|source| source.borrow_mut().take()) {
                let _ = Retained::autorelease_ptr(source);
            }
        }
    }
);

impl RecoveryDragSource {
    fn new(mtm: MainThreadMarker, ivars: SourceIvars) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(ivars);
        // SAFETY: NSObject's designated initializer on a freshly allocated instance.
        unsafe { msg_send![super(this), init] }
    }
}

thread_local! {
    // The source of the session in progress, owned by the main thread until it ends.
    static ACTIVE_SOURCE: RefCell<Option<Retained<RecoveryDragSource>>> = const { RefCell::new(None) };
}

// @spec FR-003: Native plain-text drag in copy mode — .specs/features/010-recovery-bubble/spec.md#fr-003
/// Begin the session from the mouse event in progress; must run on the main thread.
pub(super) fn begin(
    window: &WebviewWindow,
    bubble_id: BubbleId,
    transcript: &str,
    ended: DragEnded,
) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or("drag off the main thread")?;
    let event = mouse_event(mtm).ok_or("no mouse drag in progress")?;
    let view = content_view(window)?;
    let location = view.convertPoint_fromView(event.locationInWindow(), None);
    let item = dragging_item(mtm, transcript, location)?;
    let ivars = SourceIvars {
        bubble_id,
        ended,
        escape: RefCell::new(None),
    };
    let source = RecoveryDragSource::new(mtm, ivars);
    let items = NSArray::from_retained_slice(&[item]);
    let session = view.beginDraggingSessionWithItems_event_source(
        &items,
        &event,
        ProtocolObject::from_ref(&*source),
    );
    session.setAnimatesToStartingPositionsOnCancelOrFail(true);
    // @spec AC-004: Escape during the drag is watched — .specs/features/010-recovery-bubble/spec.md#ac-004
    source.ivars().escape.replace(Some(EscapeWatch::start(mtm)));
    ACTIVE_SOURCE.with(|active| active.replace(Some(source)));
    set_dragging(window, true);
    Ok(())
}

/// The left-button press or drag the session starts from (AppKit's current event).
fn mouse_event(mtm: MainThreadMarker) -> Option<Retained<objc2_app_kit::NSEvent>> {
    let event = NSApplication::sharedApplication(mtm).currentEvent()?;
    matches!(
        event.r#type(),
        NSEventType::LeftMouseDown | NSEventType::LeftMouseDragged
    )
    .then_some(event)
}

fn content_view(window: &WebviewWindow) -> Result<Retained<objc2_app_kit::NSView>, String> {
    use tauri_nspanel::ManagerExt;
    let panel = window
        .app_handle()
        .get_webview_panel(super::RECOVERY_WINDOW)
        .map_err(|error| format!("{error:?}"))?;
    Ok(panel.content_view())
}

/// One item carrying the full transcript as plain text, with its preview image at `location`.
fn dragging_item(
    mtm: MainThreadMarker,
    transcript: &str,
    location: NSPoint,
) -> Result<Retained<NSDraggingItem>, String> {
    let pasteboard_item = NSPasteboardItem::new();
    // SAFETY: NSPasteboardTypeString is an immutable AppKit constant.
    let plain_text = unsafe { NSPasteboardTypeString };
    if !pasteboard_item.setString_forType(&NSString::from_str(transcript), plain_text) {
        return Err("the drag payload was refused".to_owned());
    }
    let writer: Retained<ProtocolObject<dyn NSPasteboardWriting>> =
        ProtocolObject::from_retained(pasteboard_item);
    let item = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), &writer);
    let origin = NSPoint::new(
        location.x - PREVIEW_SIZE.width / 2.0,
        location.y - PREVIEW_SIZE.height / 2.0,
    );
    let image = preview_image(mtm, transcript);
    // SAFETY: `image` is an NSImage, the type AppKit expects for the contents of a frame.
    unsafe { item.setDraggingFrame_contents(NSRect::new(origin, PREVIEW_SIZE), Some(&image)) };
    Ok(item)
}

/// The beginning of the transcript drawn as a one-line label, or a blank image.
fn preview_image(mtm: MainThreadMarker, transcript: &str) -> Retained<NSImage> {
    let preview: String = transcript.chars().take(PREVIEW_CHARS).collect();
    let label = NSTextField::labelWithString(&NSString::from_str(&preview), mtm);
    let bounds = NSRect::new(NSPoint::new(0.0, 0.0), PREVIEW_SIZE);
    label.setFrame(bounds);
    let data = label.dataWithPDFInsideRect(bounds);
    NSImage::initWithData(NSImage::alloc(), &data)
        .unwrap_or_else(|| NSImage::initWithSize(NSImage::alloc(), PREVIEW_SIZE))
}
