//! Experimental paste verification (AC-019, finding N11) through the
//! production coordinator: a page that receives the paste but inserts
//! nothing, the single retry under the clipboard ownership guard, the
//! unreadable field and the setting off (behavior unchanged). The N28 web
//! text rule and the last read before the retry are in `verification_web`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError};

use handy_app_lib::smart_paste::ax_port::Utf16Range;
use handy_app_lib::smart_paste::keyboard::{InputSourceKind, ABC_LAYOUT_ID};
use handy_app_lib::smart_paste::outcome::{InsertionPath, PasteOutcome, PasteOutcomeKind};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::DontModify;

use super::fakes::{utf16, FakeInputSources, ScriptedField, WriteBehavior};
use super::rig::{request, Content, Rig, ScriptedReads};

// @spec FR-012: Paste verification and single retry — .specs/features/007-smart-paste/spec.md#fr-012
// @spec AC-019: Verified paste, retry, failure, unreadable — .specs/features/007-smart-paste/spec.md#ac-019

const CHROME: Option<&str> = Some("com.google.Chrome");
pub const TRANSCRIPT: &str = "hello world";
const FRENCH_LAYOUT_ID: &str = "com.apple.keylayout.French";

/// A focused field of the destination (pid 7) that ignores the direct AX
/// write, like the Google page of N11, so smart paste falls back to Cmd+V.
pub fn page_field(rig: &mut Rig) -> ScriptedField {
    let caret = Utf16Range {
        location: 0,
        length: 0,
    };
    let field = ScriptedField::new(&rig.clock, 7, "", caret);
    field.state().behavior = WriteBehavior::Ignore;
    rig.ax.field = Some(field.clone());
    field
}

/// The page inserts the transcript on the `nth` Cmd+V only (1-based);
/// `None`: it never inserts anything.
pub fn page_inserts_on(rig: &Rig, field: &ScriptedField, nth: Option<usize>) {
    let chords = Arc::new(AtomicUsize::new(0));
    let field = field.clone();
    rig.keys.state().on_chord = Some(Arc::new(move || {
        let count = chords.fetch_add(1, Ordering::SeqCst) + 1;
        if Some(count) == nth {
            field.state().value.extend(utf16(TRANSCRIPT));
        }
    }));
}

fn insert(rig: &Rig, verifies_paste: bool, offsets: &[i64]) -> PasteOutcome {
    insert_dictation(rig, 1, verifies_paste, offsets)
}

/// Inserts the transcript as dictation `id` into the Chrome destination.
pub fn insert_dictation(rig: &Rig, id: u64, verifies_paste: bool, offsets: &[i64]) -> PasteOutcome {
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(rig, offsets);
    let mut request = request(id, TRANSCRIPT, CHROME, DontModify);
    request.verifies_paste = verifies_paste;
    coordinator.insert(&request, &mut reads)
}

/// Number of Cmd+V chords posted so far.
pub fn chords(rig: &Rig) -> usize {
    rig.keys.state().chords.len()
}

#[test]
fn a_field_changed_by_the_first_paste_is_verified_without_a_retry() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, Some(1));
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(outcome.path, InsertionPath::ClipboardPaste);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn an_unchanged_field_gets_one_more_paste_and_is_verified_by_it() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, Some(2));
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chords(&rig), 2);
    assert_eq!(field.value_text(), TRANSCRIPT);
}

#[test]
fn a_field_unchanged_after_the_retry_fails_to_recovery_without_enter() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::FieldUnchanged);
    assert!(outcome.kind.requires_recovery());
    assert!(!outcome.kind.is_auto_submit_eligible());
    assert!(outcome.transaction_id.is_some());
    assert_eq!(chords(&rig), 2, "exactly one retry");
}

#[test]
fn the_retry_reuses_the_published_item_and_never_rewrites_the_clipboard() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    insert(&rig, true, &[20]);
    let state = rig.pasteboard.state();
    assert_eq!(state.writes, ["publish"], "one publication, no rewrite");
    assert!(matches!(state.content, Content::Lazy(_)));
}

#[test]
fn no_retry_once_the_clipboard_no_longer_holds_the_transcript() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    let pasteboard = rig.pasteboard.clone();
    // Another write takes the clipboard over after the page read the item:
    // only the change count moves, so the scripted read is still served.
    rig.keys.state().on_chord = Some(Arc::new(move || {
        pasteboard.state().change_count += 1;
    }));
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::FieldUnchanged);
    assert_eq!(chords(&rig), 1);
    assert_eq!(field.value_text(), "");
}

