//! `handy settings destination` (009, gate `destination-return` open only): `pin <identifier>`,
//! `unpin` and `status`, answered by the running Handy because the pin lives in its destination
//! manager's RAM (live operations: without a running Handy they are refused with `unavailable`,
//! exit 5, empty details). They never write settings, never advance the revision and never
//! persist the pin. Precedence, identical offline and routed: usage (clap and the framework),
//! then the platform (Linux: `validation_failed`, `unsupported_platform`), then liveness, then the
//! lookup (`not_found`, exit 4). A pin or unpin has the badge's effect on the current dictation.
//! 012 adds `favorite-add <identifier>` (live) and `favorite-remove <identifier>` (a settings
//! change, offline or routed): the only actions of this domain that write settings.
use super::destination_favorites::{remove_plan, FavoriteAddOperation};
use super::live_operation::{managed_state, LiveAnswer, LiveCall, LiveContext, LiveOperation};
use super::{Domain, DomainPlan, DomainRequest};
use crate::destination::manager::DestinationManager;
use crate::destination::snapshot::DestinationSnapshot;
use crate::settings::AppSettings;
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use crate::settings_service::platform_capability::PlatformCapability;
use serde_json::{json, Value};
use std::sync::Arc;

/// The destination domain.
pub struct DestinationDomain;
impl Domain for DestinationDomain {
    fn name(&self) -> &'static str {
        "destination"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["pin", "unpin", "status", "favorite-add", "favorite-remove"]
    }
    // @spec AC-006: destination pin, unpin and status — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-006
    // @spec AC-006: favorite-add and favorite-remove — .specs/features/012-destination-favorites/spec.md#ac-006
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        // The pin is native state, never a setting; only `favorite-remove` reads `settings`.
        let usage = || SettingsError::new(SettingsErrorCode::UsageError);
        let operation: Box<dyn LiveOperation> = match (request.action, request.arguments) {
            ("pin", [identifier]) => Box::new(PinOperation {
                identifier: identifier.clone(),
            }),
            ("unpin", []) => Box::new(UnpinOperation),
            ("status", []) => Box::new(StatusOperation),
            ("favorite-add", [identifier]) => Box::new(FavoriteAddOperation {
                identifier: identifier.clone(),
            }),
            ("favorite-remove", [identifier]) => {
                platform_check(request)?;
                return remove_plan(identifier, settings);
            }
            _ => return Err(usage()),
        };
        platform_check(request)?;
        Ok(DomainPlan::Live(operation))
    }
}

/// The Linux refusal of every destination action (after usage, before liveness).
fn platform_check(request: &DomainRequest<'_>) -> Result<(), SettingsError> {
    request
        .platform
        .check(Some(PlatformCapability::Destination), None)
}

/// `pin <identifier>`, run inside the instance.
struct PinOperation {
    identifier: String,
}
impl LiveOperation for PinOperation {
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError> {
        refuse_change_options(call)?;
        let manager = managed_state::<Arc<DestinationManager>>(context)?;
        let (data, snapshot) = pin_answer(&manager, &self.identifier)?;
        notify(context, snapshot.as_ref());
        Ok(LiveAnswer::Read(data))
    }
}
/// `unpin`, run inside the instance.
struct UnpinOperation;
impl LiveOperation for UnpinOperation {
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError> {
        refuse_change_options(call)?;
        let manager = managed_state::<Arc<DestinationManager>>(context)?;
        let (data, snapshot) = unpin_answer(&manager);
        notify(context, snapshot.as_ref());
        Ok(LiveAnswer::Read(data))
    }
}
/// `status`, run inside the instance.
struct StatusOperation;
impl LiveOperation for StatusOperation {
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError> {
        refuse_change_options(call)?;
        let manager = managed_state::<Arc<DestinationManager>>(context)?;
        Ok(LiveAnswer::Read(status_answer(&manager)))
    }
}

/// The three actions declare no change option (decision `cli_pin_change_options`): clap refuses
/// `--dry-run` and `--expected-revision` before routing; a raw request carrying one is refused
/// the same way here, before anything changes.
fn refuse_change_options(call: &LiveCall) -> Result<(), SettingsError> {
    if call.dry_run || call.expected.is_some() {
        return Err(SettingsError::new(SettingsErrorCode::UsageError));
    }
    Ok(())
}

/// Reply of `pin`: `{pinned: {identity: {kind, value}, display_name}}`, and the snapshot of the
/// current dictation for the badge.
///
/// # Errors
/// `not_found` (lookup message) when no running application other than Handy has the identity.
pub fn pin_answer(
    manager: &DestinationManager,
    identifier: &str,
) -> Result<(Value, Option<DestinationSnapshot>), SettingsError> {
    let change = manager
        .pin_identifier(identifier)
        .map_err(|_| SettingsError::no_running_application())?;
    Ok((json!({ "pinned": change.pinned }), change.snapshot))
}
/// Reply of `unpin`: `{pinned: null}` (a no-op without a pin), and the current snapshot.
pub fn unpin_answer(manager: &DestinationManager) -> (Value, Option<DestinationSnapshot>) {
    (json!({ "pinned": null }), manager.unpin())
}
/// Reply of `status`: the pinned application, or `null`.
pub fn status_answer(manager: &DestinationManager) -> Value {
    json!({ "pinned": manager.pinned() })
}

/// Tell the badge that the current dictation changed (when there is one and a UI to tell).
fn notify(context: &dyn LiveContext, snapshot: Option<&DestinationSnapshot>) {
    if let (Some(app), Some(snapshot)) = (context.app(), snapshot) {
        crate::destination::app::emit_changed(app, snapshot);
    }
}

#[cfg(test)]
#[path = "destination_tests.rs"]
mod tests;
