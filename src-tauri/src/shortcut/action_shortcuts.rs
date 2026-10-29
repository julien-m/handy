//! Pure edits of the ordered shortcut list of one action.
//!
//! An action keeps 1 to [`MAX_SHORTCUTS_PER_ACTION`] shortcuts: the first
//! stays in `current_binding` (where earlier versions read it) and the
//! others live in `additional_bindings`. Every edit is validated here,
//! including the shared conflict check, before anything is registered or
//! saved; the caller applies the returned list.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::conflicts::{find_conflict_of, InventoryEntry, ShortcutConflict};
use super::prompt_targets::prompt_of;
use crate::dictation_keys::is_dictation_key;
use crate::prompts::types::MAX_PROMPT_SHORTCUTS;
use crate::settings::{KeyboardImplementation, ShortcutBinding};
use crate::settings_service::command_error::SettingsCommandError;

// @spec FR-004: 1-4 shortcuts, last one kept — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-004
/// Upper bound of shortcuts per action. Spec assumption: four covers "at
/// least two" without cluttering the settings.
pub const MAX_SHORTCUTS_PER_ACTION: usize = 4;

/// Actions that accept alternative shortcuts. Cancel keeps its single
/// dynamic shortcut; prompt shortcuts (005) bring their own policy.
const MULTI_SHORTCUT_ACTIONS: [&str; 2] = ["transcribe", "transcribe_with_post_process"];

/// Whether `action_id` accepts more than one shortcut.
pub fn accepts_alternatives(action_id: &str) -> bool {
    MULTI_SHORTCUT_ACTIONS.contains(&action_id) || prompt_of(action_id).is_some()
}

// @spec AC-006: 0-2 shortcuts per prompt — .specs/features/005-custom-ai-prompts/spec.md#ac-006
/// Most shortcuts `action_id` may have: 2 for a prompt target (005), 1 for a key read during a
/// dictation (014), 4 otherwise.
pub fn max_shortcuts(action_id: &str) -> usize {
    if prompt_of(action_id).is_some() {
        MAX_PROMPT_SHORTCUTS
    } else if is_dictation_key(action_id) {
        1
    } else {
        MAX_SHORTCUTS_PER_ACTION
    }
}

/// A requested change to the shortcut list of one action.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShortcutEdit {
    /// Append an alternative shortcut.
    Add { shortcut: String },
    /// Remove one shortcut; removing the first promotes the next one.
    Remove { shortcut: String },
    /// Replace one shortcut in place.
    Replace { previous: String, shortcut: String },
    /// Restore the single default shortcut.
    Reset,
}

/// Why an edit was refused. Codes map to the settings CLI envelope (008):
/// `shortcut_conflict` and `limit_exceeded` as is, `not_found` for unknown
/// entries, every other code as `validation_failed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ShortcutEditError {
    ShortcutConflict {
        conflict: ShortcutConflict,
    },
    LimitExceeded {
        limit: u32,
    },
    LastShortcut,
    AlternativesUnsupported,
    InvalidShortcut {
        message: String,
    },
    NotFound {
        shortcut: String,
    },
    ActionNotFound {
        action_id: String,
    },
    RegistrationFailed {
        message: String,
    },
    /// The settings service refused or could not save the change (persistence, unavailable):
    /// nothing was saved, and the registrations are as before.
    SettingsRefused {
        error: SettingsCommandError,
    },
}

/// Inputs shared by every edit check.
pub struct EditContext<'a> {
    pub inventory: &'a [InventoryEntry],
    pub implementation: KeyboardImplementation,
}

/// Returns the action's shortcut list after `edit`, or why it is refused.
/// An unchanged result means the edit is a no-op.
///
/// # Errors
/// Any [`ShortcutEditError`] except `RegistrationFailed`, which only the
/// registration step can produce.
pub fn plan_edit(
    binding: &ShortcutBinding,
    edit: &ShortcutEdit,
    context: &EditContext,
) -> Result<ShortcutBinding, ShortcutEditError> {
    match edit {
        ShortcutEdit::Add { shortcut } => plan_add(binding, shortcut, context),
        ShortcutEdit::Remove { shortcut } => plan_remove(binding, shortcut),
        ShortcutEdit::Replace { previous, shortcut } => {
            plan_replace(binding, previous, shortcut, context)
        }
        ShortcutEdit::Reset => plan_reset(binding, context),
    }
}

fn plan_add(
    binding: &ShortcutBinding,
    shortcut: &str,
    context: &EditContext,
) -> Result<ShortcutBinding, ShortcutEditError> {
    // @spec FR-001: One key or none, so the CLI can change it — .specs/features/014-in-dictation-keys/spec.md#fr-001
    // A key read during a dictation (014) holds one key or none: Add fills an empty list.
    if !accepts_alternatives(&binding.id) && !is_dictation_key(&binding.id) {
        return Err(ShortcutEditError::AlternativesUnsupported);
    }
    let limit = max_shortcuts(&binding.id);
    if binding.shortcuts().len() >= limit {
        return Err(ShortcutEditError::LimitExceeded {
            limit: limit as u32,
        });
    }
    check_candidate(&binding.id, shortcut, &[], context)?;
    // `with_shortcuts` puts the first shortcut of an empty prompt target in `current_binding`.
    let mut shortcuts = binding.shortcuts();
    shortcuts.push(shortcut.to_string());
    Ok(with_shortcuts(binding, shortcuts))
}

