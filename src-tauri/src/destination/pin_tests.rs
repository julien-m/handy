//! Session pin over the real manager, orchestrator and guard with a fake workspace and clock
//! (009 AC-001, AC-002, AC-003): the pin wins at start while 006's capture is kept, badge choices
//! and the cross change the current dictation only, Unpin before/at/after the freeze, return off,
//! CLI pins with and without a dictation, lookups, several instances, restarts, an unreadable
//! foreground and the exact reason of each pinned failure.
use super::*;
use crate::destination::context::{DestinationContext, NoRecoveryConsumer, RecoveryDisposition};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::guard::DispatchGuard;
use crate::destination::insertion::{InsertionPort, InsertionRequest, Orchestrator};
use crate::destination::outcome::{DestinationFailure, InsertionStatus, MethodResult};
use crate::destination::session::{DestinationSupport, SupportReason};
use crate::destination::snapshot::MenuChoice;
use crate::destination::workspace::Foreground;
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::{Arc, Mutex, PoisonError};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
fn comet() -> RunningApp {
    app(22, "ai.perplexity.comet", "Comet")
}
fn safari() -> RunningApp {
    app(33, "com.apple.Safari", "Safari")
}
fn setup(
    front: RunningApp,
    running: Vec<RunningApp>,
) -> (Arc<FakeWorkspace>, Arc<DestinationManager>) {
    let workspace = Arc::new(FakeWorkspace::new(Foreground::Application(front), running));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let clock = Arc::new(FakeClock::new());
    let manager = DestinationManager::new(workspace.clone(), clock, platform);
    (workspace, Arc::new(manager))
}
fn source_and_name(manager: &DestinationManager) -> Option<(DestinationSource, String)> {
    let snapshot = manager.snapshot()?;
    let name = match snapshot.selection {
        crate::destination::snapshot::SelectionView::Application { display_name, .. } => {
            display_name
        }
        crate::destination::snapshot::SelectionView::FrontmostAtEnd => "frontmost".to_owned(),
    };
    Some((snapshot.source, name))
}
fn pinned(name: &str) -> Option<(DestinationSource, String)> {
    Some((DestinationSource::Pinned, name.to_owned()))
}

