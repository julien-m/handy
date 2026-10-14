//! Input-source handling around the paste chord (AC-011).
//!
//! When the current source is a keyboard layout other than ABC or US, the
//! ABC layout is selected (US if ABC cannot be), the switch is given 50 ms to
//! apply, and the previous source is restored 100 ms after the last key of
//! the chord is released, even when the paste failed. Input methods and
//! failed selections leave the source unchanged.

use std::fmt;
use std::time::{Duration, Instant};

use super::clock::Clock;

// @spec FR-007: Input-source switch and restore — .specs/features/007-smart-paste/spec.md#fr-007

/// ABC keyboard layout identifier (preferred).
pub const ABC_LAYOUT_ID: &str = "com.apple.keylayout.ABC";
/// US keyboard layout identifier (fallback).
pub const US_LAYOUT_ID: &str = "com.apple.keylayout.US";
/// Time given to a layout switch before the chord (AC-011).
pub const LAYOUT_SETTLE_DELAY: Duration = Duration::from_millis(50);
/// Delay after the last key-up before restoring the source (AC-011).
pub const LAYOUT_RESTORE_DELAY: Duration = Duration::from_millis(100);

/// Kind of the current input source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSourceKind {
    /// A keyboard layout (AZERTY, Dvorak, ABC...).
    KeyboardLayout,
    /// An input method or input mode (for example Japanese Kana).
    InputMethod,
}

/// The current input source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputSourceInfo {
    /// Text Input Sources identifier.
    pub id: String,
    /// Layout or input method.
    pub kind: InputSourceKind,
}

/// Errors of input-source and key-event operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardError {
    /// The current input source could not be read.
    SourceUnavailable,
    /// The source could not be selected (missing, disabled or refused).
    SelectFailed,
    /// A key event could not be created or posted.
    PostFailed,
    /// The main thread did not run the operation in time.
    MainThreadUnavailable,
}

impl fmt::Display for KeyboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for KeyboardError {}

/// Text Input Sources access.
pub trait InputSourcePort: Send {
    /// Reads the current keyboard input source.
    fn current_source(&mut self) -> Result<InputSourceInfo, KeyboardError>;
    /// Selects an enabled source by identifier.
    fn select_source(&mut self, id: &str) -> Result<(), KeyboardError>;
}

/// Decision for the current source, before any selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDecision {
    /// Keep the current source.
    Keep,
    /// Try ABC, then US.
    TryAbcThenUs,
}

/// Pure decision: only layouts other than ABC and US are switched.
pub fn decide_layout(current: &InputSourceInfo) -> LayoutDecision {
    let is_abc_or_us = current.id == ABC_LAYOUT_ID || current.id == US_LAYOUT_ID;
    match current.kind {
        InputSourceKind::KeyboardLayout if !is_abc_or_us => LayoutDecision::TryAbcThenUs,
        _ => LayoutDecision::Keep,
    }
}

/// Outcome of the pre-chord switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutSwitch {
    /// The source was not changed.
    Unchanged,
    /// The source was switched; `previous` must be restored.
    Switched {
        /// Identifier of the source to restore.
        previous: String,
        /// Identifier that was selected.
        selected: &'static str,
    },
}

/// Switches to ABC (or US) when needed and waits for the switch to apply.
/// Any failure leaves the source unchanged; Cmd+V is then sent with it.
pub fn switch_for_paste(port: &mut dyn InputSourcePort, clock: &dyn Clock) -> LayoutSwitch {
    let Ok(current) = port.current_source() else {
        return LayoutSwitch::Unchanged;
    };
    if decide_layout(&current) == LayoutDecision::Keep {
        return LayoutSwitch::Unchanged;
    }
    // ABC is preferred; US is tried only when ABC cannot be selected.
    for candidate in [ABC_LAYOUT_ID, US_LAYOUT_ID] {
        if port.select_source(candidate).is_ok() {
            clock.sleep(LAYOUT_SETTLE_DELAY);
            return LayoutSwitch::Switched {
                previous: current.id,
                selected: candidate,
            };
        }
    }
    LayoutSwitch::Unchanged
}

/// Restores the previous source `LAYOUT_RESTORE_DELAY` after `last_key_up`.
///
/// # Errors
/// Returns the selection error; callers must report it, never swallow it.
pub fn restore_after_chord(
    port: &mut dyn InputSourcePort,
    clock: &dyn Clock,
    switch: &LayoutSwitch,
    last_key_up: Instant,
) -> Result<(), KeyboardError> {
    let LayoutSwitch::Switched { previous, .. } = switch else {
        return Ok(());
    };
    let due = last_key_up + LAYOUT_RESTORE_DELAY;
    clock.sleep(due.saturating_duration_since(clock.now()));
    port.select_source(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: &str, kind: InputSourceKind) -> InputSourceInfo {
        InputSourceInfo {
            id: id.to_string(),
            kind,
        }
    }

    #[test]
    fn non_us_layouts_are_switched() {
        let french = source(
            "com.apple.keylayout.French",
            InputSourceKind::KeyboardLayout,
        );
        let dvorak = source(
            "com.apple.keylayout.Dvorak",
            InputSourceKind::KeyboardLayout,
        );
        assert_eq!(decide_layout(&french), LayoutDecision::TryAbcThenUs);
        assert_eq!(decide_layout(&dvorak), LayoutDecision::TryAbcThenUs);
    }

    #[test]
    fn abc_us_and_input_methods_are_kept() {
        let abc = source(ABC_LAYOUT_ID, InputSourceKind::KeyboardLayout);
        let us = source(US_LAYOUT_ID, InputSourceKind::KeyboardLayout);
        let kana = source(
            "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese",
            InputSourceKind::InputMethod,
        );
        for current in [abc, us, kana] {
            assert_eq!(decide_layout(&current), LayoutDecision::Keep);
        }
    }
}
