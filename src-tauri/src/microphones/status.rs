//! The microphone status read by `handy settings microphones list` and the settings window:
//! the committed order, the availability of each choice and the effective input. The configured
//! order comes from a settings revision; availability comes from a separately timestamped device
//! observation: two distinct clocks, never one atomic promise about changing hardware.
use super::catalog::{availability, Catalog, ChoiceStatus};
use super::effective::{EffectiveScope, EffectiveView};
use super::types::EffectiveReason;
use serde::Serialize;

/// Typed warning of a status read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StatusWarning {
    /// The devices could not be listed in time: every status is unknown.
    EnumerationFailed,
}

/// The input in use, or the last one used, as reported to the user.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct EffectiveReport {
    pub scope: EffectiveScope,
    /// Device name read from the opened stream (null when not used yet or unnamed).
    pub name: Option<String>,
    pub reason: Option<EffectiveReason>,
    pub capture_generation: Option<u64>,
}

/// One status read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct MicrophoneStatus {
    /// Committed order of normal choices.
    pub configured_order: Vec<String>,
    /// Status of each configured choice, in saved order.
    pub availability: Vec<ChoiceStatus>,
    /// RFC 3339 time of the device observation.
    pub availability_observed_at: String,
    pub effective_input: EffectiveReport,
    pub warning: Option<StatusWarning>,
}

// @spec FR-002: Configured order apart from effective input — .specs/features/001-microphone-priority/spec.md#fr-002
/// Status of `configured` against one observation (`None`: the enumeration failed) taken at
/// `observed_at`, with the effective input `effective`.
pub fn status<D>(
    configured: Vec<String>,
    catalog: Option<&Catalog<D>>,
    observed_at: String,
    effective: EffectiveView,
) -> MicrophoneStatus {
    let input = effective.input;
    MicrophoneStatus {
        availability: availability(&configured, catalog),
        configured_order: configured,
        availability_observed_at: observed_at,
        effective_input: EffectiveReport {
            scope: effective.scope,
            name: input.as_ref().and_then(|input| input.name.clone()),
            reason: input.as_ref().map(|input| input.reason),
            capture_generation: input.map(|input| input.capture_generation),
        },
        warning: catalog
            .is_none()
            .then_some(StatusWarning::EnumerationFailed),
    }
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
