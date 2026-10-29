//! Terminal targets of feature 013 through an addressing tool (cmux today): the argument vectors
//! of its commands, the parsing of its JSON answers (identifiers and names only), the capture of
//! the focused workspace, its resolution at insertion and the escape-safe typing of the text.
//! Every call goes through a [`CliRunner`] with `CLI_TIMEOUT`; nothing read here is logged.
use super::model::{AddressingTool, TerminalTarget, MAX_TEXT_CHARS};
use super::runner::{CliError, CliRunner};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Bound of one tool call (spec AC-003, AC-007).
pub const CLI_TIMEOUT: Duration = Duration::from_secs(2);
/// Characters cmux's `send` turns into keys after a backslash: `\n`/`\r` press Enter, `\t` Tab.
const ESCAPED: [char; 3] = ['n', 'r', 't'];
/// Longest workspace or surface identifier accepted from or passed to the tool (a UUID is 36).
const MAX_HANDLE_CHARS: usize = 64;

/// The terminal a delivery types into, resolved at insertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedTerminal {
    pub cli: PathBuf,
    pub workspace_id: String,
    pub surface_id: String,
}

/// Why a terminal could not be read or resolved (logged as such, never with output).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalError {
    /// The tool failed, timed out or could not start.
    Tool(CliError),
    /// The tool exited with a failure (for a resolution: the workspace does not exist).
    Refused(Option<i32>),
    /// The answer was not the expected JSON, or had no usable terminal.
    Unreadable,
}

/// Business rule: an identifier passed to the tool is a UUID-like handle (hexadecimal digits and
/// dashes), so a stored value can never be read by the tool as an option or a reference.
pub fn is_handle(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_HANDLE_CHARS
        && value.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// `identify`: the focused window, workspace, pane and surface of the tool's application.
pub fn identify_arguments() -> Vec<String> {
    owned(&["--json", "--id-format", "both", "identify", "--no-caller"])
}
/// `tree --workspace`: one workspace with its panes and surfaces.
pub fn tree_arguments(workspace_id: &str) -> Vec<String> {
    let mut arguments = owned(&["--json", "--id-format", "both", "tree", "--workspace"]);
    arguments.push(workspace_id.to_owned());
    arguments
}
/// `send`: type `chunk` into one surface; `--` keeps a chunk starting with `-` a text.
pub fn send_arguments(terminal: &ResolvedTerminal, chunk: &str) -> Vec<String> {
    let mut arguments = owned(&["send", "--workspace"]);
    arguments.extend([
        terminal.workspace_id.clone(),
        "--surface".to_owned(),
        terminal.surface_id.clone(),
        "--".to_owned(),
        chunk.to_owned(),
    ]);
    arguments
}
fn owned(arguments: &[&str]) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect()
}

/// The focused workspace UUID of an `identify` answer.
pub fn focused_workspace(answer: &str) -> Option<String> {
    let value: Value = serde_json::from_str(answer).ok()?;
    let id = value.get("focused")?.get("workspace_id")?.as_str()?;
    is_handle(id).then(|| id.to_owned())
}

/// One workspace of a `tree` answer: its name and the terminal surface that receives text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceView {
    pub name: String,
    /// The selected terminal surface of the focused pane (else the first pane), if any.
    pub surface_id: Option<String>,
}

// @spec AC-007: Workspace's focused terminal surface — .specs/features/013-default-input-per-app/spec.md#ac-007
/// The workspace `workspace_id` of a `tree --json` answer (identifiers compared without case).
pub fn workspace_view(answer: &str, workspace_id: &str) -> Option<WorkspaceView> {
    let value: Value = serde_json::from_str(answer).ok()?;
    let workspace = value
        .get("windows")?
        .as_array()?
        .iter()
        .filter_map(|window| window.get("workspaces")?.as_array())
        .flatten()
        .find(|workspace| {
            workspace
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.eq_ignore_ascii_case(workspace_id))
        })?;
    let name = workspace
        .get("title")
        .and_then(Value::as_str)
        .map(|title| title.chars().take(MAX_TEXT_CHARS).collect::<String>())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| workspace_id.to_owned());
    Some(WorkspaceView {
        name,
        surface_id: receiving_surface(workspace),
    })
}

