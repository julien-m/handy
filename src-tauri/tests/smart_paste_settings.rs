//! Smart paste through the settings service of feature 008 (feature 007, FR-009, AC-014, AC-015):
//! the one-time macOS migration on real stores, the validator shared by the UI and the CLI, and the
//! `handy settings output` domain through the real binary. Every expectation follows the rollout
//! of this build: run once as is (gate closed: nothing moves to smart paste) and once with
//! `--features smart-paste-dispatch` (gate open, the state step 7 ships).

#[path = "smart_paste_settings/cli.rs"]
mod cli;
// Shared process harness of the settings CLI tests, public because this target does not use all
// of it.
#[path = "settings_cli/harness.rs"]
#[cfg(unix)]
pub mod harness;
#[path = "smart_paste_settings/store.rs"]
mod store;
#[path = "smart_paste_settings/validation.rs"]
mod validation;

use handy_app_lib::smart_paste::setting_policy::Rollout;

/// True when smart paste is selectable in this build on this platform.
pub(crate) fn active() -> bool {
    Rollout::CURRENT.is_active()
}
