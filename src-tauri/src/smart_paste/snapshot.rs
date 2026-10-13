//! Bounded multi-type snapshot of the previous pasteboard content (AC-008).
//!
//! The snapshot keeps every item in order with every saved type as raw
//! bytes, without decoding or normalizing anything. Saving is bounded to
//! 50 MB in total and 200 ms: a type that cannot be saved within those bounds
//! is skipped and listed by type name only, and "restored exactly" then
//! applies to the saved types. Bytes never reach a log or the disk.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use super::snapshot_protocol::ProtocolError;

// @spec FR-005: Bounded multi-type save — .specs/features/007-smart-paste/spec.md#fr-005

/// Maximum total bytes saved from the previous clipboard (AC-008).
pub const MAX_SNAPSHOT_BYTES: usize = 50_000_000;
/// Maximum time to save the previous clipboard, helper startup included
/// (AC-008).
pub const SNAPSHOT_DEADLINE: Duration = Duration::from_millis(200);
/// Pasteboard type of plain text (`NSPasteboardTypeString`).
pub const PLAIN_TEXT_TYPE: &str = "public.utf8-plain-text";

/// Bounds applied to one snapshot capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotBudget {
    /// Maximum total saved bytes.
    pub max_bytes: usize,
    /// Maximum capture time.
    pub deadline: Duration,
}

impl Default for SnapshotBudget {
    fn default() -> Self {
        Self {
            max_bytes: MAX_SNAPSHOT_BYTES,
            deadline: SNAPSHOT_DEADLINE,
        }
    }
}

/// Why no usable snapshot was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotError {
    /// The helper could not be started.
    Spawn,
    /// The deadline passed before the inventory arrived.
    NoInventory,
    /// The stream was malformed, unauthenticated or incomplete.
    Protocol(ProtocolError),
    /// The pasteboard changed while it was being read.
    SourceChanged,
    /// The pasteboard was not empty but no type could be saved; starting
    /// would lose it, so the transaction refuses to start.
    NothingSaved,
}

/// Why a type was not saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipCause {
    /// Saving it would exceed the byte budget.
    Oversize,
    /// The capture deadline passed before its data arrived.
    Timeout,
    /// Its data could not be read (for example a failing promise).
    Unreadable,
}

/// A type that was not saved, identified by name only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedType {
    /// Index of the item in pasteboard order.
    pub item_index: usize,
    /// Uniform type identifier.
    pub uti: String,
    /// Why it was skipped.
    pub cause: SkipCause,
}

/// One saved type with its raw bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct SavedType {
    /// Uniform type identifier.
    pub uti: String,
    /// Raw data, shared so restoration can move it to the UI thread cheaply.
    pub data: Arc<[u8]>,
}

impl fmt::Debug for SavedType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Clipboard bytes are user content and must never reach a log.
        write!(
            formatter,
            "SavedType({}, {} bytes)",
            self.uti,
            self.data.len()
        )
    }
}

/// One pasteboard item with its saved types in their original order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedItem {
    /// Saved types of the item.
    pub types: Vec<SavedType>,
}

/// Saved previous pasteboard content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteboardSnapshot {
    item_count: usize,
    items: Vec<SavedItem>,
    skipped: Vec<SkippedType>,
    saved_bytes: usize,
    source_change_count: i64,
}

impl PasteboardSnapshot {
    /// Snapshot of a pasteboard that holds only `text` as plain text, used
    /// when the content is Handy's own earlier transcript.
    pub fn from_plain_text(text: &str, source_change_count: i64) -> Self {
        let saved = SavedType {
            uti: PLAIN_TEXT_TYPE.to_string(),
            data: Arc::from(text.as_bytes()),
        };
        Self {
            item_count: 1,
            saved_bytes: text.len(),
            items: vec![SavedItem { types: vec![saved] }],
            skipped: Vec::new(),
            source_change_count,
        }
    }

