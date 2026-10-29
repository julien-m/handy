// LiveSpec traceability: @spec(AC-001) @spec(AC-002)
//! Shared validation and changes of the favorites (012 FR-001, AC-002): cap, duplicates, blank
//! values, toggle order (newest last) and the add/remove changes applied to locked settings.
use super::*;
use crate::settings::get_default_settings;
use crate::settings_service::errors::SettingsErrorCode;
use crate::settings_service::transaction::SettingsEdit;

fn favorite(bundle: &str) -> FavoriteApp {
    FavoriteApp {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: format!("App {bundle}"),
    }
}
fn bundles(favorites: &[FavoriteApp]) -> Vec<String> {
    favorites
        .iter()
        .map(|favorite| identity_value(&favorite.identity))
        .collect()
}
fn reason(result: Result<(), SettingsError>) -> Option<ValidationReason> {
    result.err().and_then(|error| error.reason())
}
fn apply(change: &DomainChange, settings: &mut AppSettings) -> Result<(), SettingsError> {
    match change.edit() {
        SettingsEdit::Apply(edit) => edit(settings),
        _ => Err(SettingsError::new(SettingsErrorCode::InternalError)),
    }
}

// @spec AC-002: Shared validation, cap of 10 — .specs/features/012-destination-favorites/spec.md#ac-002
#[test]
fn a_full_list_of_unique_named_favorites_is_valid() {
    let favorites: Vec<_> = (0..MAX_FAVORITES)
        .map(|index| favorite(&format!("app.{index}")))
        .collect();
    assert!(validate_favorites(&favorites).is_ok());
}

#[test]
fn one_more_than_the_cap_is_refused_as_full() {
    let favorites: Vec<_> = (0..=MAX_FAVORITES)
        .map(|index| favorite(&format!("app.{index}")))
        .collect();
    assert_eq!(
        reason(validate_favorites(&favorites)),
        Some(ValidationReason::FavoritesFull)
    );
}

#[test]
fn duplicates_blank_values_and_long_names_are_invalid() {
    let blank_identity = FavoriteApp {
        identity: ApplicationIdentity::BundleId("  ".to_owned()),
        ..favorite("x")
    };
    let blank_name = FavoriteApp {
        display_name: " ".to_owned(),
        ..favorite("y")
    };
    let long_name = FavoriteApp {
        display_name: "n".repeat(MAX_DISPLAY_NAME_CHARS + 1),
        ..favorite("z")
    };
    for list in [
        vec![favorite("a"), favorite("a")],
        vec![blank_identity],
        vec![blank_name],
        vec![long_name],
    ] {
        assert_eq!(
            reason(validate_favorites(&list)),
            Some(ValidationReason::FavoriteInvalid)
        );
    }
}

#[test]
fn toggling_appends_the_newest_last_and_removes_a_known_one() -> Result<(), SettingsError> {
    let first = toggled(&[], &favorite("notes"))?;
    let second = toggled(&first, &favorite("claude"))?;
    assert_eq!(bundles(&second), ["notes", "claude"]);
    let removed = toggled(&second, &favorite("notes"))?;
    assert_eq!(bundles(&removed), ["claude"]);
    Ok(())
}

#[test]
fn the_toggle_change_refuses_an_eleventh_favorite_and_keeps_the_list() {
    let mut settings = get_default_settings();
    settings.destination_favorites = (0..MAX_FAVORITES)
        .map(|index| favorite(&format!("app.{index}")))
        .collect();
    let before = settings.destination_favorites.clone();
    let result = apply(&toggle_change(favorite("extra")), &mut settings);
    assert_eq!(reason(result), Some(ValidationReason::FavoritesFull));
    assert_eq!(settings.destination_favorites, before);
}

#[test]
fn add_is_idempotent_and_remove_finds_any_identity_kind() -> Result<(), SettingsError> {
    let mut settings = get_default_settings();
    let path = FavoriteApp {
        identity: ApplicationIdentity::ExecutablePath("C:\\Tools\\app.exe".into()),
        display_name: "Tool".to_owned(),
    };
    apply(&add_change(path.clone()), &mut settings)?;
    apply(&add_change(path.clone()), &mut settings)?;
    assert_eq!(settings.destination_favorites, vec![path.clone()]);
    apply(
        &remove_change(identity_value(&path.identity)),
        &mut settings,
    )?;
    assert!(settings.destination_favorites.is_empty());
    let missing = apply(&remove_change("com.absent".to_owned()), &mut settings);
    assert_eq!(
        missing.err().map(|error| error.code()),
        Some(SettingsErrorCode::NotFound)
    );
    Ok(())
}

#[test]
fn the_stored_shape_is_identity_kind_value_and_display_name() -> Result<(), serde_json::Error> {
    let stored = serde_json::to_value(favorite("com.apple.Notes"))?;
    let read: FavoriteApp = serde_json::from_value(stored.clone())?;
    assert_eq!(read, favorite("com.apple.Notes"));
    assert_eq!(stored["identity"]["kind"], "bundle_id");
    assert_eq!(stored["identity"]["value"], "com.apple.Notes");
    Ok(())
}
