//! Test backend holding one registration per parsed combination, like the Tauri plugin; the OS
//! refuses the accelerators listed in `refused`, and probes record what they checked.
use std::collections::HashMap;
use tauri_plugin_global_shortcut::Shortcut;

use super::identity::ShortcutRegistration;
use super::registration_delta::RegistrationPort;

/// Fake registry: one registration per native slot, refusals and probes recorded.
#[derive(Default)]
pub(crate) struct FakeBackend {
    pub slots: HashMap<u32, ShortcutRegistration>,
    pub refused: Vec<String>,
    pub probed: Vec<ShortcutRegistration>,
}

/// Native slot of `accelerator`, as the Tauri plugin computes it.
pub(crate) fn slot_of(accelerator: &str) -> Result<u32, String> {
    accelerator
        .parse::<Shortcut>()
        .map(|shortcut| shortcut.id())
        .map_err(|e| e.to_string())
}

impl FakeBackend {
    /// The registration holding the slot of `accelerator`, if any.
    pub fn holder(&self, accelerator: &str) -> Option<&ShortcutRegistration> {
        self.slots.get(&slot_of(accelerator).ok()?)
    }
}

impl RegistrationPort for FakeBackend {
    fn register(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        if self.refused.contains(&registration.accelerator) {
            return Err("refused by the OS".into());
        }
        let slot = slot_of(&registration.accelerator)?;
        if self.slots.contains_key(&slot) {
            return Err("already in use".into());
        }
        self.slots.insert(slot, registration.clone());
        Ok(())
    }

    fn unregister(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        let slot = slot_of(&registration.accelerator)?;
        self.slots
            .remove(&slot)
            .map(|_| ())
            .ok_or("not registered".into())
    }

    fn probe(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        self.probed.push(registration.clone());
        if self.refused.contains(&registration.accelerator) {
            return Err("refused by the OS".into());
        }
        Ok(())
    }
}
