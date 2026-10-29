//! Output contract: one envelope per JSON invocation, readable text otherwise, stable error codes
//! and exits, both `--json` placements, usage errors and help, and no secret or rejected argument
//! on any channel.
use super::harness::{Portable, TestResult};
use serde_json::json;

/// Default paste method of this build: smart paste is the macOS default only once dictation
/// delivers through it (feature 007 rollout gate, Cargo feature `smart-paste-dispatch`).
pub(super) fn default_paste_method() -> &'static str {
    if cfg!(all(target_os = "macos", feature = "smart-paste-dispatch")) {
        "smart_paste"
    } else {
        "ctrl_v"
    }
}

#[test]
fn json_get_prints_one_envelope_with_revision_and_every_public_key() -> TestResult {
    let app = Portable::new()?;
    for args in [
        &["settings", "get", "--json"][..],
        &["--json", "settings", "get"],
        &["--json", "settings", "get", "--json"],
    ] {
        let run = app.run(args)?;
        let envelope = run.envelope()?;
        assert_eq!(
            (run.code, &envelope["ok"], &envelope["revision"]),
            (0, &json!(true), &json!(0))
        );
        // 010 adds `show_recovery_bubble_without_text_field` with the gate open; 007 AC-019
        // adds `paste_verification` in every build.
        let keys = 68 + usize::from(cfg!(feature = "destination-return"));
        assert_eq!(
            envelope["data"].as_object().map(|data| data.len()),
            Some(keys)
        );
    }
    let one = app
        .run(&["settings", "get", "paste_method", "--json"])?
        .envelope()?;
    assert_eq!(one["data"], json!({"paste_method": default_paste_method()}));
    Ok(())
}
#[test]
fn human_output_is_readable_and_diagnostics_stay_on_stderr() -> TestResult {
    let app = Portable::new()?;
    let get = app.run(&["settings", "get", "paste_method"])?;
    assert_eq!(
        (get.code, get.stdout.as_str(), get.stderr.as_str()),
        (0, format!("{}\n", default_paste_method()).as_str(), "")
    );
    let set = app.run(&["settings", "set", "paste_method", "direct"])?;
    assert_eq!(
        (set.code, set.stdout.as_str()),
        (0, "paste_method = direct (revision 1)\n")
    );
    let refused = app.run(&["settings", "set", "paste_method", "invalid_method"])?;
    assert_eq!((refused.code, refused.stdout.as_str()), (2, ""));
    assert!(refused
        .stderr
        .starts_with("Error: Setting value failed validation."));
    Ok(())
}
#[test]
fn keys_describe_type_allowed_values_range_and_access() -> TestResult {
    let app = Portable::new()?;
    let envelope = app.run(&["settings", "keys", "--json"])?.envelope()?;
    let keys = envelope["data"]["keys"]
        .as_array()
        .ok_or("keys is not a list")?;
    let find = |name: &str| keys.iter().find(|row| row["key"] == name).cloned();
    let paste = find("paste_method").ok_or("paste_method missing")?;
    assert_eq!(
        (&paste["value_type"], &paste["read_only"]),
        (&json!("string"), &json!(false))
    );
    assert!(paste["allowed_values"]
        .as_array()
        .is_some_and(|values| values.contains(&json!("direct"))));
    assert_eq!(
        find("paste_delay_ms").ok_or("missing")?["range"],
        json!({"min": 10.0, "max": 5000.0})
    );
    assert_eq!(
        find("selected_model").ok_or("missing")?["read_only"],
        json!(true)
    );
    assert_eq!(
        find("post_process_api_keys").ok_or("missing")?["secret"],
        json!(true)
    );
    Ok(())
}
/// (arguments, exit code, error code) of every error this feature owns, except those needing an
/// instance (`unavailable`) or a damaged store (`persistence_failed`), tested in `offline`.
const ERROR_CASES: [(&[&str], i32, &str); 6] = [
    (
        &["settings", "set", "paste_delay_ms", "-5", "--json"],
        2,
        "validation_failed",
    ),
    (
        &[
            "settings",
            "set",
            "paste_method",
            "ctrl_v",
            "--expected-revision",
            "9",
            "--json",
        ],
        3,
        "revision_conflict",
    ),
    (&["settings", "get", "not_a_key", "--json"], 4, "not_found"),
    (
        &["settings", "set", "not_a_key", "1", "--json"],
        4,
        "not_found",
    ),
    (&["settings", "frobnicate", "--json"], 2, "usage_error"),
    (
        &["--start-hidden", "settings", "get", "--json"],
        2,
        "usage_error",
    ),
];
#[test]
fn every_owned_error_code_has_its_documented_exit() -> TestResult {
    let app = Portable::new()?;
    for (args, code, name) in ERROR_CASES {
        let run = app.run(args)?;
        let envelope = run.envelope()?;
        assert_eq!(
            (run.code, &envelope["error"]["code"]),
            (code, &json!(name)),
            "{args:?}"
        );
        assert!(envelope["error"]["details"].is_object());
    }
    assert_eq!(
        app.store_bytes()?,
        None,
        "no refused write created the store"
    );
    Ok(())
}
#[test]
fn stale_expected_revision_reports_the_current_revision() -> TestResult {
    let app = Portable::new()?;
    let first = app.run(&["settings", "set", "paste_method", "direct"])?;
    assert_eq!(first.code, 0, "{}", first.stderr);
    let run = app.run(&[
        "settings",
        "set",
        "paste_method",
        "none",
        "--expected-revision",
        "0",
        "--json",
    ])?;
    let envelope = run.envelope()?;
    assert_eq!(envelope["error"]["details"], json!({"current_revision": 1}));
    let ok = app.run(&[
        "settings",
        "set",
        "paste_method",
        "none",
        "--expected-revision",
        "1",
        "--json",
    ])?;
    assert_eq!((ok.code, &ok.envelope()?["revision"]), (0, &json!(2)));
    Ok(())
}
const UI_ONLY: &str = "This setting is changed in the settings UI.";
/// Keys generic `set` refuses, with the reason naming where they are changed.
const MICROPHONES: &str =
    "Use handy settings microphones set-priority to change the microphone choices.";
