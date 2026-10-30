//! The two in-dictation key actions: defaults and the one-modifier syntax (014 AC-005).
use super::*;
use crate::settings::get_default_settings;

#[test]
fn both_actions_are_default_bindings_with_one_left_modifier_key() -> Result<(), String> {
    let defaults = get_default_settings();
    for id in [RETARGET_ACTION, SKIP_ENTER_ACTION] {
        let binding = &defaults.bindings[id];
        assert!(is_dictation_key(id));
        let key = parse_key(&binding.default_binding).map_err(|e| format!("{id}: {e}"))?;
        let left = Modifiers::CMD_LEFT
            | Modifiers::SHIFT_LEFT
            | Modifiers::CTRL_LEFT
            | Modifiers::OPT_LEFT;
        assert!(left.contains(key), "{id} is on the left of the keyboard");
        assert_eq!(binding.shortcuts(), vec![binding.default_binding.clone()]);
    }
    assert_ne!(
        defaults.bindings[RETARGET_ACTION].default_binding,
        defaults.bindings[SKIP_ENTER_ACTION].default_binding
    );
    assert!(!is_dictation_key("transcribe"));
    assert!(!is_dictation_key("cancel"));
    Ok(())
}

#[test]
fn a_key_is_exactly_one_modifier_key() {
    for accepted in [
        "option_left",
        "control_left",
        "shift_right",
        "command_right",
        "option",
    ] {
        assert!(parse_key(accepted).is_ok(), "{accepted}");
    }
    for refused in [
        "option+space",
        "control_left+option_left",
        "f13",
        "space",
        "",
        "nope",
    ] {
        assert!(parse_key(refused).is_err(), "{refused}");
    }
}

mod conflicts {
    use super::*;
    use crate::settings::KeyboardImplementation::{self, HandyKeys, Tauri};
    use crate::shortcut::action_shortcuts::{ShortcutEdit, ShortcutEditError};
    use crate::shortcut::conflicts::{find_conflict_of, ConflictKind, InventoryEntry};

    fn entry(action_id: &str, shortcut: &str) -> InventoryEntry {
        InventoryEntry {
            action_id: action_id.to_owned(),
            shortcut: shortcut.to_owned(),
        }
    }
    fn kind_of(
        action: &str,
        candidate: &str,
        inventory: &[InventoryEntry],
        implementation: KeyboardImplementation,
    ) -> Option<ConflictKind> {
        find_conflict_of(action, candidate, inventory, None, implementation)
            .ok()
            .flatten()
            .map(|conflict| conflict.kind)
    }

    // @spec AC-006: Option+Space and Option left coexist — .specs/features/014-in-dictation-keys/spec.md#ac-006
    #[test]
    fn a_lone_press_key_never_conflicts_with_a_combination_holding_it() {
        let julien = [
            entry("transcribe", "option+space"),
            entry("transcribe", "command_left+j"),
            entry("transcribe", "command_right"),
            entry("transcribe_with_post_process", "option+shift+space"),
            entry("cancel", "escape"),
        ];
        for implementation in [HandyKeys, Tauri] {
            for key in ["option_left", "control_left", "shift_left"] {
                let found = kind_of(RETARGET_ACTION, key, &julien, implementation);
                assert_eq!(found, None, "{key} {implementation:?}");
            }
        }
        let taps = [
            entry(RETARGET_ACTION, "option_left"),
            entry(SKIP_ENTER_ACTION, "control_left"),
        ];
        for combination in ["option+space", "ctrl+c", "option+shift+space"] {
            let found = kind_of("transcribe", combination, &taps, HandyKeys);
            assert_eq!(found, None, "{combination} against the lone-press keys");
        }
    }

    #[test]
    fn one_press_that_would_fire_two_actions_is_refused() {
        let taps = [entry(RETARGET_ACTION, "option_left")];
        let duplicate = Some(ConflictKind::Duplicate);
        let other_tap = kind_of(SKIP_ENTER_ACTION, "option_left", &taps, Tauri);
        assert_eq!(other_tap, duplicate);
        let other_side = kind_of(SKIP_ENTER_ACTION, "option_right", &taps, HandyKeys);
        assert_eq!(other_side, None);
        assert_eq!(kind_of("transcribe", "option", &taps, HandyKeys), duplicate);
        let dictation = [entry("transcribe", "command_right")];
        let same = kind_of(RETARGET_ACTION, "command_right", &dictation, HandyKeys);
        assert_eq!(same, duplicate);
        let left = kind_of(RETARGET_ACTION, "command_left", &dictation, HandyKeys);
        assert_eq!(left, None);
    }

    #[test]
    fn the_settings_accept_one_modifier_key_and_refuse_anything_else() {
        let settings = get_default_settings();
        let previous = settings.bindings[RETARGET_ACTION].current_binding.clone();
        let replace = |shortcut: &str| ShortcutEdit::Replace {
            previous: previous.clone(),
            shortcut: shortcut.to_owned(),
        };
        let check = |edit: &ShortcutEdit| {
            crate::shortcut::edits::check_shortcut_edit(&settings, RETARGET_ACTION, edit)
        };
        assert_eq!(check(&replace("shift_right")), None);
        assert!(matches!(
            check(&replace("option+space")),
            Some(ShortcutEditError::InvalidShortcut { .. })
        ));
        let add = ShortcutEdit::Add {
            shortcut: "shift_right".to_owned(),
        };
        assert_eq!(
            check(&add),
            Some(ShortcutEditError::LimitExceeded { limit: 1 })
        );
        let remove = ShortcutEdit::Remove {
            shortcut: previous.clone(),
        };
        assert_eq!(check(&remove), None, "the key may be turned off");
        assert!(!crate::shortcut::is_statically_registered(
            RETARGET_ACTION,
            &settings
        ));
    }
}
