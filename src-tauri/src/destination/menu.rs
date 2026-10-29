//! Badge menu of the current dictation: the application frontmost when the menu opens first
//! (Handy excluded), then other running applications with windows, deduplicated, then
//! "Frontmost app at the end". Entries are opaque tokens valid for this dictation only; a
//! choice is accepted only for the exact dictation and `selection_version`, before insertion,
//! and only while the chosen application is still running. 012 lists the favorites last
//! (`menu_order`), including those that are not running, which are never launched.
use super::context::{DestinationSource, DictationId};
use super::favorites::FavoriteApp;
use super::manager::{DestinationManager, ManagerState};
use super::menu_order::{order_menu, MenuTarget};
use super::session::{Destination, Selection, SelectionRefusal};
use super::snapshot::{DestinationMenu, DestinationSnapshot, MenuChoice, MenuEntry};
use super::workspace::{resolve_instance, Foreground, Resolution, RunningApp};

// @spec FR-003: Menu lists frontmost first, then windowed apps — .specs/features/006-destination-application/spec.md#fr-003
// @spec FR-002: Favorites last, nearest the badge — .specs/features/012-destination-favorites/spec.md#fr-002
/// Open the menu of `dictation_id` at `version`, with the saved `favorites` (012) listed last.
/// Native reads run outside the session lock; the session is checked again before the tokens
/// are stored.
///
/// # Errors
/// `Stale` for another dictation or version, `Frozen` once insertion has begun.
pub(super) fn open(
    manager: &DestinationManager,
    dictation_id: DictationId,
    version: u64,
    favorites: &[FavoriteApp],
) -> Result<DestinationMenu, SelectionRefusal> {
    still_open(&manager.lock(), dictation_id, version)?;
    let frontmost = front_application(manager.workspace.frontmost());
    let running = manager.workspace.running_windowed_apps();
    let rows = order_menu(frontmost, running, favorites);
    let mut state = manager.lock();
    let current = still_open(&state, dictation_id, version)?;
    state.menu.clear();
    for row in rows {
        state.next_token += 1;
        let token = state.next_token;
        state.menu.push((token, row));
    }
    Ok(describe(&state, dictation_id, version, &current))
}

/// The selection of the exact session, while it still accepts choices.
///
/// # Errors
/// `Stale` for another dictation or version, `Frozen` once insertion has begun.
pub(super) fn still_open(
    state: &ManagerState,
    dictation_id: DictationId,
    version: u64,
) -> Result<Selection, SelectionRefusal> {
    let session = state.session.as_ref().ok_or(SelectionRefusal::Stale)?;
    if session.dictation_id != dictation_id || session.selection_version != version {
        return Err(SelectionRefusal::Stale);
    }
    if !session.accepts_choices() {
        return Err(SelectionRefusal::Frozen);
    }
    Ok(session.selection.clone())
}

/// The menu described from the rows stored in `state`, in their order.
pub(super) fn describe(
    state: &ManagerState,
    dictation_id: DictationId,
    version: u64,
    current: &Selection,
) -> DestinationMenu {
    let pinned = state.pin.as_ref().map(|pin| pin.identity.clone());
    let marks = Marks {
        current,
        pinned: pinned.as_ref(),
    };
    let entries = state
        .menu
        .iter()
        .map(|(token, row)| entry(*token, row, &marks))
        .collect();
    DestinationMenu {
        dictation_id,
        selection_version: version,
        entries,
        frontmost_at_end_current: *current == Selection::FrontmostAtEnd,
    }
}

/// What marks a menu entry besides its row: the current selection and the pin.
struct Marks<'a> {
    current: &'a Selection,
    pinned: Option<&'a super::context::ApplicationIdentity>,
}

/// The application in front, unless it is Handy or unreadable.
fn front_application(foreground: Foreground) -> Option<RunningApp> {
    match foreground {
        Foreground::Application(app) => Some(app),
        Foreground::Handy | Foreground::Unreadable => None,
    }
}

fn entry(token: u64, row: &MenuTarget, marks: &Marks<'_>) -> MenuEntry {
    let is_current = matches!(marks.current, Selection::Application(chosen)
        if chosen.identity == row.identity);
    MenuEntry {
        token,
        display_name: row.display_name.clone(),
        icon_ref: row.icon_ref.clone(),
        current: is_current,
        frontmost: row.frontmost,
        pinned: marks.pinned == Some(&row.identity),
        favorite: row.favorite,
        running: row.pid.is_some(),
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
    still_open(&manager.lock(), dictation_id, version)?;
    let selection = match choice {
        MenuChoice::FrontmostAtEnd => Selection::FrontmostAtEnd,
        MenuChoice::Application(token) => chosen_application(manager, token)?,
    };
    let mut state = manager.lock();
    let selection = back_to_pin(&state, selection);
    let session = state.session.as_mut().ok_or(SelectionRefusal::Stale)?;
    session.choose(dictation_id, version, selection)?;
    let snapshot = DestinationSnapshot::from(&*session);
    // 011 AC-003: another application or the cross releases the remembered input.
    super::field_lifetime::reconcile_field(&mut state);
    Ok(snapshot)
}

/// Business rule (009): choosing the pinned application itself in the menu returns this
/// dictation to its pinned route (source `pinned`); any other choice stays a badge choice.
fn back_to_pin(state: &ManagerState, selection: Selection) -> Selection {
    match (&state.pin, selection) {
        (Some(pin), Selection::Application(chosen)) if chosen.identity == pin.identity => {
            Selection::Application(Destination {
                source: DestinationSource::Pinned,
                ..chosen
            })
        }
        (_, selection) => selection,
    }
}

// @spec AC-004: A favorite not open is never launched — .specs/features/012-destination-favorites/spec.md#ac-004
/// The still-running application behind `token`, as a badge destination. A favorite listed as
/// not running is resolved like any other row: accepted only if exactly one instance of its
/// identity runs now; nothing is ever launched.
fn chosen_application(
    manager: &DestinationManager,
    token: u64,
) -> Result<Selection, SelectionRefusal> {
    let row = manager
        .lock()
        .menu
        .iter()
        .find(|(known, _)| *known == token)
        .map(|(_, row)| row.clone())
        .ok_or(SelectionRefusal::Unavailable)?;
    let instances = manager.workspace.instances(&row.identity);
    // Without a listed process, 0 matches no instance: only a single running one is accepted.
    let listed_pid = row.pid.unwrap_or(0);
    match resolve_instance(&row.identity, listed_pid, &instances) {
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