const OWNED_ELSEWHERE: [(&str, &str); 7] = [
    ("selected_microphone", MICROPHONES),
    ("microphone_priority", MICROPHONES),
    (
        "bindings",
        "Use handy settings shortcuts to change shortcuts.",
    ),
    (
        "custom_words",
        "Use handy settings vocabulary to change this list.",
    ),
    // The vocabulary commands do not manage filler words: they are changed in the app.
    ("custom_filler_words", UI_ONLY),
    ("post_process_api_keys", UI_ONLY),
    ("selected_model", UI_ONLY),
];
#[test]
fn list_secret_and_read_only_keys_point_to_their_owner() -> TestResult {
    let app = Portable::new()?;
    for (key, reason) in OWNED_ELSEWHERE {
        let run = app.run(&["settings", "set", key, "x", "--json"])?;
        let envelope = run.envelope()?;
        assert_eq!(
            (run.code, &envelope["error"]["code"]),
            (2, &json!("validation_failed"))
        );
        assert_eq!(
            envelope["error"]["details"]["issues"][0]["reason"],
            json!(reason),
            "{key}"
        );
    }
    Ok(())
}
#[test]
fn secrets_and_rejected_arguments_never_reach_stdout_or_stderr() -> TestResult {
    let app = Portable::new()?;
    let sentinel = "sk-SENTINEL-4f1c";
    let store = json!({"settings": {"post_process_api_keys": {"openai": sentinel}}});
    std::fs::write(app.store()?, serde_json::to_vec(&store)?)?;
    let runs = [
        app.run(&["settings", "get", "--json"])?,
        app.run(&["settings", "get"])?,
        app.run(&[
            "settings",
            "set",
            "post_process_api_keys",
            sentinel,
            "--json",
        ])?,
        app.run(&["settings", "get", "paste_method", sentinel])?,
        app.run(&["settings", "set", "paste_method", sentinel])?,
    ];
    for run in &runs {
        assert!(!run.stdout.contains(sentinel) && !run.stderr.contains(sentinel));
    }
    let envelope = runs[0].envelope()?;
    assert_eq!(
        envelope["data"]["post_process_api_keys"]["openai"],
        json!({"present": true})
    );
    Ok(())
}
#[test]
fn help_lists_generic_operations_in_both_modes() -> TestResult {
    let app = Portable::new()?;
    let human = app.run(&["settings", "--help"])?;
    assert_eq!(human.code, 0);
    for operation in ["get", "keys", "set", "Exit codes"] {
        assert!(human.stdout.contains(operation), "{operation}");
    }
    let json = app.run(&["settings", "--help", "--json"])?;
    let envelope = json.envelope()?;
    assert_eq!(
        (json.code, &envelope["ok"], &envelope["revision"]),
        (0, &json!(true), &json!(0))
    );
    assert!(envelope["data"]["help"]
        .as_str()
        .is_some_and(|text| text.contains("keys")));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
