//! Order of the badge menu (features 006 and 012), as one pure function: the application frontmost
//! now, then the other running applications, then the favorites in stored order (oldest first, so
//! the most recent favorite is the last row, nearest the badge). Each application appears once:
//! a favorite is listed only among the favorites, running or not.
use super::context::ApplicationIdentity;
use super::favorites::FavoriteApp;
use super::workspace::RunningApp;

/// What one menu row designates. A favorite that is not running has no process (`pid: None`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuTarget {
    pub identity: ApplicationIdentity,
    pub display_name: String,
    pub icon_ref: Option<String>,
    pub pid: Option<u32>,
    /// The application frontmost when the menu opened.
    pub frontmost: bool,
    pub favorite: bool,
}
impl MenuTarget {
    fn running(app: RunningApp, frontmost: bool, favorite: bool) -> Self {
        Self {
            identity: app.identity,
            display_name: app.display_name,
            icon_ref: app.icon_ref,
            pid: Some(app.pid),
            frontmost,
            favorite,
        }
    }
    fn not_running(favorite: &FavoriteApp) -> Self {
        Self {
            identity: favorite.identity.clone(),
            display_name: favorite.display_name.clone(),
            icon_ref: None,
            pid: None,
            frontmost: false,
            favorite: true,
        }
    }
}

// @spec FR-002: Pure bottom-up menu order — .specs/features/012-destination-favorites/spec.md#fr-002
/// Menu rows from top to bottom. Strategy: collect the running applications once each (the
/// frontmost first, duplicates by identity dropped), split them into favorites and the others,
/// then append the favorites in stored order, each taking its running instance when there is
/// one and a not-running row otherwise.
pub fn order_menu(
    frontmost: Option<RunningApp>,
    running: Vec<RunningApp>,
    favorites: &[FavoriteApp],
) -> Vec<MenuTarget> {
    let front = frontmost.as_ref().map(|app| app.identity.clone());
    let rows: Vec<MenuTarget> = unique_running(frontmost, running)
        .into_iter()
        .map(|app| {
            let is_front = front.as_ref() == Some(&app.identity);
            MenuTarget::running(app, is_front, false)
        })
        .collect();
    let mut ordered: Vec<MenuTarget> = rows
        .iter()
        .filter(|row| !is_favorite(favorites, &row.identity))
        .cloned()
        .collect();
    ordered.extend(favorites.iter().map(|favorite| {
        match rows.iter().find(|row| row.identity == favorite.identity) {
            Some(row) => with_favorite(row, true),
            None => MenuTarget::not_running(favorite),
        }
    }));
    ordered
}

/// The frontmost application first, then the running ones, each identity once (first wins).
fn unique_running(frontmost: Option<RunningApp>, running: Vec<RunningApp>) -> Vec<RunningApp> {
    let mut unique: Vec<RunningApp> = Vec::new();
    for app in frontmost.into_iter().chain(running) {
        if !unique.iter().any(|known| known.identity == app.identity) {
            unique.push(app);
        }
    }
    unique
}

fn is_favorite(favorites: &[FavoriteApp], identity: &ApplicationIdentity) -> bool {
    favorites
        .iter()
        .any(|favorite| favorite.identity == *identity)
}

fn with_favorite(row: &MenuTarget, favorite: bool) -> MenuTarget {
    MenuTarget {
        favorite,
        ..row.clone()
    }
}

/// The same rows re-ordered for a new favorites list, keeping each row (and so its token):
/// used after a star toggle, without reading the running applications again. A favorite that is
/// not running and is no longer a favorite disappears; nothing new can appear.
pub fn reorder<T: Clone>(
    rows: &[(T, MenuTarget)],
    favorites: &[FavoriteApp],
) -> Vec<(T, MenuTarget)> {
    let mut ordered: Vec<(T, MenuTarget)> = rows
        .iter()
        .filter(|(_, row)| row.pid.is_some() && !is_favorite(favorites, &row.identity))
        .map(|(token, row)| (token.clone(), with_favorite(row, false)))
        .collect();
    // An unstarred frontmost application goes back to the top, as `order_menu` lists it (stable).
    ordered.sort_by_key(|(_, row)| !row.frontmost);
    ordered.extend(favorites.iter().filter_map(|favorite| {
        rows.iter()
            .find(|(_, row)| row.identity == favorite.identity)
            .map(|(token, row)| (token.clone(), with_favorite(row, true)))
    }));
    ordered
}

#[cfg(test)]
#[path = "menu_order_tests.rs"]
mod tests;
