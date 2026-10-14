//! Per-application clipboard timings (AC-009).
//!
//! The values come from the static analysis of Spokenly 2.30.9 and from the
//! 2026-10-13 decisions recorded in the spec; native checks (AC-016) measure
//! them. They are the single timing source for the clipboard lifecycle.

use std::time::Duration;

/// Bundle identifier of Jump Desktop, which reads the clipboard slowly.
pub const JUMP_DESKTOP_BUNDLE_ID: &str = "com.p5sys.jump.mac.viewer";

/// Default delay without any read after the keystroke before "not read".
pub const DEFAULT_NO_READ_DECISION: Duration = Duration::from_millis(800);
/// Default quiet period after the last read before restoring.
pub const DEFAULT_QUIET_AFTER_LAST_READ: Duration = Duration::from_millis(300);
/// Jump Desktop: no-read decision delay (spec AC-009).
pub const JUMP_NO_READ_DECISION: Duration = Duration::from_secs(3);
/// Jump Desktop: quiet period after the last read (spec AC-009).
pub const JUMP_QUIET_AFTER_LAST_READ: Duration = Duration::from_millis(1500);
/// Restoration happens at the latest this long after the first read, even if
/// reads keep coming (AC-009 cap).
pub const RESTORE_CAP_AFTER_FIRST_READ: Duration = Duration::from_secs(8);

/// Timing parameters of one clipboard paste transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardTimings {
    /// Delay after the first key-down without a qualifying read before the
    /// outcome becomes "not read" and the snapshot is discarded.
    pub no_read_decision: Duration,
    /// Quiet period after the last qualifying read before restoring.
    pub quiet_after_last_read: Duration,
    /// Latest restoration time measured from the first qualifying read.
    pub restore_cap_after_first_read: Duration,
}

impl ClipboardTimings {
    /// Timings for every application without a dedicated profile.
    pub const DEFAULT: Self = Self {
        no_read_decision: DEFAULT_NO_READ_DECISION,
        quiet_after_last_read: DEFAULT_QUIET_AFTER_LAST_READ,
        restore_cap_after_first_read: RESTORE_CAP_AFTER_FIRST_READ,
    };

    /// Timings for Jump Desktop.
    pub const JUMP_DESKTOP: Self = Self {
        no_read_decision: JUMP_NO_READ_DECISION,
        quiet_after_last_read: JUMP_QUIET_AFTER_LAST_READ,
        restore_cap_after_first_read: RESTORE_CAP_AFTER_FIRST_READ,
    };

    /// Selects the timings for a destination bundle identifier.
    pub fn for_bundle(bundle_id: Option<&str>) -> Self {
        match bundle_id {
            Some(JUMP_DESKTOP_BUNDLE_ID) => Self::JUMP_DESKTOP,
            _ => Self::DEFAULT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_desktop_uses_its_slow_profile() {
        let timings = ClipboardTimings::for_bundle(Some(JUMP_DESKTOP_BUNDLE_ID));
        assert_eq!(timings.no_read_decision, Duration::from_secs(3));
        assert_eq!(timings.quiet_after_last_read, Duration::from_millis(1500));
        assert_eq!(timings.restore_cap_after_first_read, Duration::from_secs(8));
    }

    #[test]
    fn other_and_unknown_destinations_use_the_default_profile() {
        for bundle in [Some("com.apple.TextEdit"), None] {
            let timings = ClipboardTimings::for_bundle(bundle);
            assert_eq!(timings.no_read_decision, Duration::from_millis(800));
            assert_eq!(timings.quiet_after_last_read, Duration::from_millis(300));
        }
    }
}
