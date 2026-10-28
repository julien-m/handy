//! Prompt shortcut targets (005 AC-006): a prompt's 0–2 shortcuts are stored as one more entry
//! of the shared `bindings` map, under the action id `prompt:<prompt id>`, so 004's conflict
//! check, registration delta, preparation and quiescence lease apply to them unchanged. A prompt
//! without shortcuts has no entry. Like the post-processing shortcut, they are registered only
//! while AI post-processing is on; their assignments stay stored (and conflict-checked) when it
//! is off.
use crate::settings::{AppSettings, ShortcutBinding};

/// Action-id prefix of a prompt target.
pub const PROMPT_TARGET_PREFIX: &str = "prompt:";
/// Action id of the post-processing shortcut.
pub const POST_PROCESS_ACTION: &str = "transcribe_with_post_process";

/// Action id of the shortcuts of prompt `prompt_id`.
pub fn target_of(prompt_id: &str) -> String {
    format!("{PROMPT_TARGET_PREFIX}{prompt_id}")
}

/// Prompt id of a prompt target, `None` for any other action.
pub fn prompt_of(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(PROMPT_TARGET_PREFIX)
}

/// Whether `action_id`'s shortcuts are registered only while post-processing is on: the
/// post-processing shortcut and every prompt target.
pub fn is_post_process_gated(action_id: &str) -> bool {
    action_id == POST_PROCESS_ACTION || prompt_of(action_id).is_some()
}

/// Whether `action_id` is a prompt target of a prompt `settings` still has.
pub fn is_live_target(settings: &AppSettings, action_id: &str) -> bool {
    prompt_of(action_id).is_some_and(|id| {
        settings
            .post_process_prompts
            .iter()
            .any(|prompt| prompt.id == id)
    })
}

/// Binding of prompt `prompt_id` before any shortcut: no shortcut and no default.
pub fn empty_binding(prompt_id: &str) -> ShortcutBinding {
    ShortcutBinding {
        id: target_of(prompt_id),
        name: "Prompt".to_owned(),
        description: String::new(),
        default_binding: String::new(),
        current_binding: String::new(),
        additional_bindings: Vec::new(),
    }
}

/// Store `planned` in `settings`: a prompt target left without shortcuts loses its entry, every
/// other binding is inserted as is.
pub fn store_binding(settings: &mut AppSettings, planned: ShortcutBinding) {
    if prompt_of(&planned.id).is_some() && planned.shortcuts().is_empty() {
        settings.bindings.remove(&planned.id);
    } else {
        settings.bindings.insert(planned.id.clone(), planned);
    }
}

/// Remove the shortcuts of prompt `prompt_id` from `settings`; returns the removed shortcuts.
pub fn remove_target(settings: &mut AppSettings, prompt_id: &str) -> Vec<String> {
    settings
        .bindings
        .remove(&target_of(prompt_id))
        .map(|binding| binding.shortcuts())
        .unwrap_or_default()
}

/// Shortcuts of prompt `prompt_id` in `settings`, in order.
pub fn shortcuts_of(settings: &AppSettings, prompt_id: &str) -> Vec<String> {
    settings
        .bindings
        .get(&target_of(prompt_id))
        .map(ShortcutBinding::shortcuts)
        .unwrap_or_default()
}

/// Prompt target ids stored in `settings` for prompts it still has, sorted.
pub fn live_targets(settings: &AppSettings) -> Vec<String> {
    let mut ids: Vec<String> = settings
        .bindings
        .keys()
        .filter(|id| is_live_target(settings, id))
        .cloned()
        .collect();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{get_default_settings, LLMPrompt};

    fn with_prompt(id: &str) -> AppSettings {
        let mut settings = get_default_settings();
        settings.post_process_prompts = vec![LLMPrompt::new(id.into(), "Notes".into(), "x".into())];
        settings
    }

    #[test]
    fn prompt_targets_are_named_after_their_prompt_and_gated_like_post_processing() {
        assert_eq!(target_of("abc"), "prompt:abc");
        assert_eq!(prompt_of("prompt:abc"), Some("abc"));
        assert_eq!(prompt_of("transcribe"), None);
        assert!(is_post_process_gated("prompt:abc"));
        assert!(is_post_process_gated(POST_PROCESS_ACTION));
        assert!(!is_post_process_gated("transcribe"));
    }
    #[test]
    fn a_target_without_shortcuts_has_no_entry() {
        let mut settings = with_prompt("abc");
        let mut binding = empty_binding("abc");
        assert!(binding.shortcuts().is_empty());
        binding.current_binding = "f13".into();
        store_binding(&mut settings, binding.clone());
        assert_eq!(shortcuts_of(&settings, "abc"), ["f13"]);
        assert_eq!(live_targets(&settings), ["prompt:abc"]);
        binding.current_binding.clear();
        store_binding(&mut settings, binding);
        assert!(!settings.bindings.contains_key("prompt:abc"));
    }
    #[test]
    fn removing_a_target_returns_its_shortcuts() {
        let mut settings = with_prompt("abc");
        let mut binding = empty_binding("abc");
        binding.current_binding = "f13".into();
        binding.additional_bindings = vec!["f14".into()];
        store_binding(&mut settings, binding);
        assert_eq!(remove_target(&mut settings, "abc"), ["f13", "f14"]);
        assert!(remove_target(&mut settings, "abc").is_empty());
    }
    #[test]
    fn a_target_of_a_deleted_prompt_is_not_live() {
        let mut settings = with_prompt("abc");
        let mut binding = empty_binding("gone");
        binding.current_binding = "f13".into();
        store_binding(&mut settings, binding);
        assert!(live_targets(&settings).is_empty());
    }
}
#[cfg(test)]
#[path = "prompt_shortcuts_tests.rs"]
mod shortcut_tests;
