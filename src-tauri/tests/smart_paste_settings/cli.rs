//! `handy settings output` with smart paste through the real binary (AC-015): values, error codes
//! and exits, `--dry-run`, JSON channel purity and parity with generic `set`, on an isolated
//! portable store, plus the offline upgrade path (read normalizes, the next write persists).
#![cfg(unix)]
use super::active;
use super::harness::{Fallible, Portable, Run, TestResult};
use serde_json::{json, Value};

const SET_SMART: [&str; 7] = [
    "settings",
    "output",
    "set",
    "paste_method",
    "smart_paste",
    "--json",
    "--dry-run",
];

/// Envelope of a JSON run: stdout carries exactly one JSON line; a success writes nothing else.
fn json_envelope(run: &Run) -> Fallible<Value> {
    let envelope = run.envelope()?;
    if envelope["ok"] == true {
        assert_eq!(run.stderr, "");
    }
    Ok(envelope)
}
/// A refusal: exit 2, `validation_failed` naming `field`, no data, and the reason on stderr.
fn assert_refused(run: &Run, field: &str) -> TestResult {
    let envelope = json_envelope(run)?;
    assert_eq!(run.code, 2);
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "validation_failed");
    assert_eq!(envelope["error"]["details"]["field"], field);
    assert!(envelope.get("data").is_none());
    assert!(run.stderr.contains(&format!("{field}: ")));
    Ok(())
}

// @spec FR-009: Output domain sets smart paste with the shared validation — .specs/features/007-smart-paste/spec.md#fr-009
#[test]
fn output_set_smart_paste_commits_or_is_refused_by_the_rollout() -> TestResult {
    let app = Portable::new()?;
    let set = app.run(&SET_SMART[..6])?;
    if !active() {
        assert_refused(&set, "paste_method")?;
        assert_eq!(app.store_bytes()?, None);
        return Ok(());
    }
    let envelope = json_envelope(&set)?;
    assert_eq!(set.code, 0);
    // A new installation already defaults to smart paste: the write is a no-op at revision 0.
    let expected = json!({"ok": true, "revision": 0, "data": {"paste_method": "smart_paste"}});
    assert_eq!(envelope, expected);
    app.run(&["settings", "output", "set", "paste_method", "ctrl_v"])?;
    let back = json_envelope(&app.run(&SET_SMART[..6])?)?;
    assert_eq!(back["revision"], 2);
    let get = app.run(&["settings", "output", "get", "paste_method", "--json"])?;
    assert_eq!(
        json_envelope(&get)?["data"],
        json!({"paste_method": "smart_paste"})
    );
    Ok(())
}

#[test]
fn output_dry_run_previews_or_refuses_smart_paste_and_never_writes() -> TestResult {
    let app = Portable::new()?;
    app.run(&["settings", "set", "paste_method", "none"])?;
    let before = app.store_bytes()?;
    let preview = app.run(&SET_SMART)?;
    if active() {
        let envelope = json_envelope(&preview)?;
        assert_eq!(preview.code, 0);
        let changes = json!({"dry_run": true, "changes": {"paste_method": "smart_paste"}});
        assert_eq!(
            (&envelope["revision"], &envelope["data"]),
            (&json!(1), &changes)
        );
    } else {
        assert_refused(&preview, "paste_method")?;
    }
    assert_eq!(app.store_bytes()?, before);
    Ok(())
}

#[test]
fn generic_set_and_output_set_give_the_same_answer() -> TestResult {
    let generic = Portable::new()?;
    let domain = Portable::new()?;
    let by_key = generic.run(&["settings", "set", "paste_method", "smart_paste", "--json"])?;
    let by_domain = domain.run(&SET_SMART[..6])?;
    assert_eq!(
        (by_key.code, by_key.envelope()?),
        (by_domain.code, by_domain.envelope()?)
    );
    Ok(())
}

#[test]
fn readable_refusal_goes_to_stderr_with_exit_two() -> TestResult {
    if active() {
        return Ok(());
    }
    let app = Portable::new()?;
    let run = app.run(&SET_SMART[..5])?;
    assert_eq!((run.code, run.stdout.as_str()), (2, ""));
    assert!(run
        .stderr
        .starts_with("Error: Setting value failed validation."));
    assert!(run.stderr.contains("Smart paste is not available"));
    Ok(())
}

#[test]
fn keys_list_smart_paste_only_when_selectable_and_the_marker_as_read_only() -> TestResult {
    let app = Portable::new()?;
    let envelope = json_envelope(&app.run(&["settings", "keys", "--json"])?)?;
    let keys = envelope["data"]["keys"].as_array().ok_or("keys")?;
    let find = |name: &str| keys.iter().find(|row| row["key"] == name).cloned();
    let paste = find("paste_method").ok_or("paste_method")?;
    let listed = paste["allowed_values"]
        .as_array()
        .is_some_and(|values| values.contains(&json!("smart_paste")));
    assert_eq!(listed, active());
    let marker = find("smart_paste_migration_version").ok_or("marker")?;
    assert_eq!(marker["read_only"], true);
    Ok(())
}

#[test]
fn output_set_of_the_beta_switch_follows_the_rollout() -> TestResult {
    let app = Portable::new()?;
    let args = [
        "settings",
        "output",
        "set",
        "reliable_paste",
        "true",
        "--json",
    ];
    let run = app.run(&args)?;
    if active() {
        assert_refused(&run, "reliable_paste")?;
    } else {
        assert_eq!(
            (run.code, &json_envelope(&run)?["revision"]),
            (0, &json!(1))
        );
    }
    Ok(())
}

#[test]
fn offline_upgrade_reads_normalized_and_the_next_write_persists_the_migration() -> TestResult {
    let app = Portable::new()?;
    let seeded = json!({"settings_revision": 3, "settings": {"paste_method": "ctrl_v", "reliable_paste": true}});
    std::fs::write(app.store()?, serde_json::to_vec(&seeded)?)?;
    let before = app.store_bytes()?;
    let method = if active() { "smart_paste" } else { "ctrl_v" };
    let get = app.run(&["settings", "output", "get", "paste_method", "--json"])?;
    let envelope = json_envelope(&get)?;
    assert_eq!(
        (&envelope["revision"], &envelope["data"]["paste_method"]),
        (&json!(3), &json!(method))
    );
    assert_eq!(app.store_bytes()?, before);
    app.run(&["settings", "output", "set", "auto_submit", "true"])?;
    let stored: Value = serde_json::from_slice(&app.store_bytes()?.ok_or("store")?)?;
    assert_eq!(stored["settings_revision"], 4);
    assert_eq!(stored["settings"]["paste_method"], method);
    assert_eq!(
        stored["settings"]["smart_paste_migration_version"],
        u32::from(active())
    );
    assert_eq!(stored["settings"]["reliable_paste"], !active());
    Ok(())
}
