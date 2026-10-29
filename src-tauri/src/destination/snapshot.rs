//! UI-facing destination DTOs: session snapshot, menu entries with opaque tokens, and the
//! choice a badge sends back. They carry names, icon references and versions only — never a
//! transcript, a window title or a caller-supplied process id.
use super::context::{DestinationSource, DictationId, RecoveryDisposition};
use super::pin::PinView;
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
    /// Where the current selection comes from; `pinned` shows the pin mark (009).
    pub source: DestinationSource,
    /// The session pin, if an application is pinned (009); the badge menu offers Unpin then.
    pub pin: Option<PinView>,
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
            source: session.selection.source(),
            pin: session.pin.clone(),
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
    /// The application of the session pin (009); the others are "This dictation only" while
    /// an application is pinned.
    pub pinned: bool,
    /// A favorite application (012), listed in the favorites section near the badge.
    pub favorite: bool,
    /// False only for a favorite with no running instance: shown "Not open", never chosen.
    pub running: bool,
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

/// Answer of a menu request: the menu, or why it is refused (a typed value, not an error, so
/// the badge handles both without exception paths).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum MenuAnswer {
    Menu(DestinationMenu),
    Refused(super::session::SelectionRefusal),
}
/// Answer of a badge choice or removal: the updated session, or why it is refused unchanged.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum ChoiceAnswer {
    Chosen(DestinationSnapshot),
    Refused(super::session::SelectionRefusal),
}

/// Why a star toggle was not saved (012): the favorites are full, or the settings service
/// refused or failed the write. The favorites and the session are unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum FavoriteNotSaved {
    Full,
    Failed,
}

/// Answer of a star toggle (012): the re-ordered menu (same tokens), the session refusal
/// (stale, frozen, unknown token), or the settings refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum FavoriteAnswer {
    Menu(DestinationMenu),
    Refused(super::session::SelectionRefusal),
    NotSaved(FavoriteNotSaved),
}

/// A badge choice: an entry token of the current menu, or "Frontmost app at the end" (the cross).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", content = "token", rename_all = "snake_case")]
pub enum MenuChoice {
    Application(u64),
    FrontmostAtEnd,
}
