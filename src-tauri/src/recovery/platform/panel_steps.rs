//! Order of the native changes that end a drag's invisibility of the recovery panel (spec
//! AC-004, native session N26). During a drag the panel stays on screen but transparent and
//! click-through; once the service has decided how the session ended, the panel either leaves
//! the screen (drop, Escape, Close) or comes back (release outside any target). Hiding takes
//! the panel off the screen before making it opaque again for the next bubble: in the other
//! order, a bubble dismissed by Escape showed again for a moment before it went away.

/// One native change of the recovery panel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelStep {
    /// Take the panel off the screen (`orderOut:`).
    OrderOut,
    /// Undo the drag's invisibility: opaque and receiving the mouse again.
    Reveal,
}

// @spec AC-004: Hide off screen before revealing — .specs/features/010-recovery-bubble/spec.md#ac-004
/// Hiding a bubble: off the screen first, then ready (opaque) for the next presentation, so a
/// bubble retired during its drag never becomes visible again, not even for one frame.
pub const HIDE_STEPS: [PanelStep; 2] = [PanelStep::OrderOut, PanelStep::Reveal];

/// Showing the same bubble again after a release outside any target: the panel never left the
/// screen, revealing it is enough.
pub const RESTORE_STEPS: [PanelStep; 1] = [PanelStep::Reveal];

#[cfg(test)]
#[path = "panel_steps_tests.rs"]
mod tests;
