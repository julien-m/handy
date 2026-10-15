//! "Show the bubble when no text field is focused" (`show_recovery_bubble_without_text_field`,
//! spec AC-002 and AC-007; plan "Rollout gate", gate-conditional mechanism). The stored field
//! is compiled in every build; its value is set in memory by 008's normalization, never as a
//! requested write: gate closed it is absent whatever is stored; gate open it is the user's
//! choice (default `true`) on macOS and `false` on Windows and Linux, where writes are refused
//! (`PlatformCapability::FocusedFieldDetection`).
use crate::settings::AppSettings;

/// The settings key.
pub const KEY: &str = "show_recovery_bubble_without_text_field";

/// Gate state and platform of one build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingBuild {
    /// The `destination-return` gate is open.
    pub wired: bool,
    /// The build targets macOS (the only platform with field detection).
    pub macos: bool,
}
impl SettingBuild {
    /// This build.
    pub const CURRENT: Self = Self {
        wired: cfg!(feature = "destination-return"),
        macos: cfg!(target_os = "macos"),
    };
}

// @spec FR-005: The no-text-field setting per build — .specs/features/010-recovery-bubble/spec.md#fr-005
/// The value of a `stored` setting in `build` (business rule above).
pub fn normalized(stored: Option<bool>, build: SettingBuild) -> Option<bool> {
    match (build.wired, build.macos) {
        (false, _) => None,
        (true, true) => Some(stored.unwrap_or(true)),
        (true, false) => Some(false),
    }
}
/// Fresh-install value of this build.
pub fn fresh_value() -> Option<bool> {
    normalized(None, SettingBuild::CURRENT)
}
/// Normalize `settings` in memory for this build.
pub fn normalize(settings: &mut AppSettings) {
    let stored = settings.show_recovery_bubble_without_text_field;
    settings.show_recovery_bubble_without_text_field = normalized(stored, SettingBuild::CURRENT);
}
/// True when field detection applies to the next dictation.
pub fn enabled(settings: &AppSettings) -> bool {
    settings.show_recovery_bubble_without_text_field == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATES: [SettingBuild; 4] = [
        SettingBuild {
            wired: false,
            macos: false,
        },
        SettingBuild {
            wired: false,
            macos: true,
        },
        SettingBuild {
            wired: true,
            macos: false,
        },
        SettingBuild {
            wired: true,
            macos: true,
        },
    ];

    #[test]
    fn closed_builds_drop_the_value_and_open_ones_fix_it_per_platform() {
        for build in STATES {
            for stored in [None, Some(false), Some(true)] {
                let expected = match (build.wired, build.macos) {
                    (false, _) => None,
                    (true, false) => Some(false),
                    (true, true) => Some(stored.unwrap_or(true)),
                };
                assert_eq!(normalized(stored, build), expected, "{build:?} {stored:?}");
            }
        }
    }
    #[test]
    fn this_build_follows_the_gate_and_the_platform() {
        let expected = match (
            cfg!(feature = "destination-return"),
            cfg!(target_os = "macos"),
        ) {
            (false, _) => None,
            (true, macos) => Some(macos),
        };
        assert_eq!(fresh_value(), expected);
    }
}
