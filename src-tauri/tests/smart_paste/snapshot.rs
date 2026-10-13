//! Bounded snapshot accounting (AC-008) and helper stream framing.

use std::io::Cursor;
use std::time::Duration;

use handy_app_lib::smart_paste::pasteboard_types::{CONCEALED_TYPE, TRANSIENT_TYPE};
use handy_app_lib::smart_paste::snapshot::{
    OmittedTypes, PasteboardSnapshot, SkipCause, SnapshotBudget, SnapshotBuilder,
    MAX_SNAPSHOT_BYTES,
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
    // Arrival order differs (derived types are read last); saved order does not.
    builder.accept(1, 0, vec![0x89, 0x50]);
    builder.accept(0, 1, b"{\\rtf}".to_vec());
    builder.accept(0, 0, b"hi".to_vec());
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
    builder.accept(0, 0, vec![0; MAX_SNAPSHOT_BYTES - 1]);
    builder.accept(0, 1, vec![0; 1]);
    builder.accept(0, 2, vec![0; 1]);
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
    builder.accept(0, 0, vec![0; 11]);
    builder.accept(0, 1, vec![1; 10]);
    let snapshot = builder.finish();
    assert_eq!(uti_list(&snapshot), vec![vec!["small".to_string()]]);
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Oversize);
}

#[test]
fn types_not_received_by_the_deadline_are_timed_out() {
    let types = inventory(&[&["fast", "slow"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, 0, vec![1]);
    let snapshot = builder.finish();
    assert_eq!(snapshot.skipped()[0].uti, "slow");
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Timeout);
}

#[test]
fn uninventoried_or_repeated_types_are_ignored() {
    let types = inventory(&[&["a"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, 0, vec![1]);
    builder.accept(0, 0, vec![2, 2]);
    builder.accept(0, 1, vec![3]);
    builder.accept(5, 0, vec![4]);
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
    builder.skip(0, 0, SkipCause::Unreadable);
    let unsaved = builder.finish();
    assert!(!unsaved.is_originally_empty());
    assert!(!unsaved.is_restorable());
}

#[test]
fn types_left_out_by_the_helper_are_counted_and_keep_the_pasteboard_nonempty() {
    let mut builder = SnapshotBuilder::new(budget(100), 1, &[]);
    builder.record_omitted(OmittedTypes { items: 2, types: 0 });
    let snapshot = builder.finish();
    assert!(!snapshot.is_originally_empty());
    assert_eq!(snapshot.omitted().items, 2);
}

#[test]
fn types_missing_after_the_helper_stopped_are_unreadable() {
    let types = inventory(&[&["a", "b"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, 0, vec![1]);
    let snapshot = builder.finish_with_missing_as(SkipCause::Unreadable);
    assert_eq!(snapshot.skipped()[0].cause, SkipCause::Unreadable);
}

fn saved_secret(marker: &str) -> PasteboardSnapshot {
    let types = inventory(&[&["public.utf8-plain-text", marker]]);
    let mut builder = SnapshotBuilder::new(budget(100), 3, &types);
    builder.accept(0, 0, b"hunter2".to_vec());
    builder.accept(0, 1, Vec::new());
    builder.finish()
}

#[test]
fn concealed_or_transient_content_is_withheld_and_its_bytes_dropped() {
    for marker in [CONCEALED_TYPE, TRANSIENT_TYPE] {
        let snapshot = saved_secret(marker).withhold_if_privacy_marked();
        assert!(snapshot.is_restoration_withheld());
        assert!(snapshot.is_restorable());
        assert_eq!(
            (snapshot.saved_type_count(), snapshot.saved_bytes()),
            (0, 0)
        );
        assert_eq!(snapshot.source_change_count(), 3);
    }
    let ordinary = saved_secret("public.rtf").withhold_if_privacy_marked();
    assert!(!ordinary.is_restoration_withheld());
    assert_eq!(ordinary.saved_type_count(), 2);
}

#[test]
fn handy_own_transcript_snapshot_is_withheld() {
    let snapshot = PasteboardSnapshot::withheld(9);
    assert!(snapshot.is_restoration_withheld() && snapshot.is_restorable());
    assert!(!snapshot.is_originally_empty());
}

#[test]
fn snapshot_debug_output_hides_clipboard_bytes() {
    let types = inventory(&[&["public.utf8-plain-text"]]);
    let mut builder = SnapshotBuilder::new(budget(100), 1, &types);
    builder.accept(0, 0, b"secret copied text".to_vec());
    let printed = format!("{:?}", builder.finish());
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
fn omitted_counts_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let omitted = Frame::Omitted {
        items: 12_000,
        types: 3,
    };
    let mut stream = Cursor::new(encoded(std::slice::from_ref(&omitted))?);
    read_header(&mut stream, &[7; 16]).map_err(|error| format!("{error:?}"))?;
    let decoded = read_frame(&mut stream, 0).map_err(|error| format!("{error:?}"))?;
    assert_eq!(decoded, Some(omitted));
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