    /// Whether the pasteboard held no item at all.
    pub fn is_originally_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Number of saved types across items.
    pub fn saved_type_count(&self) -> usize {
        self.items.iter().map(|item| item.types.len()).sum()
    }

    /// Whether restoring it is meaningful: an empty pasteboard can be
    /// cleared, but a nonempty one with no saved type must not be treated as
    /// empty, so the transaction refuses to start instead.
    pub fn is_restorable(&self) -> bool {
        self.is_originally_empty() || self.saved_type_count() > 0
    }

    /// Saved items in pasteboard order.
    pub fn items(&self) -> &[SavedItem] {
        &self.items
    }

    /// Types that were not saved.
    pub fn skipped(&self) -> &[SkippedType] {
        &self.skipped
    }

    /// Total saved bytes.
    pub fn saved_bytes(&self) -> usize {
        self.saved_bytes
    }

    /// Pasteboard `changeCount` observed when the capture started.
    pub fn source_change_count(&self) -> i64 {
        self.source_change_count
    }
}

/// Incremental, budget-enforcing builder fed by the capture helper.
#[derive(Debug)]
pub struct SnapshotBuilder {
    budget: SnapshotBudget,
    item_count: usize,
    items: Vec<SavedItem>,
    skipped: Vec<SkippedType>,
    pending: BTreeSet<(usize, String)>,
    saved_bytes: usize,
    source_change_count: i64,
}

impl SnapshotBuilder {
    /// Starts a snapshot from the pasteboard inventory (types per item).
    pub fn new(
        budget: SnapshotBudget,
        source_change_count: i64,
        inventory: &[Vec<String>],
    ) -> Self {
        let pending = inventory
            .iter()
            .enumerate()
            .flat_map(|(index, types)| types.iter().map(move |uti| (index, uti.clone())))
            .collect();
        Self {
            budget,
            item_count: inventory.len(),
            items: vec![SavedItem::default(); inventory.len()],
            skipped: Vec::new(),
            pending,
            saved_bytes: 0,
            source_change_count,
        }
    }

    /// Bytes still allowed by the budget.
    pub fn remaining_bytes(&self) -> usize {
        self.budget.max_bytes - self.saved_bytes
    }

    /// Accepts the data of an inventoried type, or skips it as oversize when
    /// it would exceed the total budget. Unknown or repeated types are
    /// ignored so a malformed helper cannot inject content.
    pub fn accept(&mut self, item_index: usize, uti: &str, data: Vec<u8>) {
        if !self.pending.remove(&(item_index, uti.to_string())) {
            return;
        }
        if data.len() > self.remaining_bytes() {
            self.push_skipped(item_index, uti, SkipCause::Oversize);
            return;
        }
        self.saved_bytes += data.len();
        if let Some(item) = self.items.get_mut(item_index) {
            item.types.push(SavedType {
                uti: uti.to_string(),
                data: Arc::from(data),
            });
        }
    }

    /// Records an inventoried type as skipped.
    pub fn skip(&mut self, item_index: usize, uti: &str, cause: SkipCause) {
        if self.pending.remove(&(item_index, uti.to_string())) {
            self.push_skipped(item_index, uti, cause);
        }
    }

    fn push_skipped(&mut self, item_index: usize, uti: &str, cause: SkipCause) {
        self.skipped.push(SkippedType {
            item_index,
            uti: uti.to_string(),
            cause,
        });
    }

    /// Finishes the snapshot; types never received count as timed out.
    pub fn finish(mut self) -> PasteboardSnapshot {
        for (item_index, uti) in std::mem::take(&mut self.pending) {
            self.push_skipped(item_index, &uti, SkipCause::Timeout);
        }
        PasteboardSnapshot {
            item_count: self.item_count,
            items: self.items,
            skipped: self.skipped,
            saved_bytes: self.saved_bytes,
            source_change_count: self.source_change_count,
        }
    }
}
