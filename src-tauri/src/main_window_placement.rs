//! Placement of the settings window when it is opened: centered in the work area of the primary
//! monitor (the one with the menu bar on macOS), whatever monitor it was left on.
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

/// Business rule: the window's top-left corner that centers `window` in the area starting at
/// `origin` with `extent`; a window larger than the area is pinned to the area's top-left corner
/// so its title bar stays reachable.
pub(crate) fn centered_origin(
    origin: PhysicalPosition<i32>,
    extent: PhysicalSize<u32>,
    window: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    let offset = |area: u32, size: u32| (i64::from(area.saturating_sub(size)) / 2) as i32;
    PhysicalPosition::new(
        origin.x + offset(extent.width, window.width),
        origin.y + offset(extent.height, window.height),
    )
}

/// Center `window` on the primary monitor's work area; leaves it in place when the monitor or the
/// window size cannot be read.
pub(crate) fn center_on_primary_monitor(window: &WebviewWindow) {
    let monitor = match window.primary_monitor() {
        Ok(Some(monitor)) => monitor,
        Ok(None) => return log::debug!("Settings window not centered: no primary monitor"),
        Err(e) => return log::debug!("Settings window not centered: {e}"),
    };
    let size = match window.outer_size() {
        Ok(size) => size,
        Err(e) => return log::debug!("Settings window not centered: {e}"),
    };
    let area = monitor.work_area();
    let position = centered_origin(area.position, area.size, size);
    if let Err(e) = window.set_position(position) {
        log::error!("Failed to center the settings window: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
        (
            PhysicalPosition::new(-1920, 25),
            PhysicalSize::new(1920, 1055),
        )
    }

    #[test]
    fn a_smaller_window_is_centered_in_the_area() {
        let (origin, extent) = area();
        let window = PhysicalSize::new(extent.width / 2, extent.height / 3);
        let at = centered_origin(origin, extent, window);
        // Same margin on both sides of each axis (to one pixel for odd remainders).
        let right = origin.x + extent.width as i32 - (at.x + window.width as i32);
        let bottom = origin.y + extent.height as i32 - (at.y + window.height as i32);
        assert!((at.x - origin.x - right).abs() <= 1);
        assert!((at.y - origin.y - bottom).abs() <= 1);
    }

    #[test]
    fn a_larger_window_is_pinned_to_the_area_top_left() {
        let (origin, extent) = area();
        let window = PhysicalSize::new(extent.width + 10, extent.height + 10);
        assert_eq!(centered_origin(origin, extent, window), origin);
    }
}
