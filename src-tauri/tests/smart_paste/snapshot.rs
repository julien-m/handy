//! Bounded snapshot accounting (AC-008) and helper stream framing.

use std::io::Cursor;
use std::time::Duration;

use handy_app_lib::smart_paste::snapshot::{
    PasteboardSnapshot, SkipCause, SnapshotBudget, SnapshotBuilder, MAX_SNAPSHOT_BYTES,
    PLAIN_TEXT_TYPE,
};
use handy_app_lib::smart_paste::snapshot_protocol::{
    read_frame, read_header, write_frame, write_header, Frame, ProtocolError,
};

// @spec FR-005: Bounded multi-type save — .specs/features/007-smart-paste/spec.md#fr-005

fn budget(max_bytes: usize) -> SnapshotBudget {
    SnapshotBudget {
        max_bytes,
        deadline: Duration::from_millis(200),
    }
}

fn inventory(types: &[&[&str]]) -> Vec<Vec<String>> {
    types
        .iter()
        .map(|item| item.iter().map(|uti| uti.to_string()).collect())
        .collect()
}

fn uti_list(snapshot: &PasteboardSnapshot) -> Vec<Vec<String>> {
    snapshot
        .items()
        .iter()
        .map(|item| item.types.iter().map(|saved| saved.uti.clone()).collect())
        .collect()
}

#[test]
fn spec_bounds_are_50_mb_and_200_ms() {
    let defaults = SnapshotBudget::default();
    assert_eq!(defaults.max_bytes, 50_000_000);
    assert_eq!(defaults.deadline, Duration::from_millis(200));
}

#[test]
fn items_and_types_keep_their_order_and_bytes() {
    let types = inventory(&[&["public.utf8-plain-text", "public.rtf"], &["public.png"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 7, &types);
    builder.accept(0, "public.utf8-plain-text", b"hi".to_vec());
    builder.accept(0, "public.rtf", b"{\\rtf}".to_vec());
    builder.accept(1, "public.png", vec![0x89, 0x50]);
    let snapshot = builder.finish();
    assert_eq!(uti_list(&snapshot), types);
    assert_eq!(&*snapshot.items()[0].types[1].data, b"{\\rtf}");
    assert_eq!(snapshot.saved_bytes(), 10);
    assert_eq!(snapshot.source_change_count(), 7);
}

#[test]
fn byte_budget_boundary_is_exact_at_50_mb() {
    let types = inventory(&[&["a", "b", "c"]]);
    let mut builder = SnapshotBuilder::new(SnapshotBudget::default(), 1, &types);
    builder.accept(0, "a", vec![0; MAX_SNAPSHOT_BYTES - 1]);
    builder.accept(0, "b", vec![0; 1]);
    builder.accept(0, "c", vec![0; 1]);
    let snapshot = builder.finish();
    assert_eq!(snapshot.saved_bytes(), MAX_SNAPSHOT_BYTES);
    assert_eq!(snapshot.skipped().len(), 1);
    assert_eq!(snapshot.skipped()[0].uti, "c");
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Oversize);
}

#[test]
fn oversize_type_is_skipped_but_later_small_types_are_kept() {
    let types = inventory(&[&["big", "small"]]);
    let mut builder = SnapshotBuilder::new(budget(10), 1, &types);
    builder.accept(0, "big", vec![0; 11]);
    builder.accept(0, "small", vec![1; 10]);
    let snapshot = builder.finish();
    assert_eq!(uti_list(&snapshot), vec![vec!["small".to_string()]]);
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Oversize);
}

