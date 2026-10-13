//! Helper-side pasteboard reader: streams the inventory, then each type's
//! data or skip reason, so the parent can enforce the snapshot bounds.
//!
//! Runs only inside the read-only helper process; it never writes the
//! pasteboard.

use std::io::Write;

use objc2::rc::Retained;
use objc2_app_kit::{NSPasteboardItem, NSPasteboardType};

use super::macos_pasteboard::pasteboard_named;
use super::snapshot::SkipCause;
use super::snapshot_protocol::{write_frame, write_header, Frame, Nonce, MAX_UTI_BYTES};

type Inventory = Vec<Vec<Retained<NSPasteboardType>>>;

/// Writes the helper's snapshot stream for the general (or a named)
/// pasteboard: inventory first, then each type's data or skip reason,
/// flushed per frame so the parent keeps completed types at its deadline.
pub(super) fn write_snapshot_stream(
    writer: &mut impl Write,
    nonce: &Nonce,
    max_bytes: usize,
    pasteboard_name: Option<&str>,
) -> std::io::Result<()> {
    // The header goes out first so the parent can tell a slow start from a
    // slow pasteboard; the source count is read before the inventory.
    write_header(writer, nonce)?;
    writer.flush()?;
    let pasteboard = pasteboard_named(pasteboard_name);
    let change_count = pasteboard.changeCount() as i64;
    let items = pasteboard
        .pasteboardItems()
        .map(|items| items.to_vec())
        .unwrap_or_default();
    let inventory = inventory_of(&items);
    write_frame(writer, &begin_frame(change_count, &inventory))?;
    writer.flush()?;
    write_type_frames(writer, &items, &inventory, max_bytes)?;
    let change_count = pasteboard.changeCount() as i64;
    write_frame(writer, &Frame::End { change_count })?;
    writer.flush()
}

/// Lists each item's types; names longer than the protocol limit are left
/// out (no real type identifier comes close to it).
fn inventory_of(items: &[Retained<NSPasteboardItem>]) -> Inventory {
    items
        .iter()
        .map(|item| {
            let types = item.types().to_vec().into_iter();
            types.filter(|uti| uti.len() <= MAX_UTI_BYTES).collect()
        })
        .collect()
}

fn begin_frame(change_count: i64, inventory: &Inventory) -> Frame {
    let names = inventory
        .iter()
        .map(|types| types.iter().map(|uti| uti.to_string()).collect())
        .collect();
    Frame::Begin {
        change_count,
        inventory: names,
    }
}

fn write_type_frames(
    writer: &mut impl Write,
    items: &[Retained<NSPasteboardItem>],
    inventory: &Inventory,
    max_bytes: usize,
) -> std::io::Result<()> {
    let mut remaining = max_bytes;
    for (item_index, (item, types)) in items.iter().zip(inventory).enumerate() {
        for (type_index, uti) in types.iter().enumerate() {
            let frame = type_frame(item, uti, (item_index, type_index), &mut remaining);
            write_frame(writer, &frame)?;
            writer.flush()?;
        }
    }
    Ok(())
}

fn type_frame(
    item: &NSPasteboardItem,
    uti: &NSPasteboardType,
    (item_index, type_index): (usize, usize),
    remaining: &mut usize,
) -> Frame {
    let skipped = |cause| Frame::Skipped {
        item: item_index,
        type_index,
        cause,
    };
    // dataForType: may synchronously wait on another application's promise;
    // the parent kills this process at its deadline. Oversize data is
    // measured without copying it.
    match item.dataForType(uti) {
        Some(data) if data.len() <= *remaining => {
            *remaining -= data.len();
            Frame::Data {
                item: item_index,
                type_index,
                data: data.to_vec(),
            }
        }
        Some(_) => skipped(SkipCause::Oversize),
        None => skipped(SkipCause::Unreadable),
    }
}
