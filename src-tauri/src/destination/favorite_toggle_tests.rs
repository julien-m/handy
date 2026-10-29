// LiveSpec traceability: @spec(AC-001) @spec(AC-004)
//! Star of the badge menu over a fake workspace (012 FR-003, AC-001, AC-004): a star never
//! changes the session, the re-ordered menu keeps its tokens, a favorite that is not running is
//! listed but never chosen, and stale or frozen menus are refused.
use super::*;
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::favorites::toggled;
use crate::destination::snapshot::{MenuChoice, MenuEntry};
use crate::destination::workspace::{Foreground, RunningApp};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn manager(
    front: RunningApp,
    running: Vec<RunningApp>,
) -> (Arc<FakeWorkspace>, DestinationManager) {
    let workspace = Arc::new(FakeWorkspace::new(Foreground::Application(front), running));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = DestinationManager::new(workspace.clone(), Arc::new(FakeClock::new()), platform);
    (workspace, manager)
}
fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
fn claude() -> RunningApp {
    app(22, "com.anthropic.claude", "Claude")
}
fn notes() -> RunningApp {
    app(33, "com.apple.Notes", "Notes")
}
fn favorite_of(running: &RunningApp) -> FavoriteApp {
    FavoriteApp {
        identity: running.identity.clone(),
        display_name: running.display_name.clone(),
    }
}
fn token_of(entries: &[MenuEntry], name: &str) -> Result<u64, String> {
    entries
        .iter()
        .find(|entry| entry.display_name == name)
        .map(|entry| entry.token)
        .ok_or_else(|| format!("no {name}"))
}
fn names(entries: &[MenuEntry]) -> Vec<&str> {
    entries
        .iter()
        .map(|entry| entry.display_name.as_str())
        .collect()
}

// @spec AC-001: Star never changes the session — .specs/features/012-destination-favorites/spec.md#ac-001
#[test]
fn starring_keeps_the_session_and_moves_the_row_to_the_favorites_with_its_token() -> TestResult {
    let (_, manager) = manager(chatgpt(), vec![chatgpt(), claude(), notes()]);
    let started = manager.commit_start(manager.capture(true));
    let id = started.dictation_id;
    let menu = manager.menu_with_favorites(id, 0, &[])?;
    let claude_token = token_of(&menu.entries, "Claude")?;
    let target = manager.favorite_target(id, 0, claude_token)?;
    assert_eq!(target, favorite_of(&claude()));
    let favorites = toggled(&[], &target)?;
    let relisted = manager.relist(id, 0, &favorites)?;
    let last = relisted.entries.last().ok_or("empty menu")?;
    assert_eq!((last.token, last.favorite), (claude_token, true));
    assert_eq!(relisted.entries.len(), menu.entries.len());
    // SC-003: the snapshot (destination, version, source) is untouched by a star.
    assert_eq!(manager.snapshot(), Some(started));
    Ok(())
}

#[test]
fn a_favorite_not_running_is_listed_dimmed_and_cannot_be_chosen() -> TestResult {
    let (workspace, manager) = manager(chatgpt(), vec![chatgpt(), notes()]);
    let started = manager.commit_start(manager.capture(true));
    let favorites = [favorite_of(&notes())];
    workspace.set_running(vec![chatgpt()]);
    let menu = manager.menu_with_favorites(started.dictation_id, 0, &favorites)?;
    let entry = menu.entries.last().ok_or("empty menu")?;
    assert_eq!((entry.favorite, entry.running), (true, false));
    let choice = MenuChoice::Application(entry.token);
    let refused = manager.choose(started.dictation_id, 0, choice);
    assert_eq!(refused, Err(SelectionRefusal::Unavailable));
    assert_eq!(manager.snapshot(), Some(started));
    assert_eq!(workspace.activation_count(), 0);
    Ok(())
}

#[test]
fn a_favorite_running_again_can_be_chosen_from_a_menu_listed_before() -> TestResult {
    let (workspace, manager) = manager(chatgpt(), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let favorites = [favorite_of(&notes())];
    let menu = manager.menu_with_favorites(started.dictation_id, 0, &favorites)?;
    let token = token_of(&menu.entries, &notes().display_name)?;
    workspace.set_running(vec![chatgpt(), notes()]);
    let chosen = manager.choose(started.dictation_id, 0, MenuChoice::Application(token))?;
    assert_eq!(chosen.selection_version, started.selection_version + 1);
    Ok(())
}

#[test]
fn unstarring_a_favorite_not_running_removes_its_row() -> TestResult {
    let (_, manager) = manager(chatgpt(), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let favorites = vec![favorite_of(&notes())];
    let menu = manager.menu_with_favorites(started.dictation_id, 0, &favorites)?;
    let token = token_of(&menu.entries, &notes().display_name)?;
    let target = manager.favorite_target(started.dictation_id, 0, token)?;
    let relisted = manager.relist(started.dictation_id, 0, &toggled(&favorites, &target)?)?;
    assert_eq!(names(&relisted.entries), [chatgpt().display_name.as_str()]);
    Ok(())
}

#[test]
fn stale_frozen_and_unknown_tokens_are_refused() -> TestResult {
    let (_, manager) = manager(chatgpt(), vec![chatgpt()]);
    let started = manager.commit_start(manager.capture(true));
    let id = started.dictation_id;
    let menu = manager.menu_with_favorites(id, 0, &[])?;
    let token = token_of(&menu.entries, "ChatGPT")?;
    assert_eq!(
        manager.favorite_target(id, 1, token),
        Err(SelectionRefusal::Stale)
    );
    assert_eq!(
        manager.favorite_target(id, 0, token + 100),
        Err(SelectionRefusal::Unavailable)
    );
    manager.mark_processing();
    manager.freeze(id, started.destination_generation)?;
    assert_eq!(manager.relist(id, 0, &[]), Err(SelectionRefusal::Frozen));
    Ok(())
}
