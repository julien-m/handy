//! Destination session commands of the settings window and the overlay badge (006). Each runs
//! on the async pool (native reads never block the main thread), takes the exact dictation and
//! `selection_version`, and returns metadata only; a refusal is a typed answer, not an error.
//! 012 adds the star toggle, saved through the settings service.
use crate::destination::{
    app::{emit_changed, manager},
    context::DictationId,
    favorites,
    rollout::Rollout,
    session::{DestinationSupport, SelectionRefusal},
    snapshot::{
        ChoiceAnswer, DestinationSnapshot, FavoriteAnswer, FavoriteNotSaved, MenuAnswer, MenuChoice,
    },
};
use crate::settings::get_settings;
use crate::settings_service::{
    command_error::SettingsCommandError, errors::ValidationReason, tauri_adapter,
};
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
/// Current session snapshot, if a dictation has started.
pub async fn destination_snapshot(app: AppHandle) -> Option<DestinationSnapshot> {
    manager(&app)?.snapshot()
}
#[tauri::command]
#[specta::specta]
/// Runtime support of destination return in this instance (AC-007 reason); `None` when this
/// build does not offer destination return (rollout gate closed), so the settings hide it.
pub async fn destination_support(app: AppHandle) -> Option<DestinationSupport> {
    if !Rollout::CURRENT.offers_setting() {
        return None;
    }
    Some(manager(&app)?.support())
}
// @spec FR-003: Menu for the current dictation only — .specs/features/006-destination-application/spec.md#fr-003
#[tauri::command]
#[specta::specta]
/// Badge menu of the exact dictation and version (frontmost first, then windowed apps, then
/// the saved favorites of 012).
pub async fn destination_menu(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> MenuAnswer {
    let favorites = get_settings(&app).destination_favorites;
    let menu = manager(&app)
        .ok_or(SelectionRefusal::Stale)
        .and_then(|manager| {
            manager.menu_with_favorites(dictation_id, selection_version, &favorites)
        });
    match menu {
        Ok(menu) => MenuAnswer::Menu(menu),
        Err(refusal) => MenuAnswer::Refused(refusal),
    }
}
#[tauri::command]
#[specta::specta]
/// Apply a badge choice for the exact dictation and version; refused unchanged otherwise.
pub async fn choose_destination(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
    choice: MenuChoice,
) -> ChoiceAnswer {
    let chosen = manager(&app)
        .ok_or(SelectionRefusal::Stale)
        .and_then(|manager| manager.choose(dictation_id, selection_version, choice));
    match chosen {
        Ok(snapshot) => {
            emit_changed(&app, &snapshot);
            ChoiceAnswer::Chosen(snapshot)
        }
        Err(refusal) => ChoiceAnswer::Refused(refusal),
    }
}
// @spec FR-001: One-click removal for this dictation only — .specs/features/006-destination-application/spec.md#fr-001
#[tauri::command]
#[specta::specta]
/// The badge cross: same mutation as "Frontmost app at the end"; the setting is untouched.
pub async fn remove_destination(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> ChoiceAnswer {
    choose_destination(
        app,
        dictation_id,
        selection_version,
        MenuChoice::FrontmostAtEnd,
    )
    .await
}
// @spec AC-001: Pin and Unpin from the badge menu — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
#[tauri::command]
#[specta::specta]
/// Badge "Pin <destination>" (009): pin the current destination of the exact dictation and
/// version; it stays the destination of the next dictations until unpinned or Handy quits.
pub async fn pin_destination(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> ChoiceAnswer {
    let pinned = manager(&app)
        .ok_or(SelectionRefusal::Stale)
        .and_then(|manager| manager.pin_current(dictation_id, selection_version));
    answer(&app, pinned)
}
#[tauri::command]
#[specta::specta]
/// Badge "Unpin" (009): before insertion this dictation gets back the route of its start.
pub async fn unpin_destination(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> ChoiceAnswer {
    let unpinned = manager(&app)
        .ok_or(SelectionRefusal::Stale)
        .and_then(|manager| manager.unpin_from_badge(dictation_id, selection_version));
    answer(&app, unpinned)
}
// @spec FR-003: Star toggle saved, menu re-ordered — .specs/features/012-destination-favorites/spec.md#fr-003
#[tauri::command]
#[specta::specta]
/// Badge star (012): add the application behind `token` to the favorites, or remove it, through
/// the settings service, then answer the same menu re-ordered. The destination, its selection
/// version and the pin never change; the menu stays open.
pub async fn toggle_destination_favorite(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
    token: u64,
) -> FavoriteAnswer {
    let Some(manager) = manager(&app) else {
        return FavoriteAnswer::Refused(SelectionRefusal::Stale);
    };
    let target = match manager.favorite_target(dictation_id, selection_version, token) {
        Ok(target) => target,
        Err(refusal) => return FavoriteAnswer::Refused(refusal),
    };
    let writer = app.clone();
    let committed = tauri_adapter::blocking(move || {
        let change = favorites::toggle_change(target);
        Ok(tauri_adapter::write_change(&writer, &change, None)?
            .after
            .destination_favorites)
    })
    .await;
    match committed {
        Ok(saved) => match manager.relist(dictation_id, selection_version, &saved) {
            Ok(menu) => FavoriteAnswer::Menu(menu),
            Err(refusal) => FavoriteAnswer::Refused(refusal),
        },
        Err(error) => FavoriteAnswer::NotSaved(not_saved(&error)),
    }
}
/// Why the favorites were not saved: full, or any other refusal or failure (logged).
fn not_saved(error: &SettingsCommandError) -> FavoriteNotSaved {
    if error.reason == Some(ValidationReason::FavoritesFull) {
        return FavoriteNotSaved::Full;
    }
    log::warn!(
        "Could not save the favorite applications: {}",
        error.message
    );
    FavoriteNotSaved::Failed
}
/// Notify the UI of an accepted change and answer it.
fn answer(app: &AppHandle, changed: Result<DestinationSnapshot, SelectionRefusal>) -> ChoiceAnswer {
    match changed {
        Ok(snapshot) => {
            emit_changed(app, &snapshot);
            ChoiceAnswer::Chosen(snapshot)
        }
        Err(refusal) => ChoiceAnswer::Refused(refusal),
    }
}
