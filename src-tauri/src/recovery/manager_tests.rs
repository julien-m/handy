//! State-machine transcripts of the recovery lifetime (plan "State Model", P0 state tests):
//! presentation results and slot release, Copy, Close, Close during Copy, drag, new dictation
//! at any age, stale callbacks, duplicate outcomes, the late read and the frozen setting.
use super::*;
use crate::destination::outcome::HistoryStatus;
use crate::destination::recovery::{PresentationReceipt, PresentationWait};
use crate::recovery::types::{BubbleReason, BubbleSpec, ReasonDetail};
use std::time::Duration;

const TEXT: &str = "safe test phrase kept by the bubble";

fn manager() -> RecoveryManager {
    RecoveryManager::new(RecoveryPlatform::MacOs)
}
fn admission(manager: &RecoveryManager, dictation: u64) -> Admission {
    Admission {
        key: manager.next_key(DictationId(dictation), dictation + 10),
        spec: BubbleSpec {
            reason: BubbleReason::InsertionFailed,
            detail: ReasonDetail::MayHaveReached,
        },
        transcript: TEXT.to_owned(),
        inserted: TEXT.to_owned(),
        transaction_id: Some("7".to_owned()),
        history: HistoryStatus::Saved,
        app_name: Some("Notes".to_owned()),
        transcript_ref: Some(TranscriptRef(dictation)),
        decided_at: None,
        consumed_at: Instant::now(),
    }
}
/// Admit dictation `dictation`; returns its bubble and receipt.
fn admit(manager: &RecoveryManager, dictation: u64) -> Option<(BubbleId, PresentationReceipt)> {
    let (receipt, view, _) = manager.admit(admission(manager, dictation))?;
    Some((view.bubble_id, receipt))
}
fn answer(receipt: PresentationReceipt) -> Option<PresentationWait> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .ok()?;
    Some(runtime.block_on(receipt.wait(Duration::from_millis(10))))
}
fn visible(manager: &RecoveryManager, dictation: u64) -> Option<BubbleId> {
    let (bubble, _) = admit(manager, dictation)?;
    manager.rendered(bubble)?;
    Some(bubble)
}
type TestResult = Result<(), &'static str>;

