//! Rule sets and imports larger than one 1 MiB channel message, offline and routed to a running
//! instance: paged lists and exports of a maximal valid set, and staged imports whose escaped
//! input exceeds one request, all with one reply and one atomic commit.
use super::harness::{Fallible, Portable, TestResult};
use super::support::{listed, path, replacements, texts, write};
use handy_app_lib::settings_service::{
    ipc, live::LiveSettings, ownership::LegacyPathProbe, store::SettingsRepository,
};
use serde_json::{json, Value};
use std::sync::Arc;

/// 1000 rules at the maximal lengths: about 2.5 MB of JSON.
fn maximal_rules() -> Vec<Value> {
    (0..1000)
        .map(|index| {
            let recognized = format!("{}{index}", "字".repeat(96));
            json!({"id": format!("id-{index}"), "recognized": recognized,
                   "replacement": "😀".repeat(500)})
        })
        .collect()
}
fn seed(app: &Portable, rules: &[Value]) -> TestResult {
    let store = json!({"settings": {"word_replacements": rules}, "settings_revision": 5});
    std::fs::write(app.store()?, serde_json::to_vec(&store)?)?;
    Ok(())
}
fn serve(app: &Portable) -> Fallible<(Arc<LiveSettings>, ipc::EndpointHandle)> {
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    let instance = LiveSettings::start_owned(SettingsRepository::new(app.store()?), &probe)?;
    let instance = Arc::new(instance);
    let endpoint = ipc::serve(&app.store()?, instance.clone())?;
    Ok((instance, endpoint))
}
fn expected_export(rules: &[Value]) -> String {
    let lines = rules.iter().map(|rule| {
        let text = |name: &str| rule[name].as_str().unwrap_or_default().to_owned();
        format!("{};{}\n", text("recognized"), text("replacement"))
    });
    std::iter::once("recognized;replacement\n".to_owned())
        .chain(lines)
        .collect()
}

fn list_and_export_a_maximal_set(app: &Portable, rules: &[Value]) -> TestResult {
    let run = app.run(&["settings", "replacements", "list", "--json"])?;
    assert!(
        run.stdout.len() > ipc::MAX_FRAME_BYTES,
        "larger than one message"
    );
    assert_eq!(
        (run.code, &run.envelope()?["data"]["rules"]),
        (0, &Value::from(rules.to_vec()))
    );
    let exported = path(app, "maximal.txt");
    let (code, envelope) = replacements(app, &["export", &exported])?;
    assert_eq!((code, &envelope["data"]["count"]), (0, &json!(1000)));
    assert_eq!(std::fs::read_to_string(&exported)?, expected_export(rules));
    Ok(())
}

#[test]
fn a_maximal_set_is_listed_and_exported_offline_and_from_a_running_instance() -> TestResult {
    let app = Portable::new()?;
    let rules = maximal_rules();
    seed(&app, &rules)?;
    list_and_export_a_maximal_set(&app, &rules)?;
    let (_instance, endpoint) = serve(&app)?;
    list_and_export_a_maximal_set(&app, &rules)?;
    let (code, refused) = replacements(&app, &["add", "one more", "x"])?;
    let reason = &refused["error"]["details"]["reason"];
    assert_eq!((code, reason), (2, &json!("too_many_rules")));
    endpoint.close();
    Ok(())
}

/// A valid input under 1,000,000 bytes whose JSON escaping exceeds one 1 MiB request: quotes in
/// every replacement and a comment line of control characters.
fn escaped_heavy_input() -> String {
    let mut input = format!("#{}\n", "\u{1}".repeat(100_000));
    for index in 0..1000 {
        input.push_str(&format!("r{index};{}\n", "\"".repeat(500)));
    }
    input
}

fn staged_import(app: &Portable) -> TestResult {
    let file = write(app, "heavy.txt", escaped_heavy_input().as_bytes())?;
    let dry = ["import", &file, "--mode", "replace", "--dry-run"];
    let (code, preview) = replacements(app, &dry)?;
    assert_eq!(
        (code, &preview["data"]["changes"]["added"]),
        (0, &json!(1000))
    );
    let (code, imported) = replacements(app, &["import", &file, "--mode", "replace"])?;
    assert_eq!((code, &imported["data"]["count"]), (0, &json!(1000)));
    let rules = listed(app)?;
    assert_eq!(texts(&rules)[999], ("r999".to_owned(), "\"".repeat(500)));
    let stale = [
        "import",
        &file,
        "--mode",
        "merge",
        "--expected-revision",
        "0",
    ];
    assert_eq!(replacements(app, &stale)?.0, 3);
    Ok(())
}

#[test]
fn an_import_larger_than_one_request_is_staged_offline_and_routed() -> TestResult {
    let input = escaped_heavy_input();
    assert!(input.len() <= 1_000_000);
    assert!(serde_json::to_string(&input)?.len() > ipc::MAX_FRAME_BYTES);
    staged_import(&Portable::new()?)?;
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    staged_import(&app)?;
    let published = serde_json::to_value(instance.snapshot().public_settings())?;
    let count = published["word_replacements"].as_array().map(Vec::len);
    assert_eq!(count, Some(1000), "committed by the instance");
    endpoint.close();
    Ok(())
}
