// LiveSpec traceability: @spec(AC-002) @spec(AC-004)
//! Shared validation and changes of the remembered inputs (013 FR-004, AC-004): cap, duplicates,
//! distinctive fingerprints, replacement in place and the remove/clear changes.
use super::*;
use crate::destination::default_input::fake_tree::TestResult;
use crate::settings::get_default_settings;
use crate::settings_service::errors::SettingsErrorCode;
use crate::settings_service::transaction::SettingsEdit;

fn field(bundle: &str, name: Option<&str>) -> DefaultInput {
    DefaultInput {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: format!("App {bundle}"),
        target: DefaultTarget::Field(FieldFingerprint {
            role: "AXTextArea".to_owned(),
            subrole: None,
            name: name.map(str::to_owned),
            placeholder: None,
            dom_id: None,
        }),
    }
}
fn terminal(bundle: &str, workspace: &str) -> DefaultInput {
    DefaultInput {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: format!("App {bundle}"),
        target: DefaultTarget::Terminal(TerminalTarget {
            tool: AddressingTool::Cmux,
            workspace_id: workspace.to_owned(),
            workspace_name: format!("work {workspace}"),
        }),
    }
}
fn reason(result: Result<(), SettingsError>) -> Option<ValidationReason> {
    result.err().and_then(|error| error.reason())
}
fn apply(change: &DomainChange, settings: &mut AppSettings) -> Result<(), SettingsError> {
    match change.edit() {
        SettingsEdit::Apply(edit) => edit(settings),
        _ => Err(SettingsError::new(SettingsErrorCode::InternalError)),
    }
}

// @spec AC-004: At most 20, one per application — .specs/features/013-default-input-per-app/spec.md#ac-004
#[test]
fn the_cap_holds_and_one_more_is_full() -> TestResult {
    let mut inputs: Vec<_> = (0..MAX_DEFAULT_INPUTS)
        .map(|index| field(&format!("app.{index}"), Some("Message")))
        .collect();
    assert!(validate_default_inputs(&inputs).is_ok());
    inputs.push(terminal("app.extra", "A1"));
    assert_eq!(
        reason(validate_default_inputs(&inputs)),
        Some(ValidationReason::DefaultInputsFull)
    );
    Ok(())
}

#[test]
fn duplicates_blank_names_and_indistinct_fields_are_invalid() -> TestResult {
    let duplicate = vec![field("a", Some("x")), terminal("a", "AB")];
    let unnamed = vec![field("b", None)];
    let blank_workspace = vec![terminal("c", " ")];
    let long_name = vec![field("d", Some(&"n".repeat(MAX_TEXT_CHARS + 1)))];
    for list in [duplicate, unnamed, blank_workspace, long_name] {
        assert_eq!(
            reason(validate_default_inputs(&list)),
            Some(ValidationReason::DefaultInputInvalid)
        );
    }
    Ok(())
}

#[test]
fn remembering_again_replaces_in_place_and_a_new_app_is_appended() -> TestResult {
    let inputs = vec![field("a", Some("old")), terminal("b", "AB")];
    let replaced = remembered(&inputs, &field("a", Some("new")))
        .map_err(|error| format!("replace: {error:?}"))?;
    assert_eq!(replaced.len(), 2);
    assert_eq!(replaced[0], field("a", Some("new")));
    let appended = remembered(&inputs, &field("c", Some("x")))
        .map_err(|error| format!("append: {error:?}"))?;
    assert_eq!(appended.last(), Some(&field("c", Some("x"))));
    Ok(())
}

#[test]
fn remove_and_clear_change_the_locked_settings() -> TestResult {
    let mut settings = get_default_settings();
    let input = field("com.example.editor", Some("Message"));
    apply(&remember_change(input.clone()), &mut settings)
        .map_err(|error| format!("remember: {error:?}"))?;
    assert_eq!(settings.destination_default_inputs, vec![input]);
    let missing = apply(&remove_change("absent".to_owned()), &mut settings);
    assert_eq!(
        missing.err().map(|error| error.code()),
        Some(SettingsErrorCode::NotFound)
    );
    apply(
        &remove_change("com.example.editor".to_owned()),
        &mut settings,
    )
    .map_err(|error| format!("remove: {error:?}"))?;
    assert!(settings.destination_default_inputs.is_empty());
    apply(&remember_change(terminal("t", "AB")), &mut settings)
        .map_err(|error| format!("remember: {error:?}"))?;
    apply(&clear_change(), &mut settings).map_err(|error| format!("clear: {error:?}"))?;
    assert!(settings.destination_default_inputs.is_empty());
    Ok(())
}

#[test]
fn the_stored_shape_is_tagged_by_kind() -> TestResult {
    let value = serde_json::to_value(terminal("t", "AB"))
        .map_err(|error| format!("serialize: {error:?}"))?;
    assert_eq!(value["target"]["kind"], "terminal");
    assert_eq!(value["target"]["tool"], "cmux");
    let field_value = serde_json::to_value(field("f", Some("Message")))
        .map_err(|error| format!("serialize: {error:?}"))?;
    assert_eq!(field_value["target"]["kind"], "field");
    assert_eq!(field_value["target"]["name"], "Message");
    Ok(())
}
