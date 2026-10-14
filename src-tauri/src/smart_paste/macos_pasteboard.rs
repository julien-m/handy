//! macOS pasteboard adapter for smart paste (AC-008, AC-009, AC-013).
//!
//! - Publication: `prepareForNewContentsWithOptions(currentHostOnly)`, then
//!   one fresh `NSPasteboardItem` whose plain text is provided on demand and
//!   whose transient/concealed/auto-generated markers are eager and empty.
//! - Every served plain-text request is timestamped and reported through the
//!   read sink; marker requests are never reads.
//! - Restoration rebuilds fresh items from the saved bytes as
//!   current-host-only content, so restoring never republishes the previous
//!   content to other devices (Universal Clipboard). Materialization writes
//!   eager plain text: current-host-only and marked like the on-demand item,
//!   or ordinary plain text for the "copy to clipboard" setting. Both are
//!   guarded by the publication `changeCount`, checked in the same
//!   main-thread operation as the write (AppKit offers no atomic
//!   compare-and-write across processes).
//! - Snapshots are captured by the read-only helper process, so a blocking
//!   promise from another application can be killed at the deadline; one
//!   helper is kept started ahead of the next capture (`warm_helper`).
//!
//! All AppKit calls run on the main thread through short bounded dispatches.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, AnyThread, DefinedClass};
use objc2_app_kit::{
    NSPasteboard, NSPasteboardContentsOptions, NSPasteboardItem, NSPasteboardItemDataProvider,
    NSPasteboardType, NSPasteboardTypeString, NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSObject, NSString};

use super::clock::Clock;
use super::ids::{TransactionId, Transcript};
use super::main_thread::run_on_main;
use super::pasteboard_port::{
    GuardedWrite, PasteboardError, PasteboardPort, ReadEvent, ReadSink, TranscriptMarking,
};
use super::pasteboard_types::HANDY_PRIVACY_MARKERS;
use super::snapshot::{PasteboardSnapshot, SnapshotBudget};
use super::snapshot_helper::HelperCommand;
use super::warm_helper::WarmHelper;

// @spec FR-005: Multi-type receipt transaction adapter — .specs/features/007-smart-paste/spec.md#fr-005
// @spec FR-006: Exact saved-type restore adapter — .specs/features/007-smart-paste/spec.md#fr-006

/// Main-thread operations are a few AppKit calls (restoring 50 MB is a
/// memory copy); one second only trips when the UI thread is stuck.
const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(1);

struct ProviderIvars {
    transaction: TransactionId,
    transcript: String,
    sink: ReadSink,
    clock: Arc<dyn Clock>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements; the ivars are plain
    // Rust values only touched on the main thread that serves requests.
    #[unsafe(super(NSObject))]
    #[name = "HandySmartPasteProvider"]
    #[ivars = ProviderIvars]
    struct SmartPasteProvider;

    unsafe impl NSObjectProtocol for SmartPasteProvider {}

    unsafe impl NSPasteboardItemDataProvider for SmartPasteProvider {
        #[unsafe(method(pasteboard:item:provideDataForType:))]
        fn provide_data(
            &self,
            _pasteboard: Option<&NSPasteboard>,
            item: &NSPasteboardItem,
            data_type: &NSPasteboardType,
        ) {
            self.serve(item, data_type);
        }

        #[unsafe(method(pasteboardFinishedWithDataProvider:))]
        fn finished_with_provider(&self, _pasteboard: &NSPasteboard) {
            // The pasteboard released this item: drop our reference too.
            let transaction = self.ivars().transaction;
            PROVIDERS.with(|providers| {
                providers.borrow_mut().remove(&transaction);
            });
        }
    }
);

impl SmartPasteProvider {
    fn new(ivars: ProviderIvars) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ivars);
        // SAFETY: NSObject's designated initializer on a freshly allocated
        // instance with initialized ivars.
        unsafe { msg_send![super(this), init] }
    }

    fn serve(&self, item: &NSPasteboardItem, data_type: &NSPasteboardType) {
        let ivars = self.ivars();
        if !data_type.isEqualToString(plain_text_type()) {
            return;
        }
        (ivars.sink)(ReadEvent {
            transaction: ivars.transaction,
            at: ivars.clock.now(),
        });
        // The full transcript is served on every request (AC-008).
        let _ = item.setString_forType(&NSString::from_str(&ivars.transcript), data_type);
    }
}

