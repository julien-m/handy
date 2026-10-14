//! Insertion strategy chosen from the destination bundle identifier (AC-010).

use super::timing::ClipboardTimings;

// @spec FR-007: Bundle strategy dispatch — .specs/features/007-smart-paste/spec.md#fr-007

/// iPhone Mirroring: receives Unicode key events, never the clipboard.
pub const IPHONE_MIRRORING_BUNDLE_ID: &str = "com.apple.ScreenContinuity";
/// Windows App (Microsoft Remote Desktop): receives Unicode key events.
pub const WINDOWS_APP_BUNDLE_ID: &str = "com.microsoft.rdc.macos";
/// Screen Sharing: receives the transcript on the clipboard, no keystroke.
pub const SCREEN_SHARING_BUNDLE_ID: &str = "com.apple.ScreenSharing";

/// How smart paste inserts into a destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionStrategy {
    /// Unicode key events without the clipboard.
    UnicodeTyping,
    /// Eager clipboard write for the remote side, without a keystroke.
    ClipboardHandOver,
    /// Verified accessibility insertion, then the clipboard paste with the
    /// destination's timings.
    DirectThenClipboard(ClipboardTimings),
}

impl InsertionStrategy {
    /// Selects the strategy for a destination bundle identifier; unknown or
    /// missing identifiers use the general path.
    pub fn for_bundle(bundle_id: Option<&str>) -> Self {
        match bundle_id {
            Some(IPHONE_MIRRORING_BUNDLE_ID | WINDOWS_APP_BUNDLE_ID) => Self::UnicodeTyping,
            Some(SCREEN_SHARING_BUNDLE_ID) => Self::ClipboardHandOver,
            other => Self::DirectThenClipboard(ClipboardTimings::for_bundle(other)),
        }
    }

    /// Whether accessibility permission is required before any effect: key
    /// events (paste chord, Unicode typing) and AX writes need it, the
    /// Screen Sharing hand-over does not.
    pub fn requires_accessibility_permission(self) -> bool {
        !matches!(self, Self::ClipboardHandOver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_paste::timing::JUMP_DESKTOP_BUNDLE_ID;

    #[test]
    fn remote_keyboards_use_unicode_typing() {
        for bundle in [IPHONE_MIRRORING_BUNDLE_ID, WINDOWS_APP_BUNDLE_ID] {
            assert_eq!(
                InsertionStrategy::for_bundle(Some(bundle)),
                InsertionStrategy::UnicodeTyping
            );
        }
    }

    #[test]
    fn screen_sharing_hands_the_clipboard_over_without_permission() {
        let strategy = InsertionStrategy::for_bundle(Some(SCREEN_SHARING_BUNDLE_ID));
        assert_eq!(strategy, InsertionStrategy::ClipboardHandOver);
        assert!(!strategy.requires_accessibility_permission());
    }

    #[test]
    fn jump_desktop_keeps_the_general_path_with_its_timings() {
        assert_eq!(
            InsertionStrategy::for_bundle(Some(JUMP_DESKTOP_BUNDLE_ID)),
            InsertionStrategy::DirectThenClipboard(ClipboardTimings::JUMP_DESKTOP)
        );
    }

    #[test]
    fn other_or_unknown_destinations_use_direct_then_clipboard() {
        for bundle in [Some("com.apple.TextEdit"), None] {
            let strategy = InsertionStrategy::for_bundle(bundle);
            assert_eq!(
                strategy,
                InsertionStrategy::DirectThenClipboard(ClipboardTimings::DEFAULT)
            );
            assert!(strategy.requires_accessibility_permission());
        }
    }
}
