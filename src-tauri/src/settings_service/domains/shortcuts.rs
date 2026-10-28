//! `handy settings shortcuts` (feature 004): list the shortcuts of every action, and add, remove
//! or reset the shortcuts of one action with the validation, limits and conflict check of the
//! settings UI (`shortcut::edits`). SHORTCUT uses the stored text syntax of the active keyboard
//! implementation. A change is live-only: the running instance registers it before committing
//! one revision (`shortcut::live_change`); without one it is refused with `unavailable`, and a
//! dry run previews it anywhere without checking the system.
use super::{ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest, Render};
use crate::settings::AppSettings;
use crate::settings_service::errors::{
    ErrorReason, SettingsError, SettingsErrorCode, ShortcutDetails,
};
use crate::shortcut::action_shortcuts::{ShortcutEdit, ShortcutEditError};
use crate::shortcut::conflicts::{normalize_shortcut, ConflictKind, ShortcutConflict};
use crate::shortcut::edits::{current_binding_or_default, plan_shortcut_edit};
use crate::shortcut::prompt_targets::{is_live_target, live_targets, store_binding};
use serde_json::{json, Value};

/// The shortcuts domain.
pub struct ShortcutsDomain;
impl Domain for ShortcutsDomain {
    fn name(&self) -> &'static str {
        "shortcuts"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["list", "add", "remove", "reset"]
    }
    // @spec FR-005: Shortcuts CLI list, add, remove, reset — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-005
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        match (request.action, request.arguments) {
            ("list", []) => Ok(DomainPlan::Read(every_action(settings)?)),
            ("list", [action]) => Ok(DomainPlan::Read(projection(
                settings,
                &known(settings, action)?,
            )?)),
            ("add", [action, shortcut]) => {
                let shortcut = shortcut.clone();
                change(
                    settings,
                    known(settings, action)?,
                    ShortcutEdit::Add { shortcut },
                )
            }
            ("remove", [action, shortcut]) => {
                let action = known(settings, action)?;
                let shortcut = stored_entry(settings, &action, shortcut);
                change(settings, action, ShortcutEdit::Remove { shortcut })
            }
            ("reset", [action]) => change(settings, known(settings, action)?, ShortcutEdit::Reset),
            _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
        }
    }
}

