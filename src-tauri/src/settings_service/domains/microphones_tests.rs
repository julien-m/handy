//! The microphones domain on the production framework, against a real store: `set-priority`
//! offline (commit, empty order, refusals with items, dry run, compare-and-set) and `list`, which
//! only a running instance with its audio manager answers.
use super::MicrophonesDomain;
use crate::settings_service::{
    domains::{execute, live_operation::LiveContext, DomainCall, DomainChange},
    errors::SettingsError,
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use serde_json::{json, Value};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn service(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = SettingsRepository::new(directory.path().join("settings_store.json"));
    LiveSettings::start_offline(store, Box::new(probe))
}
/// One `microphones` call, rendered as the CLI envelope.
fn call(
    backend: &dyn SettingsBackend,
    action: &str,
    names: &[&str],
    flags: (bool, Option<u64>),
) -> Value {
    let arguments = names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    let call = DomainCall {
        domain: "microphones",
        action,
        arguments: &arguments,
        input: None,
        dry_run: flags.0,
        expected: flags.1.map(Revision),
        platform: PlatformCapabilities::current(),
    };
    let reply = match execute(&call, backend, &[&MicrophonesDomain]) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    };
    reply.envelope().clone()
}

// @spec AC-008: set-priority commits the complete order — .specs/features/001-microphone-priority/spec.md#ac-008
#[test]
fn set_priority_commits_the_complete_order_and_zero_names_empty_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let set = call(
        &live,
        "set-priority",
        &["USB headset", "default"],
        (false, None),
    );
    assert_eq!(set["revision"], 1);
    assert_eq!(
        set["data"],
        json!({"microphone_priority": ["USB headset", "default"]})
    );
    let cleared = call(&live, "set-priority", &[], (false, Some(1)));
    assert_eq!(cleared["data"], json!({"microphone_priority": []}));
    assert_eq!(cleared["revision"], 2);
    Ok(())
}
#[test]
fn refused_orders_name_positions_and_write_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let refused = call(&live, "set-priority", &["Mac", "Mac", " "], (false, None));
    assert_eq!(refused["error"]["code"], "validation_failed");
    let codes = refused["error"]["details"]["items"]
        .as_array()
        .ok_or("items")?;
    let codes = codes
        .iter()
        .map(|item| (&item["index"], &item["code"]))
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            (&json!(1), &json!("microphone_name_duplicate")),
            (&json!(2), &json!("microphone_name_empty"))
        ]
    );
    assert!(!refused.to_string().contains("Mac"));
    let stale = call(&live, "set-priority", &["Mac"], (false, Some(9)));
    assert_eq!(stale["error"]["code"], "revision_conflict");
    assert_eq!(live.snapshot().revision, Revision(0));
    Ok(())
}
#[test]
fn a_dry_run_previews_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let preview = call(&live, "set-priority", &["USB headset"], (true, None));
    let expected = json!({"dry_run": true, "changes": {"microphone_priority": ["USB headset"]}});
    assert_eq!(preview["data"], expected);
    assert_eq!(live.snapshot().revision, Revision(0));
    Ok(())
}

/// A running instance without a Tauri handle, hence without an audio manager.
struct Instance<'a>(&'a LiveSettings);
impl SettingsBackend for Instance<'_> {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit(
        &self,
        m: SettingsMutation,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(self.0, m, e)
    }
    fn is_live(&self) -> bool {
        true
    }
    fn live_context(&self) -> Option<&dyn LiveContext> {
        Some(self)
    }
}
impl LiveContext for Instance<'_> {
    fn app(&self) -> Option<&tauri::AppHandle> {
        None
    }
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit_change(
        &self,
        c: &DomainChange,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self.0, c, e)
    }
}
// @spec AC-008: list is live-only, never fabricated offline — .specs/features/001-microphone-priority/spec.md#ac-008
#[test]
fn list_is_refused_offline_and_without_an_audio_manager() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let offline = call(&live, "list", &[], (false, None));
    assert_eq!(offline["error"]["code"], "unavailable");
    assert_eq!(offline["error"]["details"], json!({}));
    let routed = call(&Instance(&live), "list", &[], (false, None));
    assert_eq!(routed["error"]["code"], "unavailable");
    assert_eq!(
        call(&live, "list", &["extra"], (false, None))["error"]["code"],
        "usage_error"
    );
    assert_eq!(
        call(&live, "remove", &[], (false, None))["error"]["code"],
        "usage_error"
    );
    Ok(())
}
