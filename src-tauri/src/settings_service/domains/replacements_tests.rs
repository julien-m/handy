//! The replacements domain through the production framework (`execute`) over a real offline
//! store: reports, dry runs, item-located refusals without rule text and the stable exit codes.
//! Imports are tested in `replacements_import_tests.rs`.
use super::super::{execute, DomainCall};
use super::ReplacementsDomain;
use crate::settings_service::{
    errors::{SettingsError, ValidationReason},
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    protocol::Revision,
    render::{render, View},
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::path::PathBuf;

#[path = "replacements_import_tests.rs"]
mod import;
#[path = "replacements_transfer_tests.rs"]
mod transfer;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn offline(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    LiveSettings::start_offline(SettingsRepository::new(store(directory)), Box::new(probe))
}
/// One replacements call with optional input, dry run and expected revision.
#[derive(Clone, Copy)]
struct Call<'a> {
    action: &'a str,
    arguments: &'a [&'a str],
    input: Option<&'a str>,
    dry_run: bool,
    expected: Option<u64>,
}
impl<'a> Call<'a> {
    fn new(action: &'a str, arguments: &'a [&'a str]) -> Self {
        Self {
            action,
            arguments,
            input: None,
            dry_run: false,
            expected: None,
        }
    }
    fn input(self, input: &'a str) -> Self {
        Self {
            input: Some(input),
            ..self
        }
    }
    fn dry_run(self) -> Self {
        Self {
            dry_run: true,
            ..self
        }
    }
    fn expected(self, revision: u64) -> Self {
        Self {
            expected: Some(revision),
            ..self
        }
    }
    fn run(&self, backend: &dyn SettingsBackend) -> Reply {
        let arguments: Vec<String> = self.arguments.iter().map(|a| (*a).to_owned()).collect();
        let call = DomainCall {
            domain: "replacements",
            action: self.action,
            arguments: &arguments,
            input: self.input,
            dry_run: self.dry_run,
            expected: self.expected.map(Revision),
        };
        match execute(&call, backend, &[&ReplacementsDomain]) {
            Ok((revision, data, warning)) => Reply::success(revision, data, warning),
            Err(error) => Reply::failure(error),
        }
    }
}
/// Exit code, revision (or null) and data (or error) of a reply.
fn outcome(reply: &Reply) -> (i32, Value, Value) {
    let envelope = reply.envelope();
    let body = match envelope.get("data") {
        Some(data) => data.clone(),
        None => envelope["error"].clone(),
    };
    (reply.exit_code(), envelope["revision"].clone(), body)
}
fn rules(service: &LiveSettings) -> Value {
    outcome(&Call::new("list", &[]).run(service)).2["rules"].clone()
}
fn texts(rules: &Value) -> Vec<(String, String)> {
    let text = |rule: &Value, name: &str| rule[name].as_str().unwrap_or_default().to_owned();
    let all = rules.as_array().map(Vec::as_slice).unwrap_or_default();
    all.iter()
        .map(|rule| (text(rule, "recognized"), text(rule, "replacement")))
        .collect()
}
fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    let owned = |(left, right): &(&str, &str)| ((*left).to_owned(), (*right).to_owned());
    expected.iter().map(owned).collect()
}

#[test]
fn add_commits_one_revision_and_reports_the_trimmed_rule() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let (code, revision, data) =
        outcome(&Call::new("add", &[" cloud code ", "Claude Code"]).run(&service));
    assert_eq!((code, revision), (0, json!(1)));
    let id = data["rule"]["id"].as_str().unwrap_or_default();
    let rule = json!({"id": id, "recognized": "cloud code", "replacement": "Claude Code"});
    assert_eq!(data, json!({"rule": rule, "position": 1, "count": 1}));
    Ok(())
}

#[test]
fn edit_and_delete_aliases_keep_the_id_then_remove_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let added = outcome(&Call::new("add", &["cloud code", "Claude Code"]).run(&service)).2;
    let id = added["rule"]["id"].as_str().unwrap_or_default();
    let (_, revision, data) =
        outcome(&Call::new("edit", &[id, "cloud code", "Claude Code CLI"]).run(&service));
    assert_eq!((revision, &data["rule"]["id"]), (json!(2), &json!(id)));
    assert_eq!(data["rule"]["replacement"], "Claude Code CLI");
    let (_, revision, data) = outcome(&Call::new("delete", &[id]).run(&service));
    assert_eq!(
        (revision, data),
        (json!(3), json!({"removed": id, "count": 0}))
    );
    assert_eq!(rules(&service), json!([]));
    Ok(())
}

