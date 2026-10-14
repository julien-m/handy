//! Smart paste as a paste-method setting (AC-014, AC-015): where it can be selected, the macOS
//! default of new installations, the one-time macOS migration from the beta reliable paste, and
//! the refusals shared by the settings UI and the settings CLI. Pure: the platform and the rollout
//! state are passed in, so both states are tested on every platform.
//!
//! Rollout gate: the paste pipeline refuses `PasteMethod::SmartPaste` until feature 007 step 7
//! wires it into dictation (destination context of feature 006). Until then the gate stays closed:
//! smart paste is neither selectable, nor a default, nor a migration target, and the beta reliable
//! paste stays available on macOS, so every installation keeps a paste method that dictation can
//! use. Step 7 opens the gate by enabling the `smart-paste-dispatch` Cargo feature in the same
//! change that wires the dispatch.

use crate::settings::{AppSettings, PasteMethod};
use crate::settings_service::errors::ValidationReason;

/// True once dictation delivers through smart paste: the `smart-paste-dispatch` Cargo feature,
/// enabled by step 7 only. Enabling it earlier makes macOS dictation fail with the default method.
pub const DICTATION_DISPATCH_WIRED: bool = cfg!(feature = "smart-paste-dispatch");

/// Stored `smart_paste_migration_version` once the macOS migration has run.
pub const MIGRATION_VERSION: u32 = 1;

/// Canonical stored value of the smart paste method.
pub const SMART_PASTE_VALUE: &str = "smart_paste";

/// Paste methods the settings validator accepts in this build (descriptor allowed values).
pub const PASTE_METHOD_VALUES: &[&str] = if Rollout::CURRENT.is_active() {
    &[
        "ctrl_v",
        "direct",
        "none",
        "shift_insert",
        "ctrl_shift_v",
        "external_script",
        SMART_PASTE_VALUE,
    ]
} else {
    &[
        "ctrl_v",
        "direct",
        "none",
        "shift_insert",
        "ctrl_shift_v",
        "external_script",
    ]
};

/// Where smart paste stands for one platform and build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rollout {
    /// Smart paste is a macOS paste method only.
    pub macos: bool,
    /// See [`DICTATION_DISPATCH_WIRED`].
    pub dispatch_wired: bool,
}

impl Rollout {
    /// This build on this platform.
    pub const CURRENT: Self = Self {
        macos: cfg!(target_os = "macos"),
        dispatch_wired: DICTATION_DISPATCH_WIRED,
    };

    /// Smart paste can be selected, is the macOS default of new installations and replaces the
    /// beta reliable paste on macOS.
    pub const fn is_active(self) -> bool {
        self.macos && self.dispatch_wired
    }

    /// Paste method of a new installation: smart paste on macOS once active, otherwise the
    /// existing `platform_default`.
    pub const fn default_paste_method(self, platform_default: PasteMethod) -> PasteMethod {
        if self.is_active() {
            PasteMethod::SmartPaste
        } else {
            platform_default
        }
    }

    /// Migration marker of a new installation: nothing is left to migrate once smart paste is its
    /// default; before that, a later activation still migrates it like an upgrade.
    pub const fn fresh_install_marker(self) -> u32 {
        if self.is_active() {
            MIGRATION_VERSION
        } else {
            0
        }
    }
}

/// One-time macOS migration from the beta reliable paste, applied on every settings read and
/// persisted by the next successful commit (startup or explicit write) with its marker.
///
/// Business rule (AC-014): once smart paste is active, users whose method is Cmd+V move to smart
/// paste whether or not the beta switch was on, users of any other method keep it, and the beta
/// switch is gone (`reliable_paste` false, also against a later hand edit). The marker, written
/// even when the method is kept, makes a later deliberate Cmd+V choice survive every reload.
/// `stored` is the raw stored payload: only its marker decides whether the migration already ran.
///
/// Returns `true` when `settings` changed.
pub fn migrate(settings: &mut AppSettings, stored: &serde_json::Value, rollout: Rollout) -> bool {
    if !rollout.is_active() {
        return false;
    }
    let mut updated = settings.reliable_paste;
    settings.reliable_paste = false;
    if stored_marker(stored) < u64::from(MIGRATION_VERSION) {
        if settings.paste_method == PasteMethod::CtrlV {
            settings.paste_method = PasteMethod::SmartPaste;
        }
        settings.smart_paste_migration_version = MIGRATION_VERSION;
        updated = true;
    }
    updated
}

/// Marker of the raw stored payload; absent or malformed means "never migrated".
fn stored_marker(stored: &serde_json::Value) -> u64 {
    stored
        .get("smart_paste_migration_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

/// Refusal shared by every settings surface for setting `key` to `candidate`, or `None`.
///
/// Business rules: smart paste is refused wherever it is not active (outside macOS, and on macOS
/// until dictation delivers through it); once it is active, the macOS beta reliable paste cannot
/// be switched back on. Every other key and value is left to the generic validation.
pub fn refusal(
    key: &str,
    candidate: &serde_json::Value,
    rollout: Rollout,
) -> Option<ValidationReason> {
    match key {
        "paste_method" if candidate.as_str() == Some(SMART_PASTE_VALUE) && !rollout.is_active() => {
            Some(ValidationReason::SmartPasteUnavailable)
        }
        "reliable_paste" if candidate.as_bool() == Some(true) && rollout.is_active() => {
            Some(ValidationReason::ReliablePasteReplaced)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "setting_policy_tests.rs"]
mod tests;
