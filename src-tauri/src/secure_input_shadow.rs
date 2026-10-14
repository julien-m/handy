//! Carbon shadow planning for the macOS Secure Input fallback.
//!
//! While Secure Input is held, HandyKeys stops seeing keyed shortcuts, so
//! [`crate::secure_input`] registers a Carbon "shadow" of each one through
//! the Tauri global-shortcut path. This module decides, without any plugin
//! call, what shadow a shortcut needs. Each shadow keeps the identity of
//! the shortcut it stands in for, so a held session started by that
//! shortcut still ends on its release.

use log::{debug, warn};

use crate::shortcut::identity::ShortcutRegistration;

/// Desired fallback for one shortcut, computed without plugin calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShadowPlan {
    /// Modifier-only or mouse-based; unaffected by secure input.
    Immune,
    /// Cannot be represented through Carbon.
    Uncovered,
    /// Register this shadow through Carbon.
    Shadow {
        shadow: ShortcutRegistration,
        degraded: bool,
    },
}

/// Whether `key` is a mouse button, which Secure Input does not block.
pub(crate) fn is_mouse_key(key: &handy_keys::Key) -> bool {
    key.to_string().to_lowercase().starts_with("mouse")
}

/// Why a hotkey needs no shadow, if it needs none.
fn immunity(hotkey: &handy_keys::Hotkey) -> Option<&'static str> {
    match &hotkey.key {
        None => Some("modifier-only"),
        Some(key) if is_mouse_key(key) => Some("mouse-based"),
        Some(_) => None,
    }
}

/// Build the Carbon-registrable equivalent of a keyed hotkey.
///
/// Carbon has no concept of left/right modifiers, so side-specific
/// modifiers widen to the whole group — returned as `degraded: true` so
/// the UI can call out the changed matching. The fn key cannot be
/// expressed at all (`None`).
fn carbon_equivalent(hotkey: &handy_keys::Hotkey) -> Option<(String, bool)> {
    use handy_keys::Modifiers as M;

    if hotkey.modifiers.contains(M::FN) {
        return None;
    }

    let mut widened = M::empty();
    let mut degraded = false;
    for group in [M::CTRL, M::OPT, M::SHIFT, M::CMD] {
        if hotkey.modifiers.intersects(group) {
            widened |= group;
            if !hotkey.modifiers.contains(group) {
                // Only one side was specified — matching gets wider
                degraded = true;
            }
        }
    }

    let carbon_hotkey = handy_keys::Hotkey::new(widened, hotkey.key).ok()?;
    Some((carbon_hotkey.to_handy_string(), degraded))
}

// @spec FR-002: Carbon shadows keep the shortcut identity — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
/// Plans the shadow of `binding`, a shortcut of action `id`.
pub(crate) fn plan_fallback_binding(id: &str, binding: &ShortcutRegistration) -> ShadowPlan {
    let Ok(hotkey) = binding.accelerator.parse::<handy_keys::Hotkey>() else {
        warn!(
            "SecureInput fallback: '{}' has unparseable binding '{}', skipping",
            id, binding.accelerator
        );
        return ShadowPlan::Uncovered;
    };
    if let Some(reason) = immunity(&hotkey) {
        debug!(
            "SecureInput fallback: '{}' ('{}') is {} — immune, no shadow needed",
            id, binding.accelerator, reason
        );
        return ShadowPlan::Immune;
    }
    let Some((carbon_binding, degraded)) = carbon_equivalent(&hotkey) else {
        warn!(
            "SecureInput fallback: '{}' ('{}') cannot be expressed via Carbon",
            id, binding.accelerator
        );
        return ShadowPlan::Uncovered;
    };
    let shadow = binding.with_accelerator(&carbon_binding);
    ShadowPlan::Shadow { shadow, degraded }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(shortcut: &str) -> ShadowPlan {
        plan_fallback_binding(
            "transcribe",
            &ShortcutRegistration::new("transcribe", shortcut),
        )
    }

    fn shadow_of(shortcut: &str) -> Result<(ShortcutRegistration, bool), String> {
        match plan(shortcut) {
            ShadowPlan::Shadow { shadow, degraded } => Ok((shadow, degraded)),
            other => Err(format!("expected a shadow for {shortcut}, got {other:?}")),
        }
    }

    #[test]
    fn widened_shadow_keeps_the_original_identity() -> Result<(), String> {
        let (shadow, degraded) = shadow_of("option_left+space")?;
        assert!(degraded);
        assert_eq!(shadow.action_id, "transcribe");
        assert_eq!(shadow.shortcut_id, "option_left+space");
        assert_ne!(shadow.accelerator, "option_left+space");
        let widened = shadow.accelerator.parse::<handy_keys::Hotkey>();
        let both_sides = handy_keys::Modifiers::OPT;
        let parsed = widened.map_err(|e| e.to_string())?;
        assert!(parsed.modifiers.contains(both_sides));
        Ok(())
    }

    #[test]
    fn full_modifier_groups_are_covered_without_degradation() -> Result<(), String> {
        let (shadow, degraded) = shadow_of("option+space")?;
        assert!(!degraded);
        assert_eq!(shadow.shortcut_id, "option+space");
        Ok(())
    }

    #[test]
    fn each_shortcut_of_an_action_gets_its_own_shadow_identity() -> Result<(), String> {
        let (first, _) = shadow_of("option+space")?;
        let (second, _) = shadow_of("ctrl+option+d")?;
        assert_eq!(first.action_id, second.action_id);
        assert_ne!(first, second);
        Ok(())
    }

    #[test]
    fn modifier_only_shortcuts_are_immune() {
        assert_eq!(plan("option"), ShadowPlan::Immune);
    }

    #[test]
    fn mouse_based_shortcuts_are_immune() {
        assert_eq!(plan("ctrl+mouse4"), ShadowPlan::Immune);
    }

    #[test]
    fn fn_and_unparseable_shortcuts_are_uncovered() {
        assert_eq!(plan("fn+space"), ShadowPlan::Uncovered);
        assert_eq!(plan("not a shortcut"), ShadowPlan::Uncovered);
    }
}
