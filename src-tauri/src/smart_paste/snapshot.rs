//! Bounded multi-type snapshot of the previous pasteboard content (AC-008).
//!
//! The snapshot keeps every item in order with every saved type as raw
//! bytes, without decoding or normalizing anything. Saving is bounded to
//! 50 MB in total and 200 ms: a type that cannot be saved within those bounds
//! is skipped and listed by type name only, and "restored exactly" then
//! applies to the saved types. Bytes never reach a log or the disk.
//! Content marked concealed or transient is never restored: its snapshot
//! keeps no bytes and only withholds the restoration.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use super::pasteboard_types::withholds_restoration;
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

/// Pasteboard items and types left out of the inventory because they exceed
/// the protocol's structural limits (more than 10,000 items, more than 1,000
/// types in an item, or a type name longer than 1,024 bytes). They are
/// counted, never saved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OmittedTypes {
    /// Items beyond the item limit.
    pub items: usize,
    /// Types beyond the per-item limit or with an over-long name.
    pub types: usize,
}

/// Saved previous pasteboard content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteboardSnapshot {
    item_count: usize,
    items: Vec<SavedItem>,
    skipped: Vec<SkippedType>,
    omitted: OmittedTypes,
    saved_bytes: usize,
    source_change_count: i64,
    is_restoration_withheld: bool,
}

impl PasteboardSnapshot {
    /// Snapshot of content that must not be restored, without its bytes:
    /// used for Handy's own earlier transcript item, which carries the
    /// transient and concealed markers.
    pub fn withheld(source_change_count: i64) -> Self {
        Self {
            item_count: 1,
            items: Vec::new(),
            skipped: Vec::new(),
            omitted: OmittedTypes::default(),
            saved_bytes: 0,
            source_change_count,
            is_restoration_withheld: true,
        }
    }

    /// Drops the saved bytes and withholds restoration when any item carries
    /// a concealed or transient marker (decision 2026-10-13, see
    /// [`withholds_restoration`]); other snapshots are returned unchanged.
    pub fn withhold_if_privacy_marked(self) -> Self {
        if !self.is_privacy_marked() {
            return self;
        }
        Self {
            items: Vec::new(),
            skipped: Vec::new(),
            saved_bytes: 0,
            is_restoration_withheld: true,
            ..self
        }
    }

    fn is_privacy_marked(&self) -> bool {
        let saved = self.items.iter().flat_map(|item| &item.types);
        let saved = saved.map(|saved| saved.uti.as_str());
        let skipped = self.skipped.iter().map(|skipped| skipped.uti.as_str());
        saved.chain(skipped).any(withholds_restoration)
    }

    /// Whether the content must not be restored (concealed or transient).
    pub fn is_restoration_withheld(&self) -> bool {
        self.is_restoration_withheld
    }

    /// Whether the pasteboard held no item at all.
    pub fn is_originally_empty(&self) -> bool {
        self.item_count == 0 && self.omitted.items == 0
    }

    /// Number of saved types across items.
    pub fn saved_type_count(&self) -> usize {
        self.items.iter().map(|item| item.types.len()).sum()
    }

    /// Whether the transaction may start: an empty pasteboard can be cleared
    /// and withheld content is deliberately not restored, but a nonempty
    /// pasteboard with no saved type must not be treated as empty.
    pub fn is_restorable(&self) -> bool {
        self.is_restoration_withheld || self.is_originally_empty() || self.saved_type_count() > 0
    }

    /// Saved items in pasteboard order.
    pub fn items(&self) -> &[SavedItem] {
        &self.items
    }

    /// Inventoried types that were not saved.
    pub fn skipped(&self) -> &[SkippedType] {
        &self.skipped
    }

    /// Items and types beyond the structural limits, never inventoried.
    pub fn omitted(&self) -> OmittedTypes {
        self.omitted
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

/// State of one inventoried type while the capture runs.
#[derive(Debug)]
enum Slot {
    Pending,
    Saved(Arc<[u8]>),
    Skipped(SkipCause),
}

/// Incremental, budget-enforcing builder fed by the capture helper. Types
/// may arrive in any order; the snapshot keeps the inventory order.
#[derive(Debug)]
pub struct SnapshotBuilder {
    budget: SnapshotBudget,
    inventory: Vec<Vec<String>>,
    slots: Vec<Vec<Slot>>,
    omitted: OmittedTypes,
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
        let slots = inventory
            .iter()
            .map(|types| types.iter().map(|_| Slot::Pending).collect())
            .collect();
        Self {
            budget,
            inventory: inventory.to_vec(),
            slots,
            omitted: OmittedTypes::default(),
            saved_bytes: 0,
            source_change_count,
        }
    }

    /// Bytes still allowed by the budget.
    pub fn remaining_bytes(&self) -> usize {
        self.budget.max_bytes - self.saved_bytes
    }

    /// Records the items and types left out of the inventory.
    pub fn record_omitted(&mut self, omitted: OmittedTypes) {
        self.omitted = omitted;
    }

    fn pending_slot(&mut self, item: usize, type_index: usize) -> Option<&mut Slot> {
        let slot = self.slots.get_mut(item)?.get_mut(type_index)?;
        matches!(slot, Slot::Pending).then_some(slot)
    }

    /// Accepts the data of an inventoried type, or skips it as oversize when
    /// it would exceed the total budget. Unknown or repeated types are
    /// ignored so a malformed helper cannot inject content.
    pub fn accept(&mut self, item: usize, type_index: usize, data: Vec<u8>) {
        let remaining = self.remaining_bytes();
        let Some(slot) = self.pending_slot(item, type_index) else {
            return;
        };
        if data.len() > remaining {
            *slot = Slot::Skipped(SkipCause::Oversize);
            return;
        }
        let length = data.len();
        *slot = Slot::Saved(Arc::from(data));
        self.saved_bytes += length;
    }

    /// Records an inventoried type as skipped.
    pub fn skip(&mut self, item: usize, type_index: usize, cause: SkipCause) {
        if let Some(slot) = self.pending_slot(item, type_index) {
            *slot = Slot::Skipped(cause);
        }
    }

    /// Finishes the snapshot at the deadline; types never received count as
    /// timed out.
    pub fn finish(self) -> PasteboardSnapshot {
        self.finish_with_missing_as(SkipCause::Timeout)
    }

    /// Finishes the snapshot, recording every type never received with
    /// `missing` (for example unreadable when the helper stopped early).
    pub fn finish_with_missing_as(self, missing: SkipCause) -> PasteboardSnapshot {
        let mut items = Vec::with_capacity(self.inventory.len());
        let mut skipped = Vec::new();
        for (item_index, (types, slots)) in self.inventory.iter().zip(self.slots).enumerate() {
            let mut saved_item = SavedItem::default();
            for (uti, slot) in types.iter().zip(slots) {
                let uti = uti.clone();
                match slot {
                    Slot::Saved(data) => saved_item.types.push(SavedType { uti, data }),
                    Slot::Skipped(cause) => skipped.push(SkippedType::new(item_index, uti, cause)),
                    Slot::Pending => skipped.push(SkippedType::new(item_index, uti, missing)),
                }
            }
            items.push(saved_item);
        }
        PasteboardSnapshot {
            item_count: self.inventory.len(),
            items,
            skipped,
            omitted: self.omitted,
            saved_bytes: self.saved_bytes,
            source_change_count: self.source_change_count,
            is_restoration_withheld: false,
        }
    }
}

impl SkippedType {
    fn new(item_index: usize, uti: String, cause: SkipCause) -> Self {
        Self {
            item_index,
            uti,
            cause,
        }
    }
}
