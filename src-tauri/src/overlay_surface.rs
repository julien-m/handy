//! State of the recording overlay surface beyond its base card: the extent the indicator asks
//! for (prompt pill and menu of 005, destination badge and menu of 006, notices of 005, 006 and
//! 007) and the hold that keeps the window up while a notice must stay visible after the
//! dictation ended. The window grows to the requested extent, never below the base size of its
//! state and never beyond the monitor that hosts it, and stays anchored to its screen edge
//! (`overlay.rs`). Pure state and geometry only: no window call.
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

/// Largest extent the indicator may request, in logical points (a menu of 51 prompts scrolls).
const MAX_EXTENT: (f64, f64) = (960.0, 720.0);
/// Longest hold one request may set: a lost release never pins the window on screen.
const MAX_HOLD: Duration = Duration::from_secs(15);
/// Extra time after a hold so the notice fade-out completes before the window hides.
const HOLD_GRACE: Duration = Duration::from_millis(300);

/// Overlay surface state shared by the show/hide path and the indicator commands.
pub(crate) struct SurfaceState {
    extent: Mutex<Option<(f64, f64)>>,
    hold_token: AtomicU64,
    held: AtomicBool,
    session_shown: AtomicBool,
    streaming: AtomicBool,
}

/// The overlay surface of this process.
pub(crate) static SURFACE: SurfaceState = SurfaceState::new();

impl SurfaceState {
    /// Empty state: no extent, no hold, no dictation shown.
    pub(crate) const fn new() -> Self {
        Self {
            extent: Mutex::new(None),
            hold_token: AtomicU64::new(0),
            held: AtomicBool::new(false),
            session_shown: AtomicBool::new(false),
            streaming: AtomicBool::new(false),
        }
    }

    /// Store the requested extent (finite, clamped); returns true when it changed.
    pub(crate) fn store_extent(&self, width: f64, height: f64) -> bool {
        let Some(extent) = sanitize_extent(width, height) else {
            return false;
        };
        let mut current = self.extent.lock().unwrap_or_else(PoisonError::into_inner);
        let changed = *current != Some(extent);
        *current = Some(extent);
        changed
    }

    /// Forget the extent once the window is hidden; the next show starts from its base size.
    pub(crate) fn clear_extent(&self) {
        *self.extent.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// `base` grown to the requested extent, never smaller than `base`.
    pub(crate) fn grow(&self, base: (f64, f64)) -> (f64, f64) {
        let extent = *self.extent.lock().unwrap_or_else(PoisonError::into_inner);
        match extent {
            Some((width, height)) => (base.0.max(width), base.1.max(height)),
            None => base,
        }
    }

    /// Record that a dictation shows (`true`, with its form) or hid (`false`) the overlay.
    pub(crate) fn mark_session(&self, shown: bool, streaming: bool) {
        self.session_shown.store(shown, Ordering::SeqCst);
        if shown {
            self.streaming.store(streaming, Ordering::SeqCst);
        }
    }

    /// True while a dictation shows the overlay.
    pub(crate) fn session_shown(&self) -> bool {
        self.session_shown.load(Ordering::SeqCst)
    }

    /// True when the last shown dictation used the Live (streaming) form.
    pub(crate) fn streaming(&self) -> bool {
        self.streaming.load(Ordering::SeqCst)
    }

    /// Start a notice hold; returns its token (a later hold supersedes it).
    pub(crate) fn start_hold(&self) -> u64 {
        self.held.store(true, Ordering::SeqCst);
        self.hold_token.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// End the hold `token` (any hold with `None`); returns false when a newer hold superseded it.
    pub(crate) fn release_hold(&self, token: Option<u64>) -> bool {
        if token.is_some_and(|token| token != self.hold_token.load(Ordering::SeqCst)) {
            return false;
        }
        self.held.store(false, Ordering::SeqCst);
        true
    }

    /// True while a notice holds the window up.
    pub(crate) fn is_held(&self) -> bool {
        self.held.load(Ordering::SeqCst)
    }
}

/// A finite, non-negative extent clamped to [`MAX_EXTENT`]; `None` for a non-finite request.
pub(crate) fn sanitize_extent(width: f64, height: f64) -> Option<(f64, f64)> {
    if !width.is_finite() || !height.is_finite() {
        return None;
    }
    Some((
        width.clamp(0.0, MAX_EXTENT.0),
        height.clamp(0.0, MAX_EXTENT.1),
    ))
}

/// Room a monitor offers the overlay, in the window's logical points: its size in physical
/// pixels divided by the content scale (`scale` × `text_scale`, the Windows accessibility text
/// size that WebView2 applies as a zoom, 1.0 elsewhere), minus the edge offset (logical points
/// scaled by DPI only, as the placement applies it). A degenerate scale leaves no room.
pub(crate) fn monitor_room(
    physical: (u32, u32),
    scale: f64,
    text_scale: f64,
    edge_offset: f64,
) -> (f64, f64) {
    let content_scale = scale * text_scale;
    if !content_scale.is_finite() || content_scale <= 0.0 {
        return (0.0, 0.0);
    }
    let width = f64::from(physical.0) / content_scale;
    let height = (f64::from(physical.1) - edge_offset * scale) / content_scale;
    (width.max(0.0), height.max(0.0))
}

/// `requested` (logical) bounded by the monitor `room`. Business rule: the growth for badges,
/// menus and notices never leaves the screen, but the base size of the form (`base`) is kept even
/// on a monitor smaller than it, so the pill alone is placed exactly as before 005/006.
pub(crate) fn fit_to_monitor(
    requested: (f64, f64),
    base: (f64, f64),
    room: (f64, f64),
) -> (f64, f64) {
    (
        requested.0.min(room.0.max(base.0)),
        requested.1.min(room.1.max(base.1)),
    )
}

/// How long a hold of `duration_ms` keeps the window up: capped, plus the fade-out grace.
pub(crate) fn hold_duration(duration_ms: u32) -> Duration {
    Duration::from_millis(u64::from(duration_ms)).min(MAX_HOLD) + HOLD_GRACE
}

#[cfg(test)]
#[path = "overlay_surface_tests.rs"]
mod tests;
