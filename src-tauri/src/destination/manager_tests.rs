//! Manager behavior over a fake workspace: capture at an accepted start (Handy and unreadable
//! foregrounds, setting off, platform without capability), immutability through app switches,
//! menu ordering and opaque choices, cross removal, insertion-entry races, cancellation and
//! stale generations, restart/PID reuse resolution and the serialized Enter claim.
use super::*;
use crate::destination::context::{ApplicationIdentity, DestinationSource, RecoveryDisposition};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::workspace::{resolve_instance, Resolution, RunningApp};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn manager(
    foreground: Foreground,
    running: Vec<RunningApp>,
) -> (Arc<FakeWorkspace>, DestinationManager) {
    let workspace = Arc::new(FakeWorkspace::new(foreground, running));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = DestinationManager::new(workspace.clone(), Arc::new(FakeClock::new()), platform);
    (workspace, manager)
}
fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
fn comet() -> RunningApp {
    app(22, "ai.perplexity.comet", "Comet")
}
fn identity(selection: &Selection) -> Option<&ApplicationIdentity> {
    match selection {
        Selection::Application(destination) => Some(&destination.identity),
        Selection::FrontmostAtEnd => None,
    }
}

#[test]
fn an_accepted_start_captures_the_frontmost_app_and_switches_do_not_change_it() -> TestResult {
    let (workspace, manager) =
        manager(Foreground::Application(chatgpt()), vec![chatgpt(), comet()]);
    let started = manager.commit_start(manager.capture(true));
    workspace.set_foreground(Foreground::Application(comet()));
    manager.mark_processing();
    let frozen = manager.freeze(started.dictation_id, started.destination_generation)?;
    assert_eq!(identity(&frozen.selection), Some(&chatgpt().identity));
    assert_eq!(frozen.selection.source(), DestinationSource::Captured);
    Ok(())
}
#[test]
fn handy_or_an_unreadable_foreground_capture_no_destination() {
    let (_, manager) = manager(Foreground::Handy, vec![]);
    let capture = manager.capture(true);
    assert_eq!(
        (capture.selection, capture.support),
        (Selection::FrontmostAtEnd, DestinationSupport::Supported)
    );
    let (_, manager) = manager_unreadable();
    let capture = manager.capture(true);
    let unreadable = DestinationSupport::Unsupported(SupportReason::FrontmostUnreadable);
    assert_eq!(
        (capture.selection, capture.support),
        (Selection::FrontmostAtEnd, unreadable)
    );
}
fn manager_unreadable() -> (Arc<FakeWorkspace>, DestinationManager) {
    manager(Foreground::Unreadable, vec![])
}
#[test]
fn setting_off_or_platform_without_capability_reads_no_foreground() {
    let (workspace, manager) = manager(Foreground::Application(chatgpt()), vec![]);
    assert_eq!(manager.capture(false).selection, Selection::FrontmostAtEnd);
    let linux = DestinationManager::new(
        workspace.clone(),
        Arc::new(FakeClock::new()),
        PlatformCapabilities::only(&[]),
    );
    let capture = linux.capture(true);
    let unsupported = DestinationSupport::Unsupported(SupportReason::PlatformUnsupported);
    assert_eq!(
        (capture.selection, capture.support),
        (Selection::FrontmostAtEnd, unsupported)
    );
    assert_eq!(
        *workspace
            .frontmost_reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
        0
    );
}
#[test]
fn a_tentative_capture_changes_nothing_until_committed() {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![]);
    let _discarded = manager.capture(true);
    assert_eq!(manager.snapshot(), None);
}
#[test]
fn the_menu_lists_frontmost_first_then_others_and_marks_the_current_one() -> TestResult {
    let notes = app(33, "com.apple.Notes", "Notes");
    let (workspace, manager) = manager(
        Foreground::Application(chatgpt()),
        vec![chatgpt(), notes.clone(), comet()],
    );
    let started = manager.commit_start(manager.capture(true));
    workspace.set_foreground(Foreground::Application(comet()));
    let menu = manager.menu(started.dictation_id, 0)?;
    let names = menu
        .entries
        .iter()
        .map(|entry| entry.display_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["Comet", "ChatGPT", "Notes"]);
    let flags = menu
        .entries
        .iter()
        .map(|entry| (entry.frontmost, entry.current))
        .collect::<Vec<_>>();
    assert_eq!(flags, [(true, false), (false, true), (false, false)]);
    assert!(!menu.frontmost_at_end_current);
    Ok(())
}
#[test]
fn choosing_comet_changes_only_this_dictation_and_a_closed_app_is_refused() -> TestResult {
    let (workspace, manager) =
        manager(Foreground::Application(chatgpt()), vec![chatgpt(), comet()]);
    let started = manager.commit_start(manager.capture(true));
    let menu = manager.menu(started.dictation_id, 0)?;
    let comet_token = menu
        .entries
        .iter()
        .find(|entry| entry.display_name == "Comet")
        .map(|entry| entry.token)
        .ok_or("no Comet")?;
    let chosen = manager.choose(
        started.dictation_id,
        0,
        MenuChoice::Application(comet_token),
    )?;
    assert_eq!(chosen.selection_version, 1);
    workspace.set_running(vec![chatgpt()]);
    let refused = manager.choose(
        started.dictation_id,
        1,
        MenuChoice::Application(comet_token),
    );
    assert_eq!(refused, Err(SelectionRefusal::Unavailable));
    let version = manager
        .snapshot()
        .map(|snapshot| snapshot.selection_version);
    assert_eq!(version, Some(1));
    Ok(())
}
#[test]
fn the_cross_removes_the_destination_and_a_queued_click_after_freeze_is_refused() -> TestResult {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.choose(id, 0, MenuChoice::FrontmostAtEnd)?;
    assert_eq!(
        manager.freeze(id, generation)?.selection,
        Selection::FrontmostAtEnd
    );
    let late = manager.choose(id, 1, MenuChoice::FrontmostAtEnd);
    assert_eq!(late, Err(SelectionRefusal::Frozen));
    assert_eq!(manager.menu(id, 1), Err(SelectionRefusal::Frozen));
    Ok(())
}
#[test]
fn a_stale_choice_cannot_redirect_the_next_dictation() -> TestResult {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt(), comet()]);
    let first = manager.commit_start(manager.capture(true));
    let menu = manager.menu(first.dictation_id, 0)?;
    manager.freeze(first.dictation_id, first.destination_generation)?;
    let second = manager.commit_start(manager.capture(true));
    let token = menu
        .entries
        .last()
        .map(|entry| entry.token)
        .ok_or("no entry")?;
    let late = manager.choose(first.dictation_id, 0, MenuChoice::Application(token));
    assert_eq!(late, Err(SelectionRefusal::Stale));
    assert_eq!(
        manager
            .snapshot()
            .map(|snapshot| snapshot.selection_version),
        Some(second.selection_version)
    );
    Ok(())
}
#[test]
fn cancellation_invalidates_the_generation_and_late_callbacks() -> TestResult {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    let cancelled = manager.cancel().ok_or("nothing cancelled")?;
    assert_eq!(cancelled.disposition, RecoveryDisposition::Cancelled);
    assert!(!manager.is_current(id, generation));
    assert_eq!(
        manager.freeze(id, generation),
        Err(DispositionRefusal::Stale)
    );
    assert_eq!(
        manager.claim_enter(id, generation),
        Err(EnterRefusal::Disposition(RecoveryDisposition::Cancelled))
    );
    Ok(())
}
#[test]
fn restart_and_pid_reuse_resolve_by_identity_and_fail_closed_when_ambiguous() {
    let captured = chatgpt();
    let restarted = app(99, "com.openai.chat", "ChatGPT");
    let reused = app(11, "com.example.other", "Other");
    let resolve =
        |running: &[RunningApp]| resolve_instance(&captured.identity, captured.pid, running);
    assert_eq!(
        resolve(&[captured.clone(), restarted.clone()]),
        Resolution::Running(captured.clone())
    );
    assert_eq!(
        resolve(&[reused.clone(), restarted.clone()]),
        Resolution::Running(restarted.clone())
    );
    assert_eq!(resolve(std::slice::from_ref(&reused)), Resolution::Closed);
    assert_eq!(
        resolve(&[restarted, app(98, "com.openai.chat", "ChatGPT")]),
        Resolution::Ambiguous
    );
}
#[test]
fn concurrent_enter_claims_and_recover_are_serialized_by_the_session_lock() -> TestResult {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.freeze(id, generation)?;
    manager.set_disposition(id, generation, RecoveryDisposition::Insert)?;
    let manager = Arc::new(manager);
    let claims = (0..8)
        .map(|_| {
            let manager = Arc::clone(&manager);
            std::thread::spawn(move || manager.claim_enter(id, generation).is_ok())
        })
        .collect::<Vec<_>>();
    let recover = manager.set_disposition(id, generation, RecoveryDisposition::Recover);
    let granted = claims
        .into_iter()
        .map(|claim| claim.join().unwrap_or(false))
        .filter(|ok| *ok)
        .count();
    assert!(granted <= 1);
    assert_eq!(recover, Ok(RecoveryDisposition::Recover));
    Ok(())
}
#[test]
fn a_failed_insertion_worker_freezes_the_dictation_and_asks_for_recovery() -> TestResult {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.mark_processing();
    let source = manager.fail_insertion(id, generation);
    assert_eq!(source, Some(DestinationSource::Captured));
    let snapshot = manager.snapshot().ok_or("no session")?;
    assert_eq!(snapshot.disposition, RecoveryDisposition::Recover);
    assert!(manager.freeze(id, generation).is_err());
    manager.finish(id, generation);
    assert!(!manager.is_current(id, generation));
    assert_eq!(manager.fail_insertion(id, generation), None);
    Ok(())
}
#[test]
fn a_failed_insertion_worker_of_a_cancelled_dictation_reports_no_source() {
    let (_, manager) = manager(Foreground::Application(chatgpt()), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    manager.cancel();
    let failed = manager.fail_insertion(started.dictation_id, started.destination_generation);
    assert_eq!(failed, None);
}
