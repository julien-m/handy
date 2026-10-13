//! Insertion chain through the production coordinator: AX first, dedicated
//! strategies and keyboard handling (AC-010, AC-011, AC-017, AC-018).

use std::time::Duration;

use handy_app_lib::smart_paste::ax_port::Utf16Range;
use handy_app_lib::smart_paste::clock::Clock;
use handy_app_lib::smart_paste::keyboard::{InputSourceKind, ABC_LAYOUT_ID};
use handy_app_lib::smart_paste::outcome::{InsertionPath, PasteOutcome, PasteOutcomeKind};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::{self, CopyToClipboard, DontModify};
use handy_app_lib::smart_paste::strategy::{
    IPHONE_MIRRORING_BUNDLE_ID, SCREEN_SHARING_BUNDLE_ID, WINDOWS_APP_BUNDLE_ID,
};
use handy_app_lib::smart_paste::timing::JUMP_DESKTOP_BUNDLE_ID;

use super::fakes::{FakeInputSources, ScriptedField, WriteBehavior};
use super::rig::{request, rich_items, Content, Rig, ScriptedReads};

// @spec FR-007: Strategy and keyboard composition — .specs/features/007-smart-paste/spec.md#fr-007
// @spec FR-011: AX then clipboard then recovery — .specs/features/007-smart-paste/spec.md#fr-011

const TEXTEDIT: Option<&str> = Some("com.apple.TextEdit");

fn insert(
    rig: &Rig,
    bundle: Option<&str>,
    policy: ClipboardPolicy,
    offsets: &[i64],
) -> PasteOutcome {
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(rig, offsets);
    coordinator.insert(&request(1, "hello world", bundle, policy), &mut reads)
}

fn with_field(rig: &mut Rig, behavior: WriteBehavior) -> ScriptedField {
    let caret = Utf16Range {
        location: 6,
        length: 0,
    };
    let field = ScriptedField::new(&rig.clock, 7, "note: ", caret);
    field.state().behavior = behavior;
    rig.ax.field = Some(field.clone());
    field
}

#[test]
fn verified_direct_insertion_leaves_the_clipboard_untouched() {
    let mut rig = Rig::new();
    let field = with_field(&mut rig, WriteBehavior::ApplyAfter(Duration::ZERO));
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertedVerified);
    assert_eq!(outcome.path, InsertionPath::DirectAccessibility);
    assert_eq!(field.value_text(), "note: hello world");
    let state = rig.pasteboard.state();
    assert_eq!((state.captures, state.writes.len()), (0, 0));
    assert!(rig.keys.state().chords.is_empty());
}

#[test]
fn verified_direct_insertion_in_copy_mode_also_copies_plain_text() {
    let mut rig = Rig::new();
    with_field(&mut rig, WriteBehavior::ApplyAfter(Duration::ZERO));
    insert(&rig, TEXTEDIT, CopyToClipboard, &[]);
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
}

#[test]
fn unchanged_field_falls_back_to_the_clipboard_paste() {
    let mut rig = Rig::new();
    with_field(&mut rig, WriteBehavior::Ignore);
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(outcome.path, InsertionPath::ClipboardPaste);
    assert_eq!(rig.keys.state().chords.len(), 1);
}

#[test]
fn unexpected_change_is_uncertain_and_never_pasted() {
    let mut rig = Rig::new();
    with_field(&mut rig, WriteBehavior::ApplyPartially);
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::Uncertain);
    assert_eq!(rig.pasteboard.state().captures, 0);
    assert!(rig.keys.state().chords.is_empty());
}

#[test]
fn write_applied_during_the_snapshot_is_verified_without_publication() {
    let mut rig = Rig::new();
    let field = with_field(
        &mut rig,
        WriteBehavior::ApplyAfter(Duration::from_millis(160)),
    );
    rig.pasteboard.state().capture_cost = Duration::from_millis(30);
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertedVerified);
    assert_eq!(field.value_text(), "note: hello world");
    assert!(!rig.pasteboard.state().writes.contains(&"publish"));
}