#[test]
fn presented_only_after_the_first_render_then_the_slot_is_released() -> TestResult {
    let manager = manager();
    let (bubble, receipt) = admit(&manager, 1).ok_or("admitted")?;
    assert!(manager.is_presenting(bubble));
    let resolved = manager.rendered(bubble).ok_or("rendered")?;
    assert_eq!(resolved.release, Some(TranscriptRef(1)));
    assert!(resolved.waiter_answered);
    assert_eq!(
        answer(receipt),
        Some(PresentationWait::Resolved(PresentationResult::Presented))
    );
    assert!(
        manager.rendered(bubble).is_none(),
        "a second render is a no-op"
    );
    Ok(())
}
#[test]
fn an_unpresentable_bubble_is_discarded_and_reported_to_a_waiter_still_there() -> TestResult {
    let manager = manager();
    let (bubble, receipt) = admit(&manager, 1).ok_or("admitted")?;
    let resolved = manager.unpresentable(bubble).ok_or("failed")?;
    assert!(resolved.waiter_answered);
    assert_eq!(resolved.release, Some(TranscriptRef(1)));
    assert_eq!(manager.view(), None);
    assert_eq!(
        answer(receipt),
        Some(PresentationWait::Resolved(
            PresentationResult::Unpresentable
        ))
    );
    assert!(
        manager.rendered(bubble).is_none(),
        "a late render is ignored"
    );
    Ok(())
}
#[test]
fn after_the_guard_expired_the_late_answer_finds_no_waiter() -> TestResult {
    let manager = manager();
    let (bubble, receipt) = admit(&manager, 1).ok_or("admitted")?;
    drop(receipt);
    let resolved = manager.unpresentable(bubble).ok_or("failed")?;
    assert!(
        !resolved.waiter_answered,
        "the service emits paste-error itself"
    );
    Ok(())
}
#[test]
fn a_new_dictation_supersedes_a_bubble_not_yet_shown_and_releases_its_slot() -> TestResult {
    let manager = manager();
    let (_, receipt) = admit(&manager, 1).ok_or("admitted")?;
    let retired = manager.new_dictation().ok_or("retired")?;
    assert_eq!(retired.release, Some(TranscriptRef(1)));
    assert_eq!(
        answer(receipt),
        Some(PresentationWait::Resolved(PresentationResult::Superseded))
    );
    Ok(())
}
#[test]
fn a_new_dictation_closes_a_visible_bubble_at_once_and_its_late_actions_are_ignored() -> TestResult
{
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    let retired = manager.new_dictation().ok_or("closed")?;
    assert_eq!(retired.release, None, "released when it was shown");
    assert!(manager.begin_copy(bubble, 11).is_none());
    assert!(manager.close(bubble).is_none());
    assert_eq!(
        manager.drag_ended(bubble, DragResult::Dropped),
        DragEnd::Ignored
    );
    Ok(())
}
#[test]
fn there_is_no_timeout_the_bubble_stays_until_an_explicit_event() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    // The manager has no timer at all: any elapsed time leaves the same view.
    let view = manager.view().ok_or("shown")?;
    assert_eq!(view.bubble_id, bubble);
    assert_eq!(manager.view(), Some(view));
    Ok(())
}
#[test]
fn copy_closes_on_success_and_keeps_the_bubble_unchanged_on_failure() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    let before = manager.view();
    let (ticket, text) = manager.begin_copy(bubble, 11).ok_or("copy")?;
    assert_eq!(text, TEXT, "the full transcript, not the preview");
    assert_eq!(manager.finish_copy(ticket, false), CopyEnd::KeptOpen);
    assert_eq!(manager.view(), before);
    let (ticket, _) = manager.begin_copy(bubble, 11).ok_or("copy again")?;
    assert!(matches!(
        manager.finish_copy(ticket, true),
        CopyEnd::Closed(_)
    ));
    assert_eq!(manager.view(), None);
    Ok(())
}
#[test]
fn a_copy_with_a_stale_generation_is_refused() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    assert!(manager.begin_copy(bubble, 99).is_none());
    Ok(())
}
#[test]
fn close_during_copy_closes_at_once_and_the_write_completion_is_ignored() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    let (ticket, _) = manager.begin_copy(bubble, 11).ok_or("copy")?;
    assert!(manager.close(bubble).is_some());
    assert_eq!(manager.finish_copy(ticket, true), CopyEnd::Ignored);
    let next = visible(&manager, 2).ok_or("next")?;
    assert_eq!(manager.finish_copy(ticket, true), CopyEnd::Ignored);
    assert_eq!(manager.view().map(|view| view.bubble_id), Some(next));
    Ok(())
}
#[test]
fn a_refused_drag_shows_the_same_bubble_again_and_an_accepted_one_closes() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    let before = manager.view().ok_or("view")?;
    assert_eq!(manager.begin_drag(bubble).as_deref(), Some(TEXT));
    assert_eq!(
        manager.drag_ended(bubble, DragResult::Released),
        DragEnd::Refused(before)
    );
    manager.begin_drag(bubble).ok_or("again")?;
    assert!(matches!(
        manager.drag_ended(bubble, DragResult::Dropped),
        DragEnd::Accepted(_)
    ));
    assert_eq!(manager.view(), None);
    Ok(())
}
// @spec AC-004: Escape cancels and dismisses the bubble — .specs/features/010-recovery-bubble/spec.md#ac-004
#[test]
fn an_accepted_drop_wins_over_escape_and_escape_alone_dismisses() {
    for accepted in [false, true] {
        for escape in [false, true] {
            let result = DragResult::of_session(accepted, escape);
            let expected = if accepted {
                DragResult::Dropped
            } else if escape {
                DragResult::CancelledByEscape
            } else {
                DragResult::Released
            };
            assert_eq!(result, expected, "accepted {accepted}, Escape {escape}");
        }
    }
}
// @spec AC-004: Escape drops the slide back at once — .specs/features/010-recovery-bubble/spec.md#ac-004
#[test]
fn the_drag_image_slides_back_only_when_the_bubble_comes_back() {
    for escape in [false, true] {
        // A session that slides back ends with nothing dropped.
        let comes_back = DragResult::of_session(false, escape) == DragResult::Released;
        assert_eq!(
            crate::recovery::types::drag_image_slides_back(escape),
            comes_back,
            "Escape {escape}"
        );
    }
}
#[test]
fn escape_during_a_drag_closes_the_bubble_like_close_and_ignores_late_ends() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    manager.begin_drag(bubble).ok_or("drag")?;
    let DragEnd::Dismissed(retired) = manager.drag_ended(bubble, DragResult::CancelledByEscape)
    else {
        return Err("Escape dismisses");
    };
    // The same bubble closed with Close during its drag: Escape must retire it identically.
    let twin = RecoveryManager::new(RecoveryPlatform::MacOs);
    let twin_bubble = visible(&twin, 1).ok_or("twin visible")?;
    twin.begin_drag(twin_bubble).ok_or("twin drag")?;
    let closed = twin.close(twin_bubble).ok_or("closed")?;
    assert_eq!(retired, closed);
    assert_eq!(manager.view(), None);
    for late in [
        DragResult::Released,
        DragResult::Dropped,
        DragResult::CancelledByEscape,
    ] {
        assert_eq!(manager.drag_ended(bubble, late), DragEnd::Ignored);
    }
    assert!(manager.begin_copy(bubble, 11).is_none());
    Ok(())
}
#[test]
fn a_failed_drag_start_leaves_the_bubble_visible() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    manager.begin_drag(bubble).ok_or("drag")?;
    assert!(manager.drag_failed(bubble));
    assert!(manager.begin_copy(bubble, 11).is_some(), "visible again");
    Ok(())
}
#[test]
fn a_new_dictation_while_dragging_closes_for_good_and_ignores_the_late_drop() -> TestResult {
    let manager = manager();
    let first = visible(&manager, 1).ok_or("visible")?;
    manager.begin_drag(first).ok_or("drag")?;
    manager.new_dictation().ok_or("closed")?;
    let second = visible(&manager, 2).ok_or("second")?;
    assert_eq!(
        manager.drag_ended(first, DragResult::Released),
        DragEnd::Ignored
    );
    assert_eq!(
        manager.drag_ended(first, DragResult::Dropped),
        DragEnd::Ignored
    );
    assert_eq!(manager.view().map(|view| view.bubble_id), Some(second));
    Ok(())
}
#[test]
fn windows_and_linux_bubbles_offer_no_drag() -> TestResult {
    for platform in [RecoveryPlatform::Windows, RecoveryPlatform::Linux] {
        let manager = RecoveryManager::new(platform);
        let bubble = visible(&manager, 1).ok_or("visible")?;
        assert_eq!(manager.view().map(|view| view.can_drag), Some(false));
        assert!(manager.begin_drag(bubble).is_none());
    }
    Ok(())
}
#[test]
fn duplicate_or_older_outcomes_are_not_admitted() -> TestResult {
    let manager = manager();
    admit(&manager, 2).ok_or("admitted")?;
    assert!(admit(&manager, 2).is_none(), "one bubble per dictation");
    assert!(admit(&manager, 1).is_none(), "an older dictation is stale");
    assert!(manager.is_stale(DictationId(2)));
    Ok(())
}
#[test]
fn a_late_read_updates_the_matching_bubble_in_place_once() -> TestResult {
    let manager = manager();
    let bubble = visible(&manager, 1).ok_or("visible")?;
    let updated = manager.late_read(DictationId(1), "7");
    let LateReadEnd::Updated(view) = updated else {
        return Err("not updated");
    };
    assert!(view.late_read);
    assert_eq!(view.bubble_id, bubble, "no second bubble");
    assert_eq!(manager.late_read(DictationId(1), "7"), LateReadEnd::Ignored);
    assert_eq!(manager.late_read(DictationId(1), "8"), LateReadEnd::Ignored);
    Ok(())
}
#[test]
fn a_late_read_before_admission_is_applied_when_its_outcome_arrives() -> TestResult {
    let manager = manager();
    assert_eq!(
        manager.late_read(DictationId(3), "7"),
        LateReadEnd::Buffered
    );
    let (_, view, _) = manager.admit(admission(&manager, 3)).ok_or("admitted")?;
    assert!(view.late_read);
    Ok(())
}
#[test]
fn a_buffered_late_read_is_dropped_at_the_next_dictation_and_stale_ones_are_ignored() -> TestResult
{
    let manager = manager();
    assert_eq!(
        manager.late_read(DictationId(3), "7"),
        LateReadEnd::Buffered
    );
    manager.new_dictation();
    let (_, view, _) = manager.admit(admission(&manager, 3)).ok_or("admitted")?;
    assert!(!view.late_read);
    assert_eq!(manager.late_read(DictationId(2), "7"), LateReadEnd::Ignored);
    Ok(())
}
#[test]
fn the_setting_is_frozen_at_the_dictation_start_and_bound_to_that_dictation() {
    let manager = manager();
    assert!(!manager.setting_for(DictationId(1)), "unknown: off");
    manager.freeze_setting(true);
    manager.bind_setting(DictationId(1));
    manager.bind_setting(DictationId(2));
    assert!(manager.setting_for(DictationId(1)));
    assert!(!manager.setting_for(DictationId(2)));
    manager.freeze_setting(false);
    manager.bind_setting(DictationId(2));
    assert!(!manager.setting_for(DictationId(2)));
}
