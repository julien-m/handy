//! UI-facing destination DTOs: session snapshot, menu entries with opaque tokens, and the
//! choice a badge sends back. They carry names, icon references and versions only — never a
//! transcript, a window title or a caller-supplied process id.
use super::context::{DictationId, RecoveryDisposition};
use super::session::{DestinationSession, DestinationSupport, Selection, SessionPhase};
use serde::{Deserialize, Serialize};

/// What the badge shows for the current selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionView {
    FrontmostAtEnd,
    Application {
        display_name: String,
        icon_ref: Option<String>,
    },
}
impl From<&Selection> for SelectionView {
    fn from(selection: &Selection) -> Self {
        match selection {
            Selection::FrontmostAtEnd => Self::FrontmostAtEnd,
            Selection::Application(destination) => Self::Application {
                display_name: destination.display_name.clone(),
                icon_ref: destination.icon_ref.clone(),
            },
        }
    }
}

/// Session snapshot sent to the UI (`destination-changed`) and returned by session commands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct DestinationSnapshot {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub selection_version: u64,
    pub phase: SessionPhase,
    pub return_enabled: bool,
    pub support: DestinationSupport,
    pub selection: SelectionView,
    pub disposition: RecoveryDisposition,
}
impl From<&DestinationSession> for DestinationSnapshot {
    fn from(session: &DestinationSession) -> Self {
        Self {
            dictation_id: session.dictation_id,
            destination_generation: session.destination_generation,
            selection_version: session.selection_version,
            phase: session.phase,
            return_enabled: session.return_enabled,
            support: session.support,
            selection: SelectionView::from(&session.selection),
            disposition: session.disposition,
        }
    }
}

/// One application entry of the badge menu, addressed by an opaque manager token.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct MenuEntry {
    pub token: u64,
    pub display_name: String,
    pub icon_ref: Option<String>,
    /// The current destination of this dictation.
    pub current: bool,
    /// The application frontmost when the menu opened (listed first).
    pub frontmost: bool,
}
/// Badge menu of one dictation: applications, then "Frontmost app at the end".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct DestinationMenu {
    pub dictation_id: DictationId,
    pub selection_version: u64,
    pub entries: Vec<MenuEntry>,
    /// True when "Frontmost app at the end" is the current choice.
    pub frontmost_at_end_current: bool,
}

/// A badge choice: an entry token of the current menu, or "Frontmost app at the end" (the cross).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", content = "token", rename_all = "snake_case")]
pub enum MenuChoice {
    Application(u64),
    FrontmostAtEnd,
}
