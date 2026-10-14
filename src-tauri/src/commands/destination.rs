//! Destination session commands of the settings window and the overlay badge (006). Each runs
//! on the async pool (native reads never block the main thread), takes the exact dictation and
//! `selection_version`, and returns metadata only; a refusal is a typed answer, not an error.
use crate::destination::{
    app::{emit_changed, manager},
    context::DictationId,
    session::{DestinationSupport, SelectionRefusal},
    snapshot::{ChoiceAnswer, DestinationSnapshot, MenuAnswer, MenuChoice},
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
/// Runtime support of destination return in this instance (AC-007 reason).
pub async fn destination_support(app: AppHandle) -> Option<DestinationSupport> {
    Some(manager(&app)?.support())
}
// @spec FR-003: Menu for the current dictation only — .specs/features/006-destination-application/spec.md#fr-003
#[tauri::command]
#[specta::specta]
/// Badge menu of the exact dictation and version (frontmost first, then windowed apps).
pub async fn destination_menu(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> MenuAnswer {
    let menu = manager(&app)
        .ok_or(SelectionRefusal::Stale)
        .and_then(|manager| manager.menu(dictation_id, selection_version));
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
