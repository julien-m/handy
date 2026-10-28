//! One-time migration of auto-submit to Automatic Enter (009 AC-004), run inside 008's
//! normalization (`settings::apply_settings_migrations`) and persisted once by its startup
//! normalization. It only adds `automatic_enter`: `auto_submit` and `auto_submit_key` are never
//! deleted or rewritten, so a build with the `destination-return` gate closed, or an older Handy,
//! keeps the user's Enter behavior.
use crate::destination::rollout::Rollout;
use crate::settings::{AppSettings, AutomaticEnterPolicy};
use serde_json::Value;

const KEY: &str = "automatic_enter";
const LEGACY_SWITCH: &str = "auto_submit";

// @spec FR-003: Migration from auto-submit, key kept — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-003
/// Migrate `settings` read from the raw `stored` payload for `rollout`. Returns `true` when the
/// typed settings changed in a way the startup normalization must persist.
///
/// Business rules (gate open): a stored valid `automatic_enter` wins and is never re-derived (an
/// `auto_submit` changed later by a closed or older build stays ignored); a store without the key
/// is migrated from the raw legacy switch (`true` → `always`, `false` or missing → `never`, the key
/// untouched); a present but invalid or null value becomes `never` with no legacy remapping, so a
/// damaged value never silently enables Enter. Gate closed: nothing is migrated or written, and a
/// value written by an open build is dropped from the in-memory settings only (it has no descriptor
/// there, so the raw stored value survives every closed-build write).
pub fn migrate(settings: &mut AppSettings, stored: &Value, rollout: Rollout) -> bool {
    if !rollout.wired {
        // Not persisted (returns false): the closed build simply does not read the value.
        settings.automatic_enter = None;
        return false;
    }
    if settings.automatic_enter.is_some() && stored.get(KEY).is_some() {
        return false;
    }
    settings.automatic_enter = Some(match stored.get(KEY) {
        None => from_legacy(stored),
        Some(_) => AutomaticEnterPolicy::Never,
    });
    true
}

/// Policy of the legacy switch, read from the raw payload (never from typed fields a salvage may
/// have reset): only a stored `true` was an active auto-submit.
fn from_legacy(stored: &Value) -> AutomaticEnterPolicy {
    match stored.get(LEGACY_SWITCH).and_then(Value::as_bool) {
        Some(true) => AutomaticEnterPolicy::Always,
        _ => AutomaticEnterPolicy::Never,
    }
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod tests;