#[test]
fn a_pin_routes_this_and_every_following_dictation_whatever_is_in_front() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt(), comet(), safari()]);
    let first = manager.commit_start(manager.capture(true));
    manager.pin_current(first.dictation_id, 0)?;
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    for front in [safari(), comet(), safari()] {
        workspace.set_foreground(Foreground::Application(front));
        manager.commit_start(manager.capture(true));
        assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    }
    assert_eq!(
        manager.pinned().map(|pinned| pinned.display_name),
        Some("ChatGPT".to_owned())
    );
    Ok(())
}
#[test]
fn another_app_from_the_badge_changes_the_current_dictation_only() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt(), comet()]);
    manager.pin_identifier("com.openai.chat")?;
    workspace.set_foreground(Foreground::Application(comet()));
    let started = manager.commit_start(manager.capture(true));
    let menu = manager.menu(started.dictation_id, 0)?;
    let token = menu
        .entries
        .iter()
        .find(|entry| entry.display_name == "Comet")
        .map(|entry| entry.token)
        .ok_or("no Comet")?;
    manager.choose(started.dictation_id, 0, MenuChoice::Application(token))?;
    assert_eq!(
        source_and_name(&manager),
        Some((DestinationSource::Badge, "Comet".to_owned()))
    );
    assert!(manager.pinned().is_some(), "the pin is unchanged");
    manager.commit_start(manager.capture(true));
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    Ok(())
}
#[test]
fn the_cross_while_pinned_is_frontmost_at_end_for_this_dictation_only() -> TestResult {
    let (_, manager) = setup(chatgpt(), vec![chatgpt()]);
    manager.pin_identifier("com.openai.chat")?;
    let started = manager.commit_start(manager.capture(true));
    manager.choose(started.dictation_id, 0, MenuChoice::FrontmostAtEnd)?;
    let frozen = manager.freeze(started.dictation_id, started.destination_generation)?;
    assert_eq!(frozen.selection, Selection::FrontmostAtEnd);
    assert!(manager.pinned().is_some());
    Ok(())
}
// @spec AC-002: Unpin before freeze restores the capture — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-002
#[test]
fn unpin_before_freeze_restores_the_captured_app_even_after_a_badge_override() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt(), comet(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    workspace.set_foreground(Foreground::Application(safari()));
    let started = manager.commit_start(manager.capture(true));
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    let menu = manager.menu(started.dictation_id, 0)?;
    let comet_token = menu
        .entries
        .iter()
        .find(|entry| entry.display_name == "Comet")
        .map(|entry| entry.token)
        .ok_or("no Comet")?;
    manager.choose(
        started.dictation_id,
        0,
        MenuChoice::Application(comet_token),
    )?;
    let unpinned = manager.unpin_from_badge(started.dictation_id, 1)?;
    assert_eq!(unpinned.source, DestinationSource::Captured);
    assert_eq!(unpinned.pin, None);
    assert_eq!(
        source_and_name(&manager),
        Some((DestinationSource::Captured, "Safari".to_owned()))
    );
    workspace.set_foreground(Foreground::Application(comet()));
    manager.commit_start(manager.capture(true));
    assert_eq!(
        source_and_name(&manager),
        Some((DestinationSource::Captured, "Comet".to_owned()))
    );
    Ok(())
}
#[test]
fn unpin_with_return_off_gives_the_frontmost_at_end_route() -> TestResult {
    let (_, manager) = setup(safari(), vec![chatgpt(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    let started = manager.commit_start(manager.capture(false));
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    manager.unpin();
    let frozen = manager.freeze(started.dictation_id, started.destination_generation)?;
    assert_eq!(frozen.selection, Selection::FrontmostAtEnd);
    Ok(())
}
#[test]
fn unpin_or_repin_after_the_freeze_changes_only_the_next_dictations() -> TestResult {
    let (_, manager) = setup(safari(), vec![chatgpt(), comet(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    let started = manager.commit_start(manager.capture(true));
    let frozen = manager.freeze(started.dictation_id, started.destination_generation)?;
    manager.unpin();
    manager.pin_identifier("ai.perplexity.comet")?;
    assert_eq!(frozen.selection.source(), DestinationSource::Pinned);
    assert_eq!(
        source_and_name(&manager),
        pinned("ChatGPT"),
        "frozen provenance"
    );
    manager.commit_start(manager.capture(true));
    assert_eq!(source_and_name(&manager), pinned("Comet"));
    Ok(())
}
#[test]
fn a_cli_pin_without_a_dictation_applies_from_the_next_one() -> TestResult {
    let (_, manager) = setup(safari(), vec![chatgpt(), safari()]);
    let change = manager.pin_identifier("com.openai.chat")?;
    assert_eq!(change.snapshot, None);
    assert_eq!(change.pinned.display_name, "ChatGPT");
    manager.commit_start(manager.capture(true));
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    Ok(())
}
#[test]
fn a_pin_or_unpin_from_the_cli_during_a_dictation_acts_like_the_badge() -> TestResult {
    let (_, manager) = setup(safari(), vec![chatgpt(), safari()]);
    manager.commit_start(manager.capture(true));
    let change = manager.pin_identifier("com.openai.chat")?;
    let snapshot = change.snapshot.ok_or("no snapshot")?;
    assert_eq!(
        (snapshot.source, snapshot.selection_version),
        (DestinationSource::Pinned, 1)
    );
    let unpinned = manager.unpin().ok_or("no snapshot")?;
    assert_eq!(
        (unpinned.source, unpinned.selection_version),
        (DestinationSource::Captured, 2)
    );
    assert_eq!(manager.unpin(), None, "a second unpin is a no-op");
    Ok(())
}
#[test]
fn the_lookup_refuses_unknown_identities_names_and_handy_itself() {
    let own = app(std::process::id(), "computer.handy", "Handy");
    let (_, manager) = setup(safari(), vec![chatgpt(), own]);
    for identifier in ["com.example.missing", "ChatGPT", "11", "", "computer.handy"] {
        assert_eq!(
            manager.pin_identifier(identifier),
            Err(NoRunningApplication),
            "{identifier}"
        );
    }
    assert_eq!(manager.pinned(), None);
}
#[test]
fn choosing_the_pinned_app_in_the_menu_returns_to_the_pinned_route() -> TestResult {
    let (_, manager) = setup(chatgpt(), vec![chatgpt(), comet()]);
    manager.pin_identifier("com.openai.chat")?;
    let started = manager.commit_start(manager.capture(true));
    manager.choose(started.dictation_id, 0, MenuChoice::FrontmostAtEnd)?;
    let menu = manager.menu(started.dictation_id, 1)?;
    let token = menu
        .entries
        .iter()
        .find(|entry| entry.display_name == "ChatGPT")
        .map(|entry| entry.token)
        .ok_or("no ChatGPT")?;
    manager.choose(started.dictation_id, 1, MenuChoice::Application(token))?;
    assert_eq!(source_and_name(&manager), pinned("ChatGPT"));
    Ok(())
}
#[test]
fn cancellation_keeps_the_pin_and_pinning_needs_an_application_destination() -> TestResult {
    let (_, manager) = setup(chatgpt(), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(false));
    assert_eq!(
        manager.pin_current(started.dictation_id, 0),
        Err(SelectionRefusal::Unavailable)
    );
    let captured = manager.commit_start(manager.capture(true));
    manager.pin_current(captured.dictation_id, 0)?;
    manager.cancel();
    assert!(manager.pinned().is_some());
    Ok(())
}

/// Insertion method double: passes the guard and records the dispatched context.
#[derive(Default)]
struct Recorder {
    contexts: Mutex<Vec<DestinationContext>>,
}
impl InsertionPort for Recorder {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        match guard.check() {
            Ok(context) => {
                self.contexts
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(context);
                MethodResult::Legacy(Ok(()))
            }
            Err(refusal) => MethodResult::Legacy(Err(refusal.to_string())),
        }
    }
}
fn orchestrator(manager: &Arc<DestinationManager>) -> Orchestrator {
    let clock = Arc::new(FakeClock::new());
    Orchestrator {
        manager: Arc::clone(manager),
        clock: clock.clone(),
        pause: clock,
        recovery: Arc::new(NoRecoveryConsumer),
    }
}
fn deliver(
    manager: &Arc<DestinationManager>,
    started: &DestinationSnapshot,
    recorder: &Recorder,
) -> crate::destination::insertion::InsertionReport {
    manager.mark_processing();
    let request = InsertionRequest {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(|| false),
    };
    orchestrator(manager).deliver(request, recorder)
}
fn disposition(manager: &DestinationManager) -> Option<RecoveryDisposition> {
    manager.snapshot().map(|snapshot| snapshot.disposition)
}

#[test]
fn a_pinned_dictation_activates_the_pin_even_with_return_off() -> TestResult {
    let (workspace, manager) = setup(safari(), vec![chatgpt(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    let started = manager.commit_start(manager.capture(false));
    let recorder = Recorder::default();
    let report = deliver(&manager, &started, &recorder);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(workspace.activation_count(), 1);
    let context = report.context.ok_or("no context")?;
    assert_eq!(context.source, DestinationSource::Pinned);
    assert_eq!(context.target.map(|target| target.pid), Some(chatgpt().pid));
    Ok(())
}
// @spec AC-003: pinned_app_not_running, pin kept — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-003
#[test]
fn a_pinned_app_not_running_inserts_nothing_with_its_reason_and_keeps_the_pin() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    workspace.set_running(vec![safari()]);
    workspace.set_foreground(Foreground::Application(safari()));
    let started = manager.commit_start(manager.capture(true));
    let recorder = Recorder::default();
    let report = deliver(&manager, &started, &recorder);
    assert_eq!(
        (report.status, report.reason),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::PinnedAppNotRunning)
        )
    );
    assert!(recorder
        .contexts
        .lock()
        .map(|list| list.is_empty())
        .unwrap_or(false));
    assert_eq!(disposition(&manager), Some(RecoveryDisposition::Recover));
    assert!(manager.pinned().is_some());
    workspace.set_running(vec![app(77, "com.openai.chat", "ChatGPT"), safari()]);
    let reopened = manager.commit_start(manager.capture(true));
    let report = deliver(&manager, &reopened, &recorder);
    assert_eq!(
        report.status,
        InsertionStatus::InsertionAttempted,
        "new PID"
    );
    Ok(())
}
#[test]
fn a_running_pin_not_brought_forward_fails_with_activation_failed() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt(), safari()]);
    manager.pin_identifier("com.openai.chat")?;
    workspace.set_foreground(Foreground::Application(safari()));
    *workspace
        .activation_works
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = false;
    let started = manager.commit_start(manager.capture(true));
    let report = deliver(&manager, &started, &Recorder::default());
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    assert_eq!(disposition(&manager), Some(RecoveryDisposition::Recover));
    assert!(manager.pinned().is_some());
    Ok(())
}
#[test]
fn several_instances_resolve_the_pinned_one_else_the_only_one_else_fail_closed() -> TestResult {
    let (workspace, manager) = setup(
        chatgpt(),
        vec![chatgpt(), app(12, "com.openai.chat", "ChatGPT")],
    );
    manager.pin_identifier("com.openai.chat")?;
    let hint = manager
        .lock()
        .pin
        .as_ref()
        .map(|pin| pin.pid_hint)
        .ok_or("no pin")?;
    workspace.set_foreground(Foreground::Application(safari()));
    let started = manager.commit_start(manager.capture(true));
    let report = deliver(&manager, &started, &Recorder::default());
    let target = report.context.and_then(|context| context.target);
    assert_eq!(
        target.map(|target| target.pid),
        Some(hint),
        "the pinned instance"
    );
    workspace.set_running(vec![
        app(40, "com.openai.chat", "ChatGPT"),
        app(41, "com.openai.chat", "ChatGPT"),
    ]);
    workspace.set_foreground(Foreground::Application(safari()));
    let ambiguous = manager.commit_start(manager.capture(true));
    let report = deliver(&manager, &ambiguous, &Recorder::default());
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    Ok(())
}
#[test]
fn an_unreadable_foreground_with_a_pin_inserts_nowhere_with_activation_failed() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt()]);
    manager.pin_identifier("com.openai.chat")?;
    workspace.set_foreground(Foreground::Unreadable);
    // An unreadable foreground never shows the activated application in front.
    *workspace
        .activation_works
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = false;
    let started = manager.commit_start(manager.capture(true));
    assert_eq!(
        started.support,
        DestinationSupport::Unsupported(SupportReason::FrontmostUnreadable)
    );
    let recorder = Recorder::default();
    let report = deliver(&manager, &started, &recorder);
    assert_eq!(report.reason, Some(DestinationFailure::ActivationFailed));
    assert!(recorder
        .contexts
        .lock()
        .map(|list| list.is_empty())
        .unwrap_or(false));
    assert!(manager.pinned().is_some());
    Ok(())
}
#[test]
fn an_unreadable_foreground_without_a_pin_inserts_as_today() -> TestResult {
    let (workspace, manager) = setup(chatgpt(), vec![chatgpt()]);
    workspace.set_foreground(Foreground::Unreadable);
    let started = manager.commit_start(manager.capture(true));
    let report = deliver(&manager, &started, &Recorder::default());
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(report.source, DestinationSource::FrontmostAtEnd);
    Ok(())
}
#[test]
fn a_platform_without_destination_support_never_applies_a_pin() -> TestResult {
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(safari()),
        vec![chatgpt(), safari()],
    ));
    let linux = DestinationManager::new(
        workspace,
        Arc::new(FakeClock::new()),
        PlatformCapabilities::only(&[]),
    );
    linux.pin_identifier("com.openai.chat")?;
    linux.commit_start(linux.capture(true));
    assert_eq!(
        linux.snapshot().map(|snapshot| snapshot.source),
        Some(DestinationSource::FrontmostAtEnd)
    );
    Ok(())
}
