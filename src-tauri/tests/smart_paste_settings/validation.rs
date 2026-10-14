//! One validator for the settings UI and the CLI (AC-015): smart paste and the macOS beta switch
//! get the same decision, code and reason on both ports; the migration marker is never settable.
use super::active;
use handy_app_lib::settings_service::{
    errors::{SettingsErrorCode, ValidationReason},
    validation::{validate_cli, validate_ui, ValidationContext},
};
use serde_json::json;
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTEXT: ValidationContext<'static> = ValidationContext {
    prompt_ids: &[],
    updates_allowed: true,
};

// @spec FR-009: Shared UI/CLI validation of smart paste — .specs/features/007-smart-paste/spec.md#fr-009
#[test]
fn smart_paste_gets_the_same_decision_from_the_ui_and_the_cli() -> TestResult {
    let cli = validate_cli("paste_method", "smart_paste", &CONTEXT);
    let ui = validate_ui("paste_method", json!("smart_paste"), &CONTEXT);
    assert_eq!(cli, ui);
    match cli {
        Ok(mutation) => {
            assert!(active());
            assert_eq!(mutation.key(), "paste_method");
        }
        Err(error) => {
            assert!(!active());
            assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
            assert_eq!(error.exit_code(), 2);
            assert_eq!(error.field(), Some("paste_method"));
            assert_eq!(
                error.reason(),
                Some(ValidationReason::SmartPasteUnavailable)
            );
        }
    }
    Ok(())
}

#[test]
fn the_beta_switch_is_refused_on_both_ports_only_once_smart_paste_replaces_it() -> TestResult {
    let cli = validate_cli("reliable_paste", "true", &CONTEXT);
    let ui = validate_ui("reliable_paste", json!(true), &CONTEXT);
    assert_eq!(cli, ui);
    let reason = cli.err().and_then(|error| error.reason());
    let expected = active().then_some(ValidationReason::ReliablePasteReplaced);
    assert_eq!(reason, expected);
    assert!(validate_cli("reliable_paste", "false", &CONTEXT).is_ok());
    Ok(())
}

#[test]
fn the_migration_marker_is_read_only_everywhere() -> TestResult {
    for error in [
        validate_cli("smart_paste_migration_version", "1", &CONTEXT).err(),
        validate_ui("smart_paste_migration_version", json!(1), &CONTEXT).err(),
    ] {
        let error = error.ok_or("marker accepted")?;
        assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
        assert_eq!(error.reason(), Some(ValidationReason::UiOnly));
    }
    Ok(())
}

#[test]
fn existing_methods_stay_valid_in_both_rollout_states() -> TestResult {
    for method in [
        "ctrl_v",
        "direct",
        "none",
        "shift_insert",
        "ctrl_shift_v",
        "external_script",
    ] {
        assert_eq!(
            validate_cli("paste_method", method, &CONTEXT)?,
            validate_ui("paste_method", json!(method), &CONTEXT)?
        );
    }
    Ok(())
}
