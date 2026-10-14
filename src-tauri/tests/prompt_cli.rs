//! Process tests of `handy settings prompts` (005 AC-008, SC-002) on the real binary and an
//! isolated portable store: offline create, update (`edit`), delete (`remove`), set-default and
//! set-provider-override with `--clear`, text from stdin, typed refusals and exit codes, the
//! live-only list (offline refusal, routed answer with readiness), and an API-key sentinel that
//! never reaches stdout or stderr.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use handy_app_lib::settings_service::{
    domains::live_operation::LiveContext,
    domains::DomainChange,
    errors::SettingsError,
    ipc,
    live::LiveSettings,
    operations::SettingsBackend,
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use harness::{Fallible, Portable, Run, TestResult};
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

const SECRET: &str = "sk-SENTINEL-PROMPT-CLI-005";

fn prompts(app: &Portable, args: &[&str]) -> Fallible<Run> {
    let mut argv = vec!["settings", "prompts"];
    argv.extend_from_slice(args);
    argv.push("--json");
    app.run(&argv)
}
/// Same as `prompts`, with `stdin` piped to the portable binary.
fn prompts_with_stdin(app: &Portable, args: &[&str], stdin: &str) -> Fallible<Value> {
    let source = std::path::Path::new(env!("CARGO_BIN_EXE_handy"));
    let binary = app
        .root
        .path()
        .join("app")
        .join(source.file_name().ok_or("no name")?);
    let mut child = Command::new(binary)
        .args(["settings", "prompts"])
        .args(args)
        .arg("--json")
        .env("HANDY_SETTINGS_TEST_LEGACY_SOCKET", app.legacy_socket())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(stdin.as_bytes())?;
    let output = child.wait_with_output()?;
    Ok(serde_json::from_slice(&output.stdout)?)
}
/// Store with a configured remote key, so any leak would show the sentinel.
fn store_with_key(app: &Portable) -> TestResult {
    let store = json!({"settings_revision": 1, "settings": {
        "post_process_api_keys": {"openai": SECRET},
        "post_process_models": {"openai": "gpt-4o-mini"}}});
    std::fs::write(app.store()?, serde_json::to_vec(&store)?)?;
    Ok(())
}
fn assert_no_secret(run: &Run) {
    assert!(!run.stdout.contains(SECRET) && !run.stderr.contains(SECRET));
}

/// Create "Formal email" and return its id.
fn create_formal(app: &Portable) -> Fallible<String> {
    let args = [
        "create",
        "--name",
        "Formal email",
        "--text",
        "Formal: ${output}",
    ];
    let created = prompts(app, &args)?;
    assert_no_secret(&created);
    let envelope = created.envelope()?;
    assert_eq!((created.code, &envelope["revision"]), (0, &json!(2)));
    Ok(envelope["data"]["prompt"]["id"]
        .as_str()
        .ok_or("id")?
        .to_owned())
}
#[test]
fn offline_crud_with_aliases_returns_ids_and_revisions_without_any_key() -> TestResult {
    let app = Portable::new()?;
    store_with_key(&app)?;
    let id = create_formal(&app)?;
    let set = [
        "set-provider-override",
        &id,
        "--provider",
        "custom",
        "--model",
        "llama3.2",
    ];
    let runs = [
        prompts(&app, &["edit", &id, "--name", "Formal mail"])?,
        prompts(&app, &["set-default", &id])?,
        prompts(&app, &set)?,
        prompts(&app, &["set-provider-override", &id, "--clear"])?,
        prompts(&app, &["remove", &id])?,
    ];
    for run in &runs {
        assert_eq!(run.code, 0, "{}", run.stderr);
        assert_no_secret(run);
    }
    let deleted = runs[4].envelope()?;
    assert_eq!(deleted["data"]["default_prompt_id"], Value::Null);
    assert_eq!(deleted["revision"], 7);
    Ok(())
}
#[test]
fn text_dash_reads_the_prompt_from_stdin() -> TestResult {
    let app = Portable::new()?;
    let text = "Multi-line\nprompt: ${output}\n";
    let created = prompts_with_stdin(&app, &["create", "--name", "Stdin", "--text", "-"], text)?;
    assert_eq!(created["data"]["prompt"]["prompt"], text);
    Ok(())
}
#[test]
fn refusals_have_stable_codes_and_exits() -> TestResult {
    let app = Portable::new()?;
    let duplicate = prompts(&app, &["create", "--name", "NOTES", "--text", "x"])?;
    assert_eq!(duplicate.code, 2);
    assert_eq!(
        duplicate.envelope()?["error"]["details"]["reason"],
        "prompt_name_duplicate"
    );
    let missing = prompts(&app, &["delete", "nope"])?;
    assert_eq!(
        (missing.code, missing.envelope()?["error"]["code"].clone()),
        (4, json!("not_found"))
    );
    let stale = prompts(&app, &["set-default", "none", "--expected-revision", "9"])?;
    assert_eq!(stale.code, 3);
    let usage = prompts(
        &app,
        &[
            "set-provider-override",
            "starter_notes",
            "--provider",
            "custom",
        ],
    )?;
    assert_eq!(usage.code, 2);
    assert_eq!(usage.envelope()?["error"]["code"], "usage_error");
    Ok(())
}
#[test]
fn offline_list_is_unavailable_but_the_library_stays_readable_without_keys() -> TestResult {
    let app = Portable::new()?;
    store_with_key(&app)?;
    let before = app.store_bytes()?;
    let list = prompts(&app, &["list"])?;
    assert_eq!(app.store_bytes()?, before, "a refused list writes nothing");
    let error = list.envelope()?["error"].clone();
    assert_eq!(
        (list.code, &error["code"], &error["details"]),
        (5, &json!("unavailable"), &json!({}))
    );
    let read = app.run(&["settings", "get", "post_process_prompts", "--json"])?;
    assert_eq!(read.code, 0);
    assert!(read.stdout.contains("starter_notes"));
    assert_no_secret(&read);
    Ok(())
}

/// The running Handy, as seen through its settings endpoint, without a Tauri handle.
struct Instance(LiveSettings);
impl SettingsBackend for Instance {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(&self.0)
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(&self.0, mutation, expected)
    }
    fn is_live(&self) -> bool {
        true
    }
    fn live_context(&self) -> Option<&dyn LiveContext> {
        Some(self)
    }
}
impl LiveContext for Instance {
    fn app(&self) -> Option<&tauri::AppHandle> {
        None
    }
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(&self.0)
    }
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(&self.0, change, expected)
    }
}
#[test]
fn routed_list_reports_readiness_and_never_the_key() -> TestResult {
    let app = Portable::new()?;
    store_with_key(&app)?;
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    let live = LiveSettings::start_owned(SettingsRepository::new(app.store()?), &probe)?;
    let _endpoint = ipc::serve(&app.store()?, Arc::new(Instance(live)))?;
    let list = prompts(&app, &["list"])?;
    assert_eq!(list.code, 0, "{}", list.stderr);
    assert_no_secret(&list);
    let data = list.envelope()?["data"].clone();
    let notes = data["prompts"]
        .as_array()
        .and_then(|all| all.iter().find(|p| p["id"] == "starter_notes"))
        .cloned();
    let readiness = notes.ok_or("notes")?["readiness"].clone();
    assert_eq!(
        (readiness["ready"].clone(), readiness["provider_id"].clone()),
        (json!(true), json!("openai"))
    );
    let human = app.run(&["settings", "prompts", "list"])?;
    assert!(
        human.stdout.contains("starter_notes: Notes (ready) []"),
        "{}",
        human.stdout
    );
    Ok(())
}
