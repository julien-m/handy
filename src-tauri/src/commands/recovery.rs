//! Recovery bubble commands (010). Registered in every build so the generated bindings are the
//! same in every rollout gate state; without the installed service (gate closed) they do
//! nothing. The bubble actions answer only the recovery window, so no other webview can read
//! the preview or act on a bubble; every action names the exact bubble.
use crate::recovery::platform::RECOVERY_WINDOW;
use crate::recovery::types::{BubbleId, RecoveryBubbleView};
use crate::settings_service::{
    command_error::SettingsCommandError, tauri_adapter as live_settings, ui_value::UiValue,
};
use tauri::{AppHandle, WebviewWindow};

fn from_recovery_window(window: &WebviewWindow) -> bool {
    window.label() == RECOVERY_WINDOW
}

#[tauri::command]
#[specta::specta]
/// The current bubble's projection (preview, reason, metadata), for the recovery window only.
pub fn recovery_bubble_view(app: AppHandle, window: WebviewWindow) -> Option<RecoveryBubbleView> {
    if !from_recovery_window(&window) {
        return None;
    }
    crate::recovery::app::service(&app)?.view()
}

#[tauri::command]
#[specta::specta]
/// The recovery view rendered bubble `bubble_id` for the first time (it is now presented).
pub fn recovery_bubble_rendered(app: AppHandle, window: WebviewWindow, bubble_id: BubbleId) {
    if let Some(service) =
        crate::recovery::app::service(&app).filter(|_| from_recovery_window(&window))
    {
        service.rendered(bubble_id);
    }
}

#[tauri::command]
#[specta::specta]
/// Close bubble `bubble_id`; the history entry is untouched.
pub fn recovery_bubble_close(app: AppHandle, window: WebviewWindow, bubble_id: BubbleId) {
    if let Some(service) =
        crate::recovery::app::service(&app).filter(|_| from_recovery_window(&window))
    {
        service.close(bubble_id);
    }
}

#[tauri::command]
#[specta::specta]
/// Copy the full transcript of bubble `bubble_id` (deliberate Copy), written on a blocking
/// worker; success closes the bubble, a failure leaves it open (logged without the text).
pub async fn recovery_bubble_copy(
    app: AppHandle,
    window: WebviewWindow,
    bubble_id: BubbleId,
    destination_generation: u64,
) {
    let Some(service) =
        crate::recovery::app::service(&app).filter(|_| from_recovery_window(&window))
    else {
        return;
    };
    let copied = tauri::async_runtime::spawn_blocking(move || {
        // The result is already logged and reflected in the bubble state.
        let _ = service.copy(bubble_id, destination_generation);
    });
    if let Err(error) = copied.await {
        log::error!("Recovery Copy worker failed: {error}");
    }
}

#[tauri::command]
#[specta::specta]
/// Start the native drag of bubble `bubble_id` from the mouse event in progress (macOS).
pub fn recovery_bubble_begin_drag(app: AppHandle, window: WebviewWindow, bubble_id: BubbleId) {
    if let Some(service) =
        crate::recovery::app::service(&app).filter(|_| from_recovery_window(&window))
    {
        service.begin_drag(bubble_id);
    }
}

#[tauri::command]
#[specta::specta]
/// The recovery view needs `width` × `height` logical points.
pub fn recovery_bubble_resize(app: AppHandle, window: WebviewWindow, width: f64, height: f64) {
    if from_recovery_window(&window) && crate::recovery::app::service(&app).is_some() {
        crate::recovery::surface::resize(&app, width, height);
    }
}

#[tauri::command]
#[specta::specta]
/// "Show the bubble when no text field is focused" (010), through the shared settings service:
/// refused off macOS (`unsupported_platform`) and unknown with the gate closed (`not_found`).
pub async fn change_show_recovery_bubble_without_text_field_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> Result<(), SettingsCommandError> {
    live_settings::write(app, crate::recovery::setting::KEY, enabled).await
}