// @spec AC-005: No action left without a shortcut — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-005
fn plan_remove(
    binding: &ShortcutBinding,
    shortcut: &str,
) -> Result<ShortcutBinding, ShortcutEditError> {
    let mut shortcuts = binding.shortcuts();
    let Some(index) = shortcuts.iter().position(|s| s == shortcut) else {
        return Err(ShortcutEditError::NotFound {
            shortcut: shortcut.to_string(),
        });
    };
    // A prompt target may have no shortcut (005); a dictation action keeps at least one.
    // A key read during a dictation (014) may be removed too: the key is then off.
    if shortcuts.len() == 1 && prompt_of(&binding.id).is_none() && !is_dictation_key(&binding.id) {
        return Err(ShortcutEditError::LastShortcut);
    }
    shortcuts.remove(index);
    // Removing the first entry promotes the next one into `current_binding`.
    Ok(with_shortcuts(binding, shortcuts))
}

fn plan_replace(
    binding: &ShortcutBinding,
    previous: &str,
    shortcut: &str,
    context: &EditContext,
) -> Result<ShortcutBinding, ShortcutEditError> {
    let mut shortcuts = binding.shortcuts();
    let Some(index) = shortcuts.iter().position(|s| s == previous) else {
        return Err(ShortcutEditError::NotFound {
            shortcut: previous.to_string(),
        });
    };
    // A true no-op keeps the entry as is, even when it is a legacy conflict.
    if previous == shortcut {
        return Ok(binding.clone());
    }
    check_candidate(&binding.id, shortcut, &[previous.to_string()], context)?;
    shortcuts[index] = shortcut.to_string();
    Ok(with_shortcuts(binding, shortcuts))
}

fn plan_reset(
    binding: &ShortcutBinding,
    context: &EditContext,
) -> Result<ShortcutBinding, ShortcutEditError> {
    // A prompt target has no default shortcut: reset removes them all (005).
    if prompt_of(&binding.id).is_some() {
        return Ok(with_shortcuts(binding, Vec::new()));
    }
    let reset = with_shortcuts(binding, vec![binding.default_binding.clone()]);
    if reset.shortcuts() == binding.shortcuts() {
        return Ok(reset);
    }
    // Reset replaces the whole list, so none of its current entries can
    // collide with the default.
    check_candidate(
        &binding.id,
        &binding.default_binding,
        &binding.shortcuts(),
        context,
    )?;
    Ok(reset)
}

/// Validates syntax for the active implementation, then checks conflicts
/// against every entry except the entries of `action_id` listed in
/// `replaced` (they leave the list with this edit).
fn check_candidate(
    action_id: &str,
    shortcut: &str,
    replaced: &[String],
    context: &EditContext,
) -> Result<(), ShortcutEditError> {
    let invalid = |message| ShortcutEditError::InvalidShortcut { message };
    validate_syntax(action_id, shortcut, context.implementation).map_err(invalid)?;
    let remaining: Vec<InventoryEntry> = context
        .inventory
        .iter()
        .filter(|entry| !(entry.action_id == action_id && replaced.contains(&entry.shortcut)))
        .cloned()
        .collect();
    match find_conflict_of(
        action_id,
        shortcut,
        &remaining,
        None,
        context.implementation,
    ) {
        Ok(None) => Ok(()),
        Ok(Some(conflict)) => Err(ShortcutEditError::ShortcutConflict { conflict }),
        Err(message) => Err(invalid(message)),
    }
}

/// Syntax check of `shortcut` for `action_id`: a key read during a dictation (014) is one
/// modifier pressed alone, in HandyKeys syntax; any other shortcut follows `implementation`.
fn validate_syntax(
    action_id: &str,
    shortcut: &str,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    if is_dictation_key(action_id) {
        crate::dictation_keys::parse_key(shortcut).map(|_| ())
    } else {
        super::validate_shortcut_for_implementation(shortcut, implementation)
    }
}

/// `binding` with `shortcuts` as its ordered list: the first one goes to
/// `current_binding`, the others to `additional_bindings`.
pub(crate) fn with_shortcuts(binding: &ShortcutBinding, shortcuts: Vec<String>) -> ShortcutBinding {
    let mut planned = binding.clone();
    let mut entries = shortcuts.into_iter();
    planned.current_binding = entries.next().unwrap_or_default();
    planned.additional_bindings = entries.collect();
    planned
}

#[cfg(test)]
mod tests;
