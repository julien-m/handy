//! Favorite destination applications (feature 012): an ordered list, oldest first, of 006's
//! stable application identities with a display name, kept in the setting `destination_favorites`.
//! The badge star and the settings CLI change it only through the changes built here, so one
//! validation and one revisioned settings commit serve every surface; nothing writes the store.
use super::context::ApplicationIdentity;
use crate::settings::AppSettings;
use crate::settings_service::domains::{ChangeKind, DomainChange};
use crate::settings_service::errors::{SettingsError, ValidationReason};
use serde::{Deserialize, Serialize};

/// Settings key of the favorites.
pub const FAVORITES_KEY: &str = "destination_favorites";
/// Most favorites kept. They form a fixed, non-scrolling menu section: ten 36-pixel rows plus
/// the menu footer still fit above the indicator on a 768-pixel-high screen (spec AC-002).
pub const MAX_FAVORITES: usize = 10;
/// Longest display name kept, in characters (an application name, never a window title).
pub const MAX_DISPLAY_NAME_CHARS: usize = 100;

/// One favorite application: its 006 identity and the name shown while it is not running.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct FavoriteApp {
    pub identity: ApplicationIdentity,
    pub display_name: String,
}

// @spec FR-001: One shared validation, AC-002: Cap and unique identities — .specs/features/012-destination-favorites/spec.md#fr-001
/// The list unchanged when it holds at most [`MAX_FAVORITES`] entries with unique identities, a
/// non-blank identity value and a non-blank display name of at most [`MAX_DISPLAY_NAME_CHARS`].
///
/// # Errors
/// `validation_failed` on `destination_favorites`: `favorites_full` above the cap,
/// `favorite_invalid` for a blank or duplicate identity or a blank or too long name.
pub fn validate_favorites(favorites: &[FavoriteApp]) -> Result<(), SettingsError> {
    if favorites.len() > MAX_FAVORITES {
        return Err(refusal(ValidationReason::FavoritesFull));
    }
    for (index, favorite) in favorites.iter().enumerate() {
        let duplicate = favorites[..index]
            .iter()
            .any(|earlier| earlier.identity == favorite.identity);
        if duplicate || !is_valid(favorite) {
            return Err(refusal(ValidationReason::FavoriteInvalid));
        }
    }
    Ok(())
}

fn is_valid(favorite: &FavoriteApp) -> bool {
    let name = favorite.display_name.trim();
    !identity_value(&favorite.identity).trim().is_empty()
        && !name.is_empty()
        && name.chars().count() <= MAX_DISPLAY_NAME_CHARS
}

fn refusal(reason: ValidationReason) -> SettingsError {
    SettingsError::validation(FAVORITES_KEY, reason)
}

/// The textual value of an identity (bundle identifier, AUMID or executable path), as the CLI
/// names it.
pub fn identity_value(identity: &ApplicationIdentity) -> String {
    match identity {
        ApplicationIdentity::BundleId(value) | ApplicationIdentity::Aumid(value) => value.clone(),
        ApplicationIdentity::ExecutablePath(path) => path.to_string_lossy().into_owned(),
    }
}

// @spec AC-001: Star adds or removes, newest last — .specs/features/012-destination-favorites/spec.md#ac-001
/// Business rule: starring an application that is not a favorite appends it (the most recent
/// favorite is the last one, nearest the badge); starring a favorite removes it.
///
/// # Errors
/// The refusals of [`validate_favorites`] for the resulting list (an 11th favorite).
pub fn toggled(
    favorites: &[FavoriteApp],
    app: &FavoriteApp,
) -> Result<Vec<FavoriteApp>, SettingsError> {
    let mut next = favorites.to_vec();
    if let Some(position) = next.iter().position(|known| known.identity == app.identity) {
        next.remove(position);
    } else {
        next.push(app.clone());
    }
    validate_favorites(&next)?;
    Ok(next)
}

/// Settings change that toggles `app` on the locked latest settings (pure: it may run twice,
/// once for a preview and once for the commit).
pub fn toggle_change(app: FavoriteApp) -> DomainChange {
    let apply = Box::new(move |locked: &mut AppSettings| {
        locked.destination_favorites = toggled(&locked.destination_favorites, &app)?;
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

/// Settings change that adds `app` (a no-op when it is already a favorite).
pub fn add_change(app: FavoriteApp) -> DomainChange {
    let apply = Box::new(move |locked: &mut AppSettings| {
        if locked
            .destination_favorites
            .iter()
            .any(|known| known.identity == app.identity)
        {
            return Ok(());
        }
        locked.destination_favorites = toggled(&locked.destination_favorites, &app)?;
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

/// Position of the favorite whose identity value is `identifier` (any identity kind).
pub fn find_identifier(favorites: &[FavoriteApp], identifier: &str) -> Option<usize> {
    favorites
        .iter()
        .position(|favorite| identity_value(&favorite.identity) == identifier)
}

/// Settings change that removes the favorite named `identifier`, decided on the locked latest
/// settings.
///
/// # Errors
/// At commit, `not_found` (application lookup message) when no favorite has this identity.
pub fn remove_change(identifier: String) -> DomainChange {
    let apply = Box::new(move |locked: &mut AppSettings| {
        let position = find_identifier(&locked.destination_favorites, &identifier)
            .ok_or_else(SettingsError::no_favorite)?;
        locked.destination_favorites.remove(position);
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

#[cfg(test)]
#[path = "favorites_tests.rs"]
mod tests;
