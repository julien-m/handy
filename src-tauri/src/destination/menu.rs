//! Badge menu of the current dictation: the application frontmost when the menu opens first
//! (Handy excluded), then other running applications with windows, deduplicated, then
//! "Frontmost app at the end". Entries are opaque tokens valid for this dictation only; a
//! choice is accepted only for the exact dictation and `selection_version`, before insertion,
//! and only while the chosen application is still running.
use super::context::{DestinationSource, DictationId};
use super::manager::{DestinationManager, ManagerState};
use super::session::{Destination, Selection, SelectionRefusal};
use super::snapshot::{DestinationMenu, DestinationSnapshot, MenuChoice, MenuEntry};
use super::workspace::{resolve_instance, Foreground, Resolution, RunningApp};

// @spec FR-003: Menu lists frontmost first, then windowed apps — .specs/features/006-destination-application/spec.md#fr-003
/// Open the menu of `dictation_id` at `version`. Native reads run outside the session lock;
/// the session is checked again before the tokens are stored.
///
/// # Errors
/// `Stale` for another dictation or version, `Frozen` once insertion has begun.
pub(super) fn open(
    manager: &DestinationManager,
    dictation_id: DictationId,
    version: u64,
) -> Result<DestinationMenu, SelectionRefusal> {
    let current = check(manager, dictation_id, version)?;
    let frontmost = front_application(manager.workspace.frontmost());
    let apps = ordered_apps(frontmost.clone(), manager.workspace.running_windowed_apps());
    let mut state = manager.lock();
    let still_open = state.session.as_ref().is_some_and(|session| {
        session.dictation_id == dictation_id && session.selection_version == version
    });
    if !still_open {
        return Err(SelectionRefusal::Stale);
    }
    let entries = store_entries(&mut state, apps, &current, frontmost.as_ref());
    let frontmost_at_end_current = current == Selection::FrontmostAtEnd;
    Ok(DestinationMenu {
        dictation_id,
        selection_version: version,
        entries,
        frontmost_at_end_current,
    })
}

/// Replace the menu tokens with fresh ones for `apps` and describe each entry.
fn store_entries(
    state: &mut ManagerState,
    apps: Vec<RunningApp>,
    current: &Selection,
    frontmost: Option<&RunningApp>,
) -> Vec<MenuEntry> {
    state.menu.clear();
    apps.into_iter()
        .map(|app| {
            state.next_token += 1;
            let token = state.next_token;
            let described = entry(token, &app, current, frontmost);
            state.menu.push((token, app));
            described
        })
        .collect()
}
/// The current selection of the exact, still-open session.
fn check(
    manager: &DestinationManager,
    dictation_id: DictationId,
    version: u64,
) -> Result<Selection, SelectionRefusal> {
    let state = manager.lock();
    let session = state.session.as_ref().ok_or(SelectionRefusal::Stale)?;
    if session.dictation_id != dictation_id || session.selection_version != version {
        return Err(SelectionRefusal::Stale);
    }
    if !session.accepts_choices() {
        return Err(SelectionRefusal::Frozen);
    }
    Ok(session.selection.clone())
}

/// The application in front, unless it is Handy or unreadable.
fn front_application(foreground: Foreground) -> Option<RunningApp> {
    match foreground {
        Foreground::Application(app) => Some(app),
        Foreground::Handy | Foreground::Unreadable => None,
    }
}
/// Frontmost first, then the others without duplicating its identity.
fn ordered_apps(frontmost: Option<RunningApp>, running: Vec<RunningApp>) -> Vec<RunningApp> {
    let mut apps = frontmost.into_iter().collect::<Vec<_>>();
    for app in running {
        if !apps.iter().any(|known| known.identity == app.identity) {
            apps.push(app);
        }
    }
    apps
}

fn entry(
    token: u64,
    app: &RunningApp,
    current: &Selection,
    frontmost: Option<&RunningApp>,
) -> MenuEntry {
    let is_current =
        matches!(current, Selection::Application(chosen) if chosen.identity == app.identity);
    MenuEntry {
        token,
        display_name: app.display_name.clone(),
        icon_ref: app.icon_ref.clone(),
        current: is_current,
        frontmost: frontmost.is_some_and(|front| front.identity == app.identity),
    }
}

// @spec FR-001: Badge choice changes only this dictation — .specs/features/006-destination-application/spec.md#fr-001
/// Apply a badge choice (the cross sends `FrontmostAtEnd`). A token must belong to the
/// current menu and its application must still be running; the session is unchanged on any
/// refusal and the persistent setting is never touched.
///
/// # Errors
/// `Stale`, `Frozen`, or `Unavailable` for an unknown token or a closed application.
pub(super) fn choose(
    manager: &DestinationManager,
    dictation_id: DictationId,
    version: u64,
    choice: MenuChoice,
) -> Result<DestinationSnapshot, SelectionRefusal> {
    check(manager, dictation_id, version)?;
    let selection = match choice {
        MenuChoice::FrontmostAtEnd => Selection::FrontmostAtEnd,
        MenuChoice::Application(token) => chosen_application(manager, token)?,
    };
    let mut state = manager.lock();
    let session = state.session.as_mut().ok_or(SelectionRefusal::Stale)?;
    session.choose(dictation_id, version, selection)?;
    Ok(DestinationSnapshot::from(&*session))
}

/// The still-running application behind `token`, as a badge destination.
fn chosen_application(
    manager: &DestinationManager,
    token: u64,
) -> Result<Selection, SelectionRefusal> {
    let app = manager
        .lock()
        .menu
        .iter()
        .find(|(known, _)| *known == token)
        .map(|(_, app)| app.clone())
        .ok_or(SelectionRefusal::Unavailable)?;
    let instances = manager.workspace.instances(&app.identity);
    match resolve_instance(&app.identity, app.pid, &instances) {
        Resolution::Running(running) => Ok(Selection::Application(Destination {
            identity: running.identity,
            display_name: running.display_name,
            icon_ref: running.icon_ref,
            captured_pid: running.pid,
            source: DestinationSource::Badge,
        })),
        Resolution::Closed | Resolution::Ambiguous => Err(SelectionRefusal::Unavailable),
    }
}
