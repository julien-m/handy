//! Registration of the shortcuts that only exist while AI post-processing is on: the
//! post-processing action and every prompt shortcut (005). Turning post-processing on registers
//! them all, leaving a slot another action holds to it; turning it off releases them all,
//! handing a shared slot back to the action that keeps it.
use log::warn;

use super::bindings_change::{other_registered, slot_sharing};
use super::edits::{current_binding_or_default, RefusedShortcut};
use super::identity::ShortcutRegistration;
use super::prompt_targets::{live_targets, POST_PROCESS_ACTION};
use super::registration_delta::{
    apply_registration_delta, plan_activation, plan_deactivation, register_activation,
    RegistrationPort,
};
use crate::settings::{AppSettings, ShortcutBinding};

/// Bindings registered only while post-processing is on: the post-processing action, then the
/// shortcuts of every prompt (005), in id order.
fn gated_bindings(settings: &AppSettings) -> Vec<ShortcutBinding> {
    let prompt_targets = live_targets(settings);
    std::iter::once(POST_PROCESS_ACTION.to_owned())
        .chain(prompt_targets)
        .filter_map(|id| current_binding_or_default(settings, &id))
        .collect()
}

// @spec AC-002: Inherited conflicts keep their registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
// @spec AC-003: Prompt shortcuts follow the post-processing switch — .specs/features/005-custom-ai-prompts/spec.md#ac-003
/// Registers the post-processing and prompt shortcuts. A slot that another action (or a
/// shortcut registered earlier in this activation) holds stays with it (inherited conflict,
/// flagged by the settings); every other shortcut is registered on its own, and the ones the
/// backend refuses are returned.
pub(crate) fn register_post_process(
    port: &mut impl RegistrationPort,
    settings: &AppSettings,
) -> Vec<RefusedShortcut> {
    let same_slot = slot_sharing(settings.keyboard_implementation);
    let mut held: Vec<ShortcutRegistration> = Vec::new();
    let mut refused = Vec::new();
    for binding in gated_bindings(settings) {
        let mut others = other_registered(settings, &binding.id);
        others.extend(held.iter().cloned());
        let delta = plan_activation(&ShortcutRegistration::all_of(&binding), &others, &same_slot);
        let refusals = register_activation(port, &delta);
        held.extend(
            delta
                .added
                .iter()
                .filter(|added| refusals.iter().all(|r| &r.registration != *added))
                .cloned(),
        );
        refused.extend(refusals.into_iter().map(|refusal| RefusedShortcut {
            action_id: refusal.registration.action_id,
            shortcut: refusal.registration.shortcut_id,
            reason: refusal.reason,
        }));
    }
    refused
}

// @spec AC-002: Inherited conflicts keep their registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
// @spec AC-003: Prompt shortcuts follow the post-processing switch — .specs/features/005-custom-ai-prompts/spec.md#ac-003
/// Releases the post-processing and prompt shortcuts; a slot shared with an action that stays
/// registered is handed over to it instead of being unregistered.
pub(crate) fn release_post_process(port: &mut impl RegistrationPort, settings: &AppSettings) {
    // Others are computed as if post-processing were already off, so no slot is handed over to
    // a gated action that this same release unregisters.
    let mut staying = settings.clone();
    staying.post_process_enabled = false;
    let same_slot = slot_sharing(settings.keyboard_implementation);
    for binding in gated_bindings(settings) {
        let others = other_registered(&staying, &binding.id);
        let delta = plan_deactivation(&ShortcutRegistration::all_of(&binding), &others, &same_slot);
        // A release delta adds nothing, so it cannot be refused; failed releases are already
        // logged by the delta.
        if let Err(reason) = apply_registration_delta(port, &delta) {
            warn!("Releasing the post-processing shortcuts failed: {}", reason);
        }
    }
}
