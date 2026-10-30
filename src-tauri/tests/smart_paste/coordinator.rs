//! Clipboard paste through the production coordinator: Stories 1, 2 and 4,
//! successions and exit (AC-008, AC-009, AC-012, AC-013).

use std::time::Duration;

use handy_app_lib::smart_paste::clock::Clock;
use handy_app_lib::smart_paste::coordinator::SmartPasteCoordinator;
use handy_app_lib::smart_paste::ids::TransactionId;
use handy_app_lib::smart_paste::outcome::{InsertionPath, PasteOutcome, PasteOutcomeKind};
use handy_app_lib::smart_paste::pasteboard_port::{PasteboardError, ReadEvent};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::{CopyToClipboard, DontModify};
use handy_app_lib::smart_paste::snapshot::PLAIN_TEXT_TYPE;

use super::rig::{request, rich_items, settle_until, Content, Rig, ScriptedReads};

// @spec FR-006: Restore, copy, exit and succession — .specs/features/007-smart-paste/spec.md#fr-006
// @spec FR-008: Terminal outcomes and late reads — .specs/features/007-smart-paste/spec.md#fr-008

const TEXTEDIT: Option<&str> = Some("com.apple.TextEdit");

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

fn paste(rig: &Rig, offsets: &[i64]) -> (SmartPasteCoordinator, PasteOutcome, ScriptedReads) {
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(rig, offsets);
    let outcome = coordinator.insert(&request(1, "hello world", TEXTEDIT, DontModify), &mut reads);
    (coordinator, outcome, reads)
}

fn key_down(rig: &Rig) -> std::time::Instant {
    rig.keys
        .state()
        .chords
        .last()
        .copied()
        .unwrap_or_else(|| rig.clock.now())
}

#[test]
fn rich_clipboard_is_restored_exactly_300_ms_after_the_read() {
    let rig = Rig::new();
    let (mut coordinator, outcome, _) = paste(&rig, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(outcome.path, InsertionPath::ClipboardPaste);
    let down = key_down(&rig);
    settle_until(&rig.clock, &mut coordinator, down + ms(319));
    assert!(matches!(rig.pasteboard.state().content, Content::Lazy(_)));
    settle_until(&rig.clock, &mut coordinator, down + ms(320));
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
    assert_eq!(coordinator.pending_transaction(), None);
}

/// Reducer logic only: this fake reports every read, while AppKit reports
/// one per item (it caches the fulfilled data). Natively the quiet period
/// is therefore measured from the first read (known release risk R-1); see
/// `with_the_appkit_cache_quiet_is_measured_from_the_first_read`.
#[test]
fn probe_then_read_restores_after_the_second_read() {
    let rig = Rig::new();
    let (mut coordinator, _, mut reads) = paste(&rig, &[10, 160]);
    reads.drain_into(&mut coordinator);
    let down = key_down(&rig);
    settle_until(&rig.clock, &mut coordinator, down + ms(459));
    assert!(matches!(rig.pasteboard.state().content, Content::Lazy(_)));
    settle_until(&rig.clock, &mut coordinator, down + ms(460));
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
}

#[test]
fn read_during_the_chord_hold_counts_and_pre_key_reads_do_not() {
    let rig = Rig::new();
    let (_, outcome, _) = paste(&rig, &[-5, 50]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    let rig = Rig::new();
    let (_, pre_key_only, _) = paste(&rig, &[-5]);
    assert_eq!(pre_key_only.kind, PasteOutcomeKind::NotRead);
}

#[test]
fn no_read_is_declared_at_800_ms_and_never_restores() {
    let rig = Rig::new();
    let (mut coordinator, outcome, _) = paste(&rig, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    assert_eq!(rig.clock.now(), key_down(&rig) + ms(800));
    settle_until(
        &rig.clock,
        &mut coordinator,
        rig.clock.now() + Duration::from_secs(60),
    );
    assert!(matches!(rig.pasteboard.state().content, Content::Lazy(_)));
    coordinator.shutdown();
    let state = rig.pasteboard.state();
    assert_eq!(state.content, Content::Private("hello world".into()));
    assert!(!state.writes.contains(&"restore"));
}

#[test]
fn late_read_after_not_read_is_reported_once_without_restoring() {
    let rig = Rig::new();
    let (mut coordinator, outcome, _) = paste(&rig, &[]);
    let transaction = outcome.transaction_id.unwrap_or(TransactionId(0));
    for late in [100, 200] {
        let at = rig.clock.now() + ms(late);
        coordinator.handle_read(ReadEvent { transaction, at });
    }
    let diagnostics = rig.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].dictation_id, outcome.dictation_id);
    assert_eq!(coordinator.next_deadline(), None);
    assert!(!rig.pasteboard.state().writes.contains(&"restore"));
}

#[test]
fn empty_clipboard_is_empty_again_after_restoration() {
    let rig = Rig::new();
    rig.pasteboard.state().content = Content::Items(Vec::new());
    let (mut coordinator, _, _) = paste(&rig, &[20]);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    assert_eq!(rig.pasteboard.state().content, Content::Items(Vec::new()));
}

#[test]
fn a_copy_made_before_restoration_is_never_overwritten() {
    for newer in [
        Content::Plain("new text".into()),
        Content::Items(vec![vec![("public.png".into(), vec![1, 2, 3])]]),
    ] {
        let rig = Rig::new();
        let (mut coordinator, _, _) = paste(&rig, &[20]);
        rig.pasteboard.external_write(newer.clone());
        settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
        let state = rig.pasteboard.state();
        assert_eq!(state.content, newer);
        assert!(!state.writes.contains(&"restore"));
    }
}

#[test]
fn copy_mode_leaves_the_transcript_as_plain_text_instead_of_restoring() {
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(
        &request(1, "hello world", TEXTEDIT, CopyToClipboard),
        &mut reads,
    );
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    let state = rig.pasteboard.state();
    assert_eq!(state.content, Content::Plain("hello world".into()));
    assert!(!state.writes.contains(&"restore"));
}

#[test]
fn copy_mode_not_read_keeps_plain_transcript_without_reviving_the_snapshot() {
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[]);
    let outcome = coordinator.insert(
        &request(1, "hello world", TEXTEDIT, CopyToClipboard),
        &mut reads,
    );
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    coordinator.shutdown();
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
}

