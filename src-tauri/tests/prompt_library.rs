//! 005 Step 2 on a real isolated store, through the `handy settings` binary: the one-time
//! starter migration keeps every existing prompt (id, name, text, unknown fields) and the
//! selected prompt byte for byte, even invalid or dangling ones (SC-003), adds the six starters
//! once, never re-adds a deleted starter, and adds nothing to a full library.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use harness::{Fallible, Portable, TestResult};
use serde_json::{json, Value};

const STARTERS: [&str; 6] = [
    "Minimal correction",
    "Friendly rephrasing",
    "Clarify",
    "Shorten",
    "Notes",
    "Translate to English",
];

/// A legacy store (no schema version) with a custom, an over-long and a duplicate prompt, and a
/// selection that names a prompt the library no longer has.
fn legacy_store(prompts: Value, selected: Value) -> Value {
    json!({
        "settings_revision": 4,
        "settings": {
            "post_process_prompts": prompts,
            "post_process_selected_prompt_id": selected,
        }
    })
}
fn custom_prompts() -> Value {
    json!([
        {"id": "prompt_1", "name": "Formal email", "prompt": "Rewrite: ${output}", "extra": "CANARY"},
        {"id": "prompt_2", "name": "n".repeat(61), "prompt": "Too long a name"},
        {"id": "prompt_3", "name": "formal EMAIL", "prompt": "   "}
    ])
}
fn write_store(app: &Portable, store: &Value) -> TestResult {
    std::fs::write(app.store()?, serde_json::to_vec(store)?)?;
    Ok(())
}
fn disk(app: &Portable) -> Fallible<Value> {
    let bytes = app.store_bytes()?.ok_or("store missing")?;
    Ok(serde_json::from_slice(&bytes)?)
}
/// Commit an unrelated scalar so the normalization (and the migration) is persisted.
fn commit_unrelated(app: &Portable) -> TestResult {
    let run = app.run(&["settings", "set", "audio_feedback", "true", "--json"])?;
    assert_eq!(run.code, 0, "{}", run.stderr);
    Ok(())
}
fn names(prompts: &Value) -> Vec<String> {
    prompts
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["name"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// The stored library must start with `fixture`, byte for byte, then hold the six starters.
fn assert_kept_then_starters(prompts: &Value, fixture: &Value) -> TestResult {
    let prompts = prompts.as_array().ok_or("prompts")?;
    let fixture = fixture.as_array().ok_or("fixture")?;
    assert_eq!(prompts.len(), fixture.len() + STARTERS.len());
    assert_eq!(&prompts[..fixture.len()], fixture.as_slice());
    assert_eq!(names(&json!(&prompts[fixture.len()..])), STARTERS);
    Ok(())
}
// @spec AC-002: Upgrade keeps prompts, adds starters once — .specs/features/005-custom-ai-prompts/spec.md#ac-002
#[test]
fn upgrade_keeps_existing_prompts_and_selection_and_adds_the_starters_once() -> TestResult {
    let app = Portable::new()?;
    write_store(&app, &legacy_store(custom_prompts(), json!("prompt_gone")))?;
    commit_unrelated(&app)?;
    // No running Handy was involved: the passive owner probe stays absent.
    assert!(!app.legacy_socket().exists());
    let stored = disk(&app)?;
    let settings = &stored["settings"];
    assert_kept_then_starters(&settings["post_process_prompts"], &custom_prompts())?;
    assert_eq!(settings["post_process_selected_prompt_id"], "prompt_gone");
    assert_eq!(settings["settings_schema_version"], 3);
    // A second commit (restart-equivalent read and write) changes nothing in the library.
    commit_unrelated(&app)?;
    let again = disk(&app)?;
    assert_eq!(
        again["settings"]["post_process_prompts"],
        settings["post_process_prompts"]
    );
    Ok(())
}
// @spec AC-002: Deleted starters never come back — .specs/features/005-custom-ai-prompts/spec.md#ac-002
#[test]
fn a_deleted_starter_never_comes_back() -> TestResult {
    let app = Portable::new()?;
    write_store(&app, &legacy_store(json!([]), Value::Null))?;
    commit_unrelated(&app)?;
    let mut stored = disk(&app)?;
    let prompts = stored["settings"]["post_process_prompts"]
        .as_array_mut()
        .ok_or("prompts")?;
    prompts.retain(|prompt| prompt["name"] != "Shorten");
    std::fs::write(app.store()?, serde_json::to_vec(&stored)?)?;
    commit_unrelated(&app)?;
    let after = names(&disk(&app)?["settings"]["post_process_prompts"]);
    assert_eq!(after.len(), 5);
    assert!(!after.contains(&"Shorten".to_owned()));
    Ok(())
}
// @spec AC-002: Starters only while there is room — .specs/features/005-custom-ai-prompts/spec.md#ac-002
#[test]
fn a_full_library_gets_no_starter() -> TestResult {
    let app = Portable::new()?;
    let prompts: Vec<Value> = (0..50)
        .map(|index| json!({"id": format!("p{index}"), "name": format!("Prompt {index}"), "prompt": "x"}))
        .collect();
    write_store(&app, &legacy_store(json!(prompts), Value::Null))?;
    commit_unrelated(&app)?;
    let stored = disk(&app)?;
    assert_eq!(stored["settings"]["post_process_prompts"], json!(prompts));
    assert_eq!(stored["settings"]["settings_schema_version"], 3);
    Ok(())
}
#[test]
fn a_fresh_install_lists_the_historical_prompt_then_the_starters_without_a_default() -> TestResult {
    let app = Portable::new()?;
    let read = app
        .run(&["settings", "get", "post_process_prompts", "--json"])?
        .envelope()?;
    let prompts = &read["data"]["post_process_prompts"];
    let mut expected = vec!["Improve Transcriptions".to_owned()];
    expected.extend(STARTERS.map(str::to_owned));
    assert_eq!(names(prompts), expected);
    let selected = app
        .run(&[
            "settings",
            "get",
            "post_process_selected_prompt_id",
            "--json",
        ])?
        .envelope()?;
    assert_eq!(
        selected["data"]["post_process_selected_prompt_id"],
        Value::Null
    );
    Ok(())
}
