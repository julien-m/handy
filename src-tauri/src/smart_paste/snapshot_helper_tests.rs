//! Collector behavior of the snapshot helper stream: complete, partial and
//! malformed streams.

use super::*;
use crate::smart_paste::clock::ManualClock;

#[test]
fn nonce_round_trips_through_its_hex_argument() {
    let nonce = new_nonce();
    assert_eq!(parse_nonce(&hex(&nonce)), Some(nonce));
    assert_eq!(parse_nonce("zz"), None);
}

#[test]
fn two_requests_get_different_nonces() {
    assert_ne!(new_nonce(), new_nonce());
}

fn begin(types: &[&str]) -> FrameResult {
    Ok(Frame::Begin {
        change_count: 4,
        inventory: vec![types.iter().map(|uti| uti.to_string()).collect()],
    })
}

fn data(type_index: usize) -> FrameResult {
    Ok(Frame::Data {
        item: 0,
        type_index,
        data: vec![1, 2],
    })
}

/// Collects `frames`; the channel then ends as if the helper stopped,
/// unless `keep_open` holds the sender (the deadline then decides).
fn collect(frames: Vec<FrameResult>, keep_open: bool) -> Result<PasteboardSnapshot, SnapshotError> {
    let (sender, receiver) = mpsc::channel();
    frames.into_iter().for_each(|frame| {
        let _ = sender.send(frame);
    });
    let open = keep_open.then_some(sender);
    let clock = ManualClock::new();
    let result = collect_frames(&receiver, &clock, clock.now(), SnapshotBudget::default());
    drop(open);
    result
}

fn skip_causes(snapshot: &PasteboardSnapshot) -> Vec<(String, SkipCause)> {
    let skipped = snapshot.skipped().iter();
    skipped.map(|s| (s.uti.clone(), s.cause)).collect()
}

#[test]
fn a_helper_stopping_mid_stream_keeps_completed_types() -> Result<(), SnapshotError> {
    let snapshot = collect(vec![begin(&["a", "b"]), data(0)], false)?;
    assert_eq!(snapshot.saved_type_count(), 1);
    assert_eq!(
        skip_causes(&snapshot),
        vec![("b".into(), SkipCause::Unreadable)]
    );
    let truncated = Err(ProtocolError::Io(std::io::ErrorKind::UnexpectedEof));
    let snapshot = collect(vec![begin(&["a", "b"]), data(1), truncated], false)?;
    assert_eq!(
        skip_causes(&snapshot),
        vec![("a".into(), SkipCause::Unreadable)]
    );
    Ok(())
}

#[test]
fn the_deadline_keeps_completed_types_and_times_out_the_rest() -> Result<(), SnapshotError> {
    let snapshot = collect(vec![begin(&["a", "b"]), data(0)], true)?;
    assert_eq!(
        skip_causes(&snapshot),
        vec![("b".into(), SkipCause::Timeout)]
    );
    Ok(())
}

#[test]
fn omitted_counts_reach_the_snapshot() -> Result<(), SnapshotError> {
    let omitted = Ok(Frame::Omitted { items: 3, types: 5 });
    let end = Ok(Frame::End { change_count: 4 });
    let snapshot = collect(vec![begin(&["a"]), omitted, data(0), end], false)?;
    assert_eq!(snapshot.omitted(), OmittedTypes { items: 3, types: 5 });
    Ok(())
}

#[test]
fn a_second_inventory_or_an_end_without_inventory_is_malformed() {
    let malformed = Err(SnapshotError::Protocol(ProtocolError::Malformed));
    let twice = vec![begin(&["a"]), begin(&["a"])];
    assert_eq!(collect(twice, false), malformed);
    let early_end = vec![Ok(Frame::End { change_count: 4 })];
    assert_eq!(collect(early_end, false), malformed);
    assert_eq!(collect(vec![data(0)], false), malformed);
}

#[test]
fn a_stream_without_inventory_or_with_a_changed_source_is_rejected() {
    assert_eq!(collect(Vec::new(), false), Err(SnapshotError::NoInventory));
    let changed = vec![begin(&[]), Ok(Frame::End { change_count: 5 })];
    assert_eq!(collect(changed, false), Err(SnapshotError::SourceChanged));
    let bad = vec![Err(ProtocolError::NonceMismatch)];
    assert_eq!(
        collect(bad, false),
        Err(SnapshotError::Protocol(ProtocolError::NonceMismatch))
    );
}
