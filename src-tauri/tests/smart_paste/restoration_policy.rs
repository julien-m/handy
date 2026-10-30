//! What restoration puts back: concealed or transient content is withheld
//! (decision 2026-10-13), other content is restored; AppKit's read cache and
//! copy mode limits are pinned down (AC-009, AC-012, AC-013).

use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::clock::Clock;
use handy_app_lib::smart_paste::coordinator::SmartPasteCoordinator;
use handy_app_lib::smart_paste::ids::TransactionId;
use handy_app_lib::smart_paste::outcome::{PasteOutcome, PasteOutcomeKind};
use handy_app_lib::smart_paste::pasteboard_port::ReadEvent;
use handy_app_lib::smart_paste::pasteboard_types::{
    AUTO_GENERATED_TYPE, CONCEALED_TYPE, TRANSIENT_TYPE,
};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::{self, CopyToClipboard, DontModify};

use super::fake_pasteboard::Items;
use super::rig::{request, settle_until, Content, Rig, ScriptedReads};

// @spec FR-006: Guarded restoration and privacy-marked content — .specs/features/007-smart-paste/spec.md#fr-006

const TEXTEDIT: Option<&str> = Some("com.apple.TextEdit");

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

/// A password copied from a password manager: text plus `marker`.
fn marked_secret(marker: &str) -> Items {
    vec![vec![
        ("public.utf8-plain-text".into(), b"hunter2".to_vec()),
        (marker.into(), Vec::new()),
    ]]
}

fn paste_with(
    rig: &Rig,
    reads: ScriptedReads,
    policy: ClipboardPolicy,
) -> (SmartPasteCoordinator, PasteOutcome, ScriptedReads) {
    let mut coordinator = rig.coordinator();
    let mut reads = reads;
    let request = request(1, "hello world", TEXTEDIT, policy);
    let outcome = coordinator.insert(&request, &mut reads);
    (coordinator, outcome, reads)
}

fn key_down(rig: &Rig) -> Instant {
    let chords = rig.keys.state().chords.clone();
    chords.last().copied().unwrap_or_else(|| rig.clock.now())
}

#[test]
fn concealed_or_transient_content_is_not_restored_and_the_transcript_stays() {
    for marker in [CONCEALED_TYPE, TRANSIENT_TYPE] {
        let rig = Rig::new();
        rig.pasteboard.state().content = Content::Items(marked_secret(marker));
        let reads = ScriptedReads::new(&rig, &[20]);
        let (mut coordinator, outcome, _) = paste_with(&rig, reads, DontModify);
        assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
        settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
        let state = rig.pasteboard.state();
        assert_eq!(
            state.content,
            Content::Private("hello world".into()),
            "{marker}"
        );
        assert!(!state.writes.contains(&"restore"));
    }
}

#[test]
fn concealed_content_is_not_restored_on_exit_either() {
    let rig = Rig::new();
    rig.pasteboard.state().content = Content::Items(marked_secret(CONCEALED_TYPE));
    let reads = ScriptedReads::new(&rig, &[20]);
    let (mut coordinator, _, _) = paste_with(&rig, reads, DontModify);
    rig.clock.advance_to(key_down(&rig) + ms(320));
    coordinator.shutdown();
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Private("hello world".into())
    );
}

#[test]
fn auto_generated_content_alone_is_restored_normally() {
    let rig = Rig::new();
    let items = marked_secret(AUTO_GENERATED_TYPE);
    rig.pasteboard.state().content = Content::Items(items.clone());
    let reads = ScriptedReads::new(&rig, &[20]);
    let (mut coordinator, _, _) = paste_with(&rig, reads, DontModify);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    assert_eq!(rig.pasteboard.state().content, Content::Items(items));
}

#[test]
fn a_marked_type_that_could_not_be_saved_still_withholds_restoration() {
    let rig = Rig::new();
    rig.pasteboard.state().content = Content::Items(marked_secret(CONCEALED_TYPE));
    rig.pasteboard.state().unreadable_types = vec![CONCEALED_TYPE.into()];
    let reads = ScriptedReads::new(&rig, &[20]);
    let (mut coordinator, _, _) = paste_with(&rig, reads, DontModify);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(320));
    assert!(!rig.pasteboard.state().writes.contains(&"restore"));
}

/// Known release risk R-1: AppKit answers repeated reads of an item from
/// its cache, so natively the quiet period runs from the first read.
#[test]
fn with_the_appkit_cache_quiet_is_measured_from_the_first_read() {
    let rig = Rig::new();
    let reads = ScriptedReads::new(&rig, &[10, 160]).with_appkit_cache();
    let (mut coordinator, _, mut reads) = paste_with(&rig, reads, DontModify);
    reads.drain_into(&mut coordinator);
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(309));
    assert!(matches!(rig.pasteboard.state().content, Content::Lazy(_)));
    settle_until(&rig.clock, &mut coordinator, key_down(&rig) + ms(310));
    assert!(matches!(rig.pasteboard.state().content, Content::Items(_)));
}

/// Known release risk R-1: a clipboard manager reading before the keystroke
/// fills AppKit's cache, so the target's read is invisible: "not read".
#[test]
fn with_the_appkit_cache_a_pre_key_read_hides_the_target_read() {
    let rig = Rig::new();
    let reads = ScriptedReads::new(&rig, &[-5, 40]).with_appkit_cache();
    let (_, outcome, _) = paste_with(&rig, reads, DontModify);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
}

/// AC-013 wins in copy mode: the transcript becomes ordinary plain text at
/// "not read", which the pasteboard then serves without any provider
/// callback, so a late read cannot be reported (documented for 010).
#[test]
fn copy_mode_makes_the_transcript_plain_at_not_read_so_late_reads_are_unseen() {
    let rig = Rig::new();
    let reads = ScriptedReads::new(&rig, &[]);
    let (mut coordinator, outcome, _) = paste_with(&rig, reads, CopyToClipboard);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    assert_eq!(coordinator.pending_transaction(), None);
    let transaction = outcome.transaction_id.unwrap_or(TransactionId(0));
    let at = rig.clock.now() + ms(100);
    coordinator.handle_read(ReadEvent { transaction, at });
    assert!(rig.diagnostics().is_empty());
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
}

#[test]
fn the_snapshot_helper_is_prepared_before_direct_insertion() {
    let rig = Rig::new();
    let reads = ScriptedReads::new(&rig, &[20]);
    paste_with(&rig, reads, DontModify);
    let state = rig.pasteboard.state();
    assert_eq!((state.prepared_captures, state.captures), (1, 1));
}
