//! 010 recovery lifecycle through the production `RecoveryService` with native boundaries
//! faked (plan Step 2, P0 state tests): presentation results and slot release (fast, past the
//! guard, never answering until the hang limit, unpresentable before and after the guard,
//! superseded), Copy, Close, Close during Copy, drag, late read, new dictation at any age, no
//! timeout, platform projections and history wording inputs.
#[path = "recovery/support.rs"]
mod support;

use handy_app_lib::destination::context::DictationId;
use handy_app_lib::destination::outcome::{DestinationFailure, HistoryStatus, MethodOutcome};
use handy_app_lib::destination::recovery::{
    ConsumerAck, OutcomeConsumer, PresentationReceipt, PresentationResult, PresentationWait,
};
use handy_app_lib::recovery::ports::CopyError;
use handy_app_lib::recovery::service::PRESENTATION_HANG_LIMIT;
use handy_app_lib::recovery::types::{
    BubbleReason, HistoryNote, ReasonDetail, RecoveryBubbleView, RecoveryPlatform,
    PREVIEW_MAX_CHARS,
};
use std::sync::mpsc;
use std::time::Duration;
use support::{lock, Bench, SurfaceCall, TestResult, PHRASE};

/// Wait for a receipt as 006 does, with a short guard.
fn wait(receipt: PresentationReceipt) -> Result<PresentationWait, std::io::Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    Ok(runtime.block_on(receipt.wait(Duration::from_millis(20))))
}
/// Consume a failed outcome of `dictation`; returns its receipt and presented view.
fn consume(
    bench: &Bench,
    dictation: u64,
    reason: Option<DestinationFailure>,
) -> Result<(PresentationReceipt, RecoveryBubbleView), Box<dyn std::error::Error>> {
    let outcome = bench.failed(dictation, reason, None);
    let ConsumerAck::PresentationPending(receipt) = bench.service.consume(&outcome) else {
        return Err("no presentation".into());
    };
    let view = bench.surface.presented().pop().ok_or("not presented")?;
    Ok((receipt, view))
}

