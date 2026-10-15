//! Automatic Enter of 009 (PIN mode and Enter rule): the single three-value policy that replaces
//! auto-submit, its one-time migration, and the per-dictation authority that sends at most one
//! Enter press after a successful insertion. Everything here applies with the `destination-return`
//! gate open; with it closed, the legacy `auto_submit` emitters of `dev` run unchanged and only
//! [`legacy_auto_submit`] is consulted (it returns the stored `auto_submit`).
pub mod app;
pub mod migration;
pub mod policy;
pub mod rendezvous;
pub mod transaction;

use crate::settings::AppSettings;
/// The policy and key types of the settings (re-exported for the integration tests and 010).
pub use crate::settings::{AutoSubmitKey, AutomaticEnterPolicy};
use std::time::Duration;

// @spec AC-005: Stabilization pause before the Enter press — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-005
/// Pause between the insertion's readiness (t0) and the Enter claim: `dev`'s 50 ms (the classic
/// block of `clipboard::paste` and `AUTO_SUBMIT_SETTLE` of smart paste), kept so applications slow
/// to process a paste do not regress (plan decision 2026-10-14).
pub const PRE_ENTER_SETTLE: Duration = Duration::from_millis(50);

/// True when this build runs 009 (`destination-return` open).
pub const WIRED: bool = crate::destination::rollout::DESTINATION_RETURN_WIRED;

/// `automatic_enter` of a fresh install: `never` with the gate open (today's auto-submit default
/// is off), absent with it closed so the closed build stays `dev`.
pub fn fresh_install_policy() -> Option<AutomaticEnterPolicy> {
    WIRED.then_some(AutomaticEnterPolicy::Never)
}

/// The policy of `settings` in a gate-open build (`never` when absent, never a guessed Enter).
pub fn policy_of(settings: &AppSettings) -> AutomaticEnterPolicy {
    settings.automatic_enter.unwrap_or_default()
}

// @spec AC-004: Gate-open builds read only Automatic Enter — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
/// Whether the legacy emitters (insertion without a destination session) press the submit key.
/// Business rule: a closed build reads the stored `auto_submit` exactly as `dev`; an open build
/// reads only Automatic Enter, and a dictation without a session has no pin, so only `always`
/// sends a key there.
pub fn legacy_auto_submit(settings: &AppSettings) -> bool {
    legacy_auto_submit_for(settings, WIRED)
}

/// [`legacy_auto_submit`] for an explicit gate state (tests cover both in one binary).
pub fn legacy_auto_submit_for(settings: &AppSettings, wired: bool) -> bool {
    if wired {
        policy_of(settings) == AutomaticEnterPolicy::Always
    } else {
        settings.auto_submit
    }
}

/// `settings` with `auto_submit` replaced by [`legacy_auto_submit`], for the legacy emitters
/// that read the field directly. Identity in a closed build.
pub fn legacy_view(mut settings: AppSettings) -> AppSettings {
    settings.auto_submit = legacy_auto_submit(&settings);
    settings
}

#[cfg(test)]
mod scan_tests;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::get_default_settings;

    #[test]
    fn a_closed_gate_reads_the_stored_auto_submit_exactly_as_dev() {
        let mut settings = get_default_settings();
        settings.automatic_enter = Some(AutomaticEnterPolicy::Always);
        for stored in [false, true] {
            settings.auto_submit = stored;
            assert_eq!(legacy_auto_submit_for(&settings, false), stored);
        }
    }
    #[test]
    fn an_open_gate_reads_only_automatic_enter_without_a_session() {
        let mut settings = get_default_settings();
        settings.auto_submit = true;
        let cases = [
            (None, false),
            (Some(AutomaticEnterPolicy::Never), false),
            (Some(AutomaticEnterPolicy::PinOnly), false),
            (Some(AutomaticEnterPolicy::Always), true),
        ];
        for (policy, expected) in cases {
            settings.automatic_enter = policy;
            assert_eq!(
                legacy_auto_submit_for(&settings, true),
                expected,
                "{policy:?}"
            );
        }
    }
    #[test]
    fn a_fresh_install_follows_the_gate_and_the_pause_is_dev_s_50_ms() {
        assert_eq!(fresh_install_policy().is_some(), WIRED);
        assert_eq!(PRE_ENTER_SETTLE, Duration::from_millis(50));
    }
}
