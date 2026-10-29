// LiveSpec traceability: @spec(AC-002) @spec(AC-006)
//! Favorite actions of `handy settings destination` (012 AC-006) against a real offline store:
//! `favorite-remove` commits through the framework (and `not_found` changes nothing),
//! `favorite-add` names a running application only, the shared cap applies, and the actions keep
//! 009's usage and platform precedence.
use super::*;
use crate::destination::context::ApplicationIdentity;
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::workspace::Foreground;
use crate::settings_service::{
    domains::{destination::DestinationDomain, execute, DomainCall, DomainChange},
    errors::ValidationReason,
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    platform_capability::{PlatformCapabilities, PlatformCapability},
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
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
/// The running instance without a Tauri handle: commits go to the offline store.
struct Instance<'a>(&'a LiveSettings);
impl LiveContext for Instance<'_> {
    fn app(&self) -> Option<&tauri::AppHandle> {
        None
    }
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self.0, change, expected)
    }
}
fn manager() -> DestinationManager {
    let running = vec![
        app(44, "ai.perplexity.comet", "Comet"),
        app(55, "com.apple.Notes", "Notes"),
    ];
    let workspace = FakeWorkspace::new(Foreground::Application(running[0].clone()), running);
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    DestinationManager::new(Arc::new(workspace), Arc::new(FakeClock::new()), platform)
}
fn favorites(live: &LiveSettings) -> Result<Vec<String>, SettingsError> {
    Ok(SettingsBackend::read(live)?
        .snapshot
        .settings
        .destination_favorites
        .iter()
        .map(|favorite| crate::destination::favorites::identity_value(&favorite.identity))
        .collect())
}

// @spec AC-006: favorite-add and favorite-remove — .specs/features/012-destination-favorites/spec.md#ac-006
#[test]
fn add_names_a_running_application_and_remove_commits_through_the_framework() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let manager = manager();
    for identifier in ["com.apple.Notes", "ai.perplexity.comet"] {
        add_favorite(&manager, &Instance(&live), identifier)?;
    }
    assert_eq!(
        favorites(&live)?,
        ["com.apple.Notes", "ai.perplexity.comet"]
    );
    let removed = call(&live, "favorite-remove", &["com.apple.Notes"], true);
    assert_eq!(removed["ok"], true);
    assert_eq!(
        removed["data"][FAVORITES_KEY][0]["identity"],
        serde_json::to_value(ApplicationIdentity::BundleId("ai.perplexity.comet".into()))?
    );
    assert_eq!(favorites(&live)?, ["ai.perplexity.comet"]);
    Ok(())
}

#[test]
fn unknown_identifiers_are_not_found_and_change_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let refused = add_favorite(&manager(), &Instance(&live), "com.openai.chat").err();
    assert_eq!(refused.map(|error| error.exit_code()), Some(4));
    let missing = call(&live, "favorite-remove", &["com.openai.chat"], true);
    assert_eq!(missing["error"]["code"], "not_found");
    assert_eq!(
        missing["error"]["message"],
        "No favorite application matches."
    );
    assert!(favorites(&live)?.is_empty());
    assert_eq!(SettingsBackend::read(&live)?.snapshot.revision, Revision(0));
    Ok(())
}

#[test]
fn add_is_live_only_offline_and_linux_refuses_both_actions() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let offline = call(&live, "favorite-add", &["com.apple.Notes"], true);
    assert_eq!(offline["error"]["code"], "unavailable");
    for action in ["favorite-add", "favorite-remove"] {
        let linux = call(&live, action, &["com.apple.Notes"], false);
        assert_eq!(linux["error"]["code"], "validation_failed", "{action}");
        let usage = call(&live, action, &[], true);
        assert_eq!(usage["error"]["code"], "usage_error", "{action}");
    }
    Ok(())
}

#[test]
fn a_generic_set_is_refused_and_names_the_destination_domain() {
    use crate::settings_service::validation::{validate_cli, ValidationContext};
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    let refused = validate_cli(FAVORITES_KEY, "x", &context).err();
    assert_eq!(
        refused.and_then(|error| error.reason()),
        Some(ValidationReason::DestinationDomain)
    );
}

#[test]
fn the_shared_cap_refuses_an_eleventh_favorite_from_the_cli() -> TestResult {
    use crate::destination::favorites::{toggle_change, FavoriteApp, MAX_FAVORITES};
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    for index in 0..MAX_FAVORITES {
        let favorite = FavoriteApp {
            identity: ApplicationIdentity::BundleId(format!("app.{index}")),
            display_name: format!("App {index}"),
        };
        SettingsBackend::commit_change(&live, &toggle_change(favorite), None)?;
    }
    let refused = add_favorite(&manager(), &Instance(&live), "com.apple.Notes").err();
    assert_eq!(
        refused.and_then(|error| error.reason()),
        Some(ValidationReason::FavoritesFull)
    );
    assert_eq!(favorites(&live)?.len(), MAX_FAVORITES);
    Ok(())
}
