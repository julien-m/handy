//! Native surface of the recording indicator around the Handy pill (features 005, 006, 007):
//! placement of the overlay window for its form grown to the indicator's extent and bounded by
//! the monitor, the extent requests of the webview, and the hold that keeps a notice on screen
//! after the dictation. Child module of `overlay.rs`, whose window helpers it uses.
use super::surface::{self, SURFACE};
use super::*;

/// Base window size (logical) of the form last shown: compact pill or Live card.
fn form_base() -> (f64, f64) {
    if SURFACE.streaming() {
        (OVERLAY_STREAM_WIDTH, OVERLAY_STREAM_HEIGHT)
    } else {
        (OVERLAY_WIDTH, OVERLAY_HEIGHT)
    }
}

// @spec FR-003: Indicator surface stays on its monitor — .specs/features/006-destination-application/spec.md#fr-003
/// `requested` (logical) bounded by `monitor` with its content scale (`text_scale`: the Windows
/// accessibility text size, 1.0 elsewhere), keeping the base size of the shown form. macOS
/// measures the work area (menu bar and Dock excluded); the larger edge offset keeps room for
/// either placement.
pub(super) fn fit_to(
    monitor: &tauri::Monitor,
    text_scale: f64,
    requested: (f64, f64),
) -> (f64, f64) {
    #[cfg(target_os = "macos")]
    let size = monitor.work_area().size;
    #[cfg(not(target_os = "macos"))]
    let size = *monitor.size();
    let edge_offset = OVERLAY_TOP_OFFSET.max(OVERLAY_BOTTOM_OFFSET);
    let physical = (size.width, size.height);
    let room = surface::monitor_room(physical, monitor.scale_factor(), text_scale, edge_offset);
    surface::fit_to_monitor(requested, form_base(), room)
}

/// Size and position the overlay window for `width` × `height` (logical), flush to its edge and
/// bounded by the monitor under the cursor (Windows bounds it in `place_windows_overlay`).
pub(super) fn place_overlay(
    app_handle: &AppHandle,
    overlay_window: &tauri::webview::WebviewWindow,
    width: f64,
    height: f64,
) {
    #[cfg(not(target_os = "windows"))]
    let (width, height) = get_monitor_with_cursor(app_handle).map_or((width, height), |monitor| {
        fit_to(&monitor, 1.0, (width, height))
    });

    #[cfg(target_os = "linux")]
    if LAYER_SHELL_ACTIVE.load(Ordering::SeqCst) {
        let position = settings::get_settings(app_handle).overlay_position;
        match overlay_window.gtk_window() {
            Ok(gtk_window) => configure_layer_shell_surface(&gtk_window, position, width, height),
            Err(error) => log::error!("Failed to access GTK overlay window: {error}"),
        }
        return;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = overlay_window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
        if let Some((x, y)) = calculate_overlay_position(app_handle, width, height) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }
    }
    #[cfg(target_os = "windows")]
    if let Err(error) = place_windows_overlay(app_handle, overlay_window, width, height) {
        log::error!("Failed to place recording overlay: {error}");
    }
}

/// Windows only: re-assert "topmost" after showing, then the bounds, because the pre-show move
/// crosses the DPI boundary and tao's WM_DPICHANGED reflow clobbers the first placement.
pub(super) fn reassert_after_show(
    app_handle: &AppHandle,
    overlay_window: &tauri::webview::WebviewWindow,
    width: f64,
    height: f64,
) {
    #[cfg(target_os = "windows")]
    {
        force_overlay_topmost(overlay_window);
        if let Err(error) = place_windows_overlay(app_handle, overlay_window, width, height) {
            log::error!("Failed to re-assert recording overlay position: {error}");
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (app_handle, overlay_window, width, height);
}

// @spec FR-003: Indicator surface grows for badge and menus — .specs/features/006-destination-application/spec.md#fr-003
/// The indicator asks for `width` × `height` (logical) to show its badge, prompt pill, menus or
/// notices: store it and, when the window is up, grow or shrink it (never below the base size of
/// its state, never beyond its monitor), anchored to its screen edge. Never focuses or activates
/// the window.
pub fn set_overlay_extent(app_handle: &AppHandle, width: f64, height: f64) {
    if !SURFACE.store_extent(width, height) {
        return;
    }
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        let Some(window) = handle.get_webview_window("recording_overlay") else {
            return;
        };
        if window.is_visible().unwrap_or(false) {
            let (width, height) = current_form_dimensions();
            place_overlay(&handle, &window, width, height);
        }
    });
}

// @spec FR-004: Reason stays visible at least 4 s — .specs/features/006-destination-application/spec.md#fr-004
/// Keep the overlay up for `duration_ms` (capped) so a notice outlives the dictation; `0` ends
/// the hold. The window hides at the end of the hold unless a dictation shows it again. Nothing
/// is shown when the overlay style is none (the settings window carries the notice then).
pub fn hold_overlay_for_notice(app_handle: &AppHandle, duration_ms: u32) {
    if duration_ms == 0 {
        return release_notice_hold(app_handle, None);
    }
    if settings::get_settings(app_handle).overlay_style == OverlayStyle::None {
        return;
    }
    let token = SURFACE.start_hold();
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || reveal_for_notice(&handle));
    let handle = app_handle.clone();
    let wait = surface::hold_duration(duration_ms);
    std::thread::spawn(move || {
        std::thread::sleep(wait);
        release_notice_hold(&handle, Some(token));
    });
}

/// Window size of the last shown indicator form (compact or Live), grown to the extent.
fn current_form_dimensions() -> (f64, f64) {
    overlay_dimensions(if SURFACE.streaming() {
        "streaming"
    } else {
        "recording"
    })
}

/// Show the overlay again for a notice whose dictation already hid it.
fn reveal_for_notice(app_handle: &AppHandle) {
    let Some(window) = app_handle.get_webview_window("recording_overlay") else {
        return;
    };
    if SURFACE.session_shown() || window.is_visible().unwrap_or(false) {
        return;
    }
    let (width, height) = current_form_dimensions();
    place_overlay(app_handle, &window, width, height);
    let _ = window.show();
    reassert_after_show(app_handle, &window, width, height);
}

/// End hold `token` (any hold with `None`) and hide the window unless a dictation shows it.
fn release_notice_hold(app_handle: &AppHandle, token: Option<u64>) {
    if !SURFACE.release_hold(token) {
        return;
    }
    let handle = app_handle.clone();
    let _ = app_handle.run_on_main_thread(move || {
        if SURFACE.session_shown() || SURFACE.is_held() {
            return;
        }
        if let Some(window) = handle.get_webview_window("recording_overlay") {
            let _ = window.hide();
            SURFACE.clear_extent();
        }
    });
}
