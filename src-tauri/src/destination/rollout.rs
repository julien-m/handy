//! Rollout gate of destination return (006): the `destination-return` Cargo feature, off by
//! default. Closed, a dictation inserts exactly as before 006 (no destination session, no
//! activation, no pre-dispatch guard) and the build declares no `Destination` capability, so the
//! settings UI and CLI refuse `return_to_dictation_app` with `unsupported_platform`. The settings
//! framework of Step 0, the destination manager and the text-only history rule stay compiled and
//! tested in both states. Opening conditions: implementation.md, section "Rollout gate".
use crate::settings_service::platform_capability::PlatformCapability;

/// True when dictation returns to its destination application: the `destination-return` Cargo
/// feature. Open it only once the native macOS activation and guard proof and the Penflow
/// certified UI exist (implementation.md, "Rollout gate").
pub const DESTINATION_RETURN_WIRED: bool = cfg!(feature = "destination-return");

/// Where destination return stands for one build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rollout {
    /// The platform can read and activate the destination (macOS, Windows).
    pub platform_capable: bool,
    /// See [`DESTINATION_RETURN_WIRED`].
    pub wired: bool,
}

impl Rollout {
    /// This build on this platform. Business rule: Linux cannot read the frontmost application
    /// reliably across Wayland compositors (006 spec), so only macOS and Windows are capable.
    pub const CURRENT: Self = Self {
        platform_capable: cfg!(any(target_os = "macos", target_os = "windows")),
        wired: DESTINATION_RETURN_WIRED,
    };
    /// Platform capabilities the build declares to the settings validators.
    pub const fn capabilities(self) -> &'static [PlatformCapability] {
        if self.platform_capable && self.wired {
            &[PlatformCapability::Destination]
        } else {
            &[]
        }
    }
    /// True when a starting dictation opens a destination session (capture, activation and
    /// guard); false keeps the insertion path of `dev` unchanged.
    pub const fn opens_sessions(self) -> bool {
        self.wired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATES: [(bool, bool); 4] = [(false, false), (false, true), (true, false), (true, true)];

    #[test]
    fn a_closed_gate_declares_no_capability_and_opens_no_session_on_any_platform() {
        for platform_capable in [false, true] {
            let closed = Rollout {
                platform_capable,
                wired: false,
            };
            assert!(closed.capabilities().is_empty());
            assert!(!closed.opens_sessions());
        }
    }
    #[test]
    fn an_open_gate_declares_destination_only_on_a_capable_platform() {
        for (platform_capable, wired) in STATES {
            let rollout = Rollout {
                platform_capable,
                wired,
            };
            let declared = rollout
                .capabilities()
                .contains(&PlatformCapability::Destination);
            assert_eq!(declared, platform_capable && wired, "{rollout:?}");
            assert_eq!(rollout.opens_sessions(), wired, "{rollout:?}");
        }
    }
    #[test]
    fn this_build_follows_the_destination_return_feature() {
        // Printed so a proof run shows which gate state it exercised.
        println!(
            "destination_return_wired={} declares_destination={}",
            Rollout::CURRENT.wired,
            !Rollout::CURRENT.capabilities().is_empty()
        );
        assert_eq!(Rollout::CURRENT.wired, cfg!(feature = "destination-return"));
    }
}
