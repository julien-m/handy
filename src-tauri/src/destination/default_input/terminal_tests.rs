// LiveSpec traceability: @spec(AC-003) @spec(AC-007) @spec(AC-010)
//! Terminal capture, resolution and escape-safe typing with a scripted tool (013 FR-003, FR-007).
use super::*;
use crate::destination::default_input::fake_tree::TestResult;
use crate::destination::default_input::runner::CliOutput;
use serde_json::json;
use std::sync::Mutex;

pub(crate) const WORKSPACE: &str = "43E000A6-4ED9-4758-BAE7-373F9D4303AB";
pub(crate) const SURFACE: &str = "3ECE0FCA-4B38-4067-9B31-32A337ECB039";

/// A scripted tool: answers `identify`, `tree` and `send` from its fields, records every call.
#[derive(Default)]
pub(crate) struct FakeRunner {
    pub calls: Mutex<Vec<(PathBuf, Vec<String>)>>,
    pub identify: Option<String>,
    /// `tree` answer per workspace id; an unknown workspace exits 1 like the tool.
    pub trees: Vec<(String, String)>,
    pub send_result: Option<Result<CliOutput, CliError>>,
}
impl FakeRunner {
    /// A tool whose focused workspace is named `name` with a selected `surface_type` surface.
    pub(crate) fn with_workspace(name: &str, surface_type: &str) -> Self {
        Self {
            identify: Some(identify_answer(WORKSPACE)),
            trees: vec![(
                WORKSPACE.to_owned(),
                tree_answer(WORKSPACE, name, surface_type),
            )],
            ..Self::default()
        }
    }
    /// The argument vectors of every call, in order.
    pub(crate) fn calls(&self) -> Vec<Vec<String>> {
        self.calls
            .lock()
            .map(|calls| {
                calls
                    .iter()
                    .map(|(_, arguments)| arguments.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
    fn answer(&self, arguments: &[String]) -> Result<CliOutput, CliError> {
        let ok = |stdout: &str| CliOutput {
            success: true,
            code: Some(0),
            stdout: stdout.to_owned(),
        };
        let refused = CliOutput {
            success: false,
            code: Some(1),
            stdout: String::new(),
        };
        if arguments.first().map(String::as_str) == Some("send") {
            return self.send_result.clone().unwrap_or_else(|| Ok(ok("")));
        }
        match arguments.get(3).map(String::as_str) {
            Some("identify") => Ok(self.identify.as_deref().map_or(refused, ok)),
            Some("tree") => {
                let wanted = arguments.last().cloned().unwrap_or_default();
                let tree = self.trees.iter().find(|(id, _)| *id == wanted);
                Ok(tree.map_or(refused, |(_, answer)| ok(answer)))
            }
            _ => Ok(refused),
        }
    }
}
impl CliRunner for FakeRunner {
    fn run(
        &self,
        program: &Path,
        arguments: &[String],
        _: Duration,
    ) -> Result<CliOutput, CliError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push((program.to_path_buf(), arguments.to_vec()));
        }
        self.answer(arguments)
    }
}

/// An `identify --json` answer focused on `workspace`.
pub(crate) fn identify_answer(workspace: &str) -> String {
    json!({"caller": null, "focused": {"workspace_id": workspace, "surface_id": SURFACE}})
        .to_string()
}
/// A `tree --json` answer holding `workspace`, its focused pane and a selected surface.
pub(crate) fn tree_answer(workspace: &str, name: &str, surface_type: &str) -> String {
    json!({"windows": [{"workspaces": [
        {"id": "11111111-2222-3333-4444-555555555555", "title": "other", "panes": []},
        {"id": workspace, "title": name, "panes": [
            {"focused": false, "surfaces": [{"id": "AAAA", "type": "terminal", "selected_in_pane": true}]},
            {"focused": true, "surfaces": [
                {"id": "BBBB", "type": "terminal", "selected_in_pane": false},
                {"id": SURFACE, "type": surface_type, "selected_in_pane": true}
            ]}
        ]}
    ]}]})
    .to_string()
}

fn cli() -> PathBuf {
    PathBuf::from("/Applications/Term.app/Contents/Resources/bin/cmux")
}

// @spec AC-003: Workspace UUID and name, never a ref — .specs/features/013-default-input-per-app/spec.md#ac-003
#[test]
fn capture_saves_the_focused_workspace_id_and_name() -> TestResult {
    let runner = FakeRunner::with_workspace("roam-radar", "terminal");
    let target = capture(&runner, AddressingTool::Cmux, &cli())
        .map_err(|error| format!("captured: {error:?}"))?;
    assert_eq!(target.workspace_id, WORKSPACE);
    assert_eq!(target.workspace_name, "roam-radar");
    let calls = runner.calls();
    assert_eq!(calls[0], identify_arguments());
    assert_eq!(calls[1], tree_arguments(WORKSPACE));
    Ok(())
}

#[test]
fn capture_fails_on_a_refused_or_unreadable_answer() -> TestResult {
    let silent = FakeRunner::default();
    assert_eq!(
        capture(&silent, AddressingTool::Cmux, &cli()),
        Err(TerminalError::Refused(Some(1)))
    );
    let garbled = FakeRunner {
        identify: Some("not json".to_owned()),
        ..FakeRunner::default()
    };
    assert_eq!(
        capture(&garbled, AddressingTool::Cmux, &cli()),
        Err(TerminalError::Unreadable)
    );
    let reference_only = FakeRunner {
        identify: Some(identify_answer("workspace:10")),
        ..FakeRunner::default()
    };
    assert_eq!(
        capture(&reference_only, AddressingTool::Cmux, &cli()),
        Err(TerminalError::Unreadable)
    );
    Ok(())
}

fn target() -> TerminalTarget {
    TerminalTarget {
        tool: AddressingTool::Cmux,
        workspace_id: WORKSPACE.to_owned(),
        workspace_name: "roam-radar".to_owned(),
    }
}

// @spec AC-007: Workspace's focused terminal surface — .specs/features/013-default-input-per-app/spec.md#ac-007
#[test]
fn resolution_picks_the_focused_panes_selected_terminal() -> TestResult {
    let runner = FakeRunner::with_workspace("roam-radar", "terminal");
    let resolved =
        resolve(&runner, &cli(), &target()).map_err(|error| format!("resolved: {error:?}"))?;
    assert_eq!(resolved.surface_id, SURFACE);
    assert_eq!(resolved.workspace_id, WORKSPACE);
    Ok(())
}

#[test]
fn a_gone_workspace_or_a_browser_surface_is_not_a_terminal() -> TestResult {
    let gone = FakeRunner::default();
    assert_eq!(
        resolve(&gone, &cli(), &target()),
        Err(TerminalError::Refused(Some(1)))
    );
    let browser = FakeRunner::with_workspace("roam-radar", "browser");
    assert_eq!(
        resolve(&browser, &cli(), &target()),
        Err(TerminalError::Unreadable)
    );
    let injected = TerminalTarget {
        workspace_id: "--all".to_owned(),
        ..target()
    };
    assert_eq!(
        resolve(&FakeRunner::default(), &cli(), &injected),
        Err(TerminalError::Unreadable)
    );
    Ok(())
}

// @spec AC-007: Escape-safe chunks, no Enter — .specs/features/013-default-input-per-app/spec.md#ac-007
#[test]
fn chunks_never_hold_an_enter_or_tab_escape_and_keep_the_text() -> TestResult {
    let text = "echo a\\nb\nthen\ttab \\x and \\\\t end\\";
    let parts = chunks(text);
    for part in &parts {
        for escape in ["\\n", "\\r", "\\t"] {
            assert!(!part.contains(escape), "{part:?}");
        }
        assert!(!part.contains(['\n', '\r', '\t']), "{part:?}");
    }
    let expected: String = text
        .chars()
        .map(|c| {
            if matches!(c, '\n' | '\r' | '\t') {
                ' '
            } else {
                c
            }
        })
        .collect();
    assert_eq!(parts.concat(), expected);
    assert!(chunks("").is_empty());
    Ok(())
}

#[test]
fn send_passes_explicit_ids_and_a_separator_before_each_chunk() -> TestResult {
    let runner = FakeRunner::default();
    let resolved = ResolvedTerminal {
        cli: cli(),
        workspace_id: WORKSPACE.to_owned(),
        surface_id: SURFACE.to_owned(),
    };
    send(&runner, &resolved, "-rf a\\nb").map_err(|error| format!("sent: {error:?}"))?;
    let calls = runner.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], send_arguments(&resolved, "-rf a\\"));
    assert_eq!(calls[1], send_arguments(&resolved, "nb"));
    let separator = calls[0].iter().position(|argument| argument == "--");
    assert_eq!(separator, Some(calls[0].len() - 2));
    Ok(())
}

#[test]
fn a_failed_send_stops_and_never_carries_the_text() -> TestResult {
    let runner = FakeRunner {
        send_result: Some(Err(CliError::Timeout)),
        ..FakeRunner::default()
    };
    let resolved = ResolvedTerminal {
        cli: cli(),
        workspace_id: WORKSPACE.to_owned(),
        surface_id: SURFACE.to_owned(),
    };
    let error = send(&runner, &resolved, "secret a\\nb").expect_err("failed");
    assert!(!error.contains("secret"));
    assert_eq!(runner.calls().len(), 1);
    Ok(())
}
