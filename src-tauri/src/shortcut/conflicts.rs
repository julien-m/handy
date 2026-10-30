//! Shared shortcut conflict checker.
//!
//! One checker for every caller (settings commands today, the settings CLI
//! once 008 lands, prompt shortcuts from 005): a candidate combination is
//! normalized with the active keyboard implementation's own parser, then
//! compared with every configured shortcut of every action. Equal
//! combinations (sides ignored) and modifier-only overlaps (on a shared
//! side) are conflicts.

use serde::Serialize;
use specta::Type;
use std::collections::HashMap;

use crate::settings::{KeyboardImplementation, ShortcutBinding};

/// Modifier families compared regardless of alias
/// (`option`/`alt`, `cmd`/`command`/`super`/`meta`); sides are kept apart
/// in [`NormalizedCombination`] for the modifier-only overlap rule.
const FAMILY_CTRL: u8 = 1;
const FAMILY_ALT: u8 = 1 << 1;
const FAMILY_SHIFT: u8 = 1 << 2;
const FAMILY_META: u8 = 1 << 3;
const FAMILY_FN: u8 = 1 << 4;

/// Sides of one modifier family a shortcut actually registers.
const SIDE_LEFT: u8 = 1;
const SIDE_RIGHT: u8 = 1 << 1;
/// A modifier without side (`cmd`, every Tauri modifier) fires on either.
const SIDE_BOTH: u8 = SIDE_LEFT | SIDE_RIGHT;

/// Families in bit order, used to index [`NormalizedCombination::sides`].
const FAMILIES: [u8; 5] = [
    FAMILY_CTRL,
    FAMILY_ALT,
    FAMILY_SHIFT,
    FAMILY_META,
    FAMILY_FN,
];

/// Comparable form of a shortcut: its modifier families, the sides each
/// family registers, and its main key.
///
/// Equality (`==`, the `Duplicate` rule) ignores sides on purpose: left and
/// right variants of a combination count as the same shortcut. Only the
/// modifier-only overlap rule reads the sides.
#[derive(Debug, Clone, Eq)]
pub struct NormalizedCombination {
    modifiers: u8,
    /// Side mask per family, in [`FAMILIES`] order; 0 when the family is
    /// absent.
    sides: [u8; FAMILIES.len()],
    key: Option<String>,
}

impl PartialEq for NormalizedCombination {
    fn eq(&self, other: &Self) -> bool {
        self.modifiers == other.modifiers && self.key == other.key
    }
}

impl NormalizedCombination {
    fn is_modifier_only(&self) -> bool {
        self.key.is_none()
    }

    /// Whether `other` holds every modifier family of `self` on at least one
    /// side `self` registers, so `self` fires whenever `other` is pressed.
    fn modifiers_within(&self, other: &NormalizedCombination) -> bool {
        self.modifiers & other.modifiers == self.modifiers
            && self
                .sides
                .iter()
                .zip(other.sides.iter())
                .all(|(mine, theirs)| *mine == 0 || mine & theirs != 0)
    }
}

/// Builds the family set and side masks from `(family, left, right)` flags.
fn families_and_sides(held: impl Iterator<Item = (u8, bool, bool)>) -> (u8, [u8; FAMILIES.len()]) {
    let mut modifiers = 0;
    let mut sides = [0; FAMILIES.len()];
    for (family, left, right) in held {
        let mask = match (left, right) {
            (true, true) => SIDE_BOTH,
            (true, false) => SIDE_LEFT,
            (false, true) => SIDE_RIGHT,
            (false, false) => continue,
        };
        modifiers |= family;
        if let Some(index) = FAMILIES.iter().position(|f| *f == family) {
            sides[index] |= mask;
        }
    }
    (modifiers, sides)
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
    // The Tauri parser exposes no side: every held family fires on both.
    let (modifiers, sides) = families_and_sides(families.iter().map(|(flags, family)| {
        let held = shortcut.mods.intersects(*flags);
        (*family, held, held)
    }));
    Ok(NormalizedCombination {
        modifiers,
        sides,
        key: Some(format!("{:?}", shortcut.key)),
    })
}

fn normalize_handy_keys(raw: &str) -> Result<NormalizedCombination, String> {
    use handy_keys::{Hotkey, Modifiers};
    let hotkey = raw
        .parse::<Hotkey>()
        .map_err(|e| format!("Invalid shortcut for HandyKeys: {}", e))?;
    // Families collapse left/right for equality (the `Duplicate` rule), but
    // the side masks keep the sides HandyKeys actually registers: a bare
    // `cmd` sets both sides, `command_right` only the right one, so a
    // right-only modifier shortcut does not overlap `command_left+j`
    // (native session 2026-10-28, finding N9). `Fn` has no side.
    let families = [
        (Modifiers::CTRL_LEFT, Modifiers::CTRL_RIGHT, FAMILY_CTRL),
        (Modifiers::OPT_LEFT, Modifiers::OPT_RIGHT, FAMILY_ALT),
        (Modifiers::SHIFT_LEFT, Modifiers::SHIFT_RIGHT, FAMILY_SHIFT),
        (Modifiers::CMD_LEFT, Modifiers::CMD_RIGHT, FAMILY_META),
        (Modifiers::FN, Modifiers::FN, FAMILY_FN),
    ];
    let (modifiers, sides) = families_and_sides(families.iter().map(|(left, right, family)| {
        (
            *family,
            hotkey.modifiers.intersects(*left),
            hotkey.modifiers.intersects(*right),
        )
    }));
    Ok(NormalizedCombination {
        modifiers,
        sides,
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
    // are held on a side it registers, so it also fires inside any
    // combination holding them on one of those sides (`command_right`
    // overlaps `cmd+j` but not `command_left+j`).
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
#[cfg(test)]
pub fn find_conflict(
    candidate: &str,
    inventory: &[InventoryEntry],
    excluded: Option<&InventoryEntry>,
    implementation: KeyboardImplementation,
) -> Result<Option<ShortcutConflict>, String> {
    find_conflict_of("", candidate, inventory, excluded, implementation)
}

/// [`find_conflict`] for a candidate of `candidate_action`, whose syntax and collision rule may
/// differ (a key read during a dictation, 014).
///
/// # Errors
/// Returns the parser message when `candidate` itself is invalid.
pub fn find_conflict_of(
    candidate_action: &str,
    candidate: &str,
    inventory: &[InventoryEntry],
    excluded: Option<&InventoryEntry>,
    implementation: KeyboardImplementation,
) -> Result<Option<ShortcutConflict>, String> {
    let normalized = normalize_shortcut(
        candidate,
        lone_press::parser_of(candidate_action, implementation),
    )?;
    let conflict = inventory
        .iter()
        .filter(|entry| Some(*entry) != excluded)
        .find_map(|entry| {
            let parser = lone_press::parser_of(&entry.action_id, implementation);
            let other = normalize_shortcut(&entry.shortcut, parser).ok()?;
            let mine = (candidate_action, &normalized);
            lone_press::conflict_of(mine, (&entry.action_id, &other)).map(|kind| ShortcutConflict {
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
            let conflict = find_conflict_of(
                &entry.action_id,
                &entry.shortcut,
                inventory,
                Some(entry),
                implementation,
            )
            .ok()??;
            Some(StoredShortcutConflict {
                action_id: entry.action_id.clone(),
                shortcut: entry.shortcut.clone(),
                conflict,
            })
        })
        .collect()
}

#[path = "conflicts_lone_press.rs"]
mod lone_press;

#[cfg(test)]
#[path = "conflicts_tests.rs"]
mod tests;
