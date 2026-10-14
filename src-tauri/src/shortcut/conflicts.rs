//! Shared shortcut conflict checker.
//!
//! One checker for every caller (settings commands today, the settings CLI
//! once 008 lands, prompt shortcuts from 005): a candidate combination is
//! normalized with the active keyboard implementation's own parser, then
//! compared with every configured shortcut of every action. Equal
//! combinations and modifier-only overlaps are conflicts.

use serde::Serialize;
use specta::Type;
use std::collections::HashMap;

use crate::settings::{KeyboardImplementation, ShortcutBinding};

/// Modifier families compared regardless of side (left/right) or alias
/// (`option`/`alt`, `cmd`/`command`/`super`/`meta`).
const FAMILY_CTRL: u8 = 1;
const FAMILY_ALT: u8 = 1 << 1;
const FAMILY_SHIFT: u8 = 1 << 2;
const FAMILY_META: u8 = 1 << 3;
const FAMILY_FN: u8 = 1 << 4;

/// Comparable form of a shortcut: its modifier families and its main key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCombination {
    modifiers: u8,
    key: Option<String>,
}

impl NormalizedCombination {
    fn is_modifier_only(&self) -> bool {
        self.key.is_none()
    }

    /// Whether every modifier family of `self` is also held by `other`.
    fn modifiers_within(&self, other: &NormalizedCombination) -> bool {
        self.modifiers & other.modifiers == self.modifiers
    }
}

/// One configured shortcut of one action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEntry {
    pub action_id: String,
    pub shortcut: String,
}

/// Why a candidate cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    /// The same combination is already assigned.
    Duplicate,
    /// A modifier-only shortcut would fire together with the other one.
    ModifierOverlap,
}

/// A conflict with an existing shortcut, naming its owning action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ShortcutConflict {
    pub kind: ConflictKind,
    pub owning_action: String,
    pub owning_shortcut: String,
}

/// A conflict between two stored shortcuts, reported as a warning. Upgrades
/// keep such shortcuts registered exactly as before (decision 2026-10-12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct StoredShortcutConflict {
    pub action_id: String,
    pub shortcut: String,
    pub conflict: ShortcutConflict,
}

