//! Configured choices that failed to open in the resolution of the current stream (feature 001,
//! AC-004). A choice that is present but cannot be opened (busy, broken driver, Bluetooth still
//! switching profile) stays preferred by the presence-only check (`policy::preferred`), so each
//! always-on recording start would close the healthy fallback, retry the failing choice for up
//! to 2 s and reopen the same fallback before capture. Remembering the failure for a short grace
//! keeps the fallback instead; a priority commit forgets it at once.
use super::policy::{Attempt, AttemptResult};
use std::time::{Duration, Instant};

/// How long a failed open keeps the current stream. Long enough to cover a burst of dictations
/// right after the failure, short enough that a device that recovers (a Bluetooth profile switch
/// ends in a few seconds) is tried again at a recording start within half a minute.
pub const FAILED_OPEN_GRACE: Duration = Duration::from_secs(30);

/// Choices whose open failed or timed out when the current stream was opened, and when.
#[derive(Debug, Default)]
pub struct FailedOpens {
    names: Vec<String>,
    at: Option<Instant>,
}
impl FailedOpens {
    /// Remember the configured choices whose open failed or timed out in `attempts`, the
    /// resolution that opened the current stream at `now` (replaces the previous record). A
    /// missing, ambiguous or skipped choice made no open call and is not remembered.
    pub fn record(&mut self, attempts: &[Attempt], now: Instant) {
        self.names = attempts
            .iter()
            .filter(|attempt| matches!(attempt.result, AttemptResult::Open(_)))
            .filter_map(|attempt| attempt.choice.clone())
            .collect();
        self.at = Some(now);
    }
    /// Forget every failure (a priority commit resolves again from scratch).
    pub fn forget(&mut self) {
        *self = Self::default();
    }
    /// Whether `choice` failed to open for the current stream less than the grace before `now`.
    pub fn recently_failed(&self, choice: &str, now: Instant) -> bool {
        let recent = self
            .at
            .is_some_and(|at| now.saturating_duration_since(at) < FAILED_OPEN_GRACE);
        recent && self.names.iter().any(|name| name == choice)
    }
}

// @spec FR-004: A recently unopenable choice keeps the fallback — .specs/features/001-microphone-priority/spec.md#fr-004
/// Business rule: the open stream (`current`, its configured choice or `None` for the system
/// default) is kept when it is what the priority prefers now, or when the preferred choice
/// failed to open for it within the grace, since reopening would most likely fail the same way.
pub fn keeps_stream(
    current: Option<Option<&str>>,
    preferred: Option<&str>,
    failed: &FailedOpens,
    now: Instant,
) -> bool {
    current == Some(preferred) || preferred.is_some_and(|name| failed.recently_failed(name, now))
}

#[cfg(test)]
#[path = "failed_opens_tests.rs"]
mod tests;
