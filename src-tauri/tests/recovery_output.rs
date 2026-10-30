//! 010 at 006's real insertion boundary (plan Step 4): the production `RecoveryService` as the
//! pre-insertion check of 006's orchestrator and guard, and as the outcome consumer of 006's
//! registry, with 009's real Enter transaction. A not-editable focus recovers before any
//! dispatch with the typed reason `no_text_field` and no Enter; editable or undetermined focus,
//! the setting off, a missing permission and remote views insert as usual (no accessibility
//! query for the last three); the guard reads cancellation and the foreground again after an
//! `Insert` verdict; a failed history save is never presented as kept; and the closed gate
//! installs nothing.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "recovery/support.rs"]
mod support;

use destination::{app, chatgpt, safari, Method, Rig};
use handy_app_lib::automatic_enter::AutomaticEnterPolicy as Policy;
use handy_app_lib::destination::context::RecoveryDisposition;
use handy_app_lib::destination::insertion::{InsertionReport, InsertionRequest, Orchestrator};
use handy_app_lib::destination::outcome::{DestinationFailure, InsertionStatus};
use handy_app_lib::destination::recovery::{Delivery, OutcomeConsumer, OutcomeRegistry};
use handy_app_lib::destination::snapshot::DestinationSnapshot;
use handy_app_lib::recovery::types::{BubbleReason, HistoryNote, ReasonDetail, RecoveryPlatform};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use support::{Bench, Focus, TestResult};

/// One dictation into `rig`'s front app with the setting `enabled`, delivered by 006's
/// orchestrator with 010's check, and 009's Enter decided after it (policy Always).
fn deliver(
    rig: &Rig,
    bench: &Bench,
    enabled: bool,
    return_on: bool,
    cancelled: Arc<AtomicBool>,
) -> (InsertionReport, Method, DestinationSnapshot) {
    bench.service.dictation_starting(enabled);
    let started = rig.start(return_on);
    bench.service.dictation_started(started.dictation_id);
    let enter = rig.enter(&started, Policy::Always);
    rig.manager.mark_processing();
    let orchestrator = Orchestrator {
        recovery: bench.service.clone(),
        ..rig.orchestrator()
    };
    let request = InsertionRequest {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(move || cancelled.load(Ordering::SeqCst)),
    };
    let method = Method::succeeding();
    let report = orchestrator.deliver(request, &method);
    enter.after_insertion(&report);
    (report, method, started)
}
fn live() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

