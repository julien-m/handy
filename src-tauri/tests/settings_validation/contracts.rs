//! Public projection and envelope assertions with synthetic secret canaries only.

use handy_app_lib::settings_service::protocol::{Revision, SettingsEnvelope, SettingsSnapshot};
use serde_json::json;

#[test]
fn explicit_projection_reports_secret_presence_without_any_raw_canary(
) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = SettingsSnapshot::from_stored_value(
        json!({"post_process_api_keys":{"custom":"CANARY_API_SECRET","empty":""},"post_process_providers":[{"id":"custom","label":"Custom","base_url":"https://example.invalid","unknown_secret":"CANARY_NESTED_SECRET"}],"unknown_secret":"CANARY_UNKNOWN_SECRET"}),
        Revision(17),
    )?;
    let public = serde_json::to_value(snapshot.public_settings())?;
    assert_eq!(
        public["post_process_api_keys"]["custom"],
        json!({"present":true})
    );
    assert_eq!(
        public["post_process_api_keys"]["empty"],
        json!({"present":false})
    );
    let rendered = serde_json::to_string(&public)?;
    assert!(!rendered.contains("CANARY_"));
    assert_eq!(public.as_object().ok_or("public object")?.len(), 63);
    let envelope = serde_json::to_value(SettingsEnvelope::success(
        snapshot.revision,
        snapshot.public_settings(),
    ))?;
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["revision"], 17);
    Ok(())
}

#[test]
fn all_documented_error_codes_have_fixed_exits_and_object_details(
) -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::errors::{SettingsError, SettingsErrorCode::*};
    for (code, label, exit) in [
        (UsageError, "usage_error", 2),
        (ValidationFailed, "validation_failed", 2),
        (ShortcutConflict, "shortcut_conflict", 2),
        (LimitExceeded, "limit_exceeded", 2),
        (RevisionConflict, "revision_conflict", 3),
        (NotFound, "not_found", 4),
        (Unavailable, "unavailable", 5),
        (PersistenceFailed, "persistence_failed", 1),
        (InternalError, "internal_error", 1),
    ] {
        let error = SettingsError::new(code);
        assert_eq!(error.exit_code(), exit);
        let value = serde_json::to_value(SettingsEnvelope::<serde_json::Value>::failure(error))?;
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], label);
        assert_eq!(value["error"]["details"], json!({}));
        assert_eq!(value.as_object().ok_or("envelope object")?.len(), 2);
    }
    let conflict = serde_json::to_value(SettingsError::revision_conflict(Revision(21)))?;
    assert_eq!(conflict["details"]["current_revision"], 21);
    let timeout = serde_json::to_value(SettingsError::unknown_outcome())?;
    assert_eq!(timeout["details"]["outcome"], "unknown");
    Ok(())
}

#[test]
fn request_roundtrip_keeps_typed_operation_without_debug_or_secret_diagnostics(
) -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::protocol::{OperationRequest, SettingsOperation};
    let request = OperationRequest {
        protocol_version: Default::default(),
        request_id: "synthetic-request".into(),
        store_identity: "synthetic-store".into(),
        operation: SettingsOperation::Set {
            key: "paste_method".into(),
            value: "ctrl_v".into(),
            expected_revision: Some(Revision(3)),
        },
    };
    let json = serde_json::to_value(request)?;
    assert_eq!(json["protocol_version"], 1);
    assert_eq!(json["operation"], "set");
    assert_eq!(json["expected_revision"], 3);
    let decoded: OperationRequest = serde_json::from_value(json)?;
    assert!(matches!(
        decoded.operation,
        SettingsOperation::Set {
            expected_revision: Some(Revision(3)),
            ..
        }
    ));
    let invalid =
        SettingsSnapshot::from_stored_value(json!({"bindings":"CANARY_PARSE_SECRET"}), Revision(0))
            .err()
            .ok_or("invalid stored shape")?;
    assert!(!serde_json::to_string(&invalid)?.contains("CANARY_PARSE_SECRET"));
    Ok(())
}

#[test]
fn generated_schema_matches_wire_shapes_and_descriptor_inventory(
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = handy_app_lib::settings_service::protocol::contract_schema()?;
    assert_eq!(
        schema["$defs"]["public_settings"]["properties"]
            .as_object()
            .ok_or("schema properties")?
            .len(),
        63
    );
    assert_eq!(
        schema["$defs"]["success"]["properties"]["ok"]["const"],
        true
    );
    assert_eq!(
        schema["$defs"]["failure"]["properties"]["ok"]["const"],
        false
    );
    assert_eq!(
        schema["$defs"]["error"]["properties"]["code"]["enum"]
            .as_array()
            .ok_or("error enum")?
            .len(),
        9
    );
    assert_eq!(schema["$defs"]["provider"]["additionalProperties"], false);
    assert_eq!(
        schema["$defs"]["public_settings"]["properties"]["post_process_api_keys"]
            ["additionalProperties"]["required"],
        json!(["present"])
    );
    println!(
        "SCHEMA_BEGIN\n{}\nSCHEMA_END",
        serde_json::to_string_pretty(&schema)?
    );
    Ok(())
}

#[test]
fn checked_schema_matches_the_actual_dto_generator() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../.specs/features/008-settings-cli/contracts/settings-cli.schema.json");
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(
        checked,
        handy_app_lib::settings_service::protocol::contract_schema()?
    );
    Ok(())
}

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
fn operation_schema_requires_the_same_variant_fields_as_serde(
) -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::protocol::{contract_schema, OperationRequest};
    let schema = contract_schema()?;
    for (operation, required) in [
        ("set", vec!["key", "value"]),
        ("domain", vec!["domain", "action", "dry_run"]),
    ] {
        for field in required {
            let mut candidate = json!({"protocol_version":1,"request_id":"fixture","store_identity":"fixture","operation":operation,"key":"paste_method","value":"ctrl_v","domain":"output","action":"import","dry_run":false});
            candidate
                .as_object_mut()
                .ok_or("request object")?
                .remove(field);
            assert!(
                serde_json::from_value::<OperationRequest>(candidate).is_err(),
                "{operation}:{field}"
            );
            let branch =
                request_variant(&schema, operation).ok_or("missing discriminated variant")?;
            assert!(
                branch["required"]
                    .as_array()
                    .ok_or("variant required")?
                    .contains(&json!(field)),
                "{operation}:{field}"
            );
        }
    }
    let set_null = json!({"protocol_version":1,"request_id":"fixture","store_identity":"fixture","operation":"set","key":null,"value":"ctrl_v"});
    assert!(serde_json::from_value::<OperationRequest>(set_null).is_err());
    assert_eq!(
        request_variant(&schema, "set").ok_or("set branch")?["properties"]["key"]["type"],
        "string"
    );
    Ok(())
}

#[test]
fn valid_get_keys_set_domain_shapes_roundtrip_with_optional_nulls(
) -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::protocol::OperationRequest;
    for candidate in [
        json!({"operation":"get"}),
        json!({"operation":"get","key":null}),
        json!({"operation":"keys"}),
        json!({"operation":"set","key":"paste_method","value":"ctrl_v"}),
        json!({"operation":"domain","domain":"output","action":"set","arguments":["paste_method","direct"],"dry_run":true,"input":null}),
    ] {
        let mut object = candidate.as_object().ok_or("variant object")?.clone();
        object.extend([
            ("protocol_version".into(), json!(1)),
            ("request_id".into(), json!("fixture")),
            ("store_identity".into(), json!("fixture")),
        ]);
        assert!(serde_json::from_value::<OperationRequest>(object.into()).is_ok());
    }
    Ok(())
}
