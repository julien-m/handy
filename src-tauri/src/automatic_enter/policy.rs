//! Pure Enter eligibility of 009 (AC-005): from the frozen policy, the route that 006 dispatched
//! and the way the insertion method completed, decide whether one Enter press may be attempted.
//! The claim, the focus check and the key itself are the transaction's (`transaction.rs`).
use crate::destination::context::DestinationSource;
use crate::destination::outcome::MethodOutcome;
use crate::destination::session::EnterRefusal;
use crate::settings::AutomaticEnterPolicy;

/// How the insertion method of one dictation completed, as the Enter authority sees it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    /// A legacy method returned (classic paste, direct typing, external script, the reliable
    /// paste's classic fallback): its own success or failure.
    Classic { succeeded: bool },
    /// 007's terminal smart-paste outcome.
    SmartPaste(MethodOutcome),
    /// Reliable paste (macOS beta, Windows): the chord succeeded and a qualifying target read was
    /// latched (the rendezvous of `rendezvous.rs`).
    ReliableConfirmed,
    /// Paste method "none": nothing was inserted.
    NoInsertion,
}

/// Why no Enter press was attempted for a dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suppression {
    /// Automatic Enter is Never.
    PolicyNever,
    /// Only in PIN mode, and the destination did not come from the pin.
    NotPinned,
    /// Paste method "none".
    PasteMethodNone,
    /// No method dispatch happened (guard refusal, destination failure, cancellation).
    NotDispatched,
    /// The legacy method reported a failure.
    MethodFailed,
    /// A smart-paste outcome other than inserted-verified, read-observed or typed.
    SmartPasteOutcome(MethodOutcome),
    /// 006 refused the claim (next dictation started, recovery, cancellation, second claim).
    Claim(EnterRefusal),
    /// The destination was no longer frontmost immediately before the key; never retried.
    FocusLost,
    /// Another completion already decided this dictation's Enter.
    AlreadyDecided,
}

/// What the eligibility needs: the frozen policy, the source of the dispatched route (`None`
/// when the method never passed 006's guard) and the completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnterFacts {
    pub policy: AutomaticEnterPolicy,
    pub source: Option<DestinationSource>,
    pub completion: Completion,
}

// @spec FR-004: At most one Enter, gated by outcome and option — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-004
/// Eligibility of `facts` (AC-005 conjunction before the claim and the focus check).
///
/// # Errors
/// The first reason the Enter press is suppressed, in the order of the spec's Story 4 flow:
/// paste method none, no dispatch, method failure, then the option and the pin source.
pub fn evaluate_enter(facts: EnterFacts) -> Result<(), Suppression> {
    completion_succeeded(facts.completion)?;
    let source = facts.source.ok_or(Suppression::NotDispatched)?;
    match facts.policy {
        AutomaticEnterPolicy::Never => Err(Suppression::PolicyNever),
        AutomaticEnterPolicy::Always => Ok(()),
        AutomaticEnterPolicy::PinOnly if source == DestinationSource::Pinned => Ok(()),
        AutomaticEnterPolicy::PinOnly => Err(Suppression::NotPinned),
    }
}

/// Business rule (AC-005, 007 AC-012): only a legacy success, 007's inserted-verified,
/// read-observed and typed results, and a confirmed reliable read are successful insertions.
fn completion_succeeded(completion: Completion) -> Result<(), Suppression> {
    use MethodOutcome::{InsertedVerified, ReadObserved, Typed};
    match completion {
        Completion::NoInsertion => Err(Suppression::PasteMethodNone),
        Completion::Classic { succeeded: false } => Err(Suppression::MethodFailed),
        Completion::Classic { succeeded: true } | Completion::ReliableConfirmed => Ok(()),
        Completion::SmartPaste(InsertedVerified | ReadObserved | Typed) => Ok(()),
        Completion::SmartPaste(other) => Err(Suppression::SmartPasteOutcome(other)),
    }
}

/// True when the route activated a destination (captured, badge or pinned): its Enter requires
/// that destination still frontmost; the frontmost-at-the-end route has none (AC-005).
pub fn route_has_destination(source: DestinationSource) -> bool {
    source != DestinationSource::FrontmostAtEnd
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
