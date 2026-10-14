//! Repair of the shortcut lists when the keyboard implementation changes.
//!
//! Each list is re-read with the destination parser: an unreadable first
//! shortcut is reset to the action default, unreadable alternatives are
//! removed, and an alternative equivalent to that reset default is merged
//! into it. Conflicts follow the inherited-conflict rule (decision
//! 2026-10-12): conflicting shortcuts are kept and registered as before,
//! and the ones the switch introduces are reported so the settings flag
//! them; nothing is removed silently.

use std::collections::HashMap;

use super::action_shortcuts::with_shortcuts;
use super::conflicts::{
    find_stored_conflicts, inventory_of, normalize_shortcut, NormalizedCombination,
    StoredShortcutConflict,
};
use crate::settings::{KeyboardImplementation, ShortcutBinding};

/// Bindings after a switch repair.
#[derive(Debug, Clone)]
pub struct SwitchRepair {
    /// Every stored binding, the repaired ones included.
    pub bindings: HashMap<String, ShortcutBinding>,
    /// Bindings to register with the destination implementation, by id.
    pub active: Vec<ShortcutBinding>,
    /// Ids of the bindings whose list changed, sorted.
    pub repaired: Vec<String>,
}

/// Comparable form of `raw` when `implementation` can register it.
///
/// # Errors
/// Returns the validation or parser message of `implementation`.
pub fn destination_form(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Result<NormalizedCombination, String> {
    super::validate_shortcut_for_implementation(raw, implementation)?;
    normalize_shortcut(raw, implementation)
}

// @spec AC-002: Switch repair keeps inherited conflicts — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Repairs one list for the destination implementation. `normalize`
/// returns the comparable form of a shortcut the destination accepts, or
/// `None` when it cannot register it. Valid entries keep their order.
pub fn repair_for_implementation(
    binding: &ShortcutBinding,
    default_shortcut: &str,
    normalize: impl Fn(&str) -> Option<NormalizedCombination>,
) -> ShortcutBinding {
    let (first, reset_default) = match normalize(&binding.current_binding) {
        Some(_) => (binding.current_binding.clone(), None),
        None => (default_shortcut.to_string(), normalize(default_shortcut)),
    };
    let mut shortcuts = vec![first];
    for alternative in &binding.additional_bindings {
        // An unreadable alternative is removed (reported with the binding).
        let Some(normalized) = normalize(alternative) else {
            continue;
        };
        // Merged into the reset default: both would claim one registration.
        let merges_into_default = reset_default.as_ref() == Some(&normalized);
        if merges_into_default || shortcuts.contains(alternative) {
            continue;
        }
        shortcuts.push(alternative.clone());
    }
    with_shortcuts(binding, shortcuts)
}

/// Repairs every binding that `is_active` (registered outside recording);
/// a binding missing from the store is repaired from its default.
pub fn plan_switch_repair(
    bindings: &HashMap<String, ShortcutBinding>,
    defaults: &HashMap<String, ShortcutBinding>,
    is_active: impl Fn(&str) -> bool,
    normalize: impl Fn(&str) -> Option<NormalizedCombination>,
) -> SwitchRepair {
    let mut actions: Vec<(&String, &ShortcutBinding)> =
        defaults.iter().filter(|(id, _)| is_active(id)).collect();
    actions.sort_by_key(|(id, _)| *id);
    let mut repair = SwitchRepair {
        bindings: bindings.clone(),
        active: Vec::new(),
        repaired: Vec::new(),
    };
    for (id, default) in actions {
        let binding = bindings.get(id).unwrap_or(default);
        let repaired = repair_for_implementation(binding, &default.current_binding, &normalize);
        if repaired.shortcuts() != binding.shortcuts() {
            repair.bindings.insert(id.clone(), repaired.clone());
            repair.repaired.push(id.clone());
        }
        repair.active.push(repaired);
    }
    repair
}

/// Stored shortcuts that conflict under `to` and did not under `from`
/// before the switch. They stay registered as inherited conflicts are, and
/// the switch reports them.
pub fn introduced_conflicts(
    before: &HashMap<String, ShortcutBinding>,
    from: KeyboardImplementation,
    after: &HashMap<String, ShortcutBinding>,
    to: KeyboardImplementation,
) -> Vec<StoredShortcutConflict> {
    let previous = find_stored_conflicts(&inventory_of(before), from);
    find_stored_conflicts(&inventory_of(after), to)
        .into_iter()
        .filter(|conflict| {
            !previous
                .iter()
                .any(|p| p.action_id == conflict.action_id && p.shortcut == conflict.shortcut)
        })
        .collect()
}

#[cfg(test)]
mod tests;
