//! Native side of a change of the stored shortcut lists, shared by the settings UI and the
//! settings CLI: before the commit, every changed action registers its difference with the
//! active keyboard backend (or probes it, see [`super::edits::preparation_for`]); when the commit
//! is refused, the same differences are undone in reverse order, so a refusal leaves both the
//! store and the registrations as they were. Everything goes through a [`RegistrationPort`], so
//! the order and the rollback are tested without a backend.

use log::error;

use super::edits::Preparation;
use super::identity::ShortcutRegistration;
use super::registration_delta::{
    apply_registration_delta, plan_registration_delta, probe_additions, RegistrationDelta,
    RegistrationPort,
};
use super::tauri_impl;
use crate::settings::{self, AppSettings, KeyboardImplementation, ShortcutBinding};

/// The shortcut list of one action before and after a change.
#[derive(Debug, Clone)]
pub(crate) struct ActionChange {
    pub before: ShortcutBinding,
    pub after: ShortcutBinding,
}

/// Actions whose shortcut list differs between `before` and `after`, sorted by id. An action
/// missing from a store is read from its default, as at startup.
pub(crate) fn changed_actions(before: &AppSettings, after: &AppSettings) -> Vec<ActionChange> {
    let mut ids: Vec<&String> = after.bindings.keys().collect();
    ids.sort();
    ids.into_iter()
        .filter_map(|id| {
            let previous = super::edits::current_binding_or_default(before, id)?;
            let next = after.bindings.get(id)?.clone();
            (previous.shortcuts() != next.shortcuts()).then_some(ActionChange {
                before: previous,
                after: next,
            })
        })
        .collect()
}

/// Whether two registrations share one native slot: only on the Tauri
/// backend, whose plugin keeps one registration per parsed combination.
pub(crate) fn slot_sharing(
    implementation: KeyboardImplementation,
) -> impl Fn(&ShortcutRegistration, &ShortcutRegistration) -> bool {
    move |a, b| {
        implementation == KeyboardImplementation::Tauri
            && tauri_impl::same_registration_slot(&a.accelerator, &b.accelerator)
    }
}

/// Registrations of every other known action registered outside recording.
/// Unknown ids (a newer Handy's actions) are skipped, as at startup: they
/// have no handler, so a slot must never be handed over to them.
pub(crate) fn other_registered(settings: &AppSettings, id: &str) -> Vec<ShortcutRegistration> {
    let known = settings::get_default_settings().bindings;
    let mut actions: Vec<(&String, &ShortcutBinding)> = settings
        .bindings
        .iter()
        .filter(|(other, _)| {
            *other != id
                && known.contains_key(*other)
                && super::is_statically_registered(other, settings)
        })
        .collect();
    // Sorted so the survivor of a shared slot does not depend on map order.
    actions.sort_by_key(|(other, _)| *other);
    actions
        .into_iter()
        .flat_map(|(_, binding)| ShortcutRegistration::all_of(binding))
        .collect()
}

/// Registration delta of `from` → `to` for one action, the other actions of `settings` being
/// registered as they are.
fn delta_of(
    settings: &AppSettings,
    from: &ShortcutBinding,
    to: &ShortcutBinding,
) -> RegistrationDelta {
    plan_registration_delta(
        &ShortcutRegistration::all_of(from),
        &ShortcutRegistration::all_of(to),
        &other_registered(settings, &from.id),
        slot_sharing(settings.keyboard_implementation),
    )
}

/// Prepare every change from the registrations of `settings` (the published settings the
/// change starts from), in order. `preparation` says how each action is prepared.
///
/// # Errors
/// The first refusal: the action and the OS or backend reason; the changes prepared before it
/// are undone first.
pub(crate) fn prepare_changes(
    port: &mut impl RegistrationPort,
    settings: &AppSettings,
    changes: &[ActionChange],
    preparation: &impl Fn(&str) -> Preparation,
) -> Result<(), (String, String)> {
    let mut current = settings.clone();
    for (index, change) in changes.iter().enumerate() {
        let delta = delta_of(&current, &change.before, &change.after);
        let prepared = match preparation(&change.after.id) {
            Preparation::ApplyDelta => apply_registration_delta(port, &delta),
            Preparation::Probe => probe_additions(port, &delta),
            Preparation::Nothing => Ok(()),
        };
        if let Err(reason) = prepared {
            compensate_changes(port, settings, &changes[..index], preparation);
            return Err((change.after.id.clone(), reason));
        }
        set_binding(&mut current, &change.after);
    }
    Ok(())
}

/// Undo `prepare_changes` after a refused commit: the stored lists are still those of
/// `settings`. Order matters: the last prepared action is undone first, each against the
/// registrations still in place, so a slot handed over between actions returns to its first
/// holder. A probe left nothing registered.
pub(crate) fn compensate_changes(
    port: &mut impl RegistrationPort,
    settings: &AppSettings,
    changes: &[ActionChange],
    preparation: &impl Fn(&str) -> Preparation,
) {
    let mut current = settings.clone();
    for change in changes {
        set_binding(&mut current, &change.after);
    }
    for change in changes.iter().rev() {
        if preparation(&change.after.id) == Preparation::ApplyDelta {
            let undo = delta_of(&current, &change.after, &change.before);
            if let Err(reason) = apply_registration_delta(port, &undo) {
                error!(
                    "Could not restore the shortcuts of '{}' after a refused commit: {}",
                    change.after.id, reason
                );
            }
        }
        set_binding(&mut current, &change.before);
    }
}

fn set_binding(settings: &mut AppSettings, binding: &ShortcutBinding) {
    settings
        .bindings
        .insert(binding.id.clone(), binding.clone());
}

#[cfg(test)]
#[path = "bindings_change_tests.rs"]
mod tests;
