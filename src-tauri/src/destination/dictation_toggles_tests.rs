//! The two in-dictation keys over the real manager with a fake workspace and clock (014 AC-001
//! to AC-004, SC-001): re-routing, second press, no pin, after the insertion freeze, the pin left
//! intact for the next dictation, and the Enter claim refused only for a skipped dictation.
use super::*;
use crate::destination::context::{DestinationSource, RecoveryDisposition};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::session::EnterRefusal;
use crate::destination::workspace::{Foreground, RunningApp};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn claude() -> RunningApp {
    app(11, "com.anthropic.claudefordesktop", "Claude")
}
fn codex() -> RunningApp {
    app(22, "com.openai.codex", "Codex")
}
fn setup() -> (Arc<FakeWorkspace>, DestinationManager) {
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(claude()),
        vec![claude(), codex()],
    ));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = DestinationManager::new(workspace.clone(), Arc::new(FakeClock::new()), platform);
    (workspace, manager)
}
/// Claude pinned, Codex in front, a dictation started with return on.
fn pinned_dictation(
) -> Result<(DestinationManager, DestinationSnapshot), Box<dyn std::error::Error>> {
    let (workspace, manager) = setup();
    manager.pin_identifier("com.anthropic.claudefordesktop")?;
    workspace.set_foreground(Foreground::Application(codex()));
    let started = manager.commit_start(manager.capture(true));
    assert_eq!(started.source, DestinationSource::Pinned);
    Ok((manager, started))
}
fn ids(snapshot: &DestinationSnapshot) -> (DictationId, u64) {
    (snapshot.dictation_id, snapshot.destination_generation)
}

// @spec AC-001: Re-route to the app in front at the end — .specs/features/014-in-dictation-keys/spec.md#ac-001
#[test]
fn a_press_reroutes_a_pinned_dictation_and_a_second_press_restores_the_pin() -> TestResult {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    let rerouted = manager
        .toggle_active_app(id, generation)?
        .ok_or("no change")?;
    assert_eq!(rerouted.source, DestinationSource::FrontmostAtEnd);
    assert_eq!(rerouted.selection_version, started.selection_version + 1);
    assert_eq!(rerouted.pin, started.pin, "the badge still offers Unpin");
    let restored = manager
        .toggle_active_app(id, generation)?
        .ok_or("no change")?;
    assert_eq!(restored.source, DestinationSource::Pinned);
    assert_eq!(restored.selection, started.selection);
    Ok(())
}

// @spec SC-001: Pin intact for the next dictation — .specs/features/014-in-dictation-keys/spec.md#sc-001
#[test]
fn the_pin_is_intact_and_routes_the_next_dictation() -> TestResult {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    let version = manager.lock().pin.as_ref().map(|pin| pin.pin_version);
    manager.toggle_active_app(id, generation)?;
    let frozen = manager.freeze(id, generation)?;
    assert_eq!(frozen.selection, Selection::FrontmostAtEnd);
    let next = manager.commit_start(manager.capture(true));
    assert_eq!(next.source, DestinationSource::Pinned);
    assert_eq!(
        manager.lock().pin.as_ref().map(|pin| pin.pin_version),
        version
    );
    Ok(())
}

#[test]
fn without_a_pin_or_on_a_frontmost_route_the_key_changes_nothing() -> TestResult {
    let (_, manager) = setup();
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = ids(&started);
    assert_eq!(manager.toggle_active_app(id, generation)?, None);
    assert_eq!(manager.snapshot(), Some(started));

    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    let version = started.selection_version;
    manager.choose(
        id,
        version,
        crate::destination::snapshot::MenuChoice::FrontmostAtEnd,
    )?;
    assert_eq!(
        manager.toggle_active_app(id, generation)?,
        None,
        "the cross set it"
    );
    Ok(())
}

#[test]
fn after_an_unpin_or_a_badge_choice_the_second_press_restores_nothing() -> TestResult {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    manager.toggle_active_app(id, generation)?;
    manager.unpin();
    assert_eq!(
        manager.toggle_active_app(id, generation)?,
        None,
        "nothing pinned"
    );

    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    let rerouted = manager
        .toggle_active_app(id, generation)?
        .ok_or("no change")?;
    let menu = manager.menu(id, rerouted.selection_version)?;
    let token = menu
        .entries
        .first()
        .map(|entry| entry.token)
        .ok_or("no app")?;
    let choice = crate::destination::snapshot::MenuChoice::Application(token);
    let chosen = manager.choose(id, rerouted.selection_version, choice)?;
    let cross = crate::destination::snapshot::MenuChoice::FrontmostAtEnd;
    manager.choose(id, chosen.selection_version, cross)?;
    assert_eq!(
        manager.toggle_active_app(id, generation)?,
        None,
        "the cross set it"
    );
    Ok(())
}

#[test]
fn after_the_insertion_freeze_or_for_another_dictation_both_keys_are_refused() -> TestResult {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    assert!(manager.accepts_keys(id, generation));
    assert_eq!(
        manager.toggle_active_app(DictationId(id.0 + 1), generation),
        Err(SelectionRefusal::Stale)
    );
    manager.freeze(id, generation)?;
    assert!(!manager.accepts_keys(id, generation));
    assert_eq!(
        manager.toggle_active_app(id, generation),
        Err(SelectionRefusal::Frozen)
    );
    assert_eq!(
        manager.toggle_skip_enter(id, generation).err(),
        Some(SelectionRefusal::Frozen)
    );
    let frozen_route = manager.snapshot().map(|snapshot| snapshot.source);
    assert_eq!(frozen_route, Some(DestinationSource::Pinned));
    Ok(())
}

/// Start, optionally skip Enter `presses` times, freeze and allow insertion, then claim.
fn claim_after(presses: usize) -> Result<Result<(), EnterRefusal>, Box<dyn std::error::Error>> {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    for _ in 0..presses {
        manager.toggle_skip_enter(id, generation)?;
    }
    manager.freeze(id, generation)?;
    manager.set_disposition(id, generation, RecoveryDisposition::Insert)?;
    Ok(manager.claim_enter(id, generation).map(|_| ()))
}

// @spec AC-003: Enter skipped for this dictation only — .specs/features/014-in-dictation-keys/spec.md#ac-003
#[test]
fn a_skipped_dictation_refuses_the_enter_claim_and_a_second_press_restores_it() -> TestResult {
    assert_eq!(claim_after(0)?, Ok(()));
    assert_eq!(claim_after(1)?, Err(EnterRefusal::UserSkipped));
    assert_eq!(claim_after(2)?, Ok(()));
    Ok(())
}

#[test]
fn the_skip_shows_in_the_snapshot_and_keeps_the_route_and_version() -> TestResult {
    let (manager, started) = pinned_dictation()?;
    let (id, generation) = ids(&started);
    let skipped = manager.toggle_skip_enter(id, generation)?;
    assert!(skipped.enter_skipped);
    assert_eq!(skipped.selection_version, started.selection_version);
    assert_eq!(skipped.source, started.source);
    let rerouted = manager
        .toggle_active_app(id, generation)?
        .ok_or("no change")?;
    assert!(rerouted.enter_skipped, "both keys combine");
    let next = manager.commit_start(manager.capture(true));
    assert!(
        !next.enter_skipped,
        "the next dictation starts without the mark"
    );
    Ok(())
}
