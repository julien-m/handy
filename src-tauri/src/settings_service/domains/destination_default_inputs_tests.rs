// LiveSpec traceability: @spec(AC-004) @spec(AC-009)
//! Default-input actions of `handy settings destination` (013 AC-009) against a real offline
//! store: remove and clear commit through the framework, an unknown identifier is `not_found`
//! and changes nothing, Linux and usage keep 009's precedence, a generic `set` names the domain.
use super::*;
use crate::destination::context::ApplicationIdentity;
use crate::destination::default_input::model::{
    remember_change, AddressingTool, DefaultInput, DefaultTarget, TerminalTarget,
};
use crate::settings_service::{
    domains::{destination::DestinationDomain, execute, DomainCall},
    errors::ValidationReason,
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    platform_capability::{PlatformCapabilities, PlatformCapability},
    protocol::Revision,
    store::SettingsRepository,
};
use serde_json::Value;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn service(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = SettingsRepository::new(directory.path().join("settings_store.json"));
    LiveSettings::start_offline(store, Box::new(probe))
}
fn call(backend: &dyn SettingsBackend, action: &str, arguments: &[&str], supported: bool) -> Value {
    let arguments = arguments
        .iter()
        .map(|a| (*a).to_owned())
        .collect::<Vec<_>>();
    let capabilities: &'static [PlatformCapability] = if supported {
        &[PlatformCapability::Destination]
    } else {
        &[]
    };
    let call = DomainCall {
        domain: "destination",
        action,
        arguments: &arguments,
        input: None,
        dry_run: false,
        expected: None,
        platform: PlatformCapabilities::only(capabilities),
    };
    let reply = match execute(&call, backend, &[&DestinationDomain]) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    };
    reply.envelope().clone()
}
fn terminal(bundle: &str) -> DefaultInput {
    DefaultInput {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: format!("App {bundle}"),
        target: DefaultTarget::Terminal(TerminalTarget {
            tool: AddressingTool::Cmux,
            workspace_id: "43E000A6-4ED9-4758-BAE7-373F9D4303AB".to_owned(),
            workspace_name: "roam-radar".to_owned(),
        }),
    }
}
fn remembered(live: &LiveSettings) -> Result<Vec<DefaultInput>, SettingsError> {
    Ok(SettingsBackend::read(live)?
        .snapshot
        .settings
        .destination_default_inputs)
}

// @spec AC-009: default-input-remove and default-input-clear — .specs/features/013-default-input-per-app/spec.md#ac-009
#[test]
fn remove_and_clear_commit_through_the_framework() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    for bundle in ["com.example.term", "com.example.other", "com.example.third"] {
        SettingsBackend::commit_change(&live, &remember_change(terminal(bundle)), None)?;
    }
    let removed = call(&live, "default-input-remove", &["com.example.term"], true);
    assert_eq!(removed["ok"], true);
    assert_eq!(remembered(&live)?.len(), 2);
    let cleared = call(&live, "default-input-clear", &[], true);
    assert_eq!(cleared["ok"], true);
    assert!(remembered(&live)?.is_empty());
    Ok(())
}

#[test]
fn an_unknown_identifier_is_not_found_and_changes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let missing = call(&live, "default-input-remove", &["com.example.none"], true);
    assert_eq!(missing["error"]["code"], "not_found");
    assert_eq!(missing["error"]["message"], "No remembered input matches.");
    assert_eq!(SettingsBackend::read(&live)?.snapshot.revision, Revision(0));
    Ok(())
}

#[test]
fn linux_and_usage_keep_the_destination_precedence() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    for (action, arguments) in [
        ("default-input-remove", vec!["com.example.term"]),
        ("default-input-clear", vec![]),
    ] {
        let linux = call(&live, action, &arguments, false);
        assert_eq!(linux["error"]["code"], "validation_failed", "{action}");
    }
    let usage = call(&live, "default-input-remove", &[], true);
    assert_eq!(usage["error"]["code"], "usage_error");
    let extra = call(&live, "default-input-clear", &["x"], true);
    assert_eq!(extra["error"]["code"], "usage_error");
    Ok(())
}

// @spec AC-004: One shared validation for every writer — .specs/features/013-default-input-per-app/spec.md#ac-004
#[test]
fn a_generic_set_is_refused_and_names_the_destination_domain() {
    use crate::settings_service::validation::{validate_cli, ValidationContext};
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    let refused = validate_cli(
        crate::destination::default_input::model::DEFAULT_INPUTS_KEY,
        "x",
        &context,
    )
    .err();
    assert_eq!(
        refused.and_then(|error| error.reason()),
        Some(ValidationReason::DestinationDomain)
    );
}