#[test]
fn types_not_received_by_the_deadline_are_timed_out() {
    let types = inventory(&[&["fast", "slow"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, "fast", vec![1]);
    let snapshot = builder.finish();
    assert_eq!(snapshot.skipped()[0].uti, "slow");
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Timeout);
}

#[test]
fn uninventoried_or_repeated_types_are_ignored() {
    let types = inventory(&[&["a"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, "a", vec![1]);
    builder.accept(0, "a", vec![2, 2]);
    builder.accept(0, "injected", vec![3]);
    builder.accept(5, "a", vec![4]);
    let snapshot = builder.finish();
    assert_eq!(snapshot.saved_bytes(), 1);
    assert_eq!(snapshot.saved_type_count(), 1);
}

#[test]
fn empty_pasteboard_differs_from_a_nonempty_one_with_nothing_saved() {
    let empty = SnapshotBuilder::new(budget(100), 1, &[]).finish();
    assert!(empty.is_originally_empty());
    assert!(empty.is_restorable());
    let mut builder = SnapshotBuilder::new(budget(100), 1, &inventory(&[&["x"]]));
    builder.skip(0, "x", SkipCause::Unreadable);
    let unsaved = builder.finish();
    assert!(!unsaved.is_originally_empty());
    assert!(!unsaved.is_restorable());
}

#[test]
fn own_transcript_snapshot_is_a_single_plain_text_item() {
    let snapshot = PasteboardSnapshot::from_plain_text("dictée", 9);
    assert_eq!(uti_list(&snapshot), vec![vec![PLAIN_TEXT_TYPE.to_string()]]);
    assert_eq!(&*snapshot.items()[0].types[0].data, "dictée".as_bytes());
}

#[test]
fn snapshot_debug_output_hides_clipboard_bytes() {
    let snapshot = PasteboardSnapshot::from_plain_text("secret copied text", 1);
    let printed = format!("{snapshot:?}");
    assert!(!printed.contains("secret"));
}

fn encoded(frames: &[Frame]) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    write_header(&mut bytes, &[7; 16])?;
    for frame in frames {
        write_frame(&mut bytes, frame)?;
    }
    Ok(bytes)
}

#[test]
fn frames_round_trip_through_the_stream() -> Result<(), Box<dyn std::error::Error>> {
    let frames = vec![
        Frame::Begin {
            change_count: 41,
            inventory: inventory(&[&["public.rtf", "public.png"]]),
        },
        Frame::Data {
            item: 0,
            type_index: 0,
            data: b"{\\rtf1}".to_vec(),
        },
        Frame::Skipped {
            item: 0,
            type_index: 1,
            cause: SkipCause::Unreadable,
        },
        Frame::End { change_count: 41 },
    ];
    let mut stream = Cursor::new(encoded(&frames)?);
    read_header(&mut stream, &[7; 16]).map_err(|error| format!("{error:?}"))?;
    let mut decoded = Vec::new();
    while let Some(frame) = read_frame(&mut stream, 1024).map_err(|error| format!("{error:?}"))? {
        decoded.push(frame);
    }
    assert_eq!(decoded, frames);
    Ok(())
}

#[test]
fn a_response_to_another_request_is_rejected() -> Result<(), std::io::Error> {
    let mut stream = Cursor::new(encoded(&[])?);
    assert_eq!(
        read_header(&mut stream, &[8; 16]),
        Err(ProtocolError::NonceMismatch)
    );
    let mut garbage = Cursor::new(b"NOPE!".to_vec());
    assert_eq!(
        read_header(&mut garbage, &[7; 16]),
        Err(ProtocolError::BadHeader)
    );
    Ok(())
}

#[test]
fn data_frames_beyond_the_remaining_budget_are_refused() -> Result<(), std::io::Error> {
    let frame = Frame::Data {
        item: 0,
        type_index: 0,
        data: vec![0; 11],
    };
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &frame)?;
    assert_eq!(
        read_frame(&mut Cursor::new(bytes), 10),
        Err(ProtocolError::DataTooLarge)
    );
    Ok(())
}

#[test]
fn truncated_or_unknown_frames_are_malformed() {
    assert_eq!(
        read_frame(&mut Cursor::new(b"X".to_vec()), 10),
        Err(ProtocolError::Malformed)
    );
    assert!(matches!(
        read_frame(&mut Cursor::new(b"E\x01".to_vec()), 10),
        Err(ProtocolError::Io(_))
    ));
}
