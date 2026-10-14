//! Helper-side pasteboard reader: streams the inventory, then each type's
//! data or skip reason, so the parent can enforce the snapshot bounds.
//!
//! Runs only inside the read-only helper process; it never writes the
//! pasteboard.

use std::io::{Read, Write};
use std::time::Duration;

use objc2::rc::Retained;
use objc2_app_kit::{NSPasteboardItem, NSPasteboardType};

use super::macos_pasteboard::pasteboard_named;
use super::pasteboard_types::read_order;
use super::snapshot::{OmittedTypes, SkipCause};
use super::snapshot_protocol::{
    write_frame, write_header, Frame, Nonce, MAX_ITEMS, MAX_TYPES_PER_ITEM, MAX_UTI_BYTES,
};

/// Byte the parent writes on the helper's stdin to start reading.
pub(super) const START_TRIGGER: u8 = b'G';
/// An armed helper waits this long for its trigger, then exits without
/// reading anything, so no idle process outlives a long pause in dictation.
/// Five minutes keeps the next dictation warm through ordinary pauses; an
/// idle helper blocked in `poll` uses no CPU and little memory.
pub(super) const HELPER_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

type Inventory = Vec<Vec<Retained<NSPasteboardType>>>;

/// Writes the helper's snapshot stream for the general (or a named)
/// pasteboard, flushed per frame so the parent keeps completed types at its
/// deadline. The header goes out as soon as the helper runs; the pasteboard
/// is only read once `is_triggered` reports the parent's start trigger, so a
/// helper started ahead of time costs no startup inside the 200 ms budget.
/// Without the trigger the helper ends without reading anything.
pub(super) fn write_snapshot_stream(
    writer: &mut impl Write,
    is_triggered: impl FnOnce() -> bool,
    nonce: &Nonce,
    max_bytes: usize,
    pasteboard_name: Option<&str>,
) -> std::io::Result<()> {
    write_header(writer, nonce)?;
    writer.flush()?;
    if !is_triggered() {
        return Ok(());
    }
    // The source count is read before the inventory.
    let pasteboard = pasteboard_named(pasteboard_name);
    let change_count = pasteboard.changeCount() as i64;
    let items = pasteboard
        .pasteboardItems()
        .map(|items| items.to_vec())
        .unwrap_or_default();
    let (inventory, omitted) = inventory_of(&items);
    write_frame(writer, &begin_frame(change_count, &inventory))?;
    write_frame(writer, &omitted_frame(omitted))?;
    writer.flush()?;
    write_type_frames(writer, &items, &inventory, max_bytes)?;
    let change_count = pasteboard.changeCount() as i64;
    write_frame(writer, &Frame::End { change_count })?;
    writer.flush()
}

/// Lists each item's types within the structural limits and counts what is
/// left out: items beyond [`MAX_ITEMS`], types beyond [`MAX_TYPES_PER_ITEM`]
/// and names longer than [`MAX_UTI_BYTES`] (no real type comes close). A
/// larger pasteboard (for example a Finder copy of many files) is therefore
/// truncated, never rejected.
fn inventory_of(items: &[Retained<NSPasteboardItem>]) -> (Inventory, OmittedTypes) {
    let mut omitted = OmittedTypes {
        items: items.len().saturating_sub(MAX_ITEMS),
        types: 0,
    };
    let inventory = items
        .iter()
        .take(MAX_ITEMS)
        .map(|item| {
            let all = item.types().to_vec();
            let total = all.len();
            let kept: Vec<_> = all
                .into_iter()
                .filter(|uti| uti.len() <= MAX_UTI_BYTES)
                .take(MAX_TYPES_PER_ITEM)
                .collect();
            omitted.types += total - kept.len();
            kept
        })
        .collect();
    (inventory, omitted)
}

/// Waits up to [`HELPER_IDLE_TIMEOUT`] for [`START_TRIGGER`] on stdin.
/// End of input (the parent exited or gave up), another byte or the idle
/// timeout all mean "do not read".
pub(super) fn wait_for_trigger() -> bool {
    let mut descriptor = libc::pollfd {
        fd: libc::STDIN_FILENO,
        events: libc::POLLIN,
        revents: 0,
    };
    let timeout_ms = i32::try_from(HELPER_IDLE_TIMEOUT.as_millis()).unwrap_or(i32::MAX);
    // SAFETY: one valid, initialized pollfd for the duration of the call.
    let ready = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
    let mut start = [0_u8; 1];
    ready > 0 && matches!(std::io::stdin().read(&mut start), Ok(1)) && start[0] == START_TRIGGER
}

fn omitted_frame(omitted: OmittedTypes) -> Frame {
    Frame::Omitted {
        items: omitted.items,
        types: omitted.types,
    }
}

fn begin_frame(change_count: i64, inventory: &Inventory) -> Frame {
    Frame::Begin {
        change_count,
        inventory: names_of(inventory),
    }
}

fn names_of(inventory: &Inventory) -> Vec<Vec<String>> {
    inventory
        .iter()
        .map(|types| types.iter().map(|uti| uti.to_string()).collect())
        .collect()
}

/// Reads every declared type first and the derived ones last, so a slow
/// AppKit conversion can only cost derived types (see `read_order`).
fn write_type_frames(
    writer: &mut impl Write,
    items: &[Retained<NSPasteboardItem>],
    inventory: &Inventory,
    max_bytes: usize,
) -> std::io::Result<()> {
    let mut remaining = max_bytes;
    for (item_index, type_index) in read_order(&names_of(inventory)) {
        // `read_order` only yields inventoried pairs; the lookup keeps the
        // indexing panic-free rather than skipping real types.
        let (Some(item), Some(uti)) = (
            items.get(item_index),
            inventory
                .get(item_index)
                .and_then(|types| types.get(type_index)),
        ) else {
            continue;
        };
        let frame = type_frame(item, uti, (item_index, type_index), &mut remaining);
        write_frame(writer, &frame)?;
        writer.flush()?;
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