// @spec AC-003: Preview, reason and actions of the bubble — .specs/features/010-recovery-bubble/spec.md#ac-003
#[test]
fn consume_copies_the_transcript_presents_a_preview_and_holds_the_slot_until_rendered() -> TestResult
{
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (receipt, view) = consume(&bench, 1, Some(DestinationFailure::AppClosed))?;
    assert_eq!(view.reason, BubbleReason::AppClosed);
    assert_eq!(view.app_name.as_deref(), Some("Notes"));
    assert!(view.truncated && view.can_drag);
    assert_eq!(view.preview.chars().count(), PREVIEW_MAX_CHARS);
    assert!(PHRASE.starts_with(&view.preview));
    assert!(
        bench.released().is_empty(),
        "the slot is held while presenting"
    );
    bench.service.rendered(view.bubble_id);
    assert_eq!(
        wait(receipt)?,
        PresentationWait::Resolved(PresentationResult::Presented)
    );
    assert_eq!(bench.released().len(), 1);
    assert!(bench.toasts().is_empty());
    Ok(())
}
#[test]
fn a_presentation_slower_than_the_guard_still_shows_the_bubble_without_paste_error() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (receipt, view) = consume(&bench, 1, None)?;
    assert_eq!(wait(receipt)?, PresentationWait::GuardExpired);
    bench.service.rendered(view.bubble_id);
    assert_eq!(
        bench.service.view().map(|shown| shown.bubble_id),
        Some(view.bubble_id)
    );
    assert_eq!(bench.released().len(), 1, "released once presented");
    assert!(bench.toasts().is_empty());
    Ok(())
}
#[test]
fn a_surface_silent_until_the_hang_limit_is_destroyed_and_reported_once() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (receipt, view) = consume(&bench, 1, None)?;
    drop(receipt); // 006's guard expired long before.
    assert_eq!(bench.timers.fire_all(), vec![PRESENTATION_HANG_LIMIT]);
    assert!(bench
        .surface
        .calls()
        .contains(&SurfaceCall::Destroy(view.bubble_id)));
    bench.service.window_destroyed(view.bubble_id);
    assert_eq!(bench.toasts(), vec![HistoryStatus::Saved]);
    assert_eq!(bench.released().len(), 1);
    bench.service.rendered(view.bubble_id);
    assert_eq!(bench.service.view(), None, "a late render never shows it");
    assert_eq!(bench.toasts().len(), 1);
    Ok(())
}
#[test]
fn unpresentable_before_the_guard_is_left_to_006_and_after_it_toasts_from_010() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (receipt, view) = consume(&bench, 1, None)?;
    bench.service.presentation_failed(view.bubble_id);
    assert_eq!(
        wait(receipt)?,
        PresentationWait::Resolved(PresentationResult::Unpresentable)
    );
    assert!(bench.toasts().is_empty(), "006 shows paste-error for it");
    let (receipt, view) = consume(&bench, 2, None)?;
    drop(receipt);
    bench.service.presentation_failed(view.bubble_id);
    assert_eq!(bench.toasts(), vec![HistoryStatus::Saved]);
    Ok(())
}
#[test]
fn a_new_dictation_before_the_render_supersedes_and_releases_the_slot() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (receipt, _) = consume(&bench, 1, None)?;
    bench.service.dictation_starting(true);
    bench.service.dictation_started(DictationId(2));
    assert_eq!(
        wait(receipt)?,
        PresentationWait::Resolved(PresentationResult::Superseded)
    );
    assert_eq!(bench.released().len(), 1);
    assert!(bench.surface.calls().contains(&SurfaceCall::Hide));
    Ok(())
}
#[test]
fn an_expired_slot_reference_or_a_non_failure_admits_nothing() {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let outcome = support::outcome(
        1,
        None,
        None,
        Some(handy_app_lib::destination::outcome::TranscriptRef(99)),
    );
    assert!(matches!(
        bench.service.consume(&outcome),
        ConsumerAck::Received
    ));
    let inserted = support::outcome(2, None, Some(MethodOutcome::InsertedVerified), None);
    let inserted = handy_app_lib::destination::outcome::InsertionOutcome {
        status: handy_app_lib::destination::outcome::InsertionStatus::InsertionAttempted,
        ..inserted
    };
    assert!(matches!(
        bench.service.consume(&inserted),
        ConsumerAck::Received
    ));
    assert!(bench.surface.presented().is_empty());
}
// @spec AC-005: Copy, Close and new dictation — .specs/features/010-recovery-bubble/spec.md#ac-005
#[test]
fn copy_writes_the_full_transcript_and_closes_and_a_failure_keeps_the_bubble() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    *lock(&bench.copier.answer) = Err(CopyError::Pasteboard);
    assert!(bench
        .service
        .copy(view.bubble_id, view.destination_generation)
        .is_err());
    assert_eq!(
        bench.service.view(),
        Some(view.clone()),
        "open and unchanged"
    );
    assert!(bench.toasts().is_empty(), "no dedicated message");
    *lock(&bench.copier.answer) = Ok(());
    assert!(bench
        .service
        .copy(view.bubble_id, view.destination_generation)
        .is_ok());
    assert_eq!(
        lock(&bench.copier.written).clone(),
        vec![PHRASE.to_owned(); 2]
    );
    assert_eq!(bench.service.view(), None);
    assert_eq!(bench.surface.calls().last(), Some(&SurfaceCall::Hide));
    Ok(())
}
#[test]
fn close_during_a_slow_copy_closes_at_once_and_the_completion_is_ignored() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    let (entered, started) = mpsc::channel();
    let (release, hold) = mpsc::channel();
    *lock(&bench.copier.entered) = Some(entered);
    *lock(&bench.copier.hold) = Some(hold);
    let copying = std::thread::spawn({
        let service = std::sync::Arc::clone(&bench.service);
        let (bubble, generation) = (view.bubble_id, view.destination_generation);
        move || service.copy(bubble, generation).is_ok()
    });
    started.recv()?;
    bench.service.close(view.bubble_id);
    assert_eq!(
        bench.service.view(),
        None,
        "closed at once while the write waits"
    );
    let (_, next) = consume(&bench, 2, None)?;
    bench.service.rendered(next.bubble_id);
    release.send(())?;
    assert!(
        copying.join().map_err(|_| "copy thread")?,
        "the admitted write finishes"
    );
    assert_eq!(
        bench.service.view().map(|shown| shown.bubble_id),
        Some(next.bubble_id),
        "its completion closes nothing"
    );
    Ok(())
}
// @spec AC-004: Drag hides, refusal restores, drop accepted closes — .specs/features/010-recovery-bubble/spec.md#ac-004
#[test]
fn a_refused_drag_shows_the_same_bubble_again_and_an_accepted_one_closes() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    bench.service.begin_drag(view.bubble_id);
    let begun = SurfaceCall::BeginDrag(view.bubble_id, PHRASE.to_owned());
    assert!(bench.surface.calls().contains(&begun), "full payload");
    bench.service.drag_ended(view.bubble_id, false);
    assert!(bench
        .surface
        .calls()
        .contains(&SurfaceCall::ShowAgain(view.clone())));
    bench.service.begin_drag(view.bubble_id);
    bench.service.drag_failed(view.bubble_id);
    assert_eq!(
        bench.service.view(),
        Some(view.clone()),
        "a failed start stays visible"
    );
    bench.service.begin_drag(view.bubble_id);
    bench.service.drag_ended(view.bubble_id, true);
    assert_eq!(bench.service.view(), None);
    bench.service.drag_ended(view.bubble_id, false);
    assert!(!bench
        .surface
        .calls()
        .iter()
        .skip_while(|call| **call != SurfaceCall::Hide)
        .any(|call| matches!(call, SurfaceCall::ShowAgain(_))));
    Ok(())
}
#[test]
fn a_late_read_received_before_or_after_admission_updates_the_same_bubble() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    bench.service.late_read(DictationId(1), 7);
    let outcome = bench.failed(1, None, Some(MethodOutcome::NotRead));
    bench.service.consume(&outcome);
    let view = bench.surface.presented().pop().ok_or("presented")?;
    assert!(view.late_read && view.detail == ReasonDetail::MayHaveReached);
    let second = bench.failed(2, None, Some(MethodOutcome::NotRead));
    bench.service.consume(&second);
    bench.service.late_read(DictationId(2), 7);
    bench.service.late_read(DictationId(2), 7);
    let updates = bench
        .surface
        .calls()
        .into_iter()
        .filter(|call| matches!(call, SurfaceCall::Update(view) if view.late_read))
        .count();
    assert_eq!(updates, 1, "updated in place once, no second bubble");
    assert_eq!(bench.surface.presented().len(), 2);
    Ok(())
}
#[test]
fn there_is_no_timeout_only_the_hang_check_which_is_moot_once_shown() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    assert_eq!(bench.timers.fire_all(), vec![PRESENTATION_HANG_LIMIT]);
    assert_eq!(
        bench.service.view().map(|shown| shown.bubble_id),
        Some(view.bubble_id)
    );
    assert!(!bench
        .surface
        .calls()
        .iter()
        .any(|call| matches!(call, SurfaceCall::Destroy(_))));
    Ok(())
}
#[test]
fn a_new_dictation_closes_a_bubble_shown_for_less_than_four_seconds() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    bench.service.dictation_starting(true);
    bench.service.dictation_started(DictationId(2));
    assert_eq!(bench.service.view(), None);
    assert_eq!(bench.surface.calls().last(), Some(&SurfaceCall::Hide));
    Ok(())
}
// @spec AC-005: A refused start keeps the bubble and its text — .specs/features/010-recovery-bubble/spec.md#ac-005
#[test]
fn a_dictation_whose_recording_fails_to_start_keeps_the_bubble_and_its_text() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let (_, view) = consume(&bench, 1, None)?;
    bench.service.rendered(view.bubble_id);
    // The start is refused (microphone unavailable): `dictation_started` is never called.
    bench.service.dictation_starting(true);
    assert_eq!(bench.service.view(), Some(view.clone()));
    assert!(!bench.surface.calls().contains(&SurfaceCall::Hide));
    assert!(bench
        .service
        .copy(view.bubble_id, view.destination_generation)
        .is_ok());
    assert_eq!(lock(&bench.copier.written).clone(), vec![PHRASE.to_owned()]);
    Ok(())
}
// @spec AC-006: Reduced bubble on Windows and Linux — .specs/features/010-recovery-bubble/spec.md#ac-006
#[test]
fn windows_and_linux_bubbles_have_no_drag_and_linux_recovers_only_insertion_failures() -> TestResult
{
    for platform in [RecoveryPlatform::Windows, RecoveryPlatform::Linux] {
        let bench = Bench::new(platform);
        let (_, view) = consume(&bench, 1, Some(DestinationFailure::InsertionFailed))?;
        assert!(!view.can_drag);
        bench.service.rendered(view.bubble_id);
        bench.service.begin_drag(view.bubble_id);
        assert!(!bench
            .surface
            .calls()
            .iter()
            .any(|call| matches!(call, SurfaceCall::BeginDrag(..))));
    }
    let linux = Bench::new(RecoveryPlatform::Linux);
    let closed = linux.failed(1, Some(DestinationFailure::AppClosed), None);
    assert!(matches!(
        linux.service.consume(&closed),
        ConsumerAck::Received
    ));
    let windows = Bench::new(RecoveryPlatform::Windows);
    assert!(consume(&windows, 1, Some(DestinationFailure::ActivationFailed)).is_ok());
    Ok(())
}
#[test]
fn the_explanation_input_says_kept_in_history_only_for_a_saved_history() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    for (dictation, status, note) in [
        (1, HistoryStatus::Saved, HistoryNote::Kept),
        (2, HistoryStatus::Failed, HistoryNote::NotKept),
        (3, HistoryStatus::NotSaved, HistoryNote::NotKept),
    ] {
        let outcome = handy_app_lib::destination::outcome::InsertionOutcome {
            history_status: status,
            ..bench.failed(dictation, None, None)
        };
        bench.service.consume(&outcome);
        let view = bench.surface.presented().pop().ok_or("presented")?;
        assert_eq!(view.history, note);
    }
    Ok(())
}
#[test]
fn the_transcript_never_enters_the_outcome_event_and_the_view_carries_a_bounded_preview(
) -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let outcome = bench.failed(1, None, None);
    let event = serde_json::to_string(&outcome)?;
    assert!(!event.contains("safe test phrase"));
    bench.service.consume(&outcome);
    let view = serde_json::to_string(&bench.surface.presented().pop().ok_or("presented")?)?;
    assert!(!view.contains("only the full payload carries"));
    Ok(())
}