/// Actions of this build (the default bindings), sorted.
fn action_ids() -> Vec<String> {
    let mut ids: Vec<String> = crate::settings::get_default_settings()
        .bindings
        .into_keys()
        .collect();
    ids.sort();
    ids
}
/// A known action id, or the shortcut target `prompt:<id>` of an existing prompt (005); an
/// unknown one is `not_found`, without echoing it.
fn known(settings: &AppSettings, action: &str) -> Result<String, SettingsError> {
    if is_live_target(settings, action) {
        return Ok(action.to_owned());
    }
    action_ids()
        .into_iter()
        .find(|id| id == action)
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))
}
/// `{actions: [...]}`: the projection of every action, then of every prompt with shortcuts.
fn every_action(settings: &AppSettings) -> Result<Value, SettingsError> {
    let actions = action_ids()
        .into_iter()
        .chain(live_targets(settings))
        .map(|id| projection(settings, &id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({ "actions": actions }))
}
/// `{action_id, shortcuts, default_shortcut, keyboard_implementation}` of one action.
fn projection(settings: &AppSettings, action: &str) -> Result<Value, SettingsError> {
    let binding = current_binding_or_default(settings, action)
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))?;
    Ok(json!({
        "action_id": action,
        "shortcuts": binding.shortcuts(),
        "default_shortcut": binding.default_binding,
        "keyboard_implementation": settings.keyboard_implementation,
    }))
}
/// The stored entry `candidate` names: itself when stored as typed, otherwise the stored entry
/// with the same normalized combination (`option+ctrl+d` removes a stored `ctrl+option+d`).
/// Unknown or unreadable candidates are returned as typed, so the plan reports `not_found`.
fn stored_entry(settings: &AppSettings, action: &str, candidate: &str) -> String {
    let implementation = settings.keyboard_implementation;
    let shortcuts = current_binding_or_default(settings, action)
        .map(|binding| binding.shortcuts())
        .unwrap_or_default();
    let wanted = normalize_shortcut(candidate, implementation).ok();
    shortcuts
        .iter()
        .find(|stored| {
            stored.as_str() == candidate
                || (wanted.is_some() && normalize_shortcut(stored, implementation).ok() == wanted)
        })
        .cloned()
        .unwrap_or_else(|| candidate.to_string())
}
/// A live change of the action's list, validated now (early typed refusal) and again on the
/// locked settings by the commit; a no-op leaves the settings untouched.
fn change(
    settings: &AppSettings,
    action: String,
    edit: ShortcutEdit,
) -> Result<DomainPlan, SettingsError> {
    plan_shortcut_edit(settings, &action, &edit).map_err(|error| refusal(&action, error))?;
    let target = action.clone();
    let apply = Box::new(move |locked: &mut AppSettings| {
        let (current, planned) =
            plan_shortcut_edit(locked, &target, &edit).map_err(|error| refusal(&target, error))?;
        if planned.shortcuts() != current.shortcuts() {
            store_binding(locked, planned);
        }
        Ok(())
    });
    let render: Render = Box::new(move |committed| projection(committed, &action));
    Ok(DomainPlan::LiveChange {
        key: "bindings",
        change: DomainChange(ChangeKind::Apply(apply)),
        render,
    })
}
// @spec AC-006: Stable shortcut error codes in the CLI envelope — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-006
/// The 008 envelope error of a refused edit of `action`: `shortcut_conflict` with its owner,
/// `limit_exceeded` with the limit, `not_found`, otherwise `validation_failed` with a stable
/// reason. Parser messages are not copied: they can quote the candidate.
pub(crate) fn refusal(action: &str, error: ShortcutEditError) -> SettingsError {
    let details = ShortcutDetails {
        action_id: Some(action.to_string()),
        ..ShortcutDetails::default()
    };
    match error {
        ShortcutEditError::ShortcutConflict { conflict } => conflict_refusal(details, conflict),
        ShortcutEditError::LimitExceeded { limit } => {
            let details = ShortcutDetails {
                limit: Some(limit),
                ..details
            };
            SettingsError::shortcut(SettingsErrorCode::LimitExceeded, None, details)
        }
        ShortcutEditError::NotFound { .. } | ShortcutEditError::ActionNotFound { .. } => {
            SettingsError::shortcut(SettingsErrorCode::NotFound, None, details)
        }
        // Only the settings UI adapter produces it, from an error it already carries.
        ShortcutEditError::SettingsRefused { .. } => {
            SettingsError::new(SettingsErrorCode::InternalError)
        }
        other => {
            let reason = Some(validation_reason(&other));
            SettingsError::shortcut(SettingsErrorCode::ValidationFailed, reason, details)
        }
    }
}
/// Stable `validation_failed` reason of a refused edit.
fn validation_reason(error: &ShortcutEditError) -> ErrorReason {
    match error {
        ShortcutEditError::LastShortcut => ErrorReason::LastShortcut,
        ShortcutEditError::AlternativesUnsupported => ErrorReason::AlternativesUnsupported,
        ShortcutEditError::RegistrationFailed { .. } => ErrorReason::RegistrationRefused,
        _ => ErrorReason::InvalidShortcut,
    }
}
/// `shortcut_conflict` naming the owning action, its stored shortcut and the kind of conflict.
fn conflict_refusal(details: ShortcutDetails, conflict: ShortcutConflict) -> SettingsError {
    let kind = match conflict.kind {
        ConflictKind::Duplicate => "duplicate",
        ConflictKind::ModifierOverlap => "modifier_overlap",
    };
    let details = ShortcutDetails {
        owning_action: Some(conflict.owning_action),
        owning_shortcut: Some(conflict.owning_shortcut),
        kind: Some(kind),
        ..details
    };
    SettingsError::shortcut(SettingsErrorCode::ShortcutConflict, None, details)
}
#[cfg(test)]
#[path = "shortcuts_tests.rs"]
mod tests;
