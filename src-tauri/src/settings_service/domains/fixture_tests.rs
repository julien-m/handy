//! A test-only domain (`words`, over the vocabulary list) built on the production framework:
//! bulk import in merge or replace mode from file/stdin content, export, alias-resolved update and
//! remove, and a live-only status. It is never registered in `production()`, so it never appears
//! in the help of a build.
use super::{execute, ChangeKind, Domain, DomainCall, DomainChange, DomainPlan, DomainRequest};
use crate::settings::AppSettings;
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode, ValidationReason},
    input::{read_bounded, MAX_INPUT_BYTES},
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    protocol::Revision,
    render::{render, View},
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use serde_json::{json, Value};
use std::path::PathBuf;
type TestResult = Result<(), Box<dyn std::error::Error>>;

struct WordsDomain;
impl Domain for WordsDomain {
    fn name(&self) -> &'static str {
        "words"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["import", "export", "update", "remove", "status"]
    }
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        let usage = || SettingsError::new(SettingsErrorCode::UsageError);
        match (request.action, request.arguments) {
            ("import", [mode]) => import(mode, request.input.ok_or_else(usage)?),
            ("export", []) => Ok(DomainPlan::Read(json!({
                "content": settings.custom_words.iter().map(|w| format!("{w}\n")).collect::<String>()
            }))),
            ("update", [old, new]) => Ok(change(replace_word(old.clone(), new.clone()))),
            ("remove", [word]) => Ok(change(remove_word(word.clone()))),
            ("status", []) => Ok(DomainPlan::LiveOnly(json!({ "listening": true }))),
            _ => Err(usage()),
        }
    }
}
fn change(apply: super::BoxedEdit) -> DomainPlan {
    DomainPlan::Change {
        change: DomainChange(ChangeKind::Apply(apply)),
        keys: vec!["custom_words"],
    }
}
/// One word per line; a tab anywhere refuses the whole import (all or nothing).
fn import(mode: &str, input: &str) -> Result<DomainPlan, SettingsError> {
    let words = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match line.contains('\t') {
            true => Err(SettingsError::invalid_input(
                ValidationReason::InvalidInput,
                None,
            )),
            false => Ok(line.to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let replace = match mode {
        "merge" => false,
        "replace" => true,
        _ => return Err(SettingsError::new(SettingsErrorCode::UsageError)),
    };
    Ok(change(Box::new(move |settings: &mut AppSettings| {
        if replace {
            settings.custom_words.clear();
        }
        for word in &words {
            if !settings.custom_words.contains(word) {
                settings.custom_words.push(word.clone());
            }
        }
        Ok(())
    })))
}
fn replace_word(old: String, new: String) -> super::BoxedEdit {
    Box::new(move |settings: &mut AppSettings| {
        let slot = settings.custom_words.iter_mut().find(|word| **word == old);
        *slot.ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))? = new.clone();
        Ok(())
    })
}
fn remove_word(word: String) -> super::BoxedEdit {
    Box::new(move |settings: &mut AppSettings| {
        settings.custom_words.retain(|known| *known != word);
        Ok(())
    })
}

/// A running instance's view of the same service (answers live-only operations).
struct Live<'a>(&'a LiveSettings);
impl SettingsBackend for Live<'_> {
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
}
fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn offline(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    LiveSettings::start_offline(SettingsRepository::new(store(directory)), Box::new(probe))
}
/// Run one `words` call; the reply as the CLI would render it.
fn call(
    backend: &dyn SettingsBackend,
    action: &str,
    arguments: &[&str],
    input: Option<&str>,
    dry_run: bool,
) -> Reply {
    let arguments = arguments
        .iter()
        .map(|a| (*a).to_owned())
        .collect::<Vec<_>>();
    let call = DomainCall {
        domain: "words",
        action,
        arguments: &arguments,
        input,
        dry_run,
        expected: None,
        platform: crate::settings_service::platform_capability::PlatformCapabilities::current(),
    };
    match execute(&call, backend, &[&WordsDomain]) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    }
}
fn words(reply: &Reply) -> Value {
    reply.envelope()["data"]["custom_words"].clone()
}

