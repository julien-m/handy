//! Wire and runtime vocabulary of microphone priority: availability of a configured choice and
//! the reason the effective input was chosen. Snake_case names are stable for the CLI JSON and
//! the settings UI; they are not translated text.
use serde::Serialize;

/// Availability of one configured choice, observed separately from the saved order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    /// Exactly one present device has this name.
    Available,
    /// A complete enumeration found no device with this name.
    Unavailable,
    /// Several present devices share this name; none is chosen arbitrarily.
    Ambiguous,
    /// The enumeration failed or timed out: absence is not proven.
    Unknown,
}

/// Why the effective input was chosen.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveReason {
    /// The first configured choice opened.
    FirstChoice,
    /// A later configured choice opened after earlier ones failed.
    Fallback,
    /// The clamshell override opened while the lid is confirmed closed.
    Clamshell,
    /// The system default opened.
    SystemDefault,
}
