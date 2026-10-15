//! 010's presentation hand-off seen from 006 (plan "Presentation hand-off"): the dictation task
//! waits for the recovery bubble's answer without blocking a thread, at most
//! `PRESENTATION_GUARD`, then decides whether 006's `paste-error` toast reports the failure.
//! A failure is reported exactly once: by the bubble, or by the toast.
use super::recovery::{PresentationReceipt, PresentationResult, PresentationWait};

/// Longest wait of a dictation task for 010's bubble presentation before ending the
/// indicator; its expiry never cancels the bubble.
pub const PRESENTATION_GUARD: std::time::Duration = std::time::Duration::from_secs(2);

/// What became of the reason presentation of one outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasonShown {
    /// No consumer took the reason: the toast reports the failure.
    NoConsumer,
    /// 010's bubble shows it (or a new dictation closed it): no toast.
    Bubble,
    /// The bubble cannot be presented at all: the toast is the only fallback.
    Unpresentable,
    /// Still pending after `PRESENTATION_GUARD`: 010 reports a later confirmed failure itself.
    Pending,
}
// @spec FR-002: The bubble replaces paste-error, never both — .specs/features/010-recovery-bubble/spec.md#fr-002
/// Wait for 010's answer without blocking a thread, at most `PRESENTATION_GUARD`.
pub(super) async fn await_presentation(receipt: Option<PresentationReceipt>) -> ReasonShown {
    let Some(receipt) = receipt else {
        return ReasonShown::NoConsumer;
    };
    match receipt.wait(PRESENTATION_GUARD).await {
        PresentationWait::Resolved(
            PresentationResult::Presented | PresentationResult::Superseded,
        ) => ReasonShown::Bubble,
        PresentationWait::Resolved(PresentationResult::Unpresentable)
        | PresentationWait::Abandoned => ReasonShown::Unpresentable,
        PresentationWait::GuardExpired => ReasonShown::Pending,
    }
}
/// Business rule (010 spec AC-001, plan "Presentation hand-off"): a failure is reported once,
/// by the bubble, or by `paste-error` only when no consumer took it or the bubble cannot be
/// presented at all; a presentation still pending at the guard reports later itself.
pub(super) fn toast_needed(shown: ReasonShown) -> bool {
    matches!(shown, ReasonShown::NoConsumer | ReasonShown::Unpresentable)
}

#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;
