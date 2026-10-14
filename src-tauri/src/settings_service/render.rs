//! Single renderer of settings replies: exactly one JSON envelope on stdout with `--json`, readable
//! text otherwise; diagnostics and warnings always go to stderr. Values of secrets never reach it:
//! replies only carry the public projection.
use super::{operations::Reply, protocol::SettingsOperation};
use serde_json::Value;
use std::io::Write;

/// What the reply answers, for readable output only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum View {
    AllSettings,
    OneSetting,
    Keys,
    Change,
    /// Dry run: the changes that would be committed, nothing written.
    Preview,
    /// Export: the exchange content itself (`data.content`), nothing else on stdout.
    Export,
    /// Shortcut lists (`handy settings shortcuts`): one line per action.
    Shortcuts,
    Help,
}
impl View {
    /// Readable layout of an operation's data.
    pub fn of(operation: &SettingsOperation) -> Self {
        match operation {
            SettingsOperation::Get { key: None } => Self::AllSettings,
            SettingsOperation::Get { key: Some(_) } => Self::OneSetting,
            SettingsOperation::Keys => Self::Keys,
            SettingsOperation::Set { .. } => Self::Change,
            SettingsOperation::Domain {
                domain,
                action,
                arguments,
                dry_run,
                ..
            } => Self::of_domain(domain, action, arguments.len(), *dry_run),
        }
    }
    fn of_domain(domain: &str, action: &str, arguments: usize, dry_run: bool) -> Self {
        match (action, arguments) {
            _ if dry_run => Self::Preview,
            _ if domain == "shortcuts" => Self::Shortcuts,
            ("export", _) => Self::Export,
            ("get", 0) => Self::AllSettings,
            ("get", _) => Self::OneSetting,
            _ => Self::Change,
        }
    }
}

/// Text written on stdout and stderr for one reply.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

/// Print `reply` and return its stable exit code. Write failures cannot change the exit code:
/// the operation already happened, so its outcome stays authoritative.
// @spec FR-006: Single renderer and exit boundary — .specs/features/008-settings-cli/spec.md#fr-006
pub fn emit(view: View, reply: &Reply, json: bool) -> i32 {
    let output = render(view, reply, json);
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(output.stdout.as_bytes());
    let _ = stdout.flush();
    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(output.stderr.as_bytes());
    let _ = stderr.flush();
    reply.exit_code()
}

/// Pure rendering of one reply.
pub fn render(view: View, reply: &Reply, json: bool) -> Output {
    let envelope = reply.envelope();
    let mut output = Output::default();
    if reply.warning() {
        output
            .stderr
            .push_str("Warning: the settings store was recovered; some values may be defaults.\n");
    }
    if json {
        output.stdout = format!("{envelope}\n");
    } else if envelope["ok"] == Value::Bool(true) {
        output.stdout = success_text(view, envelope);
    }
    if envelope["ok"] == Value::Bool(false) {
        output.stderr.push_str(&error_text(&envelope["error"]));
    }
    output
}

