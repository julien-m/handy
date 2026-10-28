//! Registration changes applied when the shortcut list of an action changes.
//!
//! Planning is pure: it compares the registrations of the action before and
//! after an edit and decides what to release, what to register, and which
//! surviving registration takes over a native slot that a removed one
//! shared. Slots are shared by inherited conflicts on the Tauri backend,
//! which keeps one native registration per combination whatever the number
//! of actions storing it. Applying goes through a [`RegistrationPort`], so
//! the order of operations and the rollback are tested without a backend.

use log::{error, warn};

use super::identity::ShortcutRegistration;

/// A removed registration whose native slot a surviving registration
/// shares (an inherited conflict). The survivor takes the slot over with its
/// own identity once every addition succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handover {
    pub removed: ShortcutRegistration,
    pub survivor: ShortcutRegistration,
}

/// What an edit changes in the backend registrations of one action.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrationDelta {
    /// Removed registrations whose slot no survivor shares: released first.
    pub released: Vec<ShortcutRegistration>,
    /// New registrations, registered after the releases.
    pub added: Vec<ShortcutRegistration>,
    /// Removed registrations sharing their slot with a survivor.
    pub handovers: Vec<Handover>,
}

/// Operations of the active keyboard backend used by a shortcut edit.
pub trait RegistrationPort {
    /// Registers `registration` so its key events reach its action.
    ///
    /// # Errors
    /// Returns the backend's refusal.
    fn register(&mut self, registration: &ShortcutRegistration) -> Result<(), String>;

    /// Releases the native slot of `registration`.
    ///
    /// # Errors
    /// Returns the backend's refusal.
    fn unregister(&mut self, registration: &ShortcutRegistration) -> Result<(), String>;

    /// Checks that the OS accepts `registration` and releases it at once;
    /// nothing stays registered and no event reaches an action.
    ///
    /// # Errors
    /// Returns the OS refusal, or why the release could not be verified.
    fn probe(&mut self, registration: &ShortcutRegistration) -> Result<(), String>;
}

// @spec AC-002: Inherited conflicts keep their registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Plans the registration change from `before` to `after` (the action's
/// registrations). `others` are the registrations of the other actions that
/// are currently registered; `same_slot` says whether two registrations
/// share one native slot.
pub fn plan_registration_delta(
    before: &[ShortcutRegistration],
    after: &[ShortcutRegistration],
    others: &[ShortcutRegistration],
    same_slot: impl Fn(&ShortcutRegistration, &ShortcutRegistration) -> bool,
) -> RegistrationDelta {
    let survivors: Vec<&ShortcutRegistration> = after
        .iter()
        .filter(|registration| before.contains(registration))
        .chain(others.iter())
        .collect();
    let mut delta = RegistrationDelta {
        added: missing_from(after, before),
        ..RegistrationDelta::default()
    };
    for removed in missing_from(before, after) {
        match survivors.iter().find(|s| same_slot(&removed, s)) {
            Some(survivor) => delta.handovers.push(Handover {
                removed,
                survivor: (*survivor).clone(),
            }),
            None => delta.released.push(removed),
        }
    }
    delta
}

// @spec AC-002: Inherited conflicts keep their registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Plans the registrations of an action that becomes active (post-processing
/// enabled). A registration whose native slot a registration of `others` (or
/// an earlier one of the same action) already holds is not added: the slot
/// stays with its holder, as at startup, and the settings flag the conflict.
pub fn plan_activation(
    registrations: &[ShortcutRegistration],
    others: &[ShortcutRegistration],
    same_slot: impl Fn(&ShortcutRegistration, &ShortcutRegistration) -> bool,
) -> RegistrationDelta {
    let mut added: Vec<ShortcutRegistration> = Vec::new();
    for registration in registrations {
        let is_held = others
            .iter()
            .chain(added.iter())
            .any(|held| same_slot(registration, held));
        if !is_held {
            added.push(registration.clone());
        }
    }
    RegistrationDelta {
        added,
        ..RegistrationDelta::default()
    }
}

// @spec AC-002: Inherited conflicts keep their registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Plans the release of every registration of an action that becomes
/// inactive (post-processing disabled): a slot shared with a registration of
/// `others` is handed over to it instead of being unregistered.
pub fn plan_deactivation(
    registrations: &[ShortcutRegistration],
    others: &[ShortcutRegistration],
    same_slot: impl Fn(&ShortcutRegistration, &ShortcutRegistration) -> bool,
) -> RegistrationDelta {
    plan_registration_delta(registrations, &[], others, same_slot)
}

