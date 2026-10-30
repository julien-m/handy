//! The constants of `src-tauri/src/recovery/platform/mod.rs`, copied verbatim, and the real
//! Windows adapter.
/// Label of the recovery window.
pub const RECOVERY_WINDOW: &str = "recovery_bubble";
/// Page of the recovery window.
pub const RECOVERY_PAGE: &str = "src/overlay/recovery.html";
/// First size of the window (logical points) before the view reports its own.
pub const INITIAL_SIZE: (f64, f64) = (430.0, 200.0);

#[path = "../../../../../../../../src-tauri/src/recovery/platform/windows.rs"]
pub mod windows;

/// Uses the adapter, so the check covers its signature.
pub fn create(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    windows::create_window(app)
}
