//! The production `prompts` domain through `domains::execute` on a real store: offline CRUD with
//! the `delete`/`edit` aliases, dry run, stale revision, typed refusals without the candidate,
//! the live-only list offline, and the routed list with shortcuts and readiness but no key.
use super::super::{execute, production, DomainCall};
use super::PromptsList;
use crate::prompts::provider::AvailabilityProbe;
use crate::settings_service::{
    domains::live_operation::{LiveCall, LiveContext, LiveOperation},
    domains::DomainChange,
    errors::SettingsError,
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
};
use serde_json::{json, Value};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const SECRET: &str = "sk-SENTINEL-PROMPTS";

fn service(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = directory.path().join("settings_store.json");
    LiveSettings::start_offline(SettingsRepository::new(store), Box::new(probe))
}
/// One `prompts` call as the CLI sends it.
fn run(backend: &dyn SettingsBackend, action: &str, arguments: &[&str], flags: Flags) -> Reply {
    let arguments: Vec<String> = arguments.iter().map(|a| (*a).to_owned()).collect();
    let call = DomainCall {
        domain: "prompts",
        action,
        arguments: &arguments,
        input: flags.input,
        dry_run: flags.dry_run,
        expected: flags.expected.map(Revision),
        platform: flags.platform,
    };
    match execute(&call, backend, production()) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    }
}
#[derive(Clone, Copy)]
struct Flags {
    input: Option<&'static str>,
    dry_run: bool,
    expected: Option<u64>,
    platform: PlatformCapabilities,
}
fn plain() -> Flags {
    Flags {
        input: None,
        dry_run: false,
        expected: None,
        platform: PlatformCapabilities::current(),
    }
}
fn data(reply: &Reply) -> Value {
    reply.envelope()["data"].clone()
}
fn error(reply: &Reply) -> (i32, Value, Value) {
    let error = &reply.envelope()["error"];
    (
        reply.exit_code(),
        error["code"].clone(),
        error["details"]["reason"].clone(),
    )
}
fn created_id(reply: &Reply) -> String {
    data(reply)["prompt"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn create_returns_the_prompt_id_and_revision_and_the_aliases_reach_update_and_remove() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let created = run(
        &backend,
        "create",
        &[" Formal email ", "Rewrite: ${output}"],
        plain(),
    );
    assert_eq!(created.envelope()["revision"], 1);
    assert_eq!(data(&created)["prompt"]["name"], "Formal email");
    let id = created_id(&created);
    let edited = run(&backend, "edit", &[&id, "text=Formal: ${output}"], plain());
    assert_eq!(data(&edited)["prompt"]["prompt"], "Formal: ${output}");
    run(&backend, "set-default", &[&id], plain());
    let removed = run(&backend, "delete", &[&id], plain());
    assert_eq!(
        data(&removed),
        json!({"removed": id, "removed_shortcuts": [], "default_prompt_id": null})
    );
    assert_eq!(removed.envelope()["revision"], 4);
    Ok(())
}
#[test]
fn text_from_stdin_is_the_request_input() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let flags = Flags {
        input: Some("From stdin: ${output}"),
        ..plain()
    };
    let created = run(&backend, "create", &["Stdin"], flags);
    assert_eq!(data(&created)["prompt"]["prompt"], "From stdin: ${output}");
    let both = run(&backend, "create", &["Twice", "text"], flags);
    assert_eq!(error(&both).1, "usage_error");
    Ok(())
}
#[test]
fn refusals_are_typed_and_never_echo_the_candidate() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let duplicate = run(&backend, "create", &["notes", "CANDIDATE-TEXT"], plain());
    let expected = (
        2,
        json!("validation_failed"),
        json!("prompt_name_duplicate"),
    );
    assert_eq!(error(&duplicate), expected);
    assert!(!duplicate.envelope().to_string().contains("CANDIDATE-TEXT"));
    let blank = run(&backend, "create", &["Blank", "  \n "], plain());
    assert_eq!(error(&blank).2, "prompt_text_empty");
    let missing = run(&backend, "remove", &["nope"], plain());
    assert_eq!(error(&missing), (4, json!("not_found"), Value::Null));
    Ok(())
}
#[test]
fn an_incomplete_or_unknown_override_is_refused() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let half = ["starter_notes", "provider=custom"];
    let refused = run(&backend, "set-provider-override", &half, plain());
    assert_eq!(error(&refused).1, "usage_error");
    let unknown = ["starter_notes", "provider=nope", "model=m"];
    let refused = run(&backend, "set-provider-override", &unknown, plain());
    assert_eq!(error(&refused).2, "unknown_provider");
    Ok(())
}
#[test]
fn apple_intelligence_override_off_apple_silicon_is_unsupported_platform() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let flags = Flags {
        platform: PlatformCapabilities::only(&[]),
        ..plain()
    };
    let arguments = [
        "starter_notes",
        "provider=apple_intelligence",
        "model=Apple Intelligence",
    ];
    let refused = run(&backend, "set-provider-override", &arguments, flags);
    assert_eq!(
        error(&refused),
        (2, json!("validation_failed"), json!("unsupported_platform"))
    );
    Ok(())
}
#[test]
fn an_override_is_set_then_cleared_atomically() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let arguments = ["starter_notes", "provider=custom", "model=llama3.2"];
    let set = run(&backend, "set-provider-override", &arguments, plain());
    let expected = json!({"provider_id": "custom", "model": "llama3.2"});
    assert_eq!(data(&set)["prompt"]["provider_override"], expected);
    let clear = ["starter_notes", "clear=true"];
    let cleared = run(&backend, "set-provider-override", &clear, plain());
    assert_eq!(data(&cleared)["prompt"]["provider_override"], Value::Null);
    Ok(())
}
#[test]
fn a_dry_run_and_a_stale_revision_write_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let dry_run = Flags {
        dry_run: true,
        ..plain()
    };
    let preview = run(&backend, "create", &["Preview", "x"], dry_run);
    assert_eq!(data(&preview)["dry_run"], true);
    let stale = Flags {
        expected: Some(1),
        ..plain()
    };
    assert_eq!(error(&run(&backend, "create", &["Stale", "x"], stale)).0, 3);
    let first = run(&backend, "create", &["Committed", "x"], plain());
    assert_eq!(
        first.envelope()["revision"],
        1,
        "nothing was committed before"
    );
    Ok(())
}
#[test]
fn the_fifty_first_prompt_is_limit_exceeded() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    for index in 0..43 {
        let name = format!("Prompt {index}");
        assert_eq!(
            run(&backend, "create", &[&name, "x"], plain()).exit_code(),
            0
        );
    }
    let refused = run(&backend, "create", &["One too many", "x"], plain());
    assert_eq!(error(&refused).0, 2);
    assert_eq!(refused.envelope()["error"]["code"], "limit_exceeded");
    assert_eq!(refused.envelope()["error"]["details"]["limit"], 50);
    Ok(())
}
#[test]
fn offline_list_is_the_live_only_refusal() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let refused = run(&backend, "list", &[], plain());
    assert_eq!(error(&refused).1, "unavailable");
    assert_eq!(refused.exit_code(), 5);
    assert_eq!(refused.envelope()["error"]["details"], json!({}));
    Ok(())
}

