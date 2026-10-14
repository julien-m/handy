//! 006 settings UI command `change_return_to_dictation_app_setting`: the write it delegates to
//! (`tauri_adapter::write` → `sequence::write_value`) against a real store, and the typed
//! `SettingsCommandError` the settings store receives when the value is refused.
use super::{owned, stored, TestResult};
use super::{write_value, SpyPort};
use crate::settings_service::{
    command_error::SettingsCommandError,
    errors::{SettingsErrorCode, ValidationReason},
    platform_capability::{PlatformCapabilities, PlatformCapability},
    validation::{validate_ui, ValidationContext},
};
use serde_json::json;

const KEY: &str = "return_to_dictation_app";

fn supported() -> bool {
    PlatformCapabilities::current().supports(PlatformCapability::Destination)
}

// @spec AC-008: Settings UI write of the destination setting — .specs/features/006-destination-application/spec.md#ac-008
#[test]
fn the_ui_write_commits_once_notifies_and_a_no_op_keeps_the_revision() -> TestResult {
    if !supported() {
        return Ok(());
    }
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    let start = live.snapshot().revision.0;
    let commit = write_value(&live, &port, KEY, json!(false))?;
    assert_eq!(commit.revision.0, start + 1);
    assert_eq!(stored(&directory, KEY)?, json!(false));
    let expected = vec![
        format!("effect:{KEY}"),
        format!("notify:{}:{KEY}", start + 1),
    ];
    assert_eq!(port.events(), expected);
    let again = write_value(&live, &port, KEY, json!(false))?;
    assert_eq!(again.revision.0, start + 1);
    assert_eq!(port.notifications(), 1);
    Ok(())
}

#[test]
fn an_invalid_ui_value_is_a_typed_refusal_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    let start = live.snapshot().revision;
    let refused = write_value(&live, &port, KEY, json!("yes"))
        .err()
        .ok_or("a string was accepted")?;
    let error = SettingsCommandError::from(refused);
    assert_eq!(error.code, SettingsErrorCode::ValidationFailed);
    assert_eq!(error.field.as_deref(), Some(KEY));
    assert_eq!(live.snapshot().revision, start);
    assert_eq!(stored(&directory, KEY)?, json!(supported()));
    assert!(port.events().is_empty());
    Ok(())
}

// @spec AC-007: Unsupported platform reason for the settings UI — .specs/features/006-destination-application/spec.md#ac-007
#[test]
fn without_the_capability_the_ui_receives_unsupported_platform_for_both_values() -> TestResult {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::only(&[]),
    };
    for value in [true, false] {
        let refused = validate_ui(KEY, json!(value), &context)
            .err()
            .ok_or("accepted without the capability")?;
        let error = SettingsCommandError::from(refused);
        assert_eq!(error.reason, Some(ValidationReason::UnsupportedPlatform));
        let wire = serde_json::to_value(&error)?;
        assert_eq!(wire["code"], json!("validation_failed"));
        assert_eq!(wire["reason"], json!("unsupported_platform"));
        assert_eq!(wire["field"], json!(KEY));
        assert_eq!(wire["outcome"], json!(null));
    }
    Ok(())
}