thread_local! {
    // Providers of published items, owned by the main thread until their
    // transaction is released or the pasteboard finishes with them.
    static PROVIDERS: RefCell<HashMap<TransactionId, Retained<SmartPasteProvider>>> =
        RefCell::new(HashMap::new());
}

fn plain_text_type() -> &'static NSPasteboardType {
    // SAFETY: NSPasteboardTypeString is an immutable AppKit constant.
    unsafe { NSPasteboardTypeString }
}

/// Returns the named pasteboard, or the general one for `None`.
pub(super) fn pasteboard_named(name: Option<&str>) -> Retained<NSPasteboard> {
    match name {
        Some(name) => NSPasteboard::pasteboardWithName(&NSString::from_str(name)),
        None => NSPasteboard::generalPasteboard(),
    }
}

/// Production [`PasteboardPort`] for the general (or a named) pasteboard.
pub struct MacPasteboard {
    pasteboard_name: Option<String>,
    helper: WarmHelper,
    sink: ReadSink,
    clock: Arc<dyn Clock>,
}

impl MacPasteboard {
    /// Creates the adapter. `pasteboard_name` is `None` for the general
    /// pasteboard; tests pass an isolated named pasteboard. Reads served by
    /// published items are reported to `sink`, stamped with `clock`. No
    /// helper process starts before the first capture is prepared.
    pub fn new(
        pasteboard_name: Option<String>,
        helper: HelperCommand,
        sink: ReadSink,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            pasteboard_name,
            helper: WarmHelper::new(helper),
            sink,
            clock,
        }
    }

    fn on_main<R: Send + 'static>(
        &self,
        work: impl FnOnce(&NSPasteboard) -> R + Send + 'static,
    ) -> Result<R, PasteboardError> {
        let name = self.pasteboard_name.clone();
        run_on_main(
            move || work(&pasteboard_named(name.as_deref())),
            MAIN_THREAD_TIMEOUT,
        )
        .map_err(|_| PasteboardError::MainThreadUnavailable)
    }
}

/// Empties the pasteboard for content that must stay on this Mac: other
/// devices never receive it through Universal Clipboard.
fn prepare_current_host_only(pasteboard: &NSPasteboard) {
    pasteboard.prepareForNewContentsWithOptions(NSPasteboardContentsOptions::CurrentHostOnly);
}

/// Adds Handy's privacy markers as eager empty data.
fn add_privacy_markers(item: &NSPasteboardItem) {
    for marker in HANDY_PRIVACY_MARKERS {
        item.setData_forType(&NSData::new(), &NSString::from_str(marker));
    }
}

fn write_item(pasteboard: &NSPasteboard, item: Retained<NSPasteboardItem>) -> bool {
    let item = ProtocolObject::<dyn NSPasteboardWriting>::from_retained(item);
    pasteboard.writeObjects(&NSArray::from_retained_slice(&[item]))
}

fn publish_item(
    pasteboard: &NSPasteboard,
    provider: &SmartPasteProvider,
) -> Result<i64, PasteboardError> {
    prepare_current_host_only(pasteboard);
    let item = NSPasteboardItem::new();
    let provided = NSArray::from_slice(&[plain_text_type()]);
    if !item.setDataProvider_forTypes(ProtocolObject::from_ref(provider), &provided) {
        return Err(PasteboardError::WriteFailed);
    }
    add_privacy_markers(&item);
    if !write_item(pasteboard, item) {
        return Err(PasteboardError::WriteFailed);
    }
    Ok(pasteboard.changeCount() as i64)
}

/// Restores the saved items as current-host-only content. Restoring is a
/// new local copy: without the option, Universal Clipboard would send the
/// previous content to the user's other devices again, possibly against the
/// original owner's choice.
fn write_saved_items(
    pasteboard: &NSPasteboard,
    snapshot: &PasteboardSnapshot,
) -> Result<(), PasteboardError> {
    prepare_current_host_only(pasteboard);
    let items: Vec<_> = snapshot
        .items()
        .iter()
        .filter(|item| !item.types.is_empty())
        .map(|saved| {
            let item = NSPasteboardItem::new();
            for saved_type in &saved.types {
                item.setData_forType(
                    &NSData::with_bytes(&saved_type.data),
                    &NSString::from_str(&saved_type.uti),
                );
            }
            ProtocolObject::<dyn NSPasteboardWriting>::from_retained(item)
        })
        .collect();
    if items.is_empty() || pasteboard.writeObjects(&NSArray::from_retained_slice(&items)) {
        Ok(())
    } else {
        Err(PasteboardError::WriteFailed)
    }
}

