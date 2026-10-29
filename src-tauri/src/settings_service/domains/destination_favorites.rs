//! Favorite actions of `handy settings destination` (012, gate `destination-return` open):
//! `favorite-add <identifier>` runs inside the running Handy because only its destination
//! manager can name a running application (never launched; `not_found` otherwise), then commits
//! through the instance's write order; `favorite-remove <identifier>` is an ordinary settings
//! change, offline or routed. Both use the validation shared with the badge star.
use super::live_operation::{managed_state, LiveAnswer, LiveCall, LiveContext, LiveOperation};
use super::DomainPlan;
use crate::destination::favorites::{
    add_change, find_identifier, remove_change, FavoriteApp, FAVORITES_KEY,
};
use crate::destination::manager::DestinationManager;
use crate::settings::AppSettings;
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use serde_json::json;
use std::sync::Arc;

// @spec FR-005: Favorites from the settings CLI — .specs/features/012-destination-favorites/spec.md#fr-005
/// Plan of `favorite-remove`: `not_found` now when no favorite has this identity value (the
/// change checks again on the locked settings at commit).
///
/// # Errors
/// `not_found` (favorite lookup message); nothing changes.
pub(super) fn remove_plan(
    identifier: &str,
    settings: &AppSettings,
) -> Result<DomainPlan, SettingsError> {
    find_identifier(&settings.destination_favorites, identifier)
        .ok_or_else(SettingsError::no_favorite)?;
    Ok(DomainPlan::Change {
        change: remove_change(identifier.to_owned()),
        keys: vec![FAVORITES_KEY],
        render: None,
    })
}

/// `favorite-add <identifier>`, run inside the instance.
pub(super) struct FavoriteAddOperation {
    pub(super) identifier: String,
}
impl LiveOperation for FavoriteAddOperation {
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError> {
        if call.dry_run || call.expected.is_some() {
            return Err(SettingsError::new(SettingsErrorCode::UsageError));
        }
        let manager = managed_state::<Arc<DestinationManager>>(context)?;
        add_favorite(&manager, context, &self.identifier)
    }
}

/// Add the running application named `identifier` to the favorites through `context`'s commit;
/// the reply is the committed list.
///
/// # Errors
/// `not_found` (application lookup message) when no running application other than Handy has
/// the identity; the shared validation (`favorites_full`); nothing changes then.
pub(super) fn add_favorite(
    manager: &DestinationManager,
    context: &dyn LiveContext,
    identifier: &str,
) -> Result<LiveAnswer, SettingsError> {
    let app = manager
        .running_application_named(identifier)
        .ok_or_else(SettingsError::no_running_application)?;
    let favorite = FavoriteApp {
        identity: app.identity,
        display_name: app.display_name,
    };
    let read = context.commit_change(&add_change(favorite), None)?;
    let data = json!({ FAVORITES_KEY: read.snapshot.settings.destination_favorites });
    Ok(LiveAnswer::Committed {
        read: Box::new(read),
        data,
    })
}

#[cfg(test)]
#[path = "destination_favorites_tests.rs"]
mod tests;
