//! Early `handy settings` path: parse, route one operation and print its reply before any Tauri,
//! window, tray, model or single-instance initialization. Other invocations return to the app.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    operations::Reply,
    protocol::SettingsOperation,
    render::{emit, View},
    route::route,
};
use crate::cli::{CliArgs, CliCommand, SettingsArgs, SettingsCommand};
use clap::{error::ErrorKind, Parser};
use serde_json::json;
use std::ffi::OsString;

/// What `main` does next.
pub enum Dispatch {
    /// Not a settings invocation: start the app with these historical arguments.
    App(CliArgs),
    /// A settings invocation ran; exit with this code.
    Exit(i32),
}

/// Parse `argv` once. Settings invocations, including their parse failures and help, end here;
/// every other invocation keeps its historical behavior (a clap error exits as before).
// @spec FR-001: Early settings subcommand path — .specs/features/008-settings-cli/spec.md#fr-001
pub fn dispatch(argv: Vec<OsString>) -> Dispatch {
    match CliArgs::try_parse_from(&argv) {
        Ok(args) => match args.command.clone() {
            Some(CliCommand::Settings(settings)) => Dispatch::Exit(run(&args, &settings)),
            None => Dispatch::App(args),
        },
        Err(error) if settings_requested(&argv) => Dispatch::Exit(parse_failure(&argv, &error)),
        Err(error) => error.exit(),
    }
}

fn run(args: &CliArgs, settings: &SettingsArgs) -> i32 {
    let operation = operation(&settings.operation);
    let reply = if mixes_app_flags(args) {
        Reply::failure(SettingsError::new(SettingsErrorCode::UsageError))
    } else {
        route(&operation)
    };
    emit(View::of(&operation), &reply, args.json || settings.json)
}
/// The wire operation of a parsed command.
pub fn operation(command: &SettingsCommand) -> SettingsOperation {
    match command {
        SettingsCommand::Get { key } => SettingsOperation::Get { key: key.clone() },
        SettingsCommand::Keys => SettingsOperation::Keys,
        SettingsCommand::Set {
            key,
            value,
            expected_revision,
        } => SettingsOperation::Set {
            key: key.clone(),
            value: value.clone(),
            expected_revision: expected_revision.map(super::protocol::Revision),
        },
    }
}
/// Recording, startup and transcription flags cannot be combined with `settings`; `--debug` and
/// `--json` can.
pub fn mixes_app_flags(args: &CliArgs) -> bool {
    args.start_hidden
        || args.no_tray
        || args.toggle_transcription
        || args.toggle_post_process
        || args.cancel
        || args.transcribe_file.is_some()
        || args.model.is_some()
        || args.device_index.is_some()
        || args.list_devices
        || args.list_models
        || args.repeat.is_some()
}
/// Whether `argv` names the settings subcommand, so its parse failure follows the settings
/// contract. Historical flags that take a value are skipped with it; the scan stops at the first
/// other positional, which can only be an error of the historical parser.
pub fn settings_requested(argv: &[OsString]) -> bool {
    let mut tokens = argv.iter().skip(1);
    while let Some(token) = tokens.next() {
        match token.to_str() {
            Some("settings") => return true,
            Some("--transcribe-file" | "-f" | "--model" | "--device-index" | "--repeat") => {
                tokens.next();
            }
            Some(flag) if flag.starts_with('-') => {}
            _ => return false,
        }
    }
    false
}
fn json_requested(argv: &[OsString]) -> bool {
    argv.iter().any(|token| token == "--json")
}
/// Help succeeds; any other parse failure is `usage_error`. clap's own message is not printed,
/// because it can quote the rejected argument (which may be a secret); only its fixed error kind
/// is shown on stderr.
fn parse_failure(argv: &[OsString], error: &clap::Error) -> i32 {
    let json = json_requested(argv);
    if error.kind() == ErrorKind::DisplayHelp {
        return help(error.render().to_string(), json);
    }
    if !json {
        if let Some(kind) = error.kind().as_str() {
            eprintln!("{kind}");
        }
    }
    let reply = Reply::failure(SettingsError::new(SettingsErrorCode::UsageError));
    emit(View::Help, &reply, json)
}
/// Human help needs no settings access; JSON help carries the current revision, read like `get`.
fn help(text: String, json: bool) -> i32 {
    let reply = if json {
        let read = route(&SettingsOperation::Get { key: None });
        match read.envelope()["revision"].as_u64() {
            Some(revision) => Reply::success(
                super::protocol::Revision(revision),
                json!({ "help": text }),
                read.warning(),
            ),
            None => read,
        }
    } else {
        Reply::success(super::protocol::Revision(0), json!({ "help": text }), false)
    };
    emit(View::Help, &reply, json)
}
#[cfg(test)]
mod tests {
    use super::settings_requested;
    use std::ffi::OsString;

    fn argv(tokens: &[&str]) -> Vec<OsString> {
        tokens.iter().map(OsString::from).collect()
    }
    #[test]
    fn only_the_settings_subcommand_position_selects_the_settings_contract() {
        assert!(settings_requested(&argv(&["handy", "settings", "bogus"])));
        assert!(settings_requested(&argv(&[
            "handy", "--json", "--debug", "settings"
        ])));
        assert!(!settings_requested(&argv(&[
            "handy", "--model", "settings"
        ])));
        assert!(!settings_requested(&argv(&[
            "handy", "-f", "settings", "--json"
        ])));
        assert!(!settings_requested(&argv(&["handy", "stray", "settings"])));
        assert!(!settings_requested(&argv(&["handy", "--start-hidden"])));
    }
}
