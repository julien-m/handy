//! Static platform capabilities of this build, shared by the settings UI, the offline CLI and the
//! running instance: a setting or domain operation that needs a capability the build lacks is
//! refused by the pure validator (`validation_failed`, `details.reason = "unsupported_platform"`),
//! never reported as `unavailable`. Runtime readability (for example a foreground that cannot be
//! read) is a separate live state of the instance and never a validation.
use super::errors::SettingsError;

/// A platform feature a setting or a domain operation may require.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformCapability {
    /// Capture and activation of the dictation's destination application (macOS, Windows).
    Destination,
    /// Apple Intelligence as a post-processing provider (Apple silicon Macs, 005).
    AppleIntelligence,
}

/// Capabilities of one build, or an injected set in tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformCapabilities {
    supported: &'static [PlatformCapability],
}

// The build declares `Destination` only on macOS and Windows with the 006 rollout gate open
// (`destination::rollout`); closed, every validator refuses `return_to_dictation_app`.
const DESTINATION_BUILD: bool = !crate::destination::rollout::Rollout::CURRENT
    .capabilities()
    .is_empty();
// Apple Intelligence exists only in Apple silicon macOS builds (the same condition that adds the
// provider to the default list and compiles the native adapter).
const APPLE_INTELLIGENCE_BUILD: bool = cfg!(all(target_os = "macos", target_arch = "aarch64"));
const BUILD_CAPABILITIES: &[PlatformCapability] =
    match (DESTINATION_BUILD, APPLE_INTELLIGENCE_BUILD) {
        (true, true) => &[
            PlatformCapability::Destination,
            PlatformCapability::AppleIntelligence,
        ],
        (true, false) => &[PlatformCapability::Destination],
        (false, true) => &[PlatformCapability::AppleIntelligence],
        (false, false) => &[],
    };

/// Setting keys that need a capability; each later feature adds its own row.
const KEY_CAPABILITIES: &[(&str, PlatformCapability)] =
    &[("return_to_dictation_app", PlatformCapability::Destination)];

impl PlatformCapabilities {
    /// Capabilities of the running build.
    pub fn current() -> Self {
        Self {
            supported: BUILD_CAPABILITIES,
        }
    }
    /// Exactly `supported` (tests inject another platform with it).
    pub fn only(supported: &'static [PlatformCapability]) -> Self {
        Self { supported }
    }
    /// True when this build has `capability`.
    pub fn supports(self, capability: PlatformCapability) -> bool {
        self.supported.contains(&capability)
    }
    // @spec FR-006: Detect platform support — .specs/features/006-destination-application/spec.md#fr-006
    /// Refuse `requirement` when this build lacks it; `key` names the setting refused, if any.
    ///
    /// # Errors
    /// `validation_failed` with `details.reason = "unsupported_platform"`.
    pub fn check(
        self,
        requirement: Option<PlatformCapability>,
        key: Option<&str>,
    ) -> Result<(), SettingsError> {
        match requirement {
            Some(capability) if !self.supports(capability) => {
                Err(SettingsError::unsupported_platform(key))
            }
            _ => Ok(()),
        }
    }
}

/// Capability that setting `key` requires, if any.
pub fn required_capability(key: &str) -> Option<PlatformCapability> {
    KEY_CAPABILITIES
        .iter()
        .find(|(known, _)| *known == key)
        .map(|(_, capability)| *capability)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_service::errors::{ErrorReason, SettingsErrorCode};

    #[test]
    fn an_unsupported_requirement_is_a_validation_refusal_with_its_reason() {
        let none = PlatformCapabilities::only(&[]);
        let refused = none.check(Some(PlatformCapability::Destination), None);
        let error = refused.err();
        assert_eq!(
            error.as_ref().map(|error| error.code()),
            Some(SettingsErrorCode::ValidationFailed)
        );
        assert_eq!(
            error.as_ref().and_then(SettingsError::detail_reason),
            Some(ErrorReason::UnsupportedPlatform)
        );
        assert_eq!(error.map(|error| error.exit_code()), Some(2));
    }
    #[test]
    fn a_supported_or_absent_requirement_passes() {
        let destination = PlatformCapabilities::only(&[PlatformCapability::Destination]);
        assert!(destination
            .check(Some(PlatformCapability::Destination), None)
            .is_ok());
        assert!(PlatformCapabilities::only(&[]).check(None, None).is_ok());
    }
    #[test]
    fn the_build_supports_apple_intelligence_exactly_on_apple_silicon() {
        assert_eq!(
            PlatformCapabilities::current().supports(PlatformCapability::AppleIntelligence),
            cfg!(all(target_os = "macos", target_arch = "aarch64"))
        );
    }
    #[test]
    fn the_build_supports_destination_exactly_on_macos_and_windows_with_the_gate_open() {
        let expected = cfg!(feature = "destination-return")
            && cfg!(any(target_os = "macos", target_os = "windows"));
        assert_eq!(
            PlatformCapabilities::current().supports(PlatformCapability::Destination),
            expected
        );
    }
}
