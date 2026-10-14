//! Identity of one registered shortcut.
//!
//! An action can own several shortcuts, so a key event must say both which
//! action it triggers and which of the action's shortcuts produced it: hold
//! modes end a session only on the release of the shortcut that started it.

use crate::settings::ShortcutBinding;

// @spec FR-002: Events carry action and shortcut identity — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
/// One shortcut of one action, as registered with a keyboard backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutRegistration {
    /// Action triggered by the shortcut (the binding id, e.g. `transcribe`).
    pub action_id: String,
    /// Stored text of the shortcut. Identifies the entry within its action
    /// independently of its list position.
    pub shortcut_id: String,
    /// Text handed to the backend parser. Equal to `shortcut_id` except for
    /// Secure Input Carbon shadows, which widen side-specific modifiers but
    /// keep the original `shortcut_id`.
    pub accelerator: String,
}

impl ShortcutRegistration {
    /// Registration of `shortcut` for `action_id`, parsed as stored.
    pub fn new(action_id: &str, shortcut: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            shortcut_id: shortcut.to_string(),
            accelerator: shortcut.to_string(),
        }
    }

    /// One registration per shortcut of `binding`, in list order.
    pub fn all_of(binding: &ShortcutBinding) -> Vec<Self> {
        binding
            .shortcuts()
            .iter()
            .map(|shortcut| Self::new(&binding.id, shortcut))
            .collect()
    }

    /// The same identity registered under another accelerator text.
    pub fn with_accelerator(&self, accelerator: &str) -> Self {
        Self {
            accelerator: accelerator.to_string(),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shortcut_of_a_binding_gets_its_own_identity() {
        let mut binding = crate::settings::get_default_settings().bindings["transcribe"].clone();
        binding.current_binding = "f13".into();
        binding.additional_bindings = vec!["f14".into()];
        let registrations = ShortcutRegistration::all_of(&binding);
        let ids: Vec<(&str, &str)> = registrations
            .iter()
            .map(|r| (r.action_id.as_str(), r.shortcut_id.as_str()))
            .collect();
        assert_eq!(ids, vec![("transcribe", "f13"), ("transcribe", "f14")]);
    }

    #[test]
    fn shadow_keeps_the_original_identity() {
        let original = ShortcutRegistration::new("transcribe", "option_left+space");
        let shadow = original.with_accelerator("option+space");
        assert_eq!(shadow.shortcut_id, "option_left+space");
        assert_eq!(shadow.accelerator, "option+space");
    }
}
