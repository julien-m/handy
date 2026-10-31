//! Quick Add window (018): a small settings window opened from the tray menu to add one
//! replacement rule (002) or one vocabulary term (003) without the settings window. There is a
//! single window; reopening it on the other kind switches its tab instead of creating a second.
use crate::main_window_placement::center_on_primary_monitor;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Label of the Quick Add window.
pub const QUICK_ADD_WINDOW: &str = "quick_add";
/// Page of the Quick Add window.
const QUICK_ADD_PAGE: &str = "src/overlay/quick-add.html";
/// Event switching an open window to another kind.
pub const QUICK_ADD_KIND_EVENT: &str = "quick-add-kind";
/// Event asking the settings window to show the list of one kind (Advanced section).
pub const SETTINGS_REVEAL_EVENT: &str = "settings-reveal";
/// Logical size of the window.
const SIZE: (f64, f64) = (440.0, 300.0);

/// What the window adds: its tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum QuickAddKind {
    Replacement,
    Vocabulary,
}

impl QuickAddKind {
    /// Tray menu item opening the window on this kind.
    pub fn from_menu_id(id: &str) -> Option<Self> {
        match id {
            "quick_add_replacement" => Some(Self::Replacement),
            "quick_add_vocabulary" => Some(Self::Vocabulary),
            _ => None,
        }
    }

    fn query(self) -> &'static str {
        match self {
            Self::Replacement => "replacement",
            Self::Vocabulary => "vocabulary",
        }
    }
}

// @spec FR-002: Single Quick Add window — .specs/features/018-quick-add-from-tray/spec.md#fr-002
// @spec AC-002: Opens centered, focused, on the kind — .specs/features/018-quick-add-from-tray/spec.md#ac-002
// @spec AC-003: Reused, never a second window — .specs/features/018-quick-add-from-tray/spec.md#ac-003
/// Show the Quick Add window on `kind`, creating it when absent; must run on the main thread
/// (tray menu events do).
pub fn open(app: &AppHandle, kind: QuickAddKind) {
    if let Some(window) = app.get_webview_window(QUICK_ADD_WINDOW) {
        if let Err(error) = window.emit(QUICK_ADD_KIND_EVENT, kind) {
            log::error!("Quick Add: failed to switch the window to {kind:?}: {error}");
        }
        bring_forward(&window);
        return;
    }
    let url = format!("{QUICK_ADD_PAGE}?kind={}", kind.query());
    let (width, height) = SIZE;
    let built = WebviewWindowBuilder::new(app, QUICK_ADD_WINDOW, WebviewUrl::App(url.into()))
        .title("Handy")
        .inner_size(width, height)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .always_on_top(true)
        .visible(false)
        .build();
    match built {
        Ok(window) => {
            center_on_primary_monitor(&window);
            bring_forward(&window);
        }
        Err(error) => log::error!("Quick Add: failed to create the window: {error}"),
    }
}

// @spec FR-006: Show the list in the settings — .specs/features/018-quick-add-from-tray/spec.md#fr-006
// @spec AC-009: Settings opened on the list — .specs/features/018-quick-add-from-tray/spec.md#ac-009
/// Show the settings window and ask it to scroll to the list of `kind` in the Advanced section.
pub fn reveal_in_settings(app: &AppHandle, kind: QuickAddKind) {
    crate::show_main_window(app);
    if let Err(error) = app.emit_to("main", SETTINGS_REVEAL_EVENT, kind) {
        log::error!("Quick Add: failed to reveal the {kind:?} list in the settings: {error}");
    }
}

/// Open the window for the tray menu item `id`; other ids are ignored.
pub fn open_from_menu(app: &AppHandle, id: &str) {
    if let Some(kind) = QuickAddKind::from_menu_id(id) {
        open(app, kind);
    }
}

fn bring_forward(window: &tauri::WebviewWindow) {
    if let Err(error) = window.show() {
        log::error!("Quick Add: failed to show the window: {error}");
    }
    if let Err(error) = window.set_focus() {
        log::error!("Quick Add: failed to focus the window: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_two_quick_add_items_open_the_window() {
        assert_eq!(
            QuickAddKind::from_menu_id("quick_add_replacement"),
            Some(QuickAddKind::Replacement)
        );
        assert_eq!(
            QuickAddKind::from_menu_id("quick_add_vocabulary"),
            Some(QuickAddKind::Vocabulary)
        );
        assert_eq!(QuickAddKind::from_menu_id("settings"), None);
    }

    #[test]
    fn the_kind_reaches_the_page_as_its_serialized_name() -> Result<(), serde_json::Error> {
        for kind in [QuickAddKind::Replacement, QuickAddKind::Vocabulary] {
            assert_eq!(serde_json::to_value(kind)?, kind.query());
        }
        Ok(())
    }
}