#[test]
fn dry_runs_preview_through_the_commit_validation_and_write_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let (code, revision, data) = outcome(
        &Call::new("add", &["get hub", "GitHub"])
            .dry_run()
            .run(&service),
    );
    assert_eq!(
        (code, revision, &data["dry_run"]),
        (0, json!(0), &json!(true))
    );
    assert_eq!(data["changes"]["count"], 1);
    let refused = Call::new("add", &["", "GitHub"]).dry_run().run(&service);
    assert_eq!(refused.exit_code(), 2);
    assert!(
        !store(&directory).exists(),
        "a dry run never creates the store"
    );
    Ok(())
}

#[test]
fn a_duplicate_key_is_located_at_both_positions() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["Cloud Code", "Claude Code"]).run(&service);
    let (code, _, error) = outcome(&Call::new("add", &["CLOUD  code", "x"]).run(&service));
    assert_eq!((code, &error["code"]), (2, &json!("validation_failed")));
    assert_eq!(error["details"]["field"], "word_replacements");
    let items = json!([{"position": 1, "field": "recognized", "kind": "duplicate_key"},
                       {"position": 2, "field": "recognized", "kind": "duplicate_key"}]);
    assert_eq!(error["details"]["items"], items);
    Ok(())
}

#[test]
fn refusals_never_echo_rule_text() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let reply = Call::new("add", &["secret;word", "private replacement"]).run(&service);
    let text = reply.envelope().to_string();
    assert!(
        !text.contains("secret") && !text.contains("private"),
        "{text}"
    );
    let items = json!([{"position": 1, "field": "recognized", "kind": "forbidden_character"}]);
    assert_eq!(outcome(&reply).2["details"]["items"], items);
    Ok(())
}

#[test]
fn unknown_rule_ids_are_not_found_and_bad_usage_is_a_usage_error() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    assert_eq!(
        Call::new("update", &["nope", "a", "b"])
            .run(&service)
            .exit_code(),
        4
    );
    assert_eq!(Call::new("remove", &["nope"]).run(&service).exit_code(), 4);
    let usage = [
        Call::new("add", &["only one"]),
        Call::new("import", &["merge"]),
        Call::new("import", &["upsert"]).input("a;b\n"),
        Call::new("list", &["extra"]),
    ];
    for call in usage {
        assert_eq!(outcome(&call.run(&service)).2["code"], "usage_error");
    }
    assert!(!store(&directory).exists());
    Ok(())
}

#[test]
fn a_stale_expected_revision_is_a_conflict_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["a", "b"]).run(&service);
    let stale = [
        Call::new("add", &["c", "d"]).expected(0),
        Call::new("import", &["merge"]).input("e;f\n").expected(0),
        Call::new("import", &["merge"])
            .input("e;f\n")
            .expected(0)
            .dry_run(),
    ];
    for call in stale {
        let (code, _, error) = outcome(&call.run(&service));
        assert_eq!(
            (code, &error["details"]["current_revision"]),
            (3, &json!(1))
        );
    }
    assert_eq!(service.snapshot().revision, Revision(1));
    Ok(())
}

#[test]
fn export_prints_the_exchange_content_and_list_one_tab_line_per_rule() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["cloud code", "Claude Code; CLI"]).run(&service);
    let export = Call::new("export", &[]).run(&service);
    let content = "recognized;replacement\ncloud code;Claude Code; CLI\n";
    assert_eq!(outcome(&export).2, json!({"content": content, "count": 1}));
    assert_eq!(render(View::Export, &export, false).stdout, content);
    let list = Call::new("list", &[]).run(&service);
    let id = rules(&service)[0]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let line = format!("{id}\tcloud code\tClaude Code; CLI\n");
    assert_eq!(render(View::List, &list, false).stdout, line);
    Ok(())
}

#[test]
fn generic_set_names_the_replacements_domain() {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
    };
    let refused = validate_cli("word_replacements", "[]", &context).err();
    let reason = refused.and_then(|error| error.reason());
    assert_eq!(reason, Some(ValidationReason::ReplacementsDomain));
}