// @spec AC-002: No text field recovers before dispatch, no Enter — .specs/features/010-recovery-bubble/spec.md#ac-002
#[test]
fn a_not_editable_focus_recovers_before_any_dispatch_with_its_typed_reason_and_no_enter() {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, Focus::role("AXList", 11));
    let (report, method, _) = deliver(&rig, &bench, true, true, live());
    assert_eq!(report.status, InsertionStatus::Failed);
    assert_eq!(report.reason, Some(DestinationFailure::NoTextField));
    assert!(method.dispatched().is_empty(), "nothing inserted");
    assert_eq!(rig.disposition(), Some(RecoveryDisposition::Recover));
    assert_eq!(
        rig.keys.count(),
        0,
        "SC-003: no Enter while the bubble is shown"
    );
}
#[test]
fn an_editable_or_undetermined_focus_inserts_as_usual() {
    for focus in [
        Focus::role("AXTextArea", 11),
        Focus::role("AXGroup", 11),
        Focus::role("AXWebArea", 11),
        Focus::role("AXWindow", 11),
        Focus::role("AXList", 99),
    ] {
        let rig = Rig::new(chatgpt(), vec![chatgpt()]);
        let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus);
        let (report, method, _) = deliver(&rig, &bench, true, true, live());
        assert_eq!(report.status, InsertionStatus::InsertionAttempted);
        assert_eq!(method.dispatched().len(), 1);
        assert_eq!(rig.disposition(), Some(RecoveryDisposition::Insert));
    }
}
#[test]
fn the_setting_off_a_missing_permission_and_remote_views_make_no_accessibility_query() {
    let not_editable = || Focus::role("AXList", 11);
    let mut untrusted = not_editable();
    untrusted.trusted = false;
    let screen_sharing = app(11, "com.apple.ScreenSharing", "Screen Sharing");
    let cases = [
        (chatgpt(), not_editable(), false, "setting off"),
        (chatgpt(), untrusted, true, "permission missing"),
        (screen_sharing, not_editable(), true, "remote view"),
    ];
    for (front, focus, enabled, case) in cases {
        let rig = Rig::new(front.clone(), vec![front]);
        let queries = focus.clone();
        let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus);
        let (report, method, _) = deliver(&rig, &bench, enabled, true, live());
        assert_eq!(report.status, InsertionStatus::InsertionAttempted, "{case}");
        assert_eq!(method.dispatched().len(), 1, "{case}");
        assert_eq!(queries.queries(), 0, "{case}");
    }
}
#[test]
fn windows_and_linux_never_classify() {
    for platform in [RecoveryPlatform::Windows, RecoveryPlatform::Linux] {
        let rig = Rig::new(chatgpt(), vec![chatgpt()]);
        let focus = Focus::role("AXList", 11);
        let queries = focus.clone();
        let bench = Bench::with_focus(platform, focus);
        let (report, _, _) = deliver(&rig, &bench, true, true, live());
        assert_eq!(report.status, InsertionStatus::InsertionAttempted);
        assert_eq!(queries.queries(), 0);
    }
}
#[test]
fn the_frontmost_at_the_end_route_is_classified_on_macos_too() {
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, Focus::role("AXList", 11));
    let (report, _, started) = deliver(&rig, &bench, true, false, live());
    assert_eq!(
        started.source,
        handy_app_lib::destination::context::DestinationSource::FrontmostAtEnd
    );
    assert_eq!(report.reason, Some(DestinationFailure::NoTextField));
}
// @spec FR-001: Recheck after an Insert verdict — .specs/features/010-recovery-bubble/spec.md#fr-001
#[test]
fn the_guard_reads_the_foreground_and_cancellation_again_after_an_insert_verdict() {
    let rig = Rig::new(chatgpt(), vec![chatgpt(), safari()]);
    let workspace = rig.workspace.clone();
    let mut focus = Focus::role("AXTextArea", 11);
    focus.during = Some(Arc::new(move || workspace.bring(safari())));
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus);
    let (report, method, _) = deliver(&rig, &bench, true, true, live());
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    assert!(method.dispatched().is_empty());
    let rig = Rig::new(chatgpt(), vec![chatgpt()]);
    let cancelled = live();
    let mut focus = Focus::role("AXTextArea", 11);
    let flag = cancelled.clone();
    focus.during = Some(Arc::new(move || flag.store(true, Ordering::SeqCst)));
    let bench = Bench::with_focus(RecoveryPlatform::MacOs, focus);
    let (report, method, _) = deliver(&rig, &bench, true, true, cancelled);
    assert_eq!(report.status, InsertionStatus::Cancelled);
    assert!(method.dispatched().is_empty());
}
// @spec AC-001: The bubble takes the reason from 006's registry — .specs/features/010-recovery-bubble/spec.md#ac-001
#[test]
fn the_registry_hands_a_receipt_only_when_010_takes_the_reason() -> TestResult {
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let without = OutcomeRegistry::default();
    let with = OutcomeRegistry::default();
    with.register(bench.service.clone());
    let first = bench.failed(1, Some(DestinationFailure::NoTextField), None);
    let Delivery::Delivered { presentation } = without.publish(&first) else {
        return Err("duplicate".into());
    };
    assert!(presentation.is_none(), "no consumer: 006 shows paste-error");
    let Delivery::Delivered { presentation } = with.publish(&first) else {
        return Err("duplicate".into());
    };
    assert!(presentation.is_some());
    let view = bench.surface.presented().pop().ok_or("presented")?;
    assert_eq!(
        (view.reason, view.detail),
        (BubbleReason::NoTextField, ReasonDetail::NotInserted)
    );
    Ok(())
}
#[test]
fn a_failed_history_save_is_never_presented_as_kept() -> TestResult {
    use handy_app_lib::destination::history::save_dictation;
    use handy_app_lib::history_store::{HistoryEntry, HistorySink, NewHistoryEntry};
    struct Failing;
    impl HistorySink for Failing {
        fn save_entry(&self, _entry: NewHistoryEntry) -> anyhow::Result<HistoryEntry> {
            Err(anyhow::anyhow!("disk full"))
        }
    }
    let entry = NewHistoryEntry {
        file_name: None,
        transcription_text: support::PHRASE.to_owned(),
        post_process_requested: false,
        post_processed_text: None,
        post_process_prompt: None,
        processing: Default::default(),
    };
    let record = save_dictation(&Failing, entry, &|| false);
    let bench = Bench::new(RecoveryPlatform::MacOs);
    let outcome = handy_app_lib::destination::outcome::InsertionOutcome {
        history_status: record.status,
        history_entry_id: record.entry_id,
        ..bench.failed(1, None, None)
    };
    bench.service.consume(&outcome);
    let view = bench.surface.presented().pop().ok_or("presented")?;
    assert_eq!(view.history, HistoryNote::NotKept);
    Ok(())
}
#[cfg(not(feature = "destination-return"))]
#[test]
fn the_closed_gate_installs_nothing_and_keeps_devs_paste_error_path() {
    use handy_app_lib::recovery::setting::{fresh_value, KEY};
    use handy_app_lib::settings_service::descriptor::descriptor;
    use handy_app_lib::settings_service::platform_capability::{
        PlatformCapabilities, PlatformCapability,
    };
    // No session opens, so `insert_final_text` takes `dev`'s `paste_as_before` (paste-error),
    // no guard asks 010 and `recovery::app::install` returns before managing anything.
    assert!(!handy_app_lib::destination::rollout::Rollout::CURRENT.opens_sessions());
    assert_eq!(fresh_value(), None);
    assert!(descriptor(KEY).is_none());
    assert!(!PlatformCapabilities::current().supports(PlatformCapability::FocusedFieldDetection));
}
