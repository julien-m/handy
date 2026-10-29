//! Closed request and intrinsic public-wire schema regressions.

use handy_app_lib::settings_service::protocol::{OperationRequest, Revision, SettingsSnapshot};
use serde_json::json;

fn request_variant<'a>(
    schema: &'a serde_json::Value,
    operation: &str,
) -> Option<&'a serde_json::Value> {
    schema["$defs"]["operation_request"]["oneOf"]
        .as_array()?
        .iter()
        .find(|branch| branch["properties"]["operation"]["const"] == operation)
}

#[test]
fn unsupported_request_versions_and_extra_fields_are_rejected_safely(
) -> Result<(), Box<dyn std::error::Error>> {
    let base = json!({"protocol_version":1,"request_id":"fixture","store_identity":"fixture","operation":"keys"});
    for (field, value) in [
        ("protocol_version", json!(0)),
        ("protocol_version", json!(2)),
        ("CANARY_UNKNOWN", json!("CANARY_SECRET")),
        ("key", json!("CANARY_SECRET")),
        ("dry_run", json!(true)),
    ] {
        let mut candidate = base.clone();
        candidate[field] = value;
        let public_error = OperationRequest::from_value(candidate.clone())
            .err()
            .ok_or("invalid request accepted")?;
        assert_eq!(
            public_error.code(),
            handy_app_lib::settings_service::errors::SettingsErrorCode::UsageError
        );
        assert!(!serde_json::to_string(&public_error)?.contains("CANARY"));
        let error = serde_json::from_value::<OperationRequest>(candidate)
            .err()
            .ok_or("malformed request accepted")?;
        assert!(!error.to_string().contains("CANARY"));
    }
    Ok(())
}

#[test]
fn request_schema_closes_each_operation_to_its_own_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let schema = handy_app_lib::settings_service::protocol::contract_schema()?;
    for (operation, extras) in [
        ("get", vec!["key"]),
        ("keys", vec![]),
        ("set", vec!["key", "value", "expected_revision"]),
        (
            "domain",
            vec![
                "domain",
                "action",
                "arguments",
                "input",
                "dry_run",
                "expected_revision",
            ],
        ),
    ] {
        let branch = request_variant(&schema, operation).ok_or("branch")?;
        assert_eq!(branch["additionalProperties"], false);
        let properties = branch["properties"].as_object().ok_or("properties")?;
        assert_eq!(properties.len(), 4 + extras.len());
        for field in [
            "protocol_version",
            "request_id",
            "store_identity",
            "operation",
        ]
        .into_iter()
        .chain(extras)
        {
            assert!(properties.contains_key(field));
        }
    }
    Ok(())
}

#[test]
fn public_schema_requires_all_fields_and_preserves_legacy_numeric_reads(
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = handy_app_lib::settings_service::protocol::contract_schema()?;
    let public = &schema["$defs"]["public_settings"];
    // 010 adds `show_recovery_bubble_without_text_field` with the gate open.
    // 005 N23 adds `plain_dictation_prompt_id`, 012 `destination_favorites` (every build).
    // 013 adds `destination_default_inputs` (every build).
    // 016 adds `smart_paragraphs_enabled`.
    let keys = 72 + usize::from(cfg!(feature = "destination-return"));
    assert_eq!(public["required"].as_array().ok_or("required")?.len(), keys);
    let fields = &public["properties"];
    assert_eq!(
        fields["paste_method"]["enum"],
        json!(super::inventory::PASTE_METHODS)
    );
    assert_eq!(fields["hold_threshold_ms"]["minimum"], 0);
    assert_eq!(fields["hold_threshold_ms"]["maximum"], json!(u64::MAX));
    assert!(fields["audio_feedback_volume"]["minimum"].is_null());
    assert!(fields["audio_feedback_volume"]["maximum"].is_null());
    assert_eq!(fields["selected_channel"]["maximum"], 65535);
    let snapshot = SettingsSnapshot::from_stored_value(
        json!({"hold_threshold_ms":0,"audio_feedback_volume":2.0}),
        Revision(0),
    )?;
    let value = serde_json::to_value(snapshot.public_settings())?;
    assert_eq!(value["hold_threshold_ms"], 0);
    assert_eq!(value["audio_feedback_volume"], 2.0);
    Ok(())
}

#[test]
fn all_canonical_public_enums_and_required_keys_match_independent_inventory(
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = handy_app_lib::settings_service::protocol::contract_schema()?;
    let public = &schema["$defs"]["public_settings"];
    let actual = SettingsSnapshot::default_field_inventory()?;
    assert_eq!(
        public["required"],
        json!(handy_app_lib::settings_service::descriptor::descriptors()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>())
    );
    // 009: with the `destination-return` gate open, `auto_submit` stays stored but is no setting.
    let retired = cfg!(feature = "destination-return").then_some("auto_submit");
    for key in actual
        .into_iter()
        .filter(|key| Some(key.as_str()) != retired)
    {
        assert!(public["required"]
            .as_array()
            .ok_or("required")?
            .contains(&json!(key)));
    }
    for (key, values) in super::inventory::ENUM_INVENTORY {
        assert_eq!(public["properties"][*key]["enum"], json!(values));
    }
    Ok(())
}

#[test]
fn every_operation_refuses_fields_owned_by_other_variants() -> Result<(), Box<dyn std::error::Error>>
{
    for (operation, extra) in [
        ("get", "value"),
        ("keys", "key"),
        ("set", "dry_run"),
        ("domain", "value"),
    ] {
        let mut candidate = json!({"protocol_version":1,"request_id":"fixture","store_identity":"fixture","operation":operation});
        if operation == "set" {
            candidate["key"] = json!("paste_method");
            candidate["value"] = json!("ctrl_v");
        }
        if operation == "domain" {
            candidate["domain"] = json!("output");
            candidate["action"] = json!("import");
            candidate["dry_run"] = json!(false);
        }
        candidate[extra] = json!("CANARY_EXTRA_SECRET");
        let error = OperationRequest::from_value(candidate)
            .err()
            .ok_or("cross-variant field accepted")?;
        assert_eq!(
            error.code(),
            handy_app_lib::settings_service::errors::SettingsErrorCode::UsageError
        );
        assert!(!serde_json::to_string(&error)?.contains("CANARY"));
    }
    Ok(())
}
