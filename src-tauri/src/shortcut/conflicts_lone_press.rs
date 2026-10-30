//! Collision rule of the keys read during a dictation (014): one modifier pressed alone fires
//! only with a modifier-only shortcut of the same modifier on a shared side, never inside a
//! combination. Their shortcuts are always read with the HandyKeys parser.
use super::{conflict_between, ConflictKind, NormalizedCombination};
use crate::settings::KeyboardImplementation;

/// Parser of `action_id`'s shortcuts: HandyKeys for a key read during a dictation (014), whose
/// observer is HandyKeys' own listener; the active implementation otherwise.
pub(super) fn parser_of(
    action_id: &str,
    implementation: KeyboardImplementation,
) -> KeyboardImplementation {
    if crate::dictation_keys::is_dictation_key(action_id) {
        KeyboardImplementation::HandyKeys
    } else {
        implementation
    }
}

// @spec FR-006: Lone-press keys conflict only when one press fires both — .specs/features/014-in-dictation-keys/spec.md#fr-006
/// How a key read during a dictation (`tap`, one modifier pressed alone) collides with `other`:
/// only a modifier-only shortcut held on a side of the same modifier fires with it. A combination
/// holding that modifier (Option+Space for Option left) never makes a lone press.
fn tap_conflict(
    tap: &NormalizedCombination,
    other: &NormalizedCombination,
) -> Option<ConflictKind> {
    if !other.is_modifier_only() || !other.modifiers_within(tap) {
        return None;
    }
    Some(if tap == other {
        ConflictKind::Duplicate
    } else {
        ConflictKind::ModifierOverlap
    })
}

/// The collision of `a` (of action `a_action`) with `b` (of `b_action`), if any.
pub(super) fn conflict_of(
    (a_action, a): (&str, &NormalizedCombination),
    (b_action, b): (&str, &NormalizedCombination),
) -> Option<ConflictKind> {
    use crate::dictation_keys::is_dictation_key;
    match (is_dictation_key(a_action), is_dictation_key(b_action)) {
        (true, _) => tap_conflict(a, b),
        (false, true) => tap_conflict(b, a),
        (false, false) => conflict_between(a, b),
    }
}