/// A registration the backend refused, with its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub registration: ShortcutRegistration,
    pub reason: String,
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Registers every addition of an activation delta (see [`plan_activation`]).
/// Activation never fails, as before this feature: a refused addition stays
/// stored and unregistered while the accepted ones stay registered, and each
/// refusal is returned so the settings can flag it. Unlike an edit (AC-004),
/// nothing is rolled back, because enabling post-processing adds no new
/// shortcut.
pub fn register_activation(
    port: &mut impl RegistrationPort,
    delta: &RegistrationDelta,
) -> Vec<Refusal> {
    delta
        .added
        .iter()
        .filter_map(|registration| {
            port.register(registration).err().map(|reason| Refusal {
                registration: registration.clone(),
                reason,
            })
        })
        .collect()
}

/// Entries of `list` that `other` lacks, in order.
fn missing_from(
    list: &[ShortcutRegistration],
    other: &[ShortcutRegistration],
) -> Vec<ShortcutRegistration> {
    list.iter()
        .filter(|registration| !other.contains(registration))
        .cloned()
        .collect()
}

// @spec AC-004: OS refusal rejects the change before saving — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
/// Applies `delta`. Order: releases first, so a replacement can reuse the
/// combination it replaces; then additions; handovers last, because an
/// addition cannot share a survivor's slot (the conflict check refuses it)
/// and a refused addition must leave the shared slot with its holder.
///
/// # Errors
/// Returns the refusal of the first addition, after unregistering the
/// additions made here and registering the released ones again.
pub fn apply_registration_delta(
    port: &mut impl RegistrationPort,
    delta: &RegistrationDelta,
) -> Result<(), String> {
    for registration in &delta.released {
        if let Err(e) = port.unregister(registration) {
            // Recording a shortcut suspends every registration, so the
            // entry may already be unregistered.
            warn!("Failed to unregister shortcut: {}", e);
        }
    }
    for (index, registration) in delta.added.iter().enumerate() {
        if let Err(message) = port.register(registration) {
            restore_registrations(port, &delta.added[..index], &delta.released);
            return Err(message);
        }
    }
    hand_over(port, &delta.handovers);
    Ok(())
}

/// Best-effort rollback of a refused delta so the user's shortcuts keep
/// working exactly as before.
fn restore_registrations(
    port: &mut impl RegistrationPort,
    added: &[ShortcutRegistration],
    released: &[ShortcutRegistration],
) {
    for registration in added {
        if let Err(e) = port.unregister(registration) {
            error!(
                "Failed to roll back shortcut '{}': {}",
                registration.shortcut_id, e
            );
        }
    }
    for registration in released {
        if let Err(e) = port.register(registration) {
            error!(
                "Failed to restore previous shortcut '{}' ({}): {}",
                registration.action_id, registration.shortcut_id, e
            );
        }
    }
}

/// Moves each shared slot to its survivor. Order matters: the slot is
/// released first (whichever action held it), then the survivor registers
/// it under its own identity. A failed release skips the survivor: the slot
/// is either still held (registering would fail and the removed shortcut
/// keeps firing until restart) or already free because every shortcut is
/// suspended for a recorder, whose resume registers the survivor.
fn hand_over(port: &mut impl RegistrationPort, handovers: &[Handover]) {
    for handover in handovers {
        if let Err(e) = port.unregister(&handover.removed) {
            warn!(
                "Could not release shortcut '{}' of '{}' for '{}': {}",
                handover.removed.shortcut_id,
                handover.removed.action_id,
                handover.survivor.action_id,
                e
            );
            continue;
        }
        if let Err(e) = port.register(&handover.survivor) {
            error!(
                "Failed to hand shortcut '{}' over to '{}': {}",
                handover.survivor.shortcut_id, handover.survivor.action_id, e
            );
        }
    }
}

// @spec AC-004: OS check of an inactive action's shortcut — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
/// Probes every addition of `delta` for an action that is not registered
/// right now (post-processing disabled): the OS must accept each one, and
/// none stays registered.
///
/// # Errors
/// Returns the first probe refusal.
pub fn probe_additions(
    port: &mut impl RegistrationPort,
    delta: &RegistrationDelta,
) -> Result<(), String> {
    delta
        .added
        .iter()
        .try_for_each(|registration| port.probe(registration))
}

#[cfg(test)]
mod tests;