#[test]
fn failed_chord_is_terminal_and_restores_only_after_a_later_read() {
    let rig = Rig::new();
    rig.keys.state().chord_fails = true;
    let (mut coordinator, outcome, _) = paste(&rig, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
    let transaction = outcome.transaction_id.unwrap_or(TransactionId(0));
    coordinator.handle_read(ReadEvent {
        transaction,
        at: rig.clock.now() + ms(30),
    });
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(400));
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
}

#[test]
fn failed_chord_without_read_never_restores_nor_reports_late_reads() {
    let rig = Rig::new();
    rig.keys.state().chord_fails = true;
    let (mut coordinator, outcome, _) = paste(&rig, &[]);
    settle_until(
        &rig.clock,
        &mut coordinator,
        key_down(&rig) + Duration::from_secs(5),
    );
    let transaction = outcome.transaction_id.unwrap_or(TransactionId(0));
    coordinator.handle_read(ReadEvent {
        transaction,
        at: rig.clock.now(),
    });
    assert!(matches!(rig.pasteboard.state().content, Content::Lazy(_)));
    assert!(rig.diagnostics().is_empty());
}

#[test]
fn permission_missing_touches_nothing_even_in_copy_mode() {
    let mut rig = Rig::new();
    rig.ax.is_trusted = false;
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[]);
    let outcome = coordinator.insert(&request(1, "x", TEXTEDIT, CopyToClipboard), &mut reads);
    assert_eq!(outcome.kind, PasteOutcomeKind::PermissionMissing);
    let state = rig.pasteboard.state();
    assert_eq!((state.captures, state.writes.len()), (0, 0));
    assert!(rig.keys.state().chords.is_empty());
}

