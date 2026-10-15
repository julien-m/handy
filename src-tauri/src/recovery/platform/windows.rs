//! Windows recovery window: a borderless, always-on-top webview that is not focusable and
//! accepts the first click, built like the recording overlay window. No drag (spec AC-006).
//! Its first-click and Narrator observation is pending release evidence.
use super::{INITIAL_SIZE, RECOVERY_PAGE, RECOVERY_WINDOW};
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Build the hidden window.
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
    builder.build().map_err(|error| error.to_string())
}
