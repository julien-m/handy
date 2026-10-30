//! Monitor the recording indicator stays on during one dictation (006 AC-009). Every placement
//! (state change, Live extent growth, position setting, notice, 010's recovery bubble) asks for
//! a monitor. Without a destination it is the monitor under the cursor, as before. Once the
//! dictation has a destination (captured at the start or pinned, 009), the indicator keeps the
//! monitor it was placed on until the next dictation starts, so switching to an application on
//! another screen no longer moves it. A held monitor that is disconnected falls back to the
//! cursor, which then becomes the held monitor. Child module of `overlay.rs`.
use super::*;
use std::sync::{Mutex, PoisonError};

/// Identity of a monitor across `available_monitors` calls: its physical origin and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MonitorKey {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl MonitorKey {
    fn of(monitor: &tauri::Monitor) -> Self {
        Self {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width,
            height: monitor.size().height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Anchor {
    /// Monitor of the last placement of the current dictation.
    monitor: Option<MonitorKey>,
    /// The dictation has a destination: placements keep `monitor`.
    held: bool,
}

/// Placement anchor of the current dictation. Pure state: no window or monitor call.
pub(crate) struct AnchorState(Mutex<Anchor>);

/// The placement anchor of this process.
pub(crate) static ANCHOR: AnchorState = AnchorState::new();

impl AnchorState {
    /// No dictation yet: placements follow the cursor.
    pub(crate) const fn new() -> Self {
        Self(Mutex::new(Anchor {
            monitor: None,
            held: false,
        }))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Anchor> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A dictation starts: forget the previous dictation's monitor and hold, so its first
    /// placement follows the cursor.
    pub(crate) fn begin(&self) {
        *self.lock() = Anchor {
            monitor: None,
            held: false,
        };
    }

    /// The dictation has a destination: keep the monitor of its placements until the next one.
    pub(crate) fn hold(&self) {
        self.lock().held = true;
    }

    /// Monitor for the next placement among the connected `monitors`: the held one while the
    /// dictation holds one still connected, else `under_cursor()`, remembered for a later hold
    /// (a disconnected held monitor is replaced by the cursor's).
    pub(crate) fn resolve(
        &self,
        monitors: &[MonitorKey],
        under_cursor: impl FnOnce() -> Option<MonitorKey>,
    ) -> Option<MonitorKey> {
        let mut anchor = self.lock();
        let held = anchor.monitor.filter(|_| anchor.held);
        if let Some(monitor) = held.filter(|monitor| monitors.contains(monitor)) {
            return Some(monitor);
        }
        let monitor = under_cursor()?;
        anchor.monitor = Some(monitor);
        Some(monitor)
    }
}

// @spec AC-009: Indicator stays on its start monitor — .specs/features/006-destination-application/spec.md#ac-009
/// Monitor for the indicator and the surfaces placed at its place: the held one while the
/// dictation has a destination, else the one under the cursor (remembered for a later hold).
pub(super) fn placement_monitor(app_handle: &AppHandle) -> Option<tauri::Monitor> {
    let monitors = app_handle.available_monitors().unwrap_or_default();
    let keys: Vec<MonitorKey> = monitors.iter().map(MonitorKey::of).collect();
    let mut under_cursor = None;
    let chosen = ANCHOR.resolve(&keys, || {
        under_cursor = get_monitor_with_cursor(app_handle);
        under_cursor.as_ref().map(MonitorKey::of)
    })?;
    under_cursor.or_else(|| {
        monitors
            .into_iter()
            .find(|monitor| MonitorKey::of(monitor) == chosen)
    })
}

/// A dictation starts: its first placement follows the cursor (006 AC-009).
pub fn begin_dictation_placement() {
    ANCHOR.begin();
}

/// The current dictation has a destination (captured or pinned): the indicator keeps its
/// monitor until the next dictation (006 AC-009).
pub fn hold_placement_for_destination() {
    ANCHOR.hold();
}

#[cfg(test)]
#[path = "overlay_anchor_tests.rs"]
mod tests;
