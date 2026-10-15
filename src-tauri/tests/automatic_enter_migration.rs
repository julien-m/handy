//! Migration of auto-submit to Automatic Enter through the real binary and an isolated portable
//! store (009 AC-004, SC-002), in the gate state of this build: a read migrates in memory without
//! writing, the next write persists it once with one revision, the legacy switch and key are
//! never rewritten, a closed build never writes `automatic_enter` and keeps a value written by an
//! open build, and an invalid stored value reads as `never`. Both gate states of the migration
//! function itself are covered in one binary by `automatic_enter::migration` unit tests.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use harness::{Fallible, Portable, TestResult};
use serde_json::{json, Value};

const OPEN: bool = cfg!(feature = "destination-return");

fn seeded(settings: Value) -> Fallible<Portable> {
    let app = Portable::new()?;
    let store = json!({"settings_revision": 3, "settings": settings});
    std::fs::write(app.store()?, serde_json::to_vec(&store)?)?;
    Ok(app)
}
fn stored(app: &Portable) -> Fallible<Value> {
    Ok(serde_json::from_slice(&app.store_bytes()?.ok_or("store")?)?)
}
fn output(app: &Portable) -> Fallible<Value> {
    app.run(&["settings", "output", "get", "--json"])?
        .envelope()
}
/// One unrelated write, so the normalized settings are persisted.
fn write_something(app: &Portable) -> TestResult {
    let run = app.run(&["settings", "set", "mute_while_recording", "true"])?;
    assert_eq!(run.code, 0);
    Ok(())
}

// @spec AC-004: Migration keeps the key and the legacy switch — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
// @spec SC-002: Every auto-submit combination migrates — .specs/features/009-pin-mode-and-enter-rule/spec.md#sc-002
#[test]
fn every_switch_and_key_reads_as_documented_and_is_persisted_once() -> TestResult {
    for (switch, policy) in [(true, "always"), (false, "never")] {
        for key in ["enter", "ctrl_enter", "cmd_enter"] {
            let app = seeded(json!({"auto_submit": switch, "auto_submit_key": key}))?;
            let before = app.store_bytes()?;
            let read = output(&app)?;
            assert_eq!(read["revision"], json!(3), "a read never writes");
            assert_eq!(app.store_bytes()?, before);
            assert_eq!(read["data"]["auto_submit_key"], json!(key));
            if OPEN {
                assert_eq!(read["data"]["automatic_enter"], json!(policy));
            } else {
                assert_eq!(read["data"]["auto_submit"], json!(switch));
            }
            write_something(&app)?;
            let disk = stored(&app)?;
            assert_eq!(disk["settings_revision"], json!(4), "one increment");
            assert_eq!(
                disk["settings"]["auto_submit"],
                json!(switch),
                "never rewritten"
            );
            assert_eq!(disk["settings"]["auto_submit_key"], json!(key));
            let written = disk["settings"].get("automatic_enter").cloned();
            assert_eq!(written, OPEN.then(|| json!(policy)));
        }
    }
    Ok(())
}
#[test]
fn an_older_handy_running_never_gets_the_migration_written_behind_it() -> TestResult {
    let app = seeded(json!({"auto_submit": true, "auto_submit_key": "ctrl_enter"}))?;
    let before = app.store_bytes()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let run = app.run(&["settings", "set", "mute_while_recording", "true", "--json"])?;
    assert_eq!(run.code, 5, "{}", run.stderr);
    assert_eq!(
        app.store_bytes()?,
        before,
        "the older Handy keeps its store"
    );
    Ok(())
}
#[test]
fn a_store_written_by_an_open_build_keeps_both_values_in_either_build() -> TestResult {
    let app = seeded(json!({"auto_submit": false, "automatic_enter": "always"}))?;
    let read = output(&app)?;
    if OPEN {
        assert_eq!(
            read["data"]["automatic_enter"],
            json!("always"),
            "stays authoritative"
        );
    } else {
        assert_eq!(
            read["data"]["auto_submit"],
            json!(false),
            "the closed build reads dev's switch"
        );
        assert!(read["data"].get("automatic_enter").is_none());
    }
    write_something(&app)?;
    let disk = stored(&app)?;
    assert_eq!(disk["settings"]["auto_submit"], json!(false));
    assert_eq!(disk["settings"]["automatic_enter"], json!("always"));
    Ok(())
}
#[test]
fn an_invalid_stored_policy_reads_as_never_and_never_enables_enter() -> TestResult {
    let app = seeded(json!({"auto_submit": true, "automatic_enter": "sometimes"}))?;
    let read = output(&app)?;
    if OPEN {
        assert_eq!(read["data"]["automatic_enter"], json!("never"));
    } else {
        assert_eq!(read["data"]["auto_submit"], json!(true));
    }
    write_something(&app)?;
    let disk = stored(&app)?;
    let expected = if OPEN {
        json!("never")
    } else {
        json!("sometimes")
    };
    assert_eq!(disk["settings"]["automatic_enter"], expected);
    assert_eq!(disk["settings"]["auto_submit"], json!(true));
    Ok(())
}
