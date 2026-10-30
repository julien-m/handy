//! Recording-boundary decisions of the audio manager (feature 001), pure so every combination
//! is tested: a committed priority change or a lost stream never touches a recording in
//! progress; it is applied when the recording ends, by one bounded reopen at most.

/// Stream facts observed under the manager's state lock.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StreamFacts {
    /// A recording is in progress (recording or stopping).
    pub recording: bool,
    /// The always-on microphone mode is active.
    pub always_on: bool,
    /// A stream is open.
    pub open: bool,
    /// The open stream reported a loss and must be rebuilt.
    pub lost: bool,
}

/// What to do with the stream after a priority change or a loss.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceChange {
    /// A recording owns the stream: wait for its end.
    Defer,
    /// On demand: close a warm stream; the next recording resolves again.
    CloseWarm,
    /// Always-on: close the stream and open the resolved input.
    Reopen,
    /// Always-on with no stream: open the resolved input.
    Open,
    /// Always-on and the open stream is still the preferred input.
    Keep,
}

// @spec FR-004: Changes applied at the next stream opening — .specs/features/001-microphone-priority/spec.md#fr-004
/// Decision for a priority change or a queued reopen; `preferred` (a fresh catalog check) is
/// asked only when an always-on stream is healthy.
pub fn device_change(facts: StreamFacts, preferred: impl FnOnce() -> bool) -> DeviceChange {
    match facts {
        StreamFacts {
            recording: true, ..
        } => DeviceChange::Defer,
        StreamFacts {
            always_on: false, ..
        } => DeviceChange::CloseWarm,
        StreamFacts { lost: true, .. } => DeviceChange::Reopen,
        StreamFacts { open: false, .. } => DeviceChange::Open,
        _ if preferred() => DeviceChange::Keep,
        _ => DeviceChange::Reopen,
    }
}

/// Whether the end of a recording queues a reopen: in always-on mode, for a priority change
/// that waited for it or a stream lost during it (review r3, warning 2).
pub fn reopen_after_recording(pending_change: bool, lost: bool, always_on: bool) -> bool {
    always_on && (pending_change || lost)
}

/// What a stream loss does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LossAction {
    /// End the recording through the normal stop (its prefix is transcribed) and tell the user.
    EndRecording,
    /// Reopen the idle always-on stream once.
    Recover,
    /// Nothing now: the next recording or user action resolves again.
    Ignore,
}

/// Business rule: a loss during a recording ends it; an idle always-on stream is recovered once
/// per user action (`armed`), so a device failing again right after opening cannot loop.
pub fn on_loss(recording: bool, always_on: bool, armed: bool) -> LossAction {
    match (recording, always_on && armed) {
        (true, _) => LossAction::EndRecording,
        (false, true) => LossAction::Recover,
        (false, false) => LossAction::Ignore,
    }
}
