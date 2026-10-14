use crate::managers::transcription::TranscriptionManager;
use crate::settings::ModelUnloadTimeout;
use crate::settings_service::{
    command_error::SettingsCommandError, tauri_adapter, ui_value::UiValue,
};
use serde::Serialize;
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Serialize, Type)]
pub struct ModelLoadStatus {
    is_loaded: bool,
    current_model: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn set_model_unload_timeout(
    app: AppHandle,
    timeout: UiValue<ModelUnloadTimeout>,
) -> Result<(), SettingsCommandError> {
    tauri_adapter::write(app, "model_unload_timeout", timeout).await
}

#[tauri::command]
#[specta::specta]
pub fn get_model_load_status(
    transcription_manager: State<Arc<TranscriptionManager>>,
) -> Result<ModelLoadStatus, String> {
    Ok(ModelLoadStatus {
        is_loaded: transcription_manager.is_model_loaded(),
        current_model: transcription_manager.get_current_model(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn unload_model_manually(
    transcription_manager: State<Arc<TranscriptionManager>>,
) -> Result<(), String> {
    transcription_manager.request_unload();
    Ok(())
}
