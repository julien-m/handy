//! Process tests of `handy settings vocabulary` on an isolated portable store: envelopes, exit
//! codes, indexed refusals, file and stdin imports, stdout export, and the live-only status.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use harness::{Fallible, Portable, Run, TestResult};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn json_run(app: &Portable, args: &[&str]) -> Fallible<(i32, Value)> {
    let mut argv = vec!["settings", "vocabulary"];
    argv.extend_from_slice(args);
    argv.push("--json");
    let run = app.run(&argv)?;
    Ok((run.code, run.envelope()?))
}
/// Run the portable binary with `stdin` piped in.
fn run_with_stdin(app: &Portable, args: &[&str], stdin: &str) -> Fallible<Run> {
    let name = Path::new(env!("CARGO_BIN_EXE_handy"))
        .file_name()
        .ok_or("binary has no file name")?;
    let mut child = Command::new(app.root.path().join("app").join(name))
        .args(args)
        .env("HANDY_SETTINGS_TEST_LEGACY_SOCKET", app.legacy_socket())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(stdin.as_bytes())?;
    let output = child.wait_with_output()?;
    Ok(Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
fn items(envelope: &Value) -> Vec<(u64, String)> {
    envelope["error"]["details"]["items"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|item| {
            let code = item["code"].as_str().unwrap_or_default().to_owned();
            (item["index"].as_u64().unwrap_or(u64::MAX), code)
        })
        .collect()
}

#[test]
fn help_lists_the_six_operations() -> TestResult {
    let app = Portable::new()?;
    let help = app.run(&["settings", "vocabulary", "--help"])?;
    assert_eq!(help.code, 0);
    for action in ["list", "add", "remove", "import", "export", "status"] {
        assert!(help.stdout.contains(action), "{action} missing from help");
    }
    Ok(())
}

#[test]
fn add_list_remove_with_revisions_and_both_json_placements() -> TestResult {
    let app = Portable::new()?;
    let (code, added) = json_run(&app, &["add", "Claude Code"])?;
    assert_eq!(
        (code, &added["data"]["custom_words"]),
        (0, &json!(["Claude Code"]))
    );
    let revision = added["revision"].as_u64().ok_or("revision")?;
    let listed = app
        .run(&["--json", "settings", "vocabulary", "list"])?
        .envelope()?;
    assert_eq!(listed["data"]["terms"][0]["term"], "Claude Code");
    assert_eq!(listed["revision"], json!(revision));
    let text = app.run(&["settings", "vocabulary", "list"])?;
    assert_eq!(text.stdout, "0: Claude Code\n");
    let (code, removed) = json_run(&app, &["remove", "claude  CODE"])?;
    assert_eq!((code, &removed["data"]["custom_words"]), (0, &json!([])));
    assert_eq!(removed["revision"], json!(revision + 1));
    Ok(())
}

#[test]
fn refusals_use_stable_codes_items_and_exits_without_echoing_terms() -> TestResult {
    let app = Portable::new()?;
    json_run(&app, &["add", "OpenClaw"])?;
    let (code, duplicate) = json_run(&app, &["add", "openclaw"])?;
    assert_eq!(
        (code, duplicate["error"]["code"].as_str()),
        (2, Some("validation_failed"))
    );
    assert_eq!(items(&duplicate), vec![(0, "term_duplicate".to_owned())]);
    let secret = format!("CANARY{}", "x".repeat(100));
    let long = app.run(&["settings", "vocabulary", "add", &secret, "--json"])?;
    assert_eq!(
        items(&long.envelope()?),
        vec![(0, "term_too_long".to_owned())]
    );
    assert!(!long.stdout.contains("CANARY") && !long.stderr.contains("CANARY"));
    let (code, missing) = json_run(&app, &["remove", "absent"])?;
    assert_eq!(
        (code, missing["error"]["code"].as_str()),
        (4, Some("not_found"))
    );
    let (code, stale) = json_run(&app, &["add", "New", "--expected-revision", "0"])?;
    assert_eq!(
        (code, stale["error"]["code"].as_str()),
        (3, Some("revision_conflict"))
    );
    Ok(())
}

#[test]
fn file_import_merges_atomically_and_invalid_lines_refuse_everything() -> TestResult {
    let app = Portable::new()?;
    let file = app.root.path().join("terms.txt");
    std::fs::write(&file, "ok\n<bad>\n\nfine\n")?;
    let path = file.to_string_lossy().into_owned();
    let (code, refused) = json_run(&app, &["import", &path, "--dry-run"])?;
    assert_eq!(code, 2);
    assert_eq!(
        items(&refused),
        vec![
            (1, "term_forbidden_character".to_owned()),
            (2, "term_empty".to_owned())
        ]
    );
    let (code, _) = json_run(&app, &["import", &path])?;
    assert_eq!(code, 2);
    assert_eq!(app.store_bytes()?, None);
    std::fs::write(&file, "Claude Code\nClaude Sonnet\nclaude code\nOpenClaw\n")?;
    let (code, merged) = json_run(&app, &["import", &path, "--mode", "merge"])?;
    assert_eq!(code, 0);
    assert_eq!(
        merged["data"]["custom_words"],
        json!(["Claude Code", "Claude Sonnet", "OpenClaw"])
    );
    Ok(())
}

#[test]
fn stdin_import_replaces_and_export_writes_the_terms_on_stdout() -> TestResult {
    let app = Portable::new()?;
    json_run(&app, &["add", "Old"])?;
    let args = "settings vocabulary import - --mode replace --json";
    let args = args.split(' ').collect::<Vec<_>>();
    let replaced = run_with_stdin(&app, &args, "Claude Opus\r\nOpenClaw\r\n")?;
    assert_eq!(replaced.code, 0);
    assert_eq!(
        replaced.envelope()?["data"]["custom_words"],
        json!(["Claude Opus", "OpenClaw"])
    );
    let raw = app.run(&["settings", "vocabulary", "export", "-"])?;
    assert_eq!(
        (raw.code, raw.stdout.as_str()),
        (0, "Claude Opus\nOpenClaw\n")
    );
    let (_, exported) = json_run(&app, &["export"])?;
    assert_eq!(exported["data"]["content"], raw.stdout);
    let (code, file_target) = json_run(&app, &["export", "words.txt"])?;
    assert_eq!(
        (code, file_target["error"]["code"].as_str()),
        (2, Some("usage_error"))
    );
    Ok(())
}

#[test]
fn status_without_a_running_handy_is_unavailable_with_empty_details() -> TestResult {
    let app = Portable::new()?;
    json_run(&app, &["add", "Claude Code"])?;
    let (code, status) = json_run(&app, &["status"])?;
    assert_eq!(
        (code, status["error"]["code"].as_str()),
        (5, Some("unavailable"))
    );
    assert_eq!(status["error"]["details"], json!({}));
    let (code, _) = json_run(&app, &["list"])?;
    assert_eq!(code, 0);
    Ok(())
}