#[test]
fn merge_then_replace_import_commit_atomically_once_each() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    let merged = call(
        &backend,
        "import",
        &["merge"],
        Some("handy\ncli\n\nhandy\n"),
        false,
    );
    assert_eq!(
        (merged.exit_code(), words(&merged)),
        (0, json!(["handy", "cli"]))
    );
    let replaced = call(&backend, "import", &["replace"], Some("tauri\n"), false);
    assert_eq!(words(&replaced), json!(["tauri"]));
    let revisions = [&merged, &replaced].map(|reply| reply.envelope()["revision"].as_u64());
    assert_eq!(revisions[1], revisions[0].map(|first| first + 1));
    Ok(())
}
#[test]
fn valid_dry_run_previews_with_exit_zero_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    call(&backend, "import", &["merge"], Some("handy\n"), false);
    let bytes = std::fs::read(store(&directory))?;
    let preview = call(&backend, "import", &["merge"], Some("cli\n"), true);
    assert_eq!(preview.exit_code(), 0);
    let data = &preview.envelope()["data"];
    assert_eq!(
        (&data["dry_run"], &data["changes"]["custom_words"]),
        (&json!(true), &json!(["handy", "cli"]))
    );
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    assert!(render(View::Preview, &preview, false)
        .stdout
        .starts_with("Dry run: nothing was changed"));
    Ok(())
}
#[test]
fn invalid_import_is_refused_whole_with_exit_two_dry_run_or_not() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    call(&backend, "import", &["merge"], Some("handy\n"), false);
    let bytes = std::fs::read(store(&directory))?;
    for dry_run in [true, false] {
        let refused = call(
            &backend,
            "import",
            &["merge"],
            Some("ok\nbad\tline\n"),
            dry_run,
        );
        assert_eq!(
            (refused.exit_code(), &refused.envelope()["error"]["code"]),
            (2, &json!("validation_failed"))
        );
        assert!(refused.envelope()["data"].is_null(), "no success preview");
    }
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    Ok(())
}
#[test]
fn edit_and_delete_aliases_resolve_to_update_and_remove() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    call(
        &backend,
        "import",
        &["replace"],
        Some("handy\ncli\n"),
        false,
    );
    assert_eq!(
        words(&call(&backend, "edit", &["cli", "CLI"], None, false)),
        json!(["handy", "CLI"])
    );
    assert_eq!(
        words(&call(&backend, "delete", &["handy"], None, false)),
        json!(["CLI"])
    );
    let missing = call(&backend, "update", &["absent", "x"], None, false);
    assert_eq!(missing.exit_code(), 4);
    assert_eq!(call(&backend, "rename", &[], None, false).exit_code(), 2);
    Ok(())
}
#[test]
fn live_only_operations_need_a_running_instance() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    let refused = call(&backend, "status", &[], None, false);
    assert_eq!(
        (refused.exit_code(), &refused.envelope()["error"]["code"]),
        (5, &json!("unavailable"))
    );
    let answered = call(&Live(&backend), "status", &[], None, false);
    assert_eq!(answered.envelope()["data"], json!({ "listening": true }));
    Ok(())
}
#[test]
fn export_prints_only_the_content_or_one_envelope_with_data_content() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = offline(&directory)?;
    call(
        &backend,
        "import",
        &["replace"],
        Some("handy\ncli\n"),
        false,
    );
    let exported = call(&backend, "export", &[], None, false);
    let human = render(View::Export, &exported, false);
    assert_eq!(
        (human.stdout.as_str(), human.stderr.as_str()),
        ("handy\ncli\n", "")
    );
    let json = render(View::Export, &exported, true);
    let envelope: Value = serde_json::from_str(json.stdout.trim_end())?;
    assert_eq!(envelope["data"]["content"], "handy\ncli\n");
    assert_eq!(json.stdout.lines().count(), 1);
    Ok(())
}
#[test]
fn input_is_utf8_text_within_the_frame_bound() -> TestResult {
    let refused = read_bounded(&[b'o', b'k', 0xff, b'!'][..])
        .err()
        .ok_or("non-UTF-8 accepted")?;
    let envelope = Reply::failure(refused).envelope().clone();
    assert_eq!(
        (
            &envelope["error"]["code"],
            &envelope["error"]["details"]["offset"]
        ),
        (&json!("validation_failed"), &json!(2))
    );
    let oversized = read_bounded(&vec![b'a'; MAX_INPUT_BYTES + 1][..])
        .err()
        .ok_or("oversized input accepted")?;
    assert_eq!(
        (oversized.code(), oversized.exit_code()),
        (SettingsErrorCode::LimitExceeded, 2)
    );
    assert_eq!(read_bounded(&b"caf\xc3\xa9\n"[..])?, "café\n");
    Ok(())
}
#[test]
fn file_input_is_read_by_path_and_a_missing_file_is_a_usage_error() -> TestResult {
    use crate::settings_service::input::read_source;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("words.txt");
    std::fs::write(&path, "handy\n")?;
    assert_eq!(read_source(&path.to_string_lossy())?, "handy\n");
    let missing = directory.path().join("absent.txt");
    let refused = read_source(&missing.to_string_lossy())
        .err()
        .ok_or("missing file accepted")?;
    assert_eq!(refused.code(), SettingsErrorCode::UsageError);
    Ok(())
}
#[path = "fixture_stdin_tests.rs"]
mod stdin;
