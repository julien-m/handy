//! Tauri glue of smart capitalization (feature 015): at the final preparation of an insertion,
//! read the settings snapshot, read the destination context on a blocking worker (never on the
//! async runtime or the main thread), and adapt the first letter. The context and the text never
//! reach a log: only the decision kind and whether the text changed are logged.
use super::reader::{context_from_units, read_frontmost, window_before, CaretPort, CONTEXT_BUDGET};
use super::{decide, prepare_text, CaseInputs, CursorContext};
use crate::destination::delivery::DictationTicket;
use crate::destination::field_context::InsertionTarget;
use crate::destination::manager::DestinationManager;
use crate::destination::workspace::{Foreground, SystemClock, WorkspacePort};
use crate::settings::AppSettings;
use std::sync::Arc;
use tauri::AppHandle;

// @spec FR-004: Final preparation on every insertion path — .specs/features/015-smart-capitalization/spec.md#fr-004
/// `text` as it will be inserted for the dictation `ticket` (none without a 006 session):
/// unchanged with the option off, for an AI-decided case or an empty text, and without any
/// accessibility query in those cases.
pub async fn prepare(
    app: &AppHandle,
    ticket: Option<DictationTicket>,
    text: String,
    case_decided: bool,
) -> String {
    let settings = crate::settings::get_settings(app);
    if !settings.smart_capitalization || case_decided || text.is_empty() {
        return text;
    }
    let context = read_on_worker(app, ticket).await;
    let protected = protected_spellings(&settings);
    let inputs = CaseInputs {
        case_decided,
        protected: &protected,
    };
    let prepared = prepare_text(&text, &context, &inputs);
    log::debug!(
        "Smart capitalization: {:?}, first letter changed: {}",
        decide(&context),
        prepared != text
    );
    prepared
}

/// The dictation's context, read on a blocking worker (accessibility calls block up to the
/// budget); a failed worker is Absent.
async fn read_on_worker(app: &AppHandle, ticket: Option<DictationTicket>) -> CursorContext {
    let manager = crate::destination::app::manager(app);
    match tauri::async_runtime::spawn_blocking(move || read_context(manager, ticket)).await {
        Ok(context) => context,
        Err(error) => {
            log::warn!("Smart capitalization context read failed ({error}); text unchanged");
            CursorContext::Absent
        }
    }
}

/// The 002 replacement values and 003 custom words of the insertion's settings snapshot.
fn protected_spellings(settings: &AppSettings) -> Vec<String> {
    settings
        .word_replacements
        .iter()
        .map(|rule| rule.replacement.clone())
        .chain(settings.custom_words.iter().cloned())
        .collect()
}

// @spec AC-007: Context source — .specs/features/015-smart-capitalization/spec.md#ac-007
/// The context of the dictation: 011's remembered caret when there is one; otherwise the
/// frontmost application's focused element when the text goes there; otherwise Absent.
fn read_context(
    manager: Option<Arc<DestinationManager>>,
    ticket: Option<DictationTicket>,
) -> CursorContext {
    let workspace = manager
        .as_ref()
        .map_or_else(crate::destination::native_workspace, |manager| {
            manager.workspace()
        });
    let (Some(manager), Some(ticket)) = (manager, ticket) else {
        return read_front(workspace.as_ref(), None);
    };
    let key = (ticket.dictation_id, ticket.destination_generation);
    if let Some(answer) = manager.read_remembered_before_caret(key, window_before, CONTEXT_BUDGET) {
        return answer.map_or(CursorContext::Absent, |(units, window)| {
            context_from_units(&units, window)
        });
    }
    match manager.insertion_target(key.0, key.1) {
        InsertionTarget::Stale => CursorContext::Absent,
        InsertionTarget::Frontmost => read_front(workspace.as_ref(), None),
        InsertionTarget::Application { pid } => read_front(workspace.as_ref(), Some(pid)),
    }
}

/// The frontmost application's context, when it is the expected destination `expected` (any
/// application when `None`); Handy itself or an unreadable foreground is Absent.
fn read_front(workspace: &dyn WorkspacePort, expected: Option<u32>) -> CursorContext {
    match readable_front(&workspace.frontmost_now(), expected) {
        Some(pid) => read_frontmost(native_caret_port().as_mut(), pid, &SystemClock),
        None => CursorContext::Absent,
    }
}

/// Business rule (spec AC-007): the process whose field may be read, only when the text goes
/// to the application in front; another destination, Handy itself in front or an unreadable
/// foreground give none.
fn readable_front(foreground: &Foreground, expected: Option<u32>) -> Option<u32> {
    let Foreground::Application(front) = foreground else {
        return None;
    };
    expected
        .is_none_or(|pid| pid == front.pid)
        .then_some(front.pid)
}

/// The context reader of this platform (macOS only in this feature, spec decision
/// `macos_only_reader`).
fn native_caret_port() -> Box<dyn CaretPort> {
    #[cfg(target_os = "macos")]
    return Box::new(super::macos::MacCaretPort);
    #[cfg(not(target_os = "macos"))]
    return Box::new(super::reader::UnsupportedCaretPort);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::destination::context::ApplicationIdentity;
    use crate::destination::workspace::RunningApp;
    use crate::word_replacements::model::ReplacementRule;

    fn front(pid: u32) -> Foreground {
        Foreground::Application(RunningApp {
            pid,
            identity: ApplicationIdentity::BundleId("com.apple.TextEdit".to_owned()),
            display_name: "TextEdit".to_owned(),
            icon_ref: None,
        })
    }

    #[test]
    fn foreign_destination_is_absent() {
        assert_eq!(readable_front(&front(7), Some(8)), None);
    }

    #[test]
    fn frontmost_destination_is_read() {
        assert_eq!(readable_front(&front(7), Some(7)), Some(7));
        assert_eq!(readable_front(&front(7), None), Some(7));
    }

    #[test]
    fn handy_or_unreadable_foreground_is_absent() {
        assert_eq!(readable_front(&Foreground::Handy, None), None);
        assert_eq!(readable_front(&Foreground::Unreadable, None), None);
    }

    #[test]
    fn protected_spellings_hold_replacements_and_custom_words() {
        let settings = AppSettings {
            custom_words: vec!["Notion".to_owned()],
            word_replacements: vec![ReplacementRule {
                id: "rule".into(),
                recognized: "cloud code".to_owned(),
                replacement: "Claude Code".to_owned(),
            }],
            ..Default::default()
        };
        assert_eq!(
            protected_spellings(&settings),
            ["Claude Code".to_owned(), "Notion".to_owned()]
        );
    }
}