fn success_text(view: View, envelope: &Value) -> String {
    let data = &envelope["data"];
    match view {
        View::AllSettings => object_lines(data),
        View::OneSetting => data
            .as_object()
            .and_then(|object| object.values().next())
            .map_or_else(String::new, |value| format!("{}\n", plain(value))),
        View::Keys => keys_table(&data["keys"]),
        View::Change => change_lines(data, &envelope["revision"]),
        View::Preview => format!(
            "Dry run: nothing was changed (revision {}). Would set:\n{}",
            envelope["revision"],
            object_lines(&data["changes"])
        ),
        View::Export => data["content"].as_str().unwrap_or_default().to_owned(),
        View::Shortcuts => shortcut_lines(data),
        View::Help => data["help"].as_str().unwrap_or_default().to_owned(),
    }
}
/// `key = value (revision N)` per changed key.
fn change_lines(data: &Value, revision: &Value) -> String {
    data.as_object().map_or_else(String::new, |object| {
        object
            .iter()
            .map(|(key, value)| format!("{key} = {} (revision {revision})\n", plain(value)))
            .collect()
    })
}
/// `action: first, second (default: shortcut)` per action of a list or of a change.
fn shortcut_lines(data: &Value) -> String {
    let actions = match data["actions"].as_array() {
        Some(actions) => actions.clone(),
        None => vec![data.clone()],
    };
    actions
        .iter()
        .map(|action| {
            let shortcuts = action["shortcuts"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(plain)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}: {shortcuts} (default: {})\n",
                plain(&action["action_id"]),
                plain(&action["default_shortcut"])
            )
        })
        .collect()
}
/// Strings print as themselves; other values print as compact JSON.
fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
fn object_lines(data: &Value) -> String {
    data.as_object().map_or_else(String::new, |object| {
        object
            .iter()
            .map(|(key, value)| format!("{key} = {value}\n"))
            .collect()
    })
}
/// One aligned row per key: name, type, allowed values or range, and access.
fn keys_table(keys: &Value) -> String {
    let rows = keys.as_array().map(Vec::as_slice).unwrap_or_default();
    let width = rows
        .iter()
        .filter_map(|row| row["key"].as_str())
        .map(str::len)
        .max()
        .unwrap_or_default();
    rows.iter()
        .map(|row| {
            let nullable = if row["nullable"] == Value::Bool(true) {
                "?"
            } else {
                ""
            };
            let kind = format!(
                "{}{nullable}",
                row["value_type"].as_str().unwrap_or_default()
            );
            format!(
                "{:width$}  {kind:9}  {:24}  {}\n",
                row["key"].as_str().unwrap_or_default(),
                constraint(row),
                access(row),
            )
        })
        .collect()
}
fn constraint(row: &Value) -> String {
    let allowed = row["allowed_values"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    if !allowed.is_empty() {
        return allowed
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("|");
    }
    match (row["range"]["min"].as_f64(), row["range"]["max"].as_f64()) {
        (Some(min), Some(max)) => format!("{min}..{max}"),
        _ => "-".to_owned(),
    }
}
fn access(row: &Value) -> &'static str {
    if row["secret"] == Value::Bool(true) {
        "secret"
    } else if row["read_only"] == Value::Bool(true) {
        "read-only"
    } else if matches!(row["value_type"].as_str(), Some("array" | "object")) {
        "domain"
    } else {
        "settable"
    }
}
/// Fixed message plus the allowlisted details, and what an agent must do next.
fn error_text(error: &Value) -> String {
    let details = &error["details"];
    let mut text = format!("Error: {}\n", error["message"].as_str().unwrap_or_default());
    for issue in details["issues"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        text.push_str(&format!(
            "  {}: {}\n",
            issue["field"].as_str().unwrap_or_default(),
            issue["reason"].as_str().unwrap_or_default()
        ));
    }
    if let Some(revision) = details["current_revision"].as_u64() {
        text.push_str(&format!("  current revision: {revision}\n"));
    }
    text.push_str(&reason_lines(details));
    text.push_str(match details["outcome"].as_str() {
        Some("unknown") => "  The outcome is unknown: read the settings again before retrying.\n",
        Some("committed") => "  The change was saved, but Handy could not apply it completely.\n",
        _ if error["code"] == "usage_error" => "  Run `handy settings --help` for usage.\n",
        _ => "",
    });
    text
}
/// Stable reason, action and shortcut owner or limit of a refusal, when present.
fn reason_lines(details: &Value) -> String {
    let mut text = String::new();
    if let Some(reason) = details["reason"].as_str() {
        text.push_str(&format!("  reason: {reason}\n"));
    }
    if let Some(action) = details["action_id"].as_str() {
        text.push_str(&format!("  action: {action}\n"));
    }
    if let Some(owner) = details["owning_action"].as_str() {
        let shortcut = details["owning_shortcut"].as_str().unwrap_or_default();
        text.push_str(&format!("  conflicts with: {shortcut} of {owner}\n"));
    }
    if let Some(limit) = details["limit"].as_u64() {
        text.push_str(&format!("  limit: {limit} shortcuts per action\n"));
    }
    text
}
#[cfg(test)]
mod tests {
    use super::{render, View};
    use crate::settings_service::{errors::SettingsError, operations::Reply, protocol::Revision};
    use serde_json::json;

    #[test]
    fn json_mode_prints_exactly_one_envelope_line_and_errors_only_on_stderr() {
        let success = Reply::success(Revision(3), json!({"paste_method": "direct"}), false);
        let output = render(View::Change, &success, true);
        assert_eq!(output.stdout.lines().count(), 1);
        assert!(output.stderr.is_empty());
        let failure = Reply::failure(SettingsError::unknown_outcome());
        let output = render(View::Change, &failure, false);
        assert!(output.stdout.is_empty());
        assert!(output
            .stderr
            .contains("read the settings again before retrying"));
        assert_eq!(failure.exit_code(), 5);
    }
    #[test]
    fn recovered_settings_warn_on_stderr_in_both_modes() {
        let reply = Reply::success(Revision(0), json!({"theme": "dark"}), true);
        for json in [true, false] {
            assert!(render(View::OneSetting, &reply, json)
                .stderr
                .starts_with("Warning:"));
        }
        assert_eq!(render(View::OneSetting, &reply, false).stdout, "dark\n");
    }
}
