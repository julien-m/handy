//! Typed key metadata; no persistence, native effect or application startup.

use serde::Serialize;

/// Canonical JSON value type; nullability is a separate descriptor dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingType {
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

/// Inclusive pure-validation bounds; widget step increments are not imposed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct NumericRange {
    pub min: f64,
    pub max: f64,
}

/// Owning live-effect category, prepared/applied by the later writer migration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEffect {
    None,
    AudioInput,
    AudioOutput,
    Shortcuts,
    Autostart,
    Updater,
    Overlay,
    Logging,
    History,
    Language,
    Theme,
    Tray,
    Model,
    Vad,
}

/// Every stored key is classified explicitly; read-only and secrets are UI-owned.
// @spec FR-002: Discover typed settings keys — .specs/features/008-settings-cli/spec.md#fr-002
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SettingDescriptor {
    pub key: &'static str,
    pub value_type: SettingType,
    pub nullable: bool,
    pub range: Option<NumericRange>,
    pub allowed_values: &'static [&'static str],
    pub read_only: bool,
    pub secret: bool,
    pub owner_domain: &'static str,
    pub runtime_effect: RuntimeEffect,
}

/// Return the current finite descriptor inventory.
pub fn descriptors() -> &'static [SettingDescriptor] {
    super::descriptor_data::DESCRIPTORS
}

/// Resolve a recognized canonical key without echoing arbitrary input.
pub fn descriptor(key: &str) -> Option<&'static SettingDescriptor> {
    descriptors().iter().find(|entry| entry.key == key)
}
