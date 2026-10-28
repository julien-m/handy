//! The vocabulary domain on the production framework: reads, atomic changes, dry runs, export, and
//! the live-only status (refused offline, and without the transcription manager).
use super::VocabularyDomain;
use crate::settings_service::{
    domains::{execute, live_operation::LiveContext, DomainCall, DomainChange},
    errors::{SettingsError, SettingsErrorCode},
    live::LiveSettings,
    operations::SettingsBackend,
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
    let store = directory.path().join("settings_store.json");
    LiveSettings::start_offline(SettingsRepository::new(store), Box::new(probe))
}
fn call(
    backend: &dyn SettingsBackend,
    action: &str,
    arguments: &[&str],
    input: Option<&str>,
    dry_run: bool,
) -> Result<(Revision, Value), SettingsError> {
    let arguments = arguments
        .iter()
        .map(|a| (*a).to_owned())
        .collect::<Vec<_>>();
    let call = DomainCall {
        domain: "vocabulary",
        action,
        arguments: &arguments,
        input,
        dry_run,
        expected: None,
        platform: PlatformCapabilities::current(),
    };
    execute(&call, backend, &[&VocabularyDomain]).map(|(revision, data, _)| (revision, data))
}

#[test]
fn add_import_remove_and_export_share_one_revision_sequence() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let (first, added) = call(&backend, "add", &["Claude Code"], None, false)?;
    assert_eq!(added["custom_words"], json!(["Claude Code"]));
    let input = "claude code\nOpenClaw\n";
    let (second, merged) = call(&backend, "import", &["merge"], Some(input), false)?;
    assert_eq!(merged["custom_words"], json!(["Claude Code", "OpenClaw"]));
    assert_eq!(second.0, first.0 + 1);
    let (_, removed) = call(&backend, "delete", &["CLAUDE CODE"], None, false)?;
    assert_eq!(removed["custom_words"], json!(["OpenClaw"]));
    let (_, exported) = call(&backend, "export", &["-"], None, false)?;
    assert_eq!(exported, json!({ "content": "OpenClaw\n", "count": 1 }));
    let (_, listed) = call(&backend, "list", &[], None, false)?;
    assert_eq!(
        listed["terms"],
        json!([{ "index": 0, "term": "OpenClaw", "issue": null }])
    );
    Ok(())
}

#[test]
fn invalid_dry_run_is_the_same_refusal_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let refused = call(&backend, "import", &["replace"], Some("ok\n<b>\n"), true)
        .err()
        .ok_or("invalid dry run succeeded")?;
    assert_eq!(refused.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(refused.items().first().map(|item| item.index()), Some(1));
    let (revision, preview) = call(&backend, "import", &["replace"], Some("ok\n"), true)?;
    assert_eq!(
        preview,
        json!({ "dry_run": true, "changes": { "custom_words": ["ok"] } })
    );
    assert_eq!(call(&backend, "list", &[], None, false)?.0, revision);
    Ok(())
}

#[test]
fn malformed_calls_are_usage_errors() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    for (action, arguments, input) in [
        ("export", vec!["words.txt"], None),
        ("import", vec!["append"], Some("a")),
        ("import", vec!["merge"], None),
        ("list", vec!["extra"], None),
    ] {
        let refused = call(&backend, action, &arguments, input, false).err();
        assert_eq!(
            refused.map(|e| e.code()),
            Some(SettingsErrorCode::UsageError),
            "{action}"
        );
    }
    let missing = call(&backend, "remove", &["absent"], None, false).err();
    assert_eq!(missing.map(|e| e.code()), Some(SettingsErrorCode::NotFound));
    Ok(())
}

/// A running instance without a Tauri handle, hence without a transcription manager.
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
        change: &DomainChange,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self.0, change, e)
    }
}

#[test]
fn status_is_live_only_and_needs_the_transcription_manager() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let offline = call(&backend, "status", &[], None, false)
        .err()
        .ok_or("offline status")?;
    assert_eq!(offline.code(), SettingsErrorCode::Unavailable);
    assert_eq!(serde_json::to_value(&offline)?["details"], json!({}));
    let live = call(&Instance(&backend), "status", &[], None, false).err();
    assert_eq!(live.map(|e| e.code()), Some(SettingsErrorCode::Unavailable));
    Ok(())
}
