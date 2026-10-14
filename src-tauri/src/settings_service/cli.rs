//! Early `handy settings` path: parse, route one operation and print its reply before any Tauri,
//! window, tray, model or single-instance initialization. Other invocations return to the app.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    operations::Reply,
    protocol::SettingsOperation,
    render::{emit, View},
    route::route,
};
use crate::cli::{
    ChangeOptions, CliArgs, CliCommand, OutputAction, PromptsAction, SettingsArgs, SettingsCommand,
    ShortcutsAction,
};
use clap::{error::ErrorKind, Parser};
use serde_json::json;
use std::ffi::OsString;

/// What `main` does next.
pub enum Dispatch {
    /// Not a settings invocation: start the app with these historical arguments (boxed: the
    /// parsed arguments are far larger than an exit code).
    App(Box<CliArgs>),
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
            None => Dispatch::App(Box::new(args)),
        },
        Err(error) if settings_requested(&argv) => Dispatch::Exit(parse_failure(&argv, &error)),
        Err(error) => error.exit(),
    }
}

fn run(args: &CliArgs, settings: &SettingsArgs) -> i32 {
    let json = args.json || settings.json;
    // A text read from stdin is read here, in the CLI process, never by the running instance.
    let operation = match read_operation(&settings.operation) {
        Ok(operation) => operation,
        Err(error) => return emit(View::Change, &Reply::failure(error), json),
    };
    let reply = if mixes_app_flags(args) {
        Reply::failure(SettingsError::new(SettingsErrorCode::UsageError))
    } else {
        route(&operation)
    };
    emit(View::of(&operation), &reply, json)
}
/// The wire operation of `command`, with a prompt text given as `-` read from stdin.
///
/// # Errors
/// The refusals of `input::read_source` (terminal stdin, size, encoding).
fn read_operation(command: &SettingsCommand) -> Result<SettingsOperation, SettingsError> {
    match command {
        SettingsCommand::Prompts(prompts) => {
            let input = match stdin_text(&prompts.action) {
                true => Some(super::input::read_source("-")?),
                false => None,
            };
            Ok(prompts_operation(&prompts.action, input))
        }
        other => Ok(operation(other)),
    }
}
/// Whether the prompt action takes its text from stdin (`--text -`).
fn stdin_text(action: &PromptsAction) -> bool {
    match action {
        PromptsAction::Create { text, .. } => text == "-",
        PromptsAction::Update { text, .. } => text.as_deref() == Some("-"),
        _ => false,
    }
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
        SettingsCommand::Output(output) => output_operation(&output.action),
        SettingsCommand::Shortcuts(shortcuts) => shortcuts_operation(&shortcuts.action),
        SettingsCommand::Prompts(prompts) => prompts_operation(&prompts.action, None),
    }
}
/// Wire operation of a prompts-domain action (arguments documented in `domains::prompts`);
/// `input` is the text already read from stdin, which then replaces the text argument.
fn prompts_operation(action: &PromptsAction, input: Option<String>) -> SettingsOperation {
    let (name, arguments, options) = prompt_arguments(action, input.is_some());
    SettingsOperation::Domain {
        domain: "prompts".to_owned(),
        action: name.to_owned(),
        arguments,
        input,
        dry_run: options.dry_run,
        expected_revision: options.expected_revision.map(super::protocol::Revision),
    }
}
/// Canonical action, wire arguments and change options of a prompts action.
fn prompt_arguments(
    action: &PromptsAction,
    from_stdin: bool,
) -> (&'static str, Vec<String>, ChangeOptions) {
    use PromptsAction as A;
    let (name, arguments) = match action {
        A::List => ("list", Vec::new()),
        A::Create { name, text, .. } => ("create", text_arguments(name, Some(text), from_stdin)),
        A::Update { id, name, text, .. } => {
            ("update", update_arguments(id, name, text, from_stdin))
        }
        A::Delete { id, .. } => ("remove", vec![id.clone()]),
        A::SetDefault { id, .. } => ("set-default", vec![id.clone()]),
        A::SetProviderOverride {
            id,
            provider,
            model,
            clear,
            ..
        } => (
            "set-provider-override",
            override_arguments(id, provider, model, *clear),
        ),
    };
    (name, arguments, prompt_options(action))
}
/// Revision and dry-run options of a prompts change; `list` has none.
fn prompt_options(action: &PromptsAction) -> ChangeOptions {
    match action {
        PromptsAction::List => ChangeOptions::default(),
        PromptsAction::Create { options, .. }
        | PromptsAction::Update { options, .. }
        | PromptsAction::Delete { options, .. }
        | PromptsAction::SetDefault { options, .. }
        | PromptsAction::SetProviderOverride { options, .. } => options.clone(),
    }
}
/// `create NAME [TEXT]`: the text is omitted when it comes from stdin.
fn text_arguments(name: &str, text: Option<&String>, from_stdin: bool) -> Vec<String> {
    let text = text.filter(|_| !from_stdin).cloned();
    std::iter::once(name.to_owned()).chain(text).collect()
}
/// `update ID [name=NAME] [text=TEXT]`: the text is omitted when it comes from stdin.
fn update_arguments(
    id: &str,
    name: &Option<String>,
    text: &Option<String>,
    from_stdin: bool,
) -> Vec<String> {
    let text = named("text", text).filter(|_| !from_stdin);
    let fields = [named("name", name), text].into_iter().flatten();
    std::iter::once(id.to_owned()).chain(fields).collect()
}
/// `set-provider-override ID provider=P model=M`, or `ID clear=true`.
fn override_arguments(
    id: &str,
    provider: &Option<String>,
    model: &Option<String>,
    clear: bool,
) -> Vec<String> {
    let fields = match clear {
        true => vec![Some("clear=true".to_owned())],
        false => vec![named("provider", provider), named("model", model)],
    };
    std::iter::once(id.to_owned())
        .chain(fields.into_iter().flatten())
        .collect()
}
/// `key=value` wire argument of an optional flag.
fn named(key: &str, value: &Option<String>) -> Option<String> {
    value.as_ref().map(|value| format!("{key}={value}"))
}
/// Wire operation of a shortcuts-domain action.
fn shortcuts_operation(action: &ShortcutsAction) -> SettingsOperation {
    let none = ChangeOptions::default();
    let (name, arguments, options) = match action {
        ShortcutsAction::List { action } => ("list", action.iter().cloned().collect(), &none),
        ShortcutsAction::Add {
            action,
            shortcut,
            options,
        } => ("add", vec![action.clone(), shortcut.clone()], options),
        ShortcutsAction::Remove {
            action,
            shortcut,
            options,
        } => ("remove", vec![action.clone(), shortcut.clone()], options),
        ShortcutsAction::Reset { action, options } => ("reset", vec![action.clone()], options),
    };
    SettingsOperation::Domain {
        domain: "shortcuts".to_owned(),
        action: name.to_owned(),
        arguments,
        input: None,
        dry_run: options.dry_run,
        expected_revision: options.expected_revision.map(super::protocol::Revision),
    }
}
/// Wire operation of an output-domain action (no file or stdin input in this domain).
fn output_operation(action: &OutputAction) -> SettingsOperation {
    let (action, arguments, dry_run, expected) = match action {
        OutputAction::Get { key } => ("get", key.iter().cloned().collect(), false, None),
        OutputAction::Set {
            key,
            value,
            expected_revision,
            dry_run,
        } => (
            "set",
            vec![key.clone(), value.clone()],
            *dry_run,
            *expected_revision,
        ),
    };
    SettingsOperation::Domain {
        domain: "output".to_owned(),
        action: action.to_owned(),
        arguments,
        input: None,
        dry_run,
        expected_revision: expected.map(super::protocol::Revision),
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
