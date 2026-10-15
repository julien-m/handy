//! Linux recovery window: the recording overlay's window type and focus hints, as a GTK layer
//! surface with `KeyboardMode::None` when available (`HANDY_NO_GTK_LAYER_SHELL=1` keeps a
//! regular window, as for the overlay). No drag; it only appears for "Insertion failed"
//! (spec AC-006). Its X11/Wayland/Orca observation is pending release evidence.
use super::{INITIAL_SIZE, RECOVERY_PAGE, RECOVERY_WINDOW};
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Build the hidden window and make it a layer surface when the compositor supports it.
pub(super) fn create_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    let (width, height) = INITIAL_SIZE;
    let mut builder =
        WebviewWindowBuilder::new(app, RECOVERY_WINDOW, WebviewUrl::App(RECOVERY_PAGE.into()))
            .title("Handy")
            .resizable(false)
            .inner_size(width, height)
            .shadow(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .accept_first_mouse(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .transparent(true)
            .focusable(false)
            .focused(false)
            .visible(false);
    if let Some(data_dir) = crate::portable::data_dir() {
        builder = builder.data_directory(data_dir.join("webview"));
    }
    let window = builder.build().map_err(|error| error.to_string())?;
    init_layer_shell(&window);
    Ok(window)
}

fn init_layer_shell(window: &WebviewWindow) {
    if crate::utils::env_flag_enabled("HANDY_NO_GTK_LAYER_SHELL")
        || !gtk_layer_shell::is_supported()
    {
        return;
    }
    let Ok(gtk_window) = window.gtk_window() else {
        return;
    };
    gtk_window.init_layer_shell();
    gtk_window.set_layer(Layer::Overlay);
    gtk_window.set_keyboard_mode(KeyboardMode::None);
    gtk_window.set_exclusive_zone(0);
    let bottom = crate::settings::get_settings(window.app_handle()).overlay_position
        == crate::settings::OverlayPosition::Bottom;
    let (edge, opposite) = if bottom {
        (Edge::Bottom, Edge::Top)
    } else {
        (Edge::Top, Edge::Bottom)
    };
    gtk_window.set_anchor(edge, true);
    gtk_window.set_anchor(opposite, false);
}
