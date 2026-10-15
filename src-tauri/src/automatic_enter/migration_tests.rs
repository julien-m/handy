//! Migration of auto-submit to Automatic Enter in both gate states, in one test binary (SC-002):
//! every legacy switch × key, the precedence of a stored value, an invalid value, and the return
//! to a closed build with an unchanged legacy Enter decision.
use super::*;
use crate::settings::{get_default_settings, AutoSubmitKey};
use serde_json::json;

const OPEN: Rollout = Rollout {
    platform_capable: true,
    wired: true,
};
const CLOSED: Rollout = Rollout {
    platform_capable: true,
    wired: false,
};
const KEYS: [(&str, AutoSubmitKey); 3] = [
    ("enter", AutoSubmitKey::Enter),
    ("ctrl_enter", AutoSubmitKey::CtrlEnter),
    ("cmd_enter", AutoSubmitKey::CmdEnter),
];

/// Typed settings of `stored`, as 008's normalization decodes them before migrating.
fn decoded(stored: &Value) -> AppSettings {
    let settings: AppSettings =
        serde_json::from_value(stored.clone()).unwrap_or_else(|_| get_default_settings());
    // A store missing the key decodes it as absent (field default), never as the fresh default.
    if stored.get(KEY).is_none() {
        assert_eq!(settings.automatic_enter, None);
    }
    settings
}

#[test]
fn every_legacy_switch_and_key_maps_to_the_documented_policy_with_the_key_kept() {
    for (switch, policy) in [
        (true, AutomaticEnterPolicy::Always),
        (false, AutomaticEnterPolicy::Never),
    ] {
        for (wire, key) in KEYS {
            let stored = json!({"auto_submit": switch, "auto_submit_key": wire});
            let mut settings = decoded(&stored);
            assert!(migrate(&mut settings, &stored, OPEN));
            assert_eq!(settings.automatic_enter, Some(policy), "{switch} {wire}");
            assert_eq!(settings.auto_submit_key, key);
            assert_eq!(
                settings.auto_submit, switch,
                "the legacy switch is never rewritten"
            );
        }
    }
}
#[test]
fn a_store_without_legacy_switch_migrates_to_never() {
    let stored = json!({"selected_model": "small"});
    let mut settings = decoded(&stored);
    assert!(migrate(&mut settings, &stored, OPEN));
    assert_eq!(settings.automatic_enter, Some(AutomaticEnterPolicy::Never));
}
#[test]
fn a_stored_policy_wins_and_a_second_pass_is_a_no_op() {
    let stored = json!({"auto_submit": true, "automatic_enter": "pin_only"});
    let mut settings = decoded(&stored);
    assert!(!migrate(&mut settings, &stored, OPEN));
    assert_eq!(
        settings.automatic_enter,
        Some(AutomaticEnterPolicy::PinOnly)
    );
    let persisted = json!({"auto_submit": false, "automatic_enter": "never"});
    let mut again = decoded(&persisted);
    assert!(!migrate(&mut again, &persisted, OPEN));
    assert_eq!(again.automatic_enter, Some(AutomaticEnterPolicy::Never));
}
#[test]
fn an_invalid_or_null_stored_policy_becomes_never_without_legacy_remapping() {
    for invalid in [json!("sometimes"), json!(3), Value::Null] {
        let stored = json!({"auto_submit": true, "automatic_enter": invalid});
        let mut settings: AppSettings = serde_json::from_value(stored.clone())
            .expect("the lenient field never fails the whole store");
        assert_eq!(settings.automatic_enter, None);
        assert!(migrate(&mut settings, &stored, OPEN));
        assert_eq!(settings.automatic_enter, Some(AutomaticEnterPolicy::Never));
    }
}
#[test]
fn a_salvaged_store_still_migrates_from_the_raw_legacy_switch() {
    // Salvage starts from the open build's defaults (`never`); the raw key decides.
    let stored = json!({"auto_submit": true});
    let mut salvaged = get_default_settings();
    salvaged.automatic_enter = Some(AutomaticEnterPolicy::Never);
    salvaged.auto_submit = false;
    assert!(migrate(&mut salvaged, &stored, OPEN));
    assert_eq!(salvaged.automatic_enter, Some(AutomaticEnterPolicy::Always));
}
#[test]
fn a_closed_gate_never_creates_or_reads_automatic_enter() {
    let stored = json!({"auto_submit": true, "auto_submit_key": "ctrl_enter"});
    let mut settings = decoded(&stored);
    assert!(!migrate(&mut settings, &stored, CLOSED));
    assert_eq!(settings.automatic_enter, None);
    let written = json!({"auto_submit": true, "automatic_enter": "never"});
    let mut reread = decoded(&written);
    assert!(
        !migrate(&mut reread, &written, CLOSED),
        "nothing to persist"
    );
    assert_eq!(reread.automatic_enter, None, "not read by the closed build");
    let serialized = serde_json::to_value(&reread).expect("serializable");
    assert!(
        serialized.get(KEY).is_none(),
        "the closed build never writes it"
    );
}
#[test]
fn open_then_closed_reread_keeps_auto_submit_key_and_the_legacy_enter_decision() {
    for (switch, wire, key) in [
        (true, "ctrl_enter", AutoSubmitKey::CtrlEnter),
        (false, "cmd_enter", AutoSubmitKey::CmdEnter),
    ] {
        let stored = json!({"auto_submit": switch, "auto_submit_key": wire});
        let before = decoded(&stored);
        let mut open = decoded(&stored);
        migrate(&mut open, &stored, OPEN);
        // What an open build persists: its typed settings merged over the raw store; the legacy
        // switch has no descriptor there, so its raw value is the one kept.
        let mut persisted = serde_json::to_value(&open).expect("serializable");
        persisted[LEGACY_SWITCH] = json!(switch);
        let mut closed = decoded(&persisted);
        migrate(&mut closed, &persisted, CLOSED);
        assert_eq!(closed.auto_submit, before.auto_submit);
        assert_eq!(closed.auto_submit_key, key);
        assert_eq!(
            crate::automatic_enter::legacy_auto_submit_for(&closed, false),
            crate::automatic_enter::legacy_auto_submit_for(&before, false)
        );
    }
}
#[test]
fn open_closed_open_keeps_the_stored_policy_authoritative() {
    let migrated = json!({"auto_submit": true, "automatic_enter": "always"});
    // A closed or older build switches auto_submit off; Automatic Enter stays authoritative.
    let mut later = migrated.clone();
    later[LEGACY_SWITCH] = json!(false);
    let mut reopened = decoded(&later);
    assert!(!migrate(&mut reopened, &later, OPEN));
    assert_eq!(reopened.automatic_enter, Some(AutomaticEnterPolicy::Always));
}
