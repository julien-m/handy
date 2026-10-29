//! Commands of feature 013 (default input per application): the badge menu's "Remember" entry
//! for the exact dictation and `selection_version`, and the settings window's Forget actions.
//! Native reads run on a blocking worker; every write goes through the settings service.
use crate::destination::{
    app::manager,
    context::{ApplicationIdentity, DictationId},
    default_input::{
        capture::{capture, offer, RememberAnswer, RememberOffer, RememberRefusal},
        model::{clear_change, remember_change, remove_change, DefaultInput},
    },
    favorites::identity_value,
    workspace::SystemClock,
};
use crate::settings_service::{
    command_error::SettingsCommandError, domains::DomainChange, errors::ValidationReason,
    tauri_adapter,
};
use tauri::AppHandle;

// @spec FR-001: One Remember entry per destination type — .specs/features/013-default-input-per-app/spec.md#fr-001
#[tauri::command]
#[specta::specta]
/// The "Remember" entry of the badge menu for the destination of the exact dictation and
/// version; `None` without an application destination or where the feature is not offered.
pub async fn default_input_offer(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> Option<RememberOffer> {
    let manager = manager(&app)?;
    let instance = manager
        .destination_instance(dictation_id, selection_version)
        .ok()?;
    let services = manager.default_inputs();
    tauri::async_runtime::spawn_blocking(move || offer(&services, &instance))
        .await
        .ok()
        .flatten()
}

// @spec AC-002: Editable, named, non-password field only — .specs/features/013-default-input-per-app/spec.md#ac-002
#[tauri::command]
#[specta::specta]
/// Remember the destination's focused field or terminal for its application; the destination,
/// its version and the pin never change.
pub async fn remember_default_input(
    app: AppHandle,
    dictation_id: DictationId,
    selection_version: u64,
) -> RememberAnswer {
    let Some(manager) = manager(&app) else {
        return RememberAnswer::Refused(RememberRefusal::Stale);
    };
    let Ok(instance) = manager.destination_instance(dictation_id, selection_version) else {
        return RememberAnswer::Refused(RememberRefusal::Stale);
    };
    let services = manager.default_inputs();
    let writer = app.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        let input = capture(&services, &instance, &SystemClock)?;
        tauri_adapter::write_change(&writer, &remember_change(input), None)
            .map_err(|error| not_saved(&error))?;
        offer(&services, &instance).ok_or(RememberRefusal::Failed)
    })
    .await;
    match saved {
        Ok(Ok(offer)) => RememberAnswer::Saved(offer),
        Ok(Err(refusal)) => RememberAnswer::Refused(refusal),
        Err(_) => RememberAnswer::Refused(RememberRefusal::Failed),
    }
}

/// Why the remembered input was not saved: the list is full, or the write failed (logged).
fn not_saved(error: &SettingsCommandError) -> RememberRefusal {
    if error.reason == Some(ValidationReason::DefaultInputsFull) {
        return RememberRefusal::Full;
    }
    log::warn!("Could not save the remembered input: {}", error.message);
    RememberRefusal::Failed
}

/// Commit `change` and answer the remembered inputs after it.
async fn commit(app: AppHandle, change: DomainChange) -> Result<Vec<DefaultInput>, String> {
    tauri_adapter::blocking(move || {
        Ok(tauri_adapter::write_change(&app, &change, None)?
            .after
            .destination_default_inputs)
    })
    .await
    .map_err(|error| error.message)
}

// @spec AC-008: Remembered inputs listed and forgotten — .specs/features/013-default-input-per-app/spec.md#ac-008
#[tauri::command]
#[specta::specta]
/// Settings window "Forget": forget the remembered input of `identity`.
pub async fn forget_default_input(
    app: AppHandle,
    identity: ApplicationIdentity,
) -> Result<Vec<DefaultInput>, String> {
    commit(app, remove_change(identity_value(&identity))).await
}

#[tauri::command]
#[specta::specta]
/// Settings window "Forget all": forget every remembered input.
pub async fn forget_all_default_inputs(app: AppHandle) -> Result<Vec<DefaultInput>, String> {
    commit(app, clear_change()).await
}
