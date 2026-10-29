//! Finding N28 through the production coordinator: a focused element that
//! is not web text (a terminal whose readable value does not follow the
//! paste) is never retried nor reported failed, and a page that applies the
//! paste after the window is not sent the transcript twice.

use std::sync::{Arc, PoisonError};

use handy_app_lib::smart_paste::ax_port::AxError;
use handy_app_lib::smart_paste::outcome::PasteOutcomeKind;
use handy_app_lib::smart_paste::verification::WEB_ANCESTRY_LIMITS;

use super::fakes::utf16;
use super::rig::Rig;
use super::verification::{
    chords, insert_dictation, page_field, page_inserts_on, verify_logs, TRANSCRIPT,
};

// @spec FR-012: Web text only, no late double paste — .specs/features/007-smart-paste/spec.md#fr-012
// @spec AC-019: Terminal unreadable, late paste kept — .specs/features/007-smart-paste/spec.md#ac-019

/// A terminal-like element: readable, but its value never shows the paste.
#[test]
fn an_element_outside_any_web_area_is_unreadable_and_never_retried() {
    super::privacy::install_logger();
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    field.state().web_area = Ok(false);
    page_inserts_on(&rig, &field, None);
    let outcome = insert_dictation(&rig, 51, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert!(outcome.kind.is_auto_submit_eligible());
    assert!(!outcome.kind.requires_recovery());
    assert_eq!(chords(&rig), 1, "no second Cmd+V");
    let logs = verify_logs(51);
    assert_eq!(logs.len(), 2, "{logs:?}");
    assert!(logs[0].contains(": Unreadable in "), "{logs:?}");
    assert!(logs[1].ends_with("unreadable: NotWebText"), "{logs:?}");
    assert!(
        logs.iter().all(|line| !line.contains(TRANSCRIPT)),
        "{logs:?}"
    );
}

#[test]
fn a_web_area_search_that_fails_is_no_proof_of_web_text() {
    for error in [AxError::Timeout, AxError::NotTrusted] {
        let mut rig = Rig::new();
        let field = page_field(&mut rig);
        field.state().web_area = Err(error);
        page_inserts_on(&rig, &field, None);
        let outcome = insert_dictation(&rig, 52, true, &[20]);
        assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved, "{error:?}");
        assert_eq!(chords(&rig), 1, "{error:?}");
    }
}

#[test]
fn the_web_area_search_uses_the_bounded_limits() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, Some(1));
    insert_dictation(&rig, 53, true, &[20]);
    assert_eq!(field.state().ancestry_searches, [WEB_ANCESTRY_LIMITS]);
}

#[test]
fn the_web_area_is_not_searched_with_the_setting_off() {
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    page_inserts_on(&rig, &field, None);
    insert_dictation(&rig, 54, false, &[20]);
    assert!(field.state().ancestry_searches.is_empty());
}

/// The page applies the first paste only once the window is over: the
/// destination check of the retry permit runs between the window and the
/// last read, so the value appears exactly there.
#[test]
fn a_paste_applied_after_the_window_is_verified_without_a_second_paste() {
    super::privacy::install_logger();
    let mut rig = Rig::new();
    let field = page_field(&mut rig);
    let late = field.clone();
    let keys = Arc::clone(&rig.keys.state);
    let hook = Arc::new(move || {
        let has_pasted = !keys
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .chords
            .is_empty();
        let mut state = late.state();
        if has_pasted && state.value.is_empty() {
            state.value = utf16(TRANSCRIPT);
        }
    });
    *rig.destination
        .on_check
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(hook);
    let outcome = insert_dictation(&rig, 55, true, &[20]);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(chords(&rig), 1, "the late paste is not sent twice");
    assert_eq!(field.value_text(), TRANSCRIPT);
    let logs = verify_logs(55);
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert!(logs[0].contains(": Verified in "), "{logs:?}");
}
