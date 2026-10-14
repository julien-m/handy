//! Early `handy settings` path: parse, route one operation and print its reply before any Tauri,
//! window, tray, model or single-instance initialization. Other invocations return to the app.
use super::{
    errors::{ErrorReason, SettingsError, SettingsErrorCode},
    operations::Reply,
    protocol::SettingsOperation,
    render::{emit, View},
    route::route,
};
use crate::cli::{
    ChangeOptions, CliArgs, CliCommand, ImportModeArg, OutputAction, PromptsAction,
    ReplacementsAction, SettingsArgs, SettingsCommand, ShortcutsAction, VocabularyAction,
    VocabularyImportMode,
};
use crate::word_replacements::exchange_file;
use clap::{error::ErrorKind, Parser};
use serde_json::json;
use std::ffi::OsString;

mod microphones;
use microphones::microphones_operation;

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
    let operation = operation(&settings.operation);
    let destination = export_destination(&settings.operation);
    let view = destination.map_or_else(|| View::of(&operation), |_| View::ExportFile);
    let reply = if mixes_app_flags(args) {
        Reply::failure(SettingsError::new(SettingsErrorCode::UsageError))
    } else {
        // Files and stdin belong to this process: read before routing, written after the reply.
        let routed = match with_input(operation, &settings.operation) {
            Ok(operation) => super::replacements_transfer::route_operation(&operation),
            Err(error) => Reply::failure(error),
        };
        match destination {
            Some(path) => write_export(path, &routed),
            None => routed,
        }
    };
    emit(view, &reply, args.json || settings.json)
}
/// File destination of an export; `-` (stdout) is rendered from the reply itself.
fn export_destination(command: &SettingsCommand) -> Option<&str> {
    match command {
        SettingsCommand::Replacements(replacements) => match &replacements.action {
            ReplacementsAction::Export { destination } if destination != "-" => {
                Some(destination.as_str())
            }
            _ => None,
        },
        _ => None,
    }
}
/// Write the exported content of a successful `reply` to `path` (atomically) and answer with
/// `{path, count, bytes, durable}`; `durable` is false when the file was replaced but its
/// directory could not be synced. A refused or failed write is `persistence_failed` and leaves
/// the destination unchanged.
fn write_export(path: &str, reply: &Reply) -> Reply {
    let envelope = reply.envelope();
    let (Some(revision), Some(content)) = (
        envelope["revision"].as_u64(),
        envelope["data"]["content"].as_str(),
    ) else {
        return reply.clone();
    };
    let written = match exchange_file::write_exchange_file(std::path::Path::new(path), content) {
        Ok(written) => written,
        Err(error) => return Reply::failure(export_failure(&error)),
    };
    let data = json!({ "path": path, "count": envelope["data"]["count"], "bytes": content.len(),
        "durable": written == exchange_file::Written::Durable });
    Reply::success(super::protocol::Revision(revision), data, reply.warning())
}
/// Typed refusal of an export write; the reason never carries the path.
fn export_failure(error: &exchange_file::ExportError) -> SettingsError {
    let reason = match error {
        exchange_file::ExportError::SymbolicLink => ErrorReason::ExportSymbolicLink,
        exchange_file::ExportError::ReadOnly => ErrorReason::ExportReadOnly,
        exchange_file::ExportError::Io(_) => ErrorReason::ExportNotWritten,
    };
    SettingsError::new(SettingsErrorCode::PersistenceFailed).with_reason(reason)
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
        SettingsCommand::Replacements(replacements) => replacements_operation(&replacements.action),
        SettingsCommand::Shortcuts(shortcuts) => shortcuts_operation(&shortcuts.action),
        SettingsCommand::Prompts(prompts) => prompts_operation(&prompts.action, None),
        SettingsCommand::Vocabulary(vocabulary) => vocabulary_operation(&vocabulary.action),
        SettingsCommand::Microphones(microphones) => microphones_operation(&microphones.action),
    }
}
/// Domain input named by the command (a replacements or vocabulary import source, or a prompt
/// text given as `-`), read as bounded UTF-8 text.
///
/// # Errors
/// The refusals of `input::read_source` (unreadable file, terminal stdin, size, encoding).
fn with_input(
    mut operation: SettingsOperation,
    command: &SettingsCommand,
) -> Result<SettingsOperation, SettingsError> {
    if let SettingsCommand::Prompts(prompts) = command {
        return prompt_with_input(operation, &prompts.action);
    }
    let Some(source) = import_source(command) else {
        return Ok(operation);
    };
    if let SettingsOperation::Domain { input, .. } = &mut operation {
        *input = Some(super::input::read_source(source)?);
    }
    Ok(operation)
}
/// A prompts operation whose text comes from stdin (`--text -`), read here, in the CLI process,
/// never by the running instance; any other prompts operation is returned unchanged.
///
/// # Errors
/// The refusals of `input::read_source` (terminal stdin, size, encoding).
fn prompt_with_input(
    operation: SettingsOperation,
    action: &PromptsAction,
) -> Result<SettingsOperation, SettingsError> {
    if !stdin_text(action) {
        return Ok(operation);
    }
    let input = super::input::read_source("-")?;
    Ok(prompts_operation(action, Some(input)))
}
/// File path or `-` named by an import command; `None` for every other command.
fn import_source(command: &SettingsCommand) -> Option<&str> {
    match command {
        SettingsCommand::Replacements(replacements) => match &replacements.action {
            ReplacementsAction::Import { source, .. } => Some(source.as_str()),
            _ => None,
        },
        SettingsCommand::Vocabulary(vocabulary) => match &vocabulary.action {
            VocabularyAction::Import { source, .. } => Some(source.as_str()),
            _ => None,
        },
        _ => None,
    }
}
/// Wire operation of a vocabulary action; an import's input is attached by `run`.
// @spec FR-005: Vocabulary CLI commands — .specs/features/003-recognition-vocabulary/spec.md#fr-005
fn vocabulary_operation(action: &VocabularyAction) -> SettingsOperation {
    let none = ChangeOptions::default();
    let (name, arguments, options) = match action {
        VocabularyAction::List => ("list", Vec::new(), &none),
        VocabularyAction::Add { term, options } => ("add", vec![term.clone()], options),
        VocabularyAction::Remove { term, options } => ("remove", vec![term.clone()], options),
        VocabularyAction::Import { mode, options, .. } => {
            let mode = match mode {
                VocabularyImportMode::Merge => "merge",
                VocabularyImportMode::Replace => "replace",
            };
            ("import", vec![mode.to_owned()], options)
        }
        VocabularyAction::Export { target } => ("export", vec![target.clone()], &none),
        VocabularyAction::Status => ("status", Vec::new(), &none),
    };
    SettingsOperation::Domain {
        domain: "vocabulary".to_owned(),
        action: name.to_owned(),
        arguments,
        input: None,
        dry_run: options.dry_run,
        expected_revision: options.expected_revision.map(super::protocol::Revision),
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
        | PromptsAction::SetProviderOverride { options, .. } => *options,
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
/// Wire operation of a replacements action. An import's input is attached by `run` (it reads a
/// file or stdin); an export always asks for the content, which `run` writes to a file if named.
fn replacements_operation(action: &ReplacementsAction) -> SettingsOperation {
    let (name, arguments, options) = replacements_parts(action);
    SettingsOperation::Domain {
        domain: "replacements".to_owned(),
        action: name.to_owned(),
        arguments,
        input: None,
        dry_run: options.dry_run,
        expected_revision: options.expected_revision.map(super::protocol::Revision),
    }
}
/// Action name, positional arguments and change options of a replacements action.
fn replacements_parts(action: &ReplacementsAction) -> (&'static str, Vec<String>, ChangeOptions) {
    let none = ChangeOptions::default();
    match action {
        ReplacementsAction::List => ("list", Vec::new(), none),
        ReplacementsAction::Export { .. } => ("export", Vec::new(), none),
        ReplacementsAction::Add {
            recognized,
            replacement,
            options,
        } => (
            "add",
            vec![recognized.clone(), replacement.clone()],
            *options,
        ),
        ReplacementsAction::Update {
            id,
            recognized,
            replacement,
            options,
        } => (
            "update",
            vec![id.clone(), recognized.clone(), replacement.clone()],
            *options,
        ),
        ReplacementsAction::Remove { id, options } => ("remove", vec![id.clone()], *options),
        ReplacementsAction::Import { mode, options, .. } => {
            ("import", vec![import_mode(*mode).to_owned()], *options)
        }
    }
}
fn import_mode(mode: ImportModeArg) -> &'static str {
    match mode {
        ImportModeArg::Merge => "merge",
        ImportModeArg::Replace => "replace",
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
/// Human help needs no settings access; JSON help carries the current revision, read with
/// `keys`, whose reply is small whatever the settings hold.
fn help(text: String, json: bool) -> i32 {
    let reply = if json {
        let read = route(&SettingsOperation::Keys);
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
