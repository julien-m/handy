//! Import and export: dry run, stdin, bounds, file destinations and the SC-002 round trip.
use super::harness::{Portable, TestResult};
use super::support::{listed, pairs, path, replacements, texts, write};
use serde_json::json;

#[test]
fn import_dry_run_previews_two_additions_and_leaves_the_rules_unchanged() -> TestResult {
    let app = Portable::new()?;
    let content = b"recognized;replacement\ncloud code;Claude Code\nget hub\tGitHub\n";
    let file = write(&app, "two.txt", content)?;
    let args = ["import", &file, "--mode", "merge", "--dry-run"];
    let (code, envelope) = replacements(&app, &args)?;
    let changes = &envelope["data"]["changes"];
    assert_eq!((code, &envelope["data"]["dry_run"]), (0, &json!(true)));
    assert_eq!(
        (&changes["added"], &changes["count"]),
        (&json!(2), &json!(2))
    );
    let lines = json!([{"line": 2, "status": "add"}, {"line": 3, "status": "add"}]);
    assert_eq!(changes["lines"], lines);
    assert_eq!(app.store_bytes()?, None, "a dry run writes nothing");
    let human = app.run(&[
        "settings",
        "replacements",
        "import",
        &file,
        "--mode",
        "merge",
        "--dry-run",
    ])?;
    assert!(human
        .stdout
        .starts_with("Dry run: nothing was changed (revision 0)"));
    Ok(())
}

#[test]
fn stdin_import_accepts_bom_and_crlf_and_refuses_non_utf8_with_its_offset() -> TestResult {
    let app = Portable::new()?;
    let args = [
        "settings",
        "replacements",
        "import",
        "-",
        "--mode",
        "replace",
        "--json",
    ];
    let input = "\u{feff}recognized;replacement\r\nget hub;GitHub\r\n";
    let accepted = app.run_input(&args, Some(input.as_bytes()))?;
    assert_eq!(
        (accepted.code, &accepted.envelope()?["data"]["added"]),
        (0, &json!(1))
    );
    let refused = app.run_input(&args, Some(b"ok;fine\n\xff;bad\n"))?;
    let error = &refused.envelope()?["error"];
    assert_eq!(
        (refused.code, &error["code"]),
        (2, &json!("validation_failed"))
    );
    assert_eq!(error["details"]["offset"], 8);
    assert_eq!(texts(&listed(&app)?), pairs(&[("get hub", "GitHub")]));
    Ok(())
}

#[test]
fn import_files_are_bounded_at_one_million_bytes_and_must_exist() -> TestResult {
    let app = Portable::new()?;
    let over = write(
        &app,
        "over.txt",
        format!("#{}\n", "x".repeat(999_999)).as_bytes(),
    )?;
    let (code, envelope) = replacements(&app, &["import", &over, "--mode", "merge"])?;
    assert_eq!(
        (code, &envelope["error"]["code"]),
        (2, &json!("limit_exceeded"))
    );
    let missing = path(&app, "missing.txt");
    let (code, envelope) = replacements(&app, &["import", &missing, "--mode", "merge"])?;
    assert_eq!(
        (code, &envelope["error"]["code"]),
        (2, &json!("usage_error"))
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

/// `count` importable rules mixing scripts, with `;` and `#` inside replacements.
fn exchange_file(count: usize) -> String {
    let mut content = String::from("recognized;replacement\n");
    for index in 0..count {
        content.push_str(&format!(
            "règle {index} 字;Remplacement {index}; #{index} 😀\n"
        ));
    }
    content
}

fn export_then_replace_import_reproduces(count: usize) -> TestResult {
    let app = Portable::new()?;
    let source = write(&app, "source.txt", exchange_file(count).as_bytes())?;
    assert_eq!(
        replacements(&app, &["import", &source, "--mode", "replace"])?.0,
        0
    );
    let before = listed(&app)?;
    let exported = path(&app, "exported.txt");
    let (code, envelope) = replacements(&app, &["export", &exported])?;
    assert_eq!((code, &envelope["data"]["count"]), (0, &json!(count)));
    let stdout = app.run(&["settings", "replacements", "export"])?.stdout;
    assert_eq!(std::fs::read_to_string(&exported)?, stdout);
    let (code, again) = replacements(&app, &["import", &exported, "--mode", "replace"])?;
    assert_eq!((code, &again["data"]["unchanged"]), (0, &json!(count)));
    assert_eq!(listed(&app)?, before, "same rules, identities and order");
    assert_eq!(texts(&before).len(), count);
    Ok(())
}

#[test]
fn export_then_replace_import_reproduces_1_100_and_1000_rules() -> TestResult {
    for count in [1, 100, 1000] {
        export_then_replace_import_reproduces(count)?;
    }
    Ok(())
}

#[test]
fn an_unwritable_export_destination_is_refused_and_left_unchanged() -> TestResult {
    let app = Portable::new()?;
    replacements(&app, &["add", "a", "b"])?;
    let occupied = path(&app, "occupied");
    std::fs::create_dir(&occupied)?;
    std::fs::write(app.root.path().join("occupied/kept"), "kept")?;
    let (code, envelope) = replacements(&app, &["export", &occupied])?;
    let error = &envelope["error"];
    assert_eq!(
        (code, &error["code"], &error["details"]["reason"]),
        (
            1,
            &json!("persistence_failed"),
            &json!("export_not_written")
        )
    );
    let kept = std::fs::read_to_string(app.root.path().join("occupied/kept"))?;
    assert_eq!(kept, "kept");
    Ok(())
}

/// Line 1001 has no separator, after 500 blank and 500 comment lines; `extra` more invalid
/// lines follow it.
fn invalid_after_ignored(extra: usize) -> String {
    let ignored = format!("{}{}", "\n".repeat(500), "# note\n".repeat(500));
    format!("{ignored}Cloud Opus\n{}", "x\n".repeat(extra))
}

#[test]
fn an_invalid_line_after_a_thousand_ignored_lines_is_located_by_import_and_dry_run() -> TestResult {
    let app = Portable::new()?;
    let file = write(&app, "late.txt", invalid_after_ignored(0).as_bytes())?;
    let item = json!([{"index": 1000, "code": "missing_separator",
                       "message": "The line has no tab or semicolon separator."}]);
    for dry_run in [false, true] {
        let mut args = vec!["import", file.as_str(), "--mode", "merge"];
        args.extend(dry_run.then_some("--dry-run"));
        let (code, envelope) = replacements(&app, &args)?;
        let error = &envelope["error"];
        assert_eq!((code, &error["code"]), (2, &json!("validation_failed")));
        assert_eq!(error["details"]["items"], item, "dry run: {dry_run}");
    }
    let many = write(&app, "many.txt", invalid_after_ignored(150).as_bytes())?;
    let (code, envelope) = replacements(&app, &["import", &many, "--mode", "replace"])?;
    let items = &envelope["error"]["details"]["items"];
    let first = (&items[0]["index"], &items[0]["code"]);
    assert_eq!(
        (code, first),
        (2, (&json!(1000), &json!("missing_separator")))
    );
    assert_eq!(items.as_array().map(Vec::len), Some(100));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
