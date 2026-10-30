//! Shared steps of the replacement CLI tests: `--json` runs, input files and rule readback.
use super::harness::{Fallible, Portable};
use serde_json::Value;

/// Run `handy settings replacements <args> --json`: exit code and envelope.
pub fn replacements(app: &Portable, args: &[&str]) -> Fallible<(i32, Value)> {
    let mut full = vec!["settings", "replacements"];
    full.extend_from_slice(args);
    full.push("--json");
    let run = app.run(&full)?;
    Ok((run.code, run.envelope()?))
}
/// Write `content` to `name` beside the portable installation; returns its path.
pub fn write(app: &Portable, name: &str, content: &[u8]) -> Fallible<String> {
    let path = app.root.path().join(name);
    std::fs::write(&path, content)?;
    Ok(path.to_string_lossy().into_owned())
}
/// A path beside the portable installation, as a command argument.
pub fn path(app: &Portable, name: &str) -> String {
    app.root.path().join(name).to_string_lossy().into_owned()
}
/// The listed rules (`data.rules`).
pub fn listed(app: &Portable) -> Fallible<Value> {
    let (code, envelope) = replacements(app, &["list"])?;
    if code != 0 {
        return Err(format!("list failed: {envelope}").into());
    }
    Ok(envelope["data"]["rules"].clone())
}
/// Recognized and replacement texts of rules, in order.
pub fn texts(rules: &Value) -> Vec<(String, String)> {
    let text = |rule: &Value, name: &str| rule[name].as_str().unwrap_or_default().to_owned();
    rules
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|rule| (text(rule, "recognized"), text(rule, "replacement")))
        .collect()
}
/// Owned pairs for comparisons with `texts`.
pub fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(recognized, replacement)| ((*recognized).to_owned(), (*replacement).to_owned()))
        .collect()
}
/// Command names of a clap help text (the indented first words of its "Commands:" section).
pub fn commands(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|line| !line.starts_with("Commands:"))
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .collect()
}
