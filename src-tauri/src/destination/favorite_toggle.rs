//! Star of the badge menu (feature 012): which application a menu token designates for the
//! favorites, and the same menu re-ordered once the favorites changed. Neither touches the
//! destination session: a star never changes the destination, its selection version or the pin.
//! The favorites themselves are saved by the caller through the settings service.
use super::context::DictationId;
use super::favorites::FavoriteApp;
use super::manager::DestinationManager;
use super::menu::{describe, still_open};
use super::menu_order::reorder;
use super::session::SelectionRefusal;
use super::snapshot::DestinationMenu;

// @spec FR-003: Star a menu token, session unchanged — .specs/features/012-destination-favorites/spec.md#fr-003
impl DestinationManager {
    /// Badge menu of the exact dictation and version with the saved `favorites` listed last.
    ///
    /// # Errors
    /// `Stale` or `Frozen`.
    pub fn menu_with_favorites(
        &self,
        dictation_id: DictationId,
        version: u64,
        favorites: &[FavoriteApp],
    ) -> Result<DestinationMenu, SelectionRefusal> {
        super::menu::open(self, dictation_id, version, favorites)
    }
    /// The application behind `token` in the open menu of the exact dictation and version, as a
    /// favorite (its identity and the name the menu shows).
    ///
    /// # Errors
    /// `Stale` or `Frozen`; `Unavailable` for a token that is not in the current menu.
    pub fn favorite_target(
        &self,
        dictation_id: DictationId,
        version: u64,
        token: u64,
    ) -> Result<FavoriteApp, SelectionRefusal> {
        let state = self.lock();
        still_open(&state, dictation_id, version)?;
        state
            .menu
            .iter()
            .find(|(known, _)| *known == token)
            .map(|(_, row)| FavoriteApp {
                identity: row.identity.clone(),
                display_name: row.display_name.clone(),
            })
            .ok_or(SelectionRefusal::Unavailable)
    }
    /// The open menu re-ordered for the committed `favorites`, with the same tokens and without
    /// reading the running applications again.
    ///
    /// # Errors
    /// `Stale` or `Frozen` when the dictation moved on meanwhile.
    pub fn relist(
        &self,
        dictation_id: DictationId,
        version: u64,
        favorites: &[FavoriteApp],
    ) -> Result<DestinationMenu, SelectionRefusal> {
        let mut state = self.lock();
        let current = still_open(&state, dictation_id, version)?;
        state.menu = reorder(&state.menu, favorites);
        Ok(describe(&state, dictation_id, version, &current))
    }
}

#[cfg(test)]
#[path = "favorite_toggle_tests.rs"]
mod tests;