#[test]
fn unsavable_or_failing_snapshot_refuses_to_start_and_keeps_the_clipboard() {
    let configurations: [fn(&Rig); 2] = [
        |rig| rig.pasteboard.state().unsavable = true,
        |rig| rig.pasteboard.state().capture_error = Some(PasteboardError::WriteFailed),
    ];
    for configure in configurations {
        let rig = Rig::new();
        configure(&rig);
        let (_, outcome, _) = paste(&rig, &[]);
        assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
        assert_eq!(outcome.transaction_id, None);
        assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
        assert!(rig.keys.state().chords.is_empty());
    }
}

#[test]
fn source_change_between_snapshot_and_publication_aborts_the_start() {
    let rig = Rig::new();
    rig.pasteboard.state().publish_error = Some(PasteboardError::SourceChanged);
    let (_, outcome, _) = paste(&rig, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
    assert!(rig.keys.state().chords.is_empty());
}

#[test]
fn destination_lost_before_the_keystroke_fails_without_keys_or_restore() {
    let rig = Rig::new();
    rig.destination
        .answers
        .lock()
        .map(|mut answers| answers.extend([true, false]))
        .unwrap_or(());
    let (_, outcome, _) = paste(&rig, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
    assert!(rig.keys.state().chords.is_empty());
    let state = rig.pasteboard.state();
    assert_eq!(state.content, Content::Private("hello world".into()));
    assert!(!state.writes.contains(&"restore"));
}

#[test]
fn second_dictation_carries_the_original_snapshot_without_intermediate_restore() {
    let rig = Rig::new();
    let (mut coordinator, _, _) = paste(&rig, &[20]);
    let mut reads = ScriptedReads::new(&rig, &[20]);
    let second = coordinator.insert(&request(2, "second", TEXTEDIT, DontModify), &mut reads);
    assert_eq!(second.kind, PasteOutcomeKind::ReadObserved);
    assert!(!rig.pasteboard.state().writes.contains(&"restore"));
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
    assert_eq!(rig.pasteboard.state().captures, 1);
}

/// The first transcript item is marked transient and concealed, so it is
/// not restored either (decision 2026-10-13): the second transcript stays.
#[test]
fn after_not_read_neither_the_pre_first_snapshot_nor_the_first_transcript_returns() {
    let rig = Rig::new();
    let (mut coordinator, _, _) = paste(&rig, &[]);
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(&request(2, "second", TEXTEDIT, DontModify), &mut reads);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    let state = rig.pasteboard.state();
    assert_eq!(state.content, Content::Private("second".into()));
    assert!(!state.writes.contains(&"restore"));
    assert_eq!(state.captures, 1);
}

#[test]
fn after_an_external_copy_the_new_content_is_the_next_snapshot() {
    let rig = Rig::new();
    let (mut coordinator, _, _) = paste(&rig, &[20]);
    rig.pasteboard
        .external_write(Content::Plain("user copy".into()));
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(&request(2, "second", TEXTEDIT, DontModify), &mut reads);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    let user_copy = vec![vec![(PLAIN_TEXT_TYPE.to_string(), b"user copy".to_vec())]];
    assert_eq!(rig.pasteboard.state().content, Content::Items(user_copy));
}

#[test]
fn reads_of_a_superseded_item_never_affect_the_current_one() {
    let rig = Rig::new();
    let (mut coordinator, first, _) = paste(&rig, &[20]);
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(&request(2, "second", TEXTEDIT, DontModify), &mut reads);
    let deadline = coordinator.next_deadline();
    let old = first.transaction_id.unwrap_or(TransactionId(0));
    coordinator.handle_read(ReadEvent {
        transaction: old,
        at: rig.clock.now() + ms(100),
    });
    assert_eq!(coordinator.next_deadline(), deadline);
    assert!(rig.pasteboard.state().released.contains(&old));
}

#[test]
fn exit_before_quiet_materializes_and_after_quiet_restores() {
    let rig = Rig::new();
    let (mut coordinator, _, _) = paste(&rig, &[20]);
    coordinator.shutdown();
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Private("hello world".into())
    );
    let rig = Rig::new();
    let (mut coordinator, _, _) = paste(&rig, &[20]);
    rig.clock.advance_to(key_down(&rig) + ms(320));
    coordinator.shutdown();
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
}