/// Normalizes `raw` with the parser of `implementation`.
///
/// # Errors
/// Returns the parser message when `raw` is not a valid shortcut for the
/// implementation.
pub fn normalize_shortcut(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Result<NormalizedCombination, String> {
    match implementation {
        KeyboardImplementation::Tauri => normalize_tauri(raw),
        KeyboardImplementation::HandyKeys => normalize_handy_keys(raw),
    }
}

fn normalize_tauri(raw: &str) -> Result<NormalizedCombination, String> {
    use tauri_plugin_global_shortcut::{Modifiers, Shortcut};
    let shortcut = raw
        .parse::<Shortcut>()
        .map_err(|e| format!("Failed to parse shortcut '{}': {}", raw, e))?;
    let families = [
        (Modifiers::CONTROL, FAMILY_CTRL),
        (Modifiers::ALT, FAMILY_ALT),
        (Modifiers::SHIFT, FAMILY_SHIFT),
        (Modifiers::SUPER | Modifiers::META, FAMILY_META),
        (Modifiers::FN, FAMILY_FN),
    ];
    let modifiers = families
        .iter()
        .filter(|(flags, _)| shortcut.mods.intersects(*flags))
        .fold(0, |acc, (_, family)| acc | family);
    Ok(NormalizedCombination {
        modifiers,
        key: Some(format!("{:?}", shortcut.key)),
    })
}

fn normalize_handy_keys(raw: &str) -> Result<NormalizedCombination, String> {
    use handy_keys::{Hotkey, Modifiers};
    let hotkey = raw
        .parse::<Hotkey>()
        .map_err(|e| format!("Invalid shortcut for HandyKeys: {}", e))?;
    // Lossy on purpose: left/right variants collapse to one family for
    // comparison only; registration keeps the side-specific raw string.
    let families = [
        (Modifiers::CTRL, FAMILY_CTRL),
        (Modifiers::OPT, FAMILY_ALT),
        (Modifiers::SHIFT, FAMILY_SHIFT),
        (Modifiers::CMD, FAMILY_META),
        (Modifiers::FN, FAMILY_FN),
    ];
    let modifiers = families
        .iter()
        .filter(|(flags, _)| hotkey.modifiers.intersects(*flags))
        .fold(0, |acc, (_, family)| acc | family);
    Ok(NormalizedCombination {
        modifiers,
        key: hotkey.key.map(|k| k.to_string().to_lowercase()),
    })
}

/// Lists every shortcut of every action, sorted by action then position,
/// so results do not depend on map iteration order.
pub fn inventory_of(bindings: &HashMap<String, ShortcutBinding>) -> Vec<InventoryEntry> {
    let mut actions: Vec<(&String, &ShortcutBinding)> = bindings.iter().collect();
    actions.sort_by_key(|(action_id, _)| *action_id);
    actions
        .into_iter()
        .flat_map(|(action_id, binding)| {
            binding
                .shortcuts()
                .into_iter()
                .map(move |shortcut| InventoryEntry {
                    action_id: action_id.clone(),
                    shortcut,
                })
        })
        .collect()
}

/// Classifies how two normalized combinations collide, if they do.
fn conflict_between(a: &NormalizedCombination, b: &NormalizedCombination) -> Option<ConflictKind> {
    if a == b {
        return Some(ConflictKind::Duplicate);
    }
    // Business rule: a modifier-only shortcut fires whenever its modifiers
    // are held, so it also fires inside any combination containing them.
    let overlaps = (a.is_modifier_only() && a.modifiers_within(b))
        || (b.is_modifier_only() && b.modifiers_within(a));
    overlaps.then_some(ConflictKind::ModifierOverlap)
}

// @spec FR-003: Shared normalized conflict checker, AC-004: Equality and modifier overlap — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-003
/// Checks `candidate` against every inventory entry except `excluded` (the
/// entry being replaced, if any).
///
/// Stored entries the active parser cannot read are skipped: they cannot be
/// registered, so they cannot fire together with the candidate.
///
/// # Errors
/// Returns the parser message when `candidate` itself is invalid.
pub fn find_conflict(
    candidate: &str,
    inventory: &[InventoryEntry],
    excluded: Option<&InventoryEntry>,
    implementation: KeyboardImplementation,
) -> Result<Option<ShortcutConflict>, String> {
    let normalized = normalize_shortcut(candidate, implementation)?;
    let conflict = inventory
        .iter()
        .filter(|entry| Some(*entry) != excluded)
        .find_map(|entry| {
            let other = normalize_shortcut(&entry.shortcut, implementation).ok()?;
            conflict_between(&normalized, &other).map(|kind| ShortcutConflict {
                kind,
                owning_action: entry.action_id.clone(),
                owning_shortcut: entry.shortcut.clone(),
            })
        });
    Ok(conflict)
}

/// Lists every stored shortcut that conflicts with another stored one, for
/// the settings warning. Each involved entry is reported once, naming the
/// first entry it collides with.
pub fn find_stored_conflicts(
    inventory: &[InventoryEntry],
    implementation: KeyboardImplementation,
) -> Vec<StoredShortcutConflict> {
    inventory
        .iter()
        .filter_map(|entry| {
            let conflict =
                find_conflict(&entry.shortcut, inventory, Some(entry), implementation).ok()??;
            Some(StoredShortcutConflict {
                action_id: entry.action_id.clone(),
                shortcut: entry.shortcut.clone(),
                conflict,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAURI: KeyboardImplementation = KeyboardImplementation::Tauri;
    const HANDY_KEYS: KeyboardImplementation = KeyboardImplementation::HandyKeys;

    fn entry(action_id: &str, shortcut: &str) -> InventoryEntry {
        InventoryEntry {
            action_id: action_id.to_string(),
            shortcut: shortcut.to_string(),
        }
    }

    fn sample_inventory() -> Vec<InventoryEntry> {
        vec![
            entry("cancel", "escape"),
            entry("transcribe", "option+space"),
            entry("transcribe_with_post_process", "option+shift+space"),
        ]
    }

    #[test]
    fn reordered_modifiers_conflict_with_the_owning_action() -> Result<(), String> {
        for implementation in [TAURI, HANDY_KEYS] {
            let conflict = find_conflict(
                "shift+option+space",
                &sample_inventory(),
                None,
                implementation,
            )?;
            let expected = ShortcutConflict {
                kind: ConflictKind::Duplicate,
                owning_action: "transcribe_with_post_process".into(),
                owning_shortcut: "option+shift+space".into(),
            };
            assert_eq!(conflict, Some(expected), "{implementation:?}");
        }
        Ok(())
    }

    #[test]
    fn modifier_aliases_normalize_to_the_same_family() -> Result<(), String> {
        let cases = [("alt+space", "option+space"), ("cmd+k", "command+k")];
        for (candidate, stored) in cases {
            for implementation in [TAURI, HANDY_KEYS] {
                let inventory = [entry("transcribe", stored)];
                let conflict = find_conflict(candidate, &inventory, None, implementation)?;
                assert!(
                    conflict.is_some(),
                    "{candidate} vs {stored} on {implementation:?}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn side_specific_modifiers_collapse_for_comparison() -> Result<(), String> {
        let inventory = [entry("transcribe", "option_left+space")];
        let conflict = find_conflict("option_right+space", &inventory, None, HANDY_KEYS)?;
        assert_eq!(
            conflict.map(|c| c.kind),
            Some(ConflictKind::Duplicate),
            "left and right option must compare equal"
        );
        Ok(())
    }

    #[test]
    fn duplicate_within_the_same_action_is_rejected() -> Result<(), String> {
        let conflict = find_conflict("option+space", &sample_inventory(), None, TAURI)?;
        assert_eq!(
            conflict.map(|c| c.owning_action),
            Some("transcribe".to_string())
        );
        Ok(())
    }

    #[test]
    fn cancel_and_disabled_actions_take_part_in_the_check() -> Result<(), String> {
        let conflict = find_conflict("escape", &sample_inventory(), None, TAURI)?;
        assert_eq!(
            conflict.map(|c| c.owning_action),
            Some("cancel".to_string())
        );
        Ok(())
    }

    #[test]
    fn distinct_combination_has_no_conflict() -> Result<(), String> {
        for implementation in [TAURI, HANDY_KEYS] {
            let conflict =
                find_conflict("ctrl+option+d", &sample_inventory(), None, implementation)?;
            assert_eq!(conflict, None, "{implementation:?}");
        }
        Ok(())
    }

    #[test]
    fn modifier_only_subset_overlaps_in_both_directions() -> Result<(), String> {
        let inventory = [entry("transcribe", "option")];
        let conflict = find_conflict("option+space", &inventory, None, HANDY_KEYS)?;
        assert_eq!(
            conflict.map(|c| c.kind),
            Some(ConflictKind::ModifierOverlap)
        );

        let inventory = [entry("transcribe", "option+shift+space")];
        let conflict = find_conflict("shift", &inventory, None, HANDY_KEYS)?;
        assert_eq!(
            conflict.map(|c| c.kind),
            Some(ConflictKind::ModifierOverlap)
        );
        Ok(())
    }

    #[test]
    fn modifier_only_without_shared_family_is_accepted() -> Result<(), String> {
        let inventory = [entry("transcribe", "option")];
        let conflict = find_conflict("ctrl+space", &inventory, None, HANDY_KEYS)?;
        assert_eq!(conflict, None);
        Ok(())
    }

    #[test]
    fn excluded_entry_is_ignored_for_a_replacement() -> Result<(), String> {
        let inventory = sample_inventory();
        let replaced = entry("transcribe", "option+space");
        let conflict = find_conflict("option+space", &inventory, Some(&replaced), TAURI)?;
        assert_eq!(conflict, None);
        Ok(())
    }

    #[test]
    fn invalid_candidate_reports_the_parser_error() {
        let result = find_conflict("option+space+k", &sample_inventory(), None, TAURI);
        assert!(result.is_err());
    }

    #[test]
    fn stored_conflicts_report_every_involved_entry() {
        let inventory = [
            entry("cancel", "escape"),
            entry("transcribe", "option+space"),
            entry("transcribe_with_post_process", "alt+space"),
        ];
        let conflicts = find_stored_conflicts(&inventory, TAURI);
        let involved: Vec<(&str, &str)> = conflicts
            .iter()
            .map(|c| (c.action_id.as_str(), c.conflict.owning_action.as_str()))
            .collect();
        assert_eq!(
            involved,
            vec![
                ("transcribe", "transcribe_with_post_process"),
                ("transcribe_with_post_process", "transcribe"),
            ]
        );
    }

    #[test]
    fn inventory_lists_alternatives_in_order() {
        let mut bindings = HashMap::new();
        let mut transcribe = crate::settings::get_default_settings().bindings["transcribe"].clone();
        transcribe.current_binding = "f13".into();
        transcribe.additional_bindings = vec!["f14".into(), "f15".into()];
        bindings.insert("transcribe".to_string(), transcribe);
        let shortcuts: Vec<String> = inventory_of(&bindings)
            .into_iter()
            .map(|e| e.shortcut)
            .collect();
        assert_eq!(shortcuts, vec!["f13", "f14", "f15"]);
    }
}
