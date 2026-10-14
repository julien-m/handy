//! Single-rule operations, aliases, refusals and revisions, offline.
use super::harness::{Portable, TestResult};
use super::support::{commands, listed, pairs, replacements, texts};
use serde_json::json;

#[test]
fn help_lists_every_replacement_action() -> TestResult {
    let app = Portable::new()?;
    let help = app.run(&["settings", "replacements", "--help"])?;
    assert_eq!(help.code, 0);
    let expected = [
        "list", "add", "update", "remove", "import", "export", "help",
    ];
    assert_eq!(commands(&help.stdout), expected);
    Ok(())
}

#[test]
fn an_empty_recognized_text_exits_two_with_a_stable_code_and_no_write() -> TestResult {
    let app = Portable::new()?;
    let (code, envelope) = replacements(&app, &["add", "", "x"])?;
    let error = &envelope["error"];
    assert_eq!((code, &error["code"]), (2, &json!("validation_failed")));
    let item = json!([{"index": 0, "code": "recognized_empty",
                       "message": "The recognized text is empty."}]);
    assert_eq!(error["details"]["items"], item);
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

#[test]
fn add_then_edit_alias_keep_the_id_and_position() -> TestResult {
    let app = Portable::new()?;
    let (code, added) = replacements(&app, &["add", "cloud code", "Claude Code"])?;
    assert_eq!((code, &added["revision"]), (0, &json!(1)));
    assert_eq!(added["data"]["position"], 1);
    let id = added["data"]["rule"]["id"].as_str().unwrap_or_default();
    let (_, edited) = replacements(&app, &["edit", id, "cloud code", "Claude Code CLI"])?;
    let rule = json!({"id": id, "recognized": "cloud code", "replacement": "Claude Code CLI"});
    assert_eq!(
        (&edited["revision"], &edited["data"]["rule"]),
        (&json!(2), &rule)
    );
    let human = app.run(&["settings", "replacements", "list"])?;
    assert_eq!(human.stdout, format!("{id}\tcloud code\tClaude Code CLI\n"));
    Ok(())
}

#[test]
fn delete_alias_removes_and_an_unknown_id_is_not_found() -> TestResult {
    let app = Portable::new()?;
    let (_, added) = replacements(&app, &["add", "get hub", "GitHub"])?;
    let id = added["data"]["rule"]["id"].as_str().unwrap_or_default();
    let (code, deleted) = replacements(&app, &["delete", id])?;
    assert_eq!(
        (code, &deleted["data"]),
        (0, &json!({"removed": id, "count": 0}))
    );
    let (code, missing) = replacements(&app, &["remove", id])?;
    assert_eq!((code, &missing["error"]["code"]), (4, &json!("not_found")));
    assert_eq!(listed(&app)?, json!([]));
    Ok(())
}

#[test]
fn human_errors_go_to_stderr_with_the_item_locations() -> TestResult {
    let app = Portable::new()?;
    let refused = app.run(&["settings", "replacements", "add", "#tag", "x"])?;
    assert_eq!((refused.code, refused.stdout.as_str()), (2, ""));
    let expected = "word_replacements: Items failed validation; see details.items.";
    assert!(refused.stderr.contains(expected), "{}", refused.stderr);
    Ok(())
}

#[test]
fn a_stale_expected_revision_is_a_conflict_that_writes_nothing() -> TestResult {
    let app = Portable::new()?;
    replacements(&app, &["add", "a", "b"])?;
    let before = app.store_bytes()?;
    let file = super::support::write(&app, "rules.txt", b"c;d\n")?;
    let stale_add = ["add", "c", "d", "--expected-revision", "0"];
    let stale_import = [
        "import",
        &file,
        "--mode",
        "merge",
        "--expected-revision",
        "0",
    ];
    for args in [&stale_add[..], &stale_import[..]] {
        let (code, envelope) = replacements(&app, args)?;
        let current = &envelope["error"]["details"]["current_revision"];
        assert_eq!((code, current), (3, &json!(1)));
    }
    assert_eq!(app.store_bytes()?, before);
    assert_eq!(texts(&listed(&app)?), pairs(&[("a", "b")]));
    Ok(())
}

#[test]
fn generic_set_of_the_rules_names_the_replacements_domain() -> TestResult {
    let app = Portable::new()?;
    let run = app.run(&["settings", "set", "word_replacements", "[]", "--json"])?;
    let issue =
        json!([{"field": "word_replacements", "reason": "Use handy settings replacements."}]);
    assert_eq!(
        (run.code, &run.envelope()?["error"]["details"]["issues"]),
        (2, &issue)
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