#[test]
fn no_retry_once_the_destination_is_no_longer_current() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    // Checks: chain start, before the first chord, before the retry.
    rig.destination
        .answers
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .extend([true, true, false]);
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::FieldUnchanged);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn the_retry_switches_and_restores_the_layout_like_the_first_paste() {
    let mut rig = Rig::new();
    let layouts = [ABC_LAYOUT_ID, FRENCH_LAYOUT_ID];
    let kind = InputSourceKind::KeyboardLayout;
    rig.sources = FakeInputSources::new(&rig.clock, FRENCH_LAYOUT_ID, kind, &layouts);
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    insert(&rig, true, &[20]);
    let state = rig.sources.state();
    let selected: Vec<&str> = state.selections.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(selected, [layouts, layouts].concat());
    let current = state.current.as_ref().map(|source| source.id.as_str());
    assert_eq!(current, Some(FRENCH_LAYOUT_ID));
}

#[test]
fn without_a_readable_field_the_outcome_is_unchanged() {
    let rig = Rig::new();
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn a_password_field_is_never_read_for_verification() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    field.state().is_secure = true;
    page_inserts_on(&rig, &field, None);
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn a_field_of_another_process_is_not_verified() {
    let mut rig = Rig::new();
    let caret = Utf16Range::default();
    let other = ScriptedField::new(&rig.clock, 99, "", caret);
    rig.ax.field = Some(other.clone());
    page_inserts_on(&rig, &other, None);
    let outcome = insert(&rig, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn not_read_is_never_verified_nor_retried() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    let outcome = insert(&rig, true, &[]);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    assert_eq!(chords(&rig), 1);
}

#[test]
fn with_the_setting_off_the_outcome_and_effects_are_unchanged() {
    let run = |verifies_paste: bool| {
        let mut rig = Rig::new();
        let field = page_field(&mut rig);
        page_inserts_on(&rig, &field, None);
        let outcome = insert(&rig, verifies_paste, &[20]);
        let focus_requests = rig.ax.focus_request_count();
        (outcome.kind, chords(&rig), focus_requests)
    };
    let (kind, chord_count, focus_requests) = run(false);
    assert_eq!(kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chord_count, 1);
    let (_, _, verified_focus_requests) = run(true);
    assert_eq!(
        focus_requests + 1,
        verified_focus_requests,
        "off: no probe of the focused field"
    );
}

/// The `verify dictation <id>` lines logged so far (the logger is shared by
/// the whole test process, so each case uses its own dictation id).
pub fn verify_logs(id: u64) -> Vec<String> {
    let marker = format!("verify dictation {id}");
    super::privacy::records()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .filter(|line| line.contains(&marker))
        .cloned()
        .collect()
}

#[test]
fn each_verdict_is_logged_once_with_its_duration_and_no_content() {
    super::privacy::install_logger();
    let cases = [
        (41, Some(1), "Verified"),
        (42, Some(2), "RetriedVerified"),
        (43, None, "Failed"),
    ];
    for (id, nth, label) in cases {
        let mut rig = Rig::new();
        let field = page_field(&mut rig);
        page_inserts_on(&rig, &field, nth);
        insert_dictation(&rig, id, true, &[20]);
        let logs = verify_logs(id);
        let prefix = format!("[smart-paste] verify dictation {id}: {label} in ");
        let verdicts: Vec<_> = logs.iter().filter(|l| l.contains(&prefix)).collect();
        assert_eq!(verdicts.len(), 1, "{logs:?}");
        assert!(verdicts[0].ends_with("ms"), "{logs:?}");
        assert!(
            logs.iter().all(|line| !line.contains(TRANSCRIPT)),
            "{logs:?}"
        );
    }
}

#[test]
fn an_unreadable_verdict_is_logged_with_its_reason_and_off_logs_nothing() {
    super::privacy::install_logger();
    insert_dictation(&Rig::new(), 44, true, &[20]);
    let unreadable = verify_logs(44);
    assert_eq!(unreadable.len(), 2, "{unreadable:?}");
    assert!(unreadable[0].contains(": Unreadable in "), "{unreadable:?}");
    assert!(
        unreadable[1].ends_with("unreadable: NoReadableField"),
        "{unreadable:?}"
    );
    insert_dictation(&Rig::new(), 45, false, &[20]);
    assert!(verify_logs(45).is_empty());
}