#[test]
fn write_applied_before_the_keystroke_cancels_the_chord() {
    let mut rig = Rig::new();
    let field = with_field(
        &mut rig,
        WriteBehavior::ApplyAfter(Duration::from_millis(200)),
    );
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::Uncertain);
    assert!(rig.keys.state().chords.is_empty());
    assert_eq!(field.value_text(), "note: hello world");
    let state = rig.pasteboard.state();
    assert_eq!(state.content, Content::Plain("hello world".into()));
    assert!(!state.writes.contains(&"restore"));
}

#[test]
fn remote_keyboards_get_unicode_typing_without_the_clipboard() {
    for bundle in [IPHONE_MIRRORING_BUNDLE_ID, WINDOWS_APP_BUNDLE_ID] {
        let rig = Rig::new();
        let outcome = insert(&rig, Some(bundle), DontModify, &[]);
        assert_eq!(outcome.kind, PasteOutcomeKind::Typed);
        assert_eq!(rig.keys.state().typed, vec!["hello world".to_string()]);
        let state = rig.pasteboard.state();
        assert_eq!((state.captures, state.writes.len()), (0, 0));
    }
}

#[test]
fn unicode_typing_in_copy_mode_leaves_the_transcript_and_failure_is_reported() {
    let rig = Rig::new();
    insert(&rig, Some(IPHONE_MIRRORING_BUNDLE_ID), CopyToClipboard, &[]);
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
    let rig = Rig::new();
    rig.keys.state().typing_fails = true;
    let outcome = insert(&rig, Some(WINDOWS_APP_BUNDLE_ID), DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
}

#[test]
fn screen_sharing_gets_a_hand_over_without_keystroke_or_permission() {
    let mut rig = Rig::new();
    rig.ax.is_trusted = false;
    let outcome = insert(&rig, Some(SCREEN_SHARING_BUNDLE_ID), DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ClipboardHandedOver);
    assert!(!outcome.kind.is_auto_submit_eligible() && !outcome.kind.requires_recovery());
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
    assert!(rig.keys.state().chords.is_empty());
}

#[test]
fn jump_desktop_decides_not_read_after_3_seconds() {
    let rig = Rig::new();
    let outcome = insert(&rig, Some(JUMP_DESKTOP_BUNDLE_ID), DontModify, &[2900]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    let rig = Rig::new();
    let outcome = insert(&rig, Some(JUMP_DESKTOP_BUNDLE_ID), DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    let key_down = rig.keys.state().chords.last().copied();
    assert_eq!(
        key_down.map(|down| rig.clock.now() - down),
        Some(Duration::from_secs(3))
    );
}

#[test]
fn azerty_switches_to_abc_before_the_chord_and_restores_after_key_up() {
    let mut rig = Rig::new();
    let french = "com.apple.keylayout.French";
    rig.sources = FakeInputSources::new(
        &rig.clock,
        french,
        InputSourceKind::KeyboardLayout,
        &[ABC_LAYOUT_ID, french],
    );
    insert(&rig, TEXTEDIT, DontModify, &[20]);
    let key_down = rig.keys.state().chords.first().copied();
    let selections = rig.sources.state().selections.clone();
    let ids: Vec<_> = selections.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, vec![ABC_LAYOUT_ID, french]);
    // 50 ms settle plus the 60 ms paste delay precede the key-down; the
    // restore follows the 100 ms chord hold by 100 ms.
    let offsets: Vec<_> = selections
        .iter()
        .map(|(_, at)| key_down.map(|down| (*at).max(down) - (*at).min(down)))
        .collect();
    assert_eq!(
        offsets,
        vec![
            Some(Duration::from_millis(110)),
            Some(Duration::from_millis(200))
        ]
    );
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Lazy(handy_app_lib::smart_paste::ids::TransactionId(1))
    );
    let _ = rich_items();
}

#[test]
fn layout_is_restored_even_when_the_chord_fails() {
    let mut rig = Rig::new();
    let french = "com.apple.keylayout.French";
    rig.sources = FakeInputSources::new(
        &rig.clock,
        french,
        InputSourceKind::KeyboardLayout,
        &[ABC_LAYOUT_ID, french],
    );
    rig.keys.state().chord_fails = true;
    let outcome = insert(&rig, TEXTEDIT, DontModify, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::InsertionFailed);
    let last = rig
        .sources
        .state()
        .selections
        .last()
        .map(|(id, _)| id.clone());
    assert_eq!(last.as_deref(), Some(french));
}
