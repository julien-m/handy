//! Smart paste setting policy in both rollout states, on every platform.
use super::{migrate, refusal, Rollout, MIGRATION_VERSION, PASTE_METHOD_VALUES};
use crate::settings::{get_default_settings, AppSettings, PasteMethod};
use crate::settings_service::errors::ValidationReason;
use serde_json::{json, Value};

const ACTIVE: Rollout = Rollout {
    macos: true,
    dispatch_wired: true,
};
const MACOS_BEFORE_WIRING: Rollout = Rollout {
    macos: true,
    dispatch_wired: false,
};
const OTHER_PLATFORM: Rollout = Rollout {
    macos: false,
    dispatch_wired: true,
};
const ALL_METHODS: [PasteMethod; 7] = [
    PasteMethod::CtrlV,
    PasteMethod::Direct,
    PasteMethod::None,
    PasteMethod::ShiftInsert,
    PasteMethod::CtrlShiftV,
    PasteMethod::ExternalScript,
    PasteMethod::SmartPaste,
];

fn upgraded(method: PasteMethod, reliable_paste: bool) -> AppSettings {
    let mut settings = get_default_settings();
    settings.paste_method = method;
    settings.reliable_paste = reliable_paste;
    settings.smart_paste_migration_version = 0;
    settings
}

#[test]
fn smart_paste_is_active_only_on_macos_once_dictation_dispatches_it() {
    assert!(ACTIVE.is_active());
    assert!(!MACOS_BEFORE_WIRING.is_active());
    assert!(!OTHER_PLATFORM.is_active());
    assert_eq!(
        Rollout::CURRENT.is_active(),
        cfg!(target_os = "macos") && cfg!(feature = "smart-paste-dispatch")
    );
}

#[test]
fn new_installations_default_to_smart_paste_only_when_active() {
    assert_eq!(
        ACTIVE.default_paste_method(PasteMethod::CtrlV),
        PasteMethod::SmartPaste
    );
    for rollout in [MACOS_BEFORE_WIRING, OTHER_PLATFORM] {
        assert_eq!(
            rollout.default_paste_method(PasteMethod::CtrlV),
            PasteMethod::CtrlV
        );
        assert_eq!(
            rollout.default_paste_method(PasteMethod::Direct),
            PasteMethod::Direct
        );
        assert_eq!(rollout.fresh_install_marker(), 0);
    }
    assert_eq!(ACTIVE.fresh_install_marker(), MIGRATION_VERSION);
}

#[test]
fn inactive_rollout_never_migrates_nor_drops_the_beta_switch() {
    for rollout in [MACOS_BEFORE_WIRING, OTHER_PLATFORM] {
        for method in ALL_METHODS {
            let mut settings = upgraded(method, true);
            assert!(!migrate(&mut settings, &json!({}), rollout));
            assert_eq!(settings.paste_method, method);
            assert!(settings.reliable_paste);
            assert_eq!(settings.smart_paste_migration_version, 0);
        }
    }
}

#[test]
fn cmd_v_moves_to_smart_paste_whether_or_not_the_beta_switch_was_on() {
    for reliable_paste in [true, false] {
        let mut settings = upgraded(PasteMethod::CtrlV, reliable_paste);
        let stored = json!({"paste_method": "ctrl_v", "reliable_paste": reliable_paste});
        assert!(migrate(&mut settings, &stored, ACTIVE));
        assert_eq!(settings.paste_method, PasteMethod::SmartPaste);
        assert!(!settings.reliable_paste);
        assert_eq!(settings.smart_paste_migration_version, MIGRATION_VERSION);
    }
}

#[test]
fn other_methods_are_kept_and_still_marked_with_the_beta_switch_dropped() {
    for method in ALL_METHODS
        .into_iter()
        .filter(|method| *method != PasteMethod::CtrlV)
    {
        let mut settings = upgraded(method, true);
        assert!(migrate(&mut settings, &json!({}), ACTIVE));
        assert_eq!(settings.paste_method, method);
        assert!(!settings.reliable_paste);
        assert_eq!(settings.smart_paste_migration_version, MIGRATION_VERSION);
    }
}

#[test]
fn a_later_cmd_v_choice_survives_every_reload() {
    let mut settings = upgraded(PasteMethod::CtrlV, false);
    settings.smart_paste_migration_version = MIGRATION_VERSION;
    let stored = json!({"paste_method": "ctrl_v", "smart_paste_migration_version": 1});
    assert!(!migrate(&mut settings, &stored, ACTIVE));
    assert_eq!(settings.paste_method, PasteMethod::CtrlV);
}

#[test]
fn a_hand_edited_beta_switch_is_turned_off_again_after_the_migration() {
    let mut settings = upgraded(PasteMethod::CtrlV, true);
    settings.smart_paste_migration_version = MIGRATION_VERSION;
    let stored = json!({"reliable_paste": true, "smart_paste_migration_version": 1});
    assert!(migrate(&mut settings, &stored, ACTIVE));
    assert!(!settings.reliable_paste);
    assert_eq!(settings.paste_method, PasteMethod::CtrlV);
}

#[test]
fn a_missing_or_malformed_marker_means_never_migrated() {
    for marker in [Value::Null, json!("1"), json!(-1), json!(0)] {
        let mut settings = upgraded(PasteMethod::CtrlV, false);
        let stored = json!({"smart_paste_migration_version": marker});
        assert!(migrate(&mut settings, &stored, ACTIVE));
        assert_eq!(settings.paste_method, PasteMethod::SmartPaste);
    }
}

#[test]
fn smart_paste_is_refused_wherever_it_is_not_active() {
    let smart = json!("smart_paste");
    for rollout in [MACOS_BEFORE_WIRING, OTHER_PLATFORM] {
        assert_eq!(
            refusal("paste_method", &smart, rollout),
            Some(ValidationReason::SmartPasteUnavailable)
        );
    }
    assert_eq!(refusal("paste_method", &smart, ACTIVE), None);
    assert_eq!(
        refusal("paste_method", &json!("ctrl_v"), OTHER_PLATFORM),
        None
    );
    assert_eq!(refusal("clipboard_handling", &smart, OTHER_PLATFORM), None);
}

#[test]
fn the_beta_switch_cannot_be_turned_back_on_once_smart_paste_is_active() {
    assert_eq!(
        refusal("reliable_paste", &json!(true), ACTIVE),
        Some(ValidationReason::ReliablePasteReplaced)
    );
    assert_eq!(refusal("reliable_paste", &json!(false), ACTIVE), None);
    for rollout in [MACOS_BEFORE_WIRING, OTHER_PLATFORM] {
        assert_eq!(refusal("reliable_paste", &json!(true), rollout), None);
    }
}

#[test]
fn allowed_paste_methods_list_smart_paste_only_when_active() {
    assert_eq!(
        PASTE_METHOD_VALUES.contains(&"smart_paste"),
        Rollout::CURRENT.is_active()
    );
    assert_eq!(
        PASTE_METHOD_VALUES.len(),
        6 + usize::from(Rollout::CURRENT.is_active())
    );
}
