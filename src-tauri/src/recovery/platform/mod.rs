//! Platform adapters of the recovery surface: a dedicated nonactivating window colocated with
//! the indicator (never the recording surface made focusable), accepting the first click, plus
//! the macOS accessibility classifier and native drag source. Windows and Linux get the reduced
//! bubble (Copy, Close, explanation) and no drag (spec AC-006).
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub mod macos_ax;
#[cfg(target_os = "macos")]
mod macos_drag;
#[cfg(target_os = "macos")]
mod macos_escape;
#[cfg(target_os = "windows")]
mod windows;

use super::types::{BubbleId, DragResult};
use tauri::{AppHandle, WebviewWindow};

/// Label of the recovery window.
pub const RECOVERY_WINDOW: &str = "recovery_bubble";
/// Page of the recovery window.
pub const RECOVERY_PAGE: &str = "src/overlay/recovery.html";
/// First size of the window (logical points) before the view reports its own.
pub const INITIAL_SIZE: (f64, f64) = (430.0, 200.0);

/// Callback of a drag end: the bubble and how its session ended (drop, release, Escape).
pub type DragEnded = Box<dyn Fn(BubbleId, DragResult) + 'static>;

/// Create the hidden recovery window on the main thread.
///
/// # Errors
/// The creation error text (never containing the transcript).
pub fn create_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    #[cfg(target_os = "macos")]
    return macos::create_window(app);
    #[cfg(target_os = "windows")]
    return windows::create_window(app);
    #[cfg(target_os = "linux")]
    return linux::create_window(app);
}

/// Show the window in front without activating Handy or taking keyboard focus.
///
/// # Errors
/// The show error text.
pub fn show(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return macos::show(window);
    #[cfg(not(target_os = "macos"))]
    return window.show().map_err(|error| error.to_string());
}

/// Hide the window; on macOS also undo the drag's invisibility for the next bubble.
pub fn hide(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::set_dragging(window, false);
    if let Err(error) = window.hide() {
        log::warn!("Could not hide the recovery bubble: {error}");
    }
}

/// Start a native drag of `transcript` (macOS); `ended` reports the end of the session.
///
/// # Errors
/// Why the drag could not start: the bubble then stays visible and unchanged.
pub fn begin_drag(
    window: &WebviewWindow,
    bubble_id: BubbleId,
    transcript: &str,
    ended: DragEnded,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return macos_drag::begin(window, bubble_id, transcript, ended);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (window, bubble_id, transcript, ended);
        Err("native drag exists on macOS only".to_owned())
    }
}

/// Make the bubble visible again after a release where no target took the drop.
pub fn restore_after_drag(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::set_dragging(window, false);
    #[cfg(not(target_os = "macos"))]
    let _ = window;
}

/// The accessibility port of this platform (macOS only; elsewhere no classification runs).
pub fn focused_element_port() -> Box<dyn super::classifier::FocusedElementPort> {
    #[cfg(target_os = "macos")]
    return Box::new(macos_ax::MacFocusedElementPort);
    #[cfg(not(target_os = "macos"))]
    return Box::new(NoAccessibility);
}

/// Platforms without the classifier: never trusted, so no query is ever made.
#[cfg(not(target_os = "macos"))]
struct NoAccessibility;
#[cfg(not(target_os = "macos"))]
impl super::classifier::FocusedElementPort for NoAccessibility {
    fn is_trusted(&mut self) -> bool {
        false
    }
    fn focused_element(
        &mut self,
        _pid: u32,
        _timeout: std::time::Duration,
    ) -> Result<Box<dyn super::classifier::FocusedElement>, super::classifier::AxQueryError> {
        Err(super::classifier::AxQueryError::Unsupported)
    }
}
