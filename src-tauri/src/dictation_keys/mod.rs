//! Keys pressed during a dictation (014): two shortcut actions that act on the dictation in
//! progress only, from its recording start until its insertion begins. « Send to the app in
//! front » re-routes this dictation to the application in front at the end (the session pin
//! stays), « Insert without Enter » skips Automatic Enter for this dictation. Each key is one
//! modifier key pressed alone: a press then a release with no other key in between
//! (`tap.rs`). Shift used for a capital letter, Control in Ctrl+C or Option in Option+Space never
//! trigger anything. The keys are read by a listener that only observes (never blocks) and runs
//! only while a dictation can still change (`watcher.rs`); they are never registered with a
//! keyboard backend, unlike every other action.
pub mod tap;
pub mod watcher;

use crate::settings::ShortcutBinding;
use handy_keys::Modifiers;

// @spec FR-001: Two in-dictation key actions — .specs/features/014-in-dictation-keys/spec.md#fr-001
/// Action id of « Send to the app in front ».
pub const RETARGET_ACTION: &str = "retarget_to_active_app";
/// Action id of « Insert without Enter ».
pub const SKIP_ENTER_ACTION: &str = "skip_automatic_enter";

/// Whether `action_id` is a key read only during a dictation (never registered with a backend).
pub fn is_dictation_key(action_id: &str) -> bool {
    action_id == RETARGET_ACTION || action_id == SKIP_ENTER_ACTION
}

// @spec AC-005: Left-hand defaults, single modifier taps — .specs/features/014-in-dictation-keys/spec.md#ac-005
/// Default of « Send to the app in front »: Option left on macOS (decision `left_hand_defaults`);
/// Shift left elsewhere, because a lone Alt press opens the menu bar of Windows applications.
fn default_retarget_key() -> &'static str {
    if cfg!(target_os = "macos") {
        "option_left"
    } else {
        "shift_left"
    }
}
/// Default of « Insert without Enter »: Control left everywhere.
const DEFAULT_SKIP_ENTER_KEY: &str = "control_left";

fn binding(id: &str, name: &str, description: &str, key: &str) -> ShortcutBinding {
    ShortcutBinding {
        id: id.to_owned(),
        name: name.to_owned(),
        description: description.to_owned(),
        default_binding: key.to_owned(),
        current_binding: key.to_owned(),
        additional_bindings: Vec::new(),
    }
}

/// The default bindings of both actions.
pub fn default_bindings() -> [ShortcutBinding; 2] {
    [
        binding(
            RETARGET_ACTION,
            "Send to the App in Front",
            "During a dictation, sends it to the app in front at the end instead of the pinned app.",
            default_retarget_key(),
        ),
        binding(
            SKIP_ENTER_ACTION,
            "Insert without Enter",
            "During a dictation, inserts the text without the automatic Enter.",
            DEFAULT_SKIP_ENTER_KEY,
        ),
    ]
}

/// The one modifier key `raw` names (a side, or both sides when no side is given), in the
/// HandyKeys syntax whatever keyboard implementation is active: the listener of `watcher.rs` is
/// HandyKeys' own (decision `single_modifier_taps`).
///
/// # Errors
/// The text is not exactly one modifier key (a key, several modifiers, or unreadable).
pub fn parse_key(raw: &str) -> Result<Modifiers, String> {
    let hotkey = raw
        .trim()
        .parse::<handy_keys::Hotkey>()
        .map_err(|error| format!("Invalid key for HandyKeys: {error}"))?;
    if hotkey.key.is_some() || !is_single_modifier(hotkey.modifiers) {
        return Err("A key used during a dictation is one modifier key pressed alone".to_owned());
    }
    Ok(hotkey.modifiers)
}

/// True when `modifiers` hold exactly one modifier family (one or both of its sides).
fn is_single_modifier(modifiers: Modifiers) -> bool {
    let families = [
        Modifiers::CMD,
        Modifiers::SHIFT,
        Modifiers::CTRL,
        Modifiers::OPT,
        Modifiers::FN,
    ];
    let held = families
        .iter()
        .filter(|family| modifiers.intersects(**family))
        .count();
    held == 1 && families.iter().any(|family| family.contains(modifiers))
}

#[cfg(test)]
mod tests;
