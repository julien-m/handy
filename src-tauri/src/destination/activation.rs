//! Activation chain of one insertion (N16): the ordered ways of bringing the resolved destination
//! forward, all inside the single 1 s activation deadline (AC-002). The workspace request comes
//! first; the accessibility request follows at once when the system declined the first one, or
//! after `ESCALATION_DELAY` when it was accepted without bringing the application in front.
//! macOS 14 and later (cooperative activation) can decline the workspace request of a background
//! accessory app such as Handy after the user switched applications; the accessibility request
//! is the second route. Every step targets the same resolved instance: the chain never launches,
//! never targets another application, and the pre-dispatch guard still decides the insertion.
use super::workspace::ActivationRequest;
use std::time::Duration;

/// One way of asking the system to bring the destination forward.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationStrategy {
    /// The platform workspace request (macOS `NSRunningApplication`, Windows foreground window).
    Workspace,
    /// The accessibility request (macOS `AXFrontmost` on the application element).
    Accessibility,
}

/// The strategies in the order they are tried.
pub const ACTIVATION_STRATEGIES: [ActivationStrategy; 2] = [
    ActivationStrategy::Workspace,
    ActivationStrategy::Accessibility,
];

// Wait after an accepted request before trying the next strategy. In the N16 diagnostic an
// accepted request put the application in front within about 10 ms and the live
// accessibility read followed within 275 ms; 300 ms leaves 700 ms of the 1 s budget to the
// next strategy.
pub const ESCALATION_DELAY: Duration = Duration::from_millis(300);

/// What the activation wait does next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChainStep {
    /// Send this request now.
    Request(ActivationStrategy),
    /// Keep sampling the foreground.
    Wait,
    /// Every strategy was declined: waiting cannot bring the destination forward.
    Exhausted,
}

/// One request sent by the chain and the system's answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationAttempt {
    pub strategy: ActivationStrategy,
    pub request: ActivationRequest,
    /// Time since the start of the activation when the request was sent.
    pub at: Duration,
}

/// Progress of the chain for one insertion.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActivationChain {
    attempts: Vec<ActivationAttempt>,
}

impl ActivationChain {
    /// The next step `elapsed` after the start of the activation.
    pub fn step(&self, elapsed: Duration) -> ChainStep {
        match ACTIVATION_STRATEGIES.get(self.attempts.len()) {
            Some(strategy) if self.is_due(elapsed) => ChainStep::Request(*strategy),
            Some(_) => ChainStep::Wait,
            None if self.any_sent() => ChainStep::Wait,
            None => ChainStep::Exhausted,
        }
    }
    /// Record the answer to the request of `strategy` sent `at` after the start.
    pub fn record(
        &mut self,
        strategy: ActivationStrategy,
        request: ActivationRequest,
        at: Duration,
    ) {
        self.attempts.push(ActivationAttempt {
            strategy,
            request,
            at,
        });
    }
    /// The requests sent so far, in order.
    pub fn attempts(&self) -> &[ActivationAttempt] {
        &self.attempts
    }
    /// Business rule: the next strategy is due at once after a declined request (or before any
    /// request), and `ESCALATION_DELAY` after the last accepted one.
    fn is_due(&self, elapsed: Duration) -> bool {
        match self.last_sent_at() {
            None => true,
            Some(at) => elapsed.saturating_sub(at) >= ESCALATION_DELAY,
        }
    }
    fn last_sent_at(&self) -> Option<Duration> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| attempt.request == ActivationRequest::Sent)
            .map(|attempt| attempt.at)
    }
    fn any_sent(&self) -> bool {
        self.last_sent_at().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ActivationRequest::{Refused, Sent};
    use ActivationStrategy::{Accessibility, Workspace};

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn the_workspace_request_comes_first() {
        assert_eq!(
            ActivationChain::default().step(ms(0)),
            ChainStep::Request(Workspace)
        );
    }
    #[test]
    fn a_declined_workspace_request_falls_back_to_accessibility_at_once() {
        let mut chain = ActivationChain::default();
        chain.record(Workspace, Refused, ms(0));
        assert_eq!(chain.step(ms(0)), ChainStep::Request(Accessibility));
    }
    #[test]
    fn an_accepted_request_is_given_the_escalation_delay_before_the_fallback() {
        let mut chain = ActivationChain::default();
        chain.record(Workspace, Sent, ms(0));
        assert_eq!(chain.step(ms(0)), ChainStep::Wait);
        assert_eq!(chain.step(ms(299)), ChainStep::Wait);
        assert_eq!(chain.step(ms(300)), ChainStep::Request(Accessibility));
    }
    #[test]
    fn after_the_last_strategy_the_chain_waits_while_one_request_was_accepted() {
        let mut chain = ActivationChain::default();
        chain.record(Workspace, Sent, ms(0));
        chain.record(Accessibility, Refused, ms(300));
        assert_eq!(chain.step(ms(900)), ChainStep::Wait);
        let mut chain = ActivationChain::default();
        chain.record(Workspace, Refused, ms(0));
        chain.record(Accessibility, Sent, ms(0));
        assert_eq!(chain.step(ms(10)), ChainStep::Wait);
    }
    #[test]
    fn every_strategy_declined_exhausts_the_chain() {
        let mut chain = ActivationChain::default();
        chain.record(Workspace, Refused, ms(0));
        chain.record(Accessibility, Refused, ms(1));
        assert_eq!(chain.step(ms(1)), ChainStep::Exhausted);
        assert_eq!(chain.attempts().len(), ACTIVATION_STRATEGIES.len());
    }
}