/// Apple Intelligence never available, for a deterministic list.
struct NoApple;
impl AvailabilityProbe for NoApple {
    fn apple_intelligence_available(&self) -> bool {
        false
    }
}
/// The running instance stood in for by an owned service, without a Tauri handle.
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
/// Commit a configured key and model so the default provider (openai) is ready.
fn configure_openai(backend: &LiveSettings) -> Result<(), SettingsError> {
    let edit = Box::new(|settings: &mut crate::settings::AppSettings| {
        let key = SECRET.to_owned();
        settings.post_process_api_keys.insert("openai".into(), key);
        let model = "gpt-4o-mini".to_owned();
        settings.post_process_models.insert("openai".into(), model);
        Ok(())
    });
    let change = DomainChange(super::super::ChangeKind::Apply(edit));
    backend.commit_change(&change, None).map(|_| ())
}
#[test]
fn the_live_list_reports_readiness_and_shortcuts_without_any_key() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    configure_openai(&backend)?;
    let call = LiveCall {
        dry_run: false,
        expected: None,
    };
    let list = PromptsList::with_probe(Box::new(NoApple));
    let answer = list.run(&Instance(&backend), &call)?;
    let super::super::live_operation::LiveAnswer::Read(listed) = answer else {
        return Err("expected a read".into());
    };
    assert!(!listed.to_string().contains(SECRET));
    let prompts = listed["prompts"].as_array().cloned().unwrap_or_default();
    let notes = prompts.iter().find(|p| p["id"] == "starter_notes").cloned();
    let notes = notes.unwrap_or_default();
    let ready = json!({"provider_id": "openai", "model": "gpt-4o-mini", "is_override": false, "ready": true, "missing": null});
    assert_eq!(
        (&notes["readiness"], &notes["shortcuts"]),
        (&ready, &json!([]))
    );
    assert_eq!(listed["default_prompt_id"], Value::Null);
    Ok(())
}
