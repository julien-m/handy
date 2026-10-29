//! Recovery bubble vocabulary (010): the bubble identity, why it appears, what its explanation
//! adds, the platform's actions and the projection the recovery window renders. The projection
//! carries a bounded preview of the transcript, never the full text: that stays in the native
//! `RecoveryManager` for Copy and drag.
use crate::destination::context::DictationId;
use crate::destination::outcome::HistoryStatus;
use serde::{Deserialize, Serialize};

/// Identity of one bubble; a closed bubble's identity is never reused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(transparent)]
pub struct BubbleId(pub u64);

/// Why the bubble appears (spec AC-003 reasons). Distinct from 006's `context::RecoveryReason`,
/// which is only the reason of a pre-insertion verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum BubbleReason {
    /// "<App> is no longer open".
    AppClosed,
    /// "<App> is pinned but not open".
    PinnedAppNotRunning,
    /// "Could not bring <App> forward".
    ActivationFailed,
    /// "Insertion failed".
    InsertionFailed,
    /// "No text field is focused".
    NoTextField,
}

/// What the explanation adds about input that may have been sent (spec AC-001).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReasonDetail {
    /// Decided before insertion: nothing was dispatched.
    NotInserted,
    /// "Some text may have reached the application" (a method that may have sent input).
    MayHaveReached,
    /// Smart paste `permission_missing`: nothing was sent to the application.
    NothingSent,
}

/// Whether the explanation may say the transcript is in History (saved only).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum HistoryNote {
    Kept,
    NotKept,
}
impl From<HistoryStatus> for HistoryNote {
    fn from(status: HistoryStatus) -> Self {
        match status {
            HistoryStatus::Saved => Self::Kept,
            HistoryStatus::NotSaved | HistoryStatus::Failed => Self::NotKept,
        }
    }
}

/// Platform of the recovery policy: a value, so every platform row is testable anywhere.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryPlatform {
    MacOs,
    Windows,
    Linux,
}
impl RecoveryPlatform {
    /// The platform of this build.
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
    /// Native drag exists on macOS only (spec AC-006).
    pub const fn offers_drag(self) -> bool {
        matches!(self, Self::MacOs)
    }
}

/// What the bubble says and offers, decided once per outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BubbleSpec {
    pub reason: BubbleReason,
    pub detail: ReasonDetail,
}

/// Lifecycle of one bubble (plan "State Model").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// Admitted with its full transcript; the surface has not reported its first render yet.
    Presenting,
    Visible,
    /// macOS native drag in progress: the bubble is hidden.
    Dragging,
    /// A deliberate Copy is being written.
    Copying,
}

// @spec AC-004: Escape cancels and dismisses the bubble — .specs/features/010-recovery-bubble/spec.md#ac-004
/// How a native drag session of the bubble ended (spec AC-004, native session N25).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DragResult {
    /// A target accepted the drop: the bubble closes (`drop accepted`).
    Dropped,
    /// Released where no target took the text: the bubble reappears for another attempt.
    Released,
    /// The user pressed Escape and nothing took the text: the bubble closes like Close.
    CancelledByEscape,
}
impl DragResult {
    /// The result of a session that `accepted` a drop or not, with Escape pressed or not
    /// during it. A drop a target accepted is never undone, so it wins over a late Escape;
    /// Escape only turns a session that inserted nothing into a dismissal.
    pub const fn of_session(accepted: bool, escape_pressed: bool) -> Self {
        match (accepted, escape_pressed) {
            (true, _) => Self::Dropped,
            (false, true) => Self::CancelledByEscape,
            (false, false) => Self::Released,
        }
    }
}

/// The identity a late callback must match to act on a bubble.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BubbleKey {
    pub bubble_id: BubbleId,
    pub dictation_id: DictationId,
    pub destination_generation: u64,
}

/// Targeted projection rendered by the recovery window (event `recovery-bubble-changed` and
/// command `recovery_bubble_view`). No full transcript, no window title.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct RecoveryBubbleView {
    pub bubble_id: BubbleId,
    pub destination_generation: u64,
    pub reason: BubbleReason,
    /// Destination name, `None` when no application can be named (localized noun then).
    pub app_name: Option<String>,
    /// Beginning of the transcript, at most `PREVIEW_MAX_CHARS` characters; the view shows
    /// its first two lines with an ellipsis.
    pub preview: String,
    /// True when the transcript is longer than `preview`.
    pub truncated: bool,
    pub detail: ReasonDetail,
    /// 007 reported a read after "not read": the text may have been pasted after all.
    pub late_read: bool,
    pub history: HistoryNote,
    /// The macOS drag handle is offered.
    pub can_drag: bool,
}

/// Longest preview handed to the view: far more than two lines of the bubble, far less than a
/// long dictation, so a long transcript never travels to the webview.
pub const PREVIEW_MAX_CHARS: usize = 240;

/// The preview of `transcript` and whether it was cut.
pub fn preview_of(transcript: &str) -> (String, bool) {
    let mut characters = transcript.chars();
    let preview: String = characters.by_ref().take(PREVIEW_MAX_CHARS).collect();
    (preview, characters.next().is_some())
}
