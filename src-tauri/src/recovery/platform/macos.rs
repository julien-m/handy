//! macOS recovery window: an `NSPanel` that never becomes key or main, joins every Space,
//! floats at the indicator's level and accepts the first click, created like the recording
//! overlay panel (`overlay.rs`) but separate from it.
use super::{INITIAL_SIZE, RECOVERY_PAGE, RECOVERY_WINDOW};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

tauri_panel! {
    panel!(RecoveryBubblePanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

// @spec AC-003: Nonactivating bubble accepting the first click — .specs/features/010-recovery-bubble/spec.md#ac-003
/// Build the hidden panel; must run on the main thread.
pub(super) fn create_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    let (width, height) = INITIAL_SIZE;
    let panel = PanelBuilder::<_, RecoveryBubblePanel>::new(app, RECOVERY_WINDOW)
        .url(WebviewUrl::App(RECOVERY_PAGE.into()))
        .title("Handy")
        .level(PanelLevel::Status)
        .size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .has_shadow(false)
        .transparent(true)
        .no_activate(true)
        .corner_radius(0.0)
        .style_mask(StyleMask::empty().borderless().nonactivating_panel())
        .with_window(|window| {
            window
                .decorations(false)
                .transparent(true)
                .focusable(false)
                .accept_first_mouse(true)
        })
        .collection_behavior(
            CollectionBehavior::new()
                .can_join_all_spaces()
                .full_screen_auxiliary(),
        )
        .build()
        .map_err(|error| error.to_string())?;
    panel.hide();
    panel
        .to_window()
        .ok_or_else(|| "recovery panel has no window".to_owned())
}

/// Order the panel front without activating Handy.
pub(super) fn show(window: &WebviewWindow) -> Result<(), String> {
    let panel = window
        .app_handle()
        .get_webview_panel(RECOVERY_WINDOW)
        .map_err(|error| format!("{error:?}"))?;
    panel.show();
    Ok(())
}

/// During a drag the panel is invisible and lets the pointer through; afterwards it is back.
pub(super) fn set_dragging(window: &WebviewWindow, dragging: bool) {
    let Ok(panel) = window.app_handle().get_webview_panel(RECOVERY_WINDOW) else {
        return;
    };
    panel.set_alpha_value(if dragging { 0.0 } else { 1.0 });
    panel.set_ignores_mouse_events(dragging);
}
