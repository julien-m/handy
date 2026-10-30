//! Recording indicator surface commands (005, 006, 007): the indicator reports the extent its
//! prompt pill, destination badge, menus and notices need, and holds the window up while a
//! notice must stay visible. Neither command focuses or activates the overlay window.
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
/// Grow (or shrink back) the overlay window to `width` × `height` logical points, never below
/// the base size of the current indicator form; the window stays anchored to its screen edge.
pub fn set_overlay_extent(app: AppHandle, width: f64, height: f64) {
    crate::overlay::set_overlay_extent(&app, width, height);
}

#[tauri::command]
#[specta::specta]
/// Keep the overlay up for `duration_ms` (capped at 15 s) so a notice stays visible after the
/// dictation ended; `0` ends the hold. Ignored when the overlay style is none.
pub fn hold_overlay_for_notice(app: AppHandle, duration_ms: u32) {
    crate::overlay::hold_overlay_for_notice(&app, duration_ms);
}
