//! Optional error details `reason` and `items` (006 Step 0, D3/D4): absent from every existing
//! error, closed in the schema, bounded and ordered, and the unsupported-platform refusal shape.

use handy_app_lib::settings_service::{
    errors::{ErrorItem, ErrorReason, SettingsError, SettingsErrorCode, MAX_ERROR_ITEMS},
    protocol::{contract_schema, Revision, SettingsEnvelope},
};
use serde_json::{json, Value};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn details_of(error: SettingsError) -> Result<Value, serde_json::Error> {
    let envelope = serde_json::to_value(SettingsEnvelope::<Value>::failure(error))?;
    Ok(envelope["error"]["details"].clone())
}
/// Every 008 constructor, so no existing envelope gains a field.
fn existing_errors() -> Vec<SettingsError> {
    use handy_app_lib::settings_service::errors::ValidationReason;
    let mut errors = vec![
        SettingsError::revision_conflict(Revision(4)),
        SettingsError::unknown_outcome(),
        SettingsError::live_state_refused("paste_method"),
        SettingsError::committed_effect_failed(Revision(9), "paste_method"),
        SettingsError::validation("paste_method", ValidationReason::CanonicalValue),
        SettingsError::invalid_input(ValidationReason::InvalidEncoding, Some(3)),
    ];
    let codes = [
        SettingsErrorCode::UsageError,
        SettingsErrorCode::Unavailable,
        SettingsErrorCode::InternalError,
    ];
    errors.extend(codes.map(SettingsError::new));
    errors
}

#[test]
fn existing_errors_carry_neither_reason_nor_items() -> TestResult {
    for error in existing_errors() {
        let details = details_of(error)?;
        assert!(details.get("reason").is_none(), "{details}");
        assert!(details.get("items").is_none(), "{details}");
    }
    Ok(())
}
#[test]
fn unsupported_platform_is_validation_failed_exit_two_with_reason_and_field() -> TestResult {
    let error = SettingsError::unsupported_platform(Some("paste_method"));
    assert_eq!(
        (error.code(), error.exit_code()),
        (SettingsErrorCode::ValidationFailed, 2)
    );
    assert_eq!(
        error.detail_reason(),
        Some(ErrorReason::UnsupportedPlatform)
    );
    let details = details_of(error)?;
    assert_eq!(details["reason"], "unsupported_platform");
    assert_eq!(details["field"], "paste_method");
    assert_eq!(details["issues"][0]["field"], "paste_method");
    let keyless = details_of(SettingsError::unsupported_platform(None))?;
    assert_eq!(keyless, json!({"reason": "unsupported_platform"}));
    Ok(())
}
#[test]
fn items_are_sorted_by_index_and_bounded_without_candidate_values() -> TestResult {
    let items = [5_u64, 2].map(|index| ErrorItem::new(index, ErrorReason::UnsupportedPlatform));
    let error = SettingsError::new(SettingsErrorCode::ValidationFailed).with_items(items);
    let listed = details_of(error)?["items"].clone();
    assert_eq!(
        listed,
        json!([
            {"index": 2, "code": "unsupported_platform", "message": "Not supported on this platform."},
            {"index": 5, "code": "unsupported_platform", "message": "Not supported on this platform."}
        ])
    );
    let many = (0..150_u64)
        .rev()
        .map(|index| ErrorItem::new(index, ErrorReason::UnsupportedPlatform));
    let bounded = SettingsError::new(SettingsErrorCode::ValidationFailed).with_items(many);
    assert_eq!(bounded.items().len(), MAX_ERROR_ITEMS);
    assert_eq!(bounded.items().first().map(ErrorItem::index), Some(0));
    assert_eq!(bounded.items().last().map(ErrorItem::index), Some(99));
    Ok(())
}
/// Every closed `details.reason` of this build, in wire-schema order.
const REASONS: [&str; 15] = [
    "unsupported_platform",
    "shortcut_session_busy",
    "instance_required",
    "last_shortcut",
    "alternatives_unsupported",
    "registration_refused",
    "invalid_shortcut",
    "prompt_name_empty",
    "prompt_name_too_long",
    "prompt_name_duplicate",
    "prompt_text_empty",
    "prompt_text_too_long",
    "override_incomplete",
    "unknown_provider",
    "prompt_invalid",
];
#[test]
fn schema_closes_reason_and_items() -> TestResult {
    let schema = contract_schema()?;
    let details = &schema["$defs"]["error_details"]["properties"];
    let reasons = json!(REASONS);
    assert_eq!(details["reason"], json!({ "enum": reasons }));
    assert_eq!(
        details["items"]["items"]["properties"]["code"]["enum"],
        reasons
    );
    let items = &details["items"];
    assert_eq!(items["maxItems"], json!(MAX_ERROR_ITEMS));
    assert_eq!(
        items["items"]["required"],
        json!(["index", "code", "message"])
    );
    assert_eq!(items["items"]["additionalProperties"], json!(false));
    assert_eq!(
        schema["$defs"]["error_details"]["additionalProperties"],
        json!(false)
    );
    Ok(())
}