fn write_plain(pasteboard: &NSPasteboard, text: &str) -> Result<(), PasteboardError> {
    pasteboard.clearContents();
    if pasteboard.setString_forType(&NSString::from_str(text), plain_text_type()) {
        Ok(())
    } else {
        Err(PasteboardError::WriteFailed)
    }
}

/// Eager transcript marked like the on-demand item it replaces.
fn write_private(pasteboard: &NSPasteboard, text: &str) -> Result<(), PasteboardError> {
    prepare_current_host_only(pasteboard);
    let item = NSPasteboardItem::new();
    let is_set = item.setString_forType(&NSString::from_str(text), plain_text_type());
    add_privacy_markers(&item);
    if is_set && write_item(pasteboard, item) {
        Ok(())
    } else {
        Err(PasteboardError::WriteFailed)
    }
}

fn write_transcript(
    pasteboard: &NSPasteboard,
    text: &str,
    marking: TranscriptMarking,
) -> Result<(), PasteboardError> {
    match marking {
        TranscriptMarking::Private => write_private(pasteboard, text),
        TranscriptMarking::Plain => write_plain(pasteboard, text),
    }
}

fn is_owned(pasteboard: &NSPasteboard, publication_count: i64) -> bool {
    pasteboard.changeCount() as i64 == publication_count
}

impl PasteboardPort for MacPasteboard {
    fn change_count(&mut self) -> Result<i64, PasteboardError> {
        self.on_main(|pasteboard| pasteboard.changeCount() as i64)
    }

    fn prepare_capture(&mut self, budget: SnapshotBudget) {
        self.helper.prepare(budget, self.clock.as_ref());
    }

    fn capture_snapshot(
        &mut self,
        budget: SnapshotBudget,
    ) -> Result<PasteboardSnapshot, PasteboardError> {
        self.helper
            .capture(budget, self.clock.as_ref())
            .map_err(PasteboardError::SnapshotFailed)
    }

    fn publish_lazy_transcript(
        &mut self,
        transaction: TransactionId,
        transcript: &Transcript,
        expected_source_count: i64,
    ) -> Result<i64, PasteboardError> {
        let ivars = ProviderIvars {
            transaction,
            transcript: transcript.as_str().to_string(),
            sink: Arc::clone(&self.sink),
            clock: Arc::clone(&self.clock),
        };
        self.on_main(move |pasteboard| {
            // The source guard sits right before the clear/write in the same
            // main-thread operation, narrowing the race with another copy.
            if pasteboard.changeCount() as i64 != expected_source_count {
                return Err(PasteboardError::SourceChanged);
            }
            let provider = SmartPasteProvider::new(ivars);
            let count = publish_item(pasteboard, &provider)?;
            PROVIDERS.with(|providers| {
                providers.borrow_mut().insert(transaction, provider);
            });
            Ok(count)
        })?
    }

    fn restore_snapshot(
        &mut self,
        snapshot: Arc<PasteboardSnapshot>,
        publication_count: i64,
    ) -> Result<GuardedWrite, PasteboardError> {
        self.on_main(move |pasteboard| {
            if !is_owned(pasteboard, publication_count) {
                return Ok(GuardedWrite::OwnershipChanged);
            }
            write_saved_items(pasteboard, &snapshot).map(|()| GuardedWrite::Written)
        })?
    }

    fn materialize_transcript(
        &mut self,
        transcript: &Transcript,
        publication_count: i64,
        marking: TranscriptMarking,
    ) -> Result<GuardedWrite, PasteboardError> {
        let text = transcript.as_str().to_string();
        self.on_main(move |pasteboard| {
            if !is_owned(pasteboard, publication_count) {
                return Ok(GuardedWrite::OwnershipChanged);
            }
            write_transcript(pasteboard, &text, marking).map(|()| GuardedWrite::Written)
        })?
    }

    fn write_plain_text(&mut self, transcript: &Transcript) -> Result<(), PasteboardError> {
        let text = transcript.as_str().to_string();
        self.on_main(move |pasteboard| write_plain(pasteboard, &text))?
    }

    fn release_transaction(&mut self, transaction: TransactionId) {
        // Best effort: a stuck main thread keeps the provider a bit longer,
        // and `pasteboardFinishedWithDataProvider:` releases it as well.
        let _ = self.on_main(move |_| {
            PROVIDERS.with(|providers| {
                providers.borrow_mut().remove(&transaction);
            });
        });
    }
}
