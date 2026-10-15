//! Session pin through 006's real manager, orchestrator and guard over a scripted workspace
//! (009 AC-001, AC-002, AC-003, SC-003 in automated form): five consecutive dictations started
//! from three other applications all go to the pinned ChatGPT, an unpin restores the start route,
//! a closed pin fails with `pinned_app_not_running` and a running one not brought forward with
//! `activation_failed`, the pin survives both and a restarted application with a new PID, and a
//! new Handy process starts without any pin.
#[path = "pin_enter/support.rs"]
mod support;

use handy_app_lib::destination::context::{DestinationSource, RecoveryDisposition};
use handy_app_lib::destination::outcome::{DestinationFailure, InsertionStatus};
use support::{app, chatgpt, comet, notes, safari, textedit, Method, Rig, TestResult};

// @spec SC-003: Five pinned dictations from three other apps — .specs/features/009-pin-mode-and-enter-rule/spec.md#sc-003
// @spec AC-001: Pin routes this and following dictations — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
#[test]
fn five_dictations_started_from_three_other_apps_all_reach_the_pinned_app() -> TestResult {
    let rig = Rig::new(chatgpt(), vec![chatgpt(), safari(), notes(), textedit()]);
    let first = rig.start(true);
    rig.manager.pin_current(first.dictation_id, 0)?;
    let starts = [safari(), notes(), textedit(), safari(), notes()];
    for (index, front) in starts.into_iter().enumerate() {
        rig.workspace.bring(front.clone());
        let started = rig.start(true);
        let method = Method::succeeding();
        let report = rig.deliver(&started, &method, false);
        assert_eq!(
            report.status,
            InsertionStatus::InsertionAttempted,
            "#{index}"
        );
        let dispatched = method.dispatched();
        assert_eq!(dispatched.len(), 1);
        assert_eq!(dispatched[0].source, DestinationSource::Pinned);
        let target = dispatched[0].target.clone().ok_or("no target")?;
        assert_eq!(
            target.identity,
            chatgpt().identity,
            "#{index} from {}",
            front.display_name
        );
    }
    assert_eq!(rig.workspace.activations().len(), 5);
    Ok(())
}
// @spec AC-002: Unpin restores the route of the start — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-002
#[test]
fn a_cli_pin_during_a_dictation_changes_it_before_insertion_then_unpin_restores_it() -> TestResult {
    let rig = Rig::new(safari(), vec![chatgpt(), safari()]);
    let started = rig.start(true);
    rig.manager.pin_identifier("com.openai.chat")?;
    rig.manager.unpin();
    let method = Method::succeeding();
    rig.deliver(&started, &method, false);
    let dispatched = method.dispatched();
    assert_eq!(dispatched[0].source, DestinationSource::Captured);
    assert_eq!(
        dispatched[0].target.as_ref().map(|target| target.pid),
        Some(safari().pid)
    );
    Ok(())
}
// @spec AC-003: Unavailable pin inserts nothing, pin kept — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-003
#[test]
fn a_pinned_app_closed_then_reopened_fails_then_receives_the_text() -> TestResult {
    let rig = Rig::new(notes(), vec![notes(), safari()]);
    rig.manager.pin_identifier("com.apple.Notes")?;
    rig.workspace.set_running(vec![safari()]);
    rig.workspace.bring(safari());
    let closed = rig.start(true);
    let method = Method::succeeding();
    let report = rig.deliver(&closed, &method, false);
    assert_eq!(
        (report.status, report.reason),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::PinnedAppNotRunning)
        )
    );
    assert!(method.dispatched().is_empty(), "no fallback to Safari");
    assert_eq!(rig.disposition(), Some(RecoveryDisposition::Recover));
    assert!(rig.manager.pinned().is_some(), "the pin stays");
    rig.workspace
        .set_running(vec![app(99, "com.apple.Notes", "Notes"), safari()]);
    let reopened = rig.start(true);
    let report = rig.deliver(&reopened, &Method::succeeding(), false);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    Ok(())
}
#[test]
fn a_running_pin_not_brought_forward_fails_with_activation_failed_and_stays() -> TestResult {
    let rig = Rig::new(notes(), vec![notes(), safari()]);
    rig.manager.pin_identifier("com.apple.Notes")?;
    rig.workspace.bring(safari());
    rig.workspace.activation_works(false);
    let started = rig.start(true);
    let report = rig.deliver(&started, &Method::succeeding(), false);
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    assert_eq!(report.source, DestinationSource::Pinned);
    assert!(rig.manager.pinned().is_some());
    Ok(())
}
#[test]
fn a_pin_activates_with_return_off_and_unpin_then_inserts_at_the_frontmost_app() -> TestResult {
    let rig = Rig::new(safari(), vec![chatgpt(), safari(), comet()]);
    rig.manager.pin_identifier("com.openai.chat")?;
    let pinned = rig.start(false);
    let method = Method::succeeding();
    rig.deliver(&pinned, &method, false);
    assert_eq!(method.dispatched()[0].source, DestinationSource::Pinned);
    rig.workspace.bring(comet());
    let started = rig.start(false);
    rig.manager.unpin();
    let method = Method::succeeding();
    rig.deliver(&started, &method, false);
    let dispatched = method.dispatched();
    assert_eq!(dispatched[0].source, DestinationSource::FrontmostAtEnd);
    Ok(())
}
#[test]
fn paste_method_none_activates_nothing_even_when_pinned() -> TestResult {
    let rig = Rig::new(safari(), vec![chatgpt(), safari()]);
    rig.manager.pin_identifier("com.openai.chat")?;
    let started = rig.start(true);
    let report = rig.deliver(&started, &Method::succeeding(), true);
    assert_eq!(report.status, InsertionStatus::Skipped);
    assert!(rig.workspace.activations().is_empty());
    Ok(())
}
#[test]
fn a_new_handy_process_starts_without_a_pin() -> TestResult {
    let rig = Rig::new(safari(), vec![chatgpt(), safari()]);
    rig.manager.pin_identifier("com.openai.chat")?;
    let restarted = Rig::new(safari(), vec![chatgpt(), safari()]);
    assert_eq!(restarted.manager.pinned(), None);
    let started = restarted.start(true);
    assert_eq!(started.source, DestinationSource::Captured);
    Ok(())
}
