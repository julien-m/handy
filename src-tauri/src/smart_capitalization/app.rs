//! Tauri glue of the insertion's final preparation (015 smart capitalization, 017 smart
//! spacing): read the settings snapshot, read the destination context once on a blocking worker
//! (never on the async runtime or the main thread), then run the steps. The context and the
//! text never reach a log: only the decision and context kinds and flags are logged.
use super::reader::{read_frontmost, read_remembered, CaretPort};
use super::{decide, prepare_text, CaseInputs, InsertionContext, Preparation, PreparedText};
use crate::destination::delivery::DictationTicket;
use crate::destination::field_context::InsertionTarget;
use crate::destination::manager::DestinationManager;
use crate::destination::workspace::{Foreground, SystemClock, WorkspacePort};
use crate::settings::AppSettings;
use crate::smart_spacing::SpacingInputs;
use std::sync::Arc;
use tauri::AppHandle;

// @spec FR-004: Final preparation on every insertion path — .specs/features/015-smart-capitalization/spec.md#fr-004
// @spec FR-004: Same preparation step and read — .specs/features/017-smart-spacing/spec.md#fr-004
/// `text` as it will be inserted for the dictation `ticket` (none without a 006 session):
/// unchanged, without any accessibility query, when no step applies (both options off, or
/// only capitalization on for an AI-decided case) or the text is empty.
pub async fn prepare(
    app: &AppHandle,
    ticket: Option<DictationTicket>,
    text: String,
    case_decided: bool,
) -> PreparedText {
    let settings = crate::settings::get_settings(app);
    let capitalizes = settings.smart_capitalization && !case_decided;
    if !(capitalizes || settings.smart_spacing) || text.is_empty() {
        return PreparedText::unchanged(text);
    }
    let context = read_on_worker(app, ticket, settings.smart_spacing).await;
    let protected = protected_spellings(&settings);
    let steps = Preparation {
        capitalization: capitalizes.then_some(CaseInputs {
            case_decided,
            protected: &protected,
        }),
        spacing: settings.smart_spacing.then_some(SpacingInputs {
            space_at_field_end: settings.append_trailing_space,
        }),
    };
    let prepared = prepare_text(&text, &context, &steps);
    log_preparation(&context, &steps, &prepared, prepared.text != text);
    prepared
}

/// Log the steps that ran: kinds and flags only, never the field text or the transcript.
fn log_preparation(
    context: &InsertionContext,
    steps: &Preparation<'_>,
    prepared: &PreparedText,
    text_changed: bool,
) {
    if steps.capitalization.is_some() {
        log::debug!(
            "Smart capitalization: {:?}, first letter changed: {}",
            decide(context.before()),
            prepared.case_changed
        );
    }
    if steps.spacing.is_some() {
        log::debug!(
            "Smart spacing: {:?}, text changed: {text_changed}, trailing space decided: {}",
            context,
            prepared.trailing_decided
        );
    }
}

/// The dictation's context, read on a blocking worker (accessibility calls block up to the
/// budget); a failed worker is Unavailable.
async fn read_on_worker(
    app: &AppHandle,
    ticket: Option<DictationTicket>,
    with_after: bool,
) -> InsertionContext {
    let manager = crate::destination::app::manager(app);
    let read = move || read_context(manager, ticket, with_after);
    match tauri::async_runtime::spawn_blocking(read).await {
        Ok(context) => context,
        Err(error) => {
            log::warn!("Insertion context read failed ({error}); no context used");
            InsertionContext::Unavailable
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
/// The context of the dictation: 011's remembered input at its remembered selection when there
/// is one; otherwise the frontmost application's focused element when the text goes there;
/// otherwise Unavailable. The side after the selection is read only `with_after`.
fn read_context(
    manager: Option<Arc<DestinationManager>>,
    ticket: Option<DictationTicket>,
    with_after: bool,
) -> InsertionContext {
    let workspace = manager
        .as_ref()
        .map_or_else(crate::destination::native_workspace, |manager| {
            manager.workspace()
        });
    let (Some(manager), Some(ticket)) = (manager, ticket) else {
        return read_front(workspace.as_ref(), None, with_after);
    };
    let key = (ticket.dictation_id, ticket.destination_generation);
    let remembered = manager.with_remembered_input(key, |node, range| {
        read_remembered(node, range, &SystemClock, with_after)
    });
    if let Some(context) = remembered {
        return context;
    }
    match manager.insertion_target(key.0, key.1) {
        InsertionTarget::Stale => InsertionContext::Unavailable,
        InsertionTarget::Frontmost => read_front(workspace.as_ref(), None, with_after),
        InsertionTarget::Application { pid } => {
            read_front(workspace.as_ref(), Some(pid), with_after)
        }
    }
}

/// The frontmost application's context, when it is the expected destination `expected` (any
/// application when `None`); Handy itself or an unreadable foreground is Unavailable.
fn read_front(
    workspace: &dyn WorkspacePort,
    expected: Option<u32>,
    with_after: bool,
) -> InsertionContext {
    match readable_front(&workspace.frontmost_now(), expected) {
        Some(pid) => read_frontmost(native_caret_port().as_mut(), pid, &SystemClock, with_after),
        None => InsertionContext::Unavailable,
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
