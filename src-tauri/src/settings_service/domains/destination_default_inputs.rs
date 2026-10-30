//! Default-input actions of `handy settings destination` (013, gate `destination-return` open):
//! `default-input-remove <identifier>` and `default-input-clear` are ordinary settings changes,
//! offline or routed, with the validation shared with the badge and the settings window.
//! Remembering needs a dictation and is offered only by the badge menu.
use super::DomainPlan;
use crate::destination::default_input::model::{
    clear_change, find_identifier, remove_change, DEFAULT_INPUTS_KEY,
};
use crate::settings::AppSettings;
use crate::settings_service::errors::SettingsError;

// @spec AC-009: default-input-remove and default-input-clear — .specs/features/013-default-input-per-app/spec.md#ac-009
/// Plan of `default-input-remove`: `not_found` now when no entry has this identity value (the
/// change checks again on the locked settings at commit).
///
/// # Errors
/// `not_found` (remembered input lookup message); nothing changes.
pub(super) fn remove_plan(
    identifier: &str,
    settings: &AppSettings,
) -> Result<DomainPlan, SettingsError> {
    find_identifier(&settings.destination_default_inputs, identifier)
        .ok_or_else(SettingsError::no_default_input)?;
    Ok(DomainPlan::Change {
        change: remove_change(identifier.to_owned()),
        keys: vec![DEFAULT_INPUTS_KEY],
        render: None,
    })
}

/// Plan of `default-input-clear`: forget every remembered input (a no-op on an empty list).
pub(super) fn clear_plan() -> DomainPlan {
    DomainPlan::Change {
        change: clear_change(),
        keys: vec![DEFAULT_INPUTS_KEY],
        render: None,
    }
}

#[cfg(test)]
#[path = "destination_default_inputs_tests.rs"]
mod tests;
