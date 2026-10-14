//! Shared validation of the ordered list and the legacy selector's order (pure).
use super::validate_priority;
use crate::settings_service::errors::{ErrorReason, SettingsErrorCode};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|name| (*name).to_owned()).collect()
}

// @spec AC-001: Duplicate or empty identities rejected — .specs/features/001-microphone-priority/spec.md#ac-001
#[test]
fn duplicate_and_blank_names_are_refused_per_position_without_echo() -> TestResult {
    let refused = validate_priority(names(&["USB", " ", "USB", "Mac", ""]))
        .err()
        .ok_or("accepted")?;
    assert_eq!(refused.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(refused.field(), Some("microphone_priority"));
    let items = refused
        .items()
        .iter()
        .map(|item| (item.index(), item.code()))
        .collect::<Vec<_>>();
    assert_eq!(
        items,
        [
            (1, ErrorReason::MicrophoneNameEmpty),
            (2, ErrorReason::MicrophoneNameDuplicate),
            (4, ErrorReason::MicrophoneNameEmpty),
        ]
    );
    assert!(!serde_json::to_string(&refused)?.contains("USB"));
    Ok(())
}
#[test]
fn exact_names_are_kept_byte_for_byte_including_default() -> TestResult {
    let order = names(&["default", "Default", " USB headset", "Mic\u{e9}"]);
    assert_eq!(validate_priority(order.clone())?, order);
    assert_eq!(validate_priority(Vec::new())?, Vec::<String>::new());
    Ok(())
}