/// Business rule (spec decision `terminal_identity_workspace`): text goes to the focused pane's
/// selected surface (the first pane when none is marked focused), and only to a terminal.
fn receiving_surface(workspace: &Value) -> Option<String> {
    let panes = workspace.get("panes")?.as_array()?;
    let is_true = |value: &Value, key: &str| value.get(key).and_then(Value::as_bool) == Some(true);
    let pane = panes
        .iter()
        .find(|pane| is_true(pane, "focused"))
        .or_else(|| panes.first())?;
    let surface = pane
        .get("surfaces")?
        .as_array()?
        .iter()
        .find(|surface| is_true(surface, "selected_in_pane") || is_true(surface, "selected"))?;
    let is_terminal = surface.get("type").and_then(Value::as_str) == Some("terminal");
    let id = surface.get("id")?.as_str()?;
    (is_terminal && is_handle(id)).then(|| id.to_owned())
}

// @spec AC-007: Escape-safe chunks, no Enter — .specs/features/013-default-input-per-app/spec.md#ac-007
/// The text cut for `send` (spec decision `terminal_text_escapes`): line breaks and tabs become
/// spaces, and a chunk ends right after any backslash followed by `n`, `r` or `t`, so that no
/// single argument holds an escape cmux would turn into Enter or Tab. Empty text gives nothing.
pub fn chunks(text: &str) -> Vec<String> {
    let flat: Vec<char> = text
        .chars()
        .map(|c| {
            if matches!(c, '\n' | '\r' | '\t') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let mut chunks = Vec::new();
    let mut current = String::new();
    for (index, character) in flat.iter().enumerate() {
        current.push(*character);
        let escapes_next = *character == '\\'
            && flat
                .get(index + 1)
                .is_some_and(|next| ESCAPED.contains(next));
        if escapes_next {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn call(runner: &dyn CliRunner, cli: &Path, arguments: &[String]) -> Result<String, TerminalError> {
    let output = runner
        .run(cli, arguments, CLI_TIMEOUT)
        .map_err(TerminalError::Tool)?;
    if !output.success {
        return Err(TerminalError::Refused(output.code));
    }
    Ok(output.stdout)
}

// @spec FR-003: Terminal captured through the bundle's tool — .specs/features/013-default-input-per-app/spec.md#fr-003
/// The focused workspace of the tool's application, as a target to remember.
///
/// # Errors
/// The tool's failure, or an answer without a workspace.
pub fn capture(
    runner: &dyn CliRunner,
    tool: AddressingTool,
    cli: &Path,
) -> Result<TerminalTarget, TerminalError> {
    let identified = call(runner, cli, &identify_arguments())?;
    let workspace_id = focused_workspace(&identified).ok_or(TerminalError::Unreadable)?;
    let tree = call(runner, cli, &tree_arguments(&workspace_id))?;
    let view = workspace_view(&tree, &workspace_id).ok_or(TerminalError::Unreadable)?;
    Ok(TerminalTarget {
        tool,
        workspace_id,
        workspace_name: view.name,
    })
}

/// The remembered workspace's receiving terminal, if it still exists.
///
/// # Errors
/// `Refused` when the tool does not know the workspace, `Unreadable` without a terminal surface,
/// or the tool's failure.
pub fn resolve(
    runner: &dyn CliRunner,
    cli: &Path,
    target: &TerminalTarget,
) -> Result<ResolvedTerminal, TerminalError> {
    if !is_handle(&target.workspace_id) {
        return Err(TerminalError::Unreadable);
    }
    let tree = call(runner, cli, &tree_arguments(&target.workspace_id))?;
    let view = workspace_view(&tree, &target.workspace_id).ok_or(TerminalError::Unreadable)?;
    let surface_id = view.surface_id.ok_or(TerminalError::Unreadable)?;
    Ok(ResolvedTerminal {
        cli: cli.to_path_buf(),
        workspace_id: target.workspace_id.clone(),
        surface_id,
    })
}

// @spec FR-007: Typed through the tool, without Enter — .specs/features/013-default-input-per-app/spec.md#fr-007
/// Type `text` into `terminal`, chunk by chunk; stops at the first failure.
///
/// # Errors
/// A short description of the failed call (exit code or tool error), never the text.
pub fn send(runner: &dyn CliRunner, terminal: &ResolvedTerminal, text: &str) -> Result<(), String> {
    for chunk in chunks(text) {
        call(runner, &terminal.cli, &send_arguments(terminal, &chunk))
            .map_err(|error| format!("terminal send failed: {error:?}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "terminal_tests.rs"]
pub(crate) mod tests;
