//! Keeps one snapshot helper started ahead of the next capture, so the
//! helper's process startup is not paid inside the 200 ms capture budget
//! (a loaded machine was measured at up to 180 ms to start it).
//!
//! The armed helper never reads the pasteboard before its trigger. It exits
//! on its own after five idle minutes, and is killed and reaped when
//! replaced or dropped; a stale or dead one is replaced by a fresh helper
//! whose startup then counts inside the deadline, as before.

use std::time::{Duration, Instant};

use super::clock::Clock;
use super::macos_snapshot_stream::HELPER_IDLE_TIMEOUT;
use super::snapshot::{PasteboardSnapshot, SnapshotBudget, SnapshotError};
use super::snapshot_helper::{capture_with_helper, ArmedHelper, HelperCommand};

// @spec FR-005: Bounded snapshot within 200 ms — .specs/features/007-smart-paste/spec.md#fr-005

/// An armed helper older than this is replaced rather than triggered, so the
/// trigger never races the helper's own idle exit.
const MAX_ARMED_AGE: Duration = Duration::from_secs(240);
const _: () = assert!(MAX_ARMED_AGE.as_secs() < HELPER_IDLE_TIMEOUT.as_secs());

/// At most one helper waiting for the next capture.
pub struct WarmHelper {
    command: HelperCommand,
    armed: Option<(ArmedHelper, Instant)>,
}

impl WarmHelper {
    /// Creates the holder; no process is started until [`Self::prepare`].
    pub fn new(command: HelperCommand) -> Self {
        Self {
            command,
            armed: None,
        }
    }

    /// Starts a helper for `budget` unless a usable one is already waiting.
    /// A spawn failure is left for the capture to report.
    pub fn prepare(&mut self, budget: SnapshotBudget, clock: &dyn Clock) {
        if let Some(armed) = self.take_usable(budget, clock) {
            self.armed = Some(armed);
            return;
        }
        self.armed = ArmedHelper::arm(&self.command, budget)
            .ok()
            .map(|helper| (helper, clock.now()));
    }

    /// Captures through the waiting helper, or a fresh one when none is
    /// usable or the trigger cannot reach it, then arms the next helper.
    ///
    /// # Errors
    /// See [`SnapshotError`]; on any error the caller must not publish.
    pub fn capture(
        &mut self,
        budget: SnapshotBudget,
        clock: &dyn Clock,
    ) -> Result<PasteboardSnapshot, SnapshotError> {
        let captured = match self.take_usable(budget, clock) {
            Some((helper, _)) => match helper.capture(clock) {
                // The helper exited before its trigger: start a fresh one.
                Err(SnapshotError::Spawn) => capture_with_helper(&self.command, budget, clock),
                other => other,
            },
            None => capture_with_helper(&self.command, budget, clock),
        };
        self.prepare(budget, clock);
        captured
    }

    /// Takes the waiting helper if it is alive, young enough and started
    /// with `budget`; any other one is dropped (killed and reaped).
    fn take_usable(
        &mut self,
        budget: SnapshotBudget,
        clock: &dyn Clock,
    ) -> Option<(ArmedHelper, Instant)> {
        let (mut helper, armed_at) = self.armed.take()?;
        let is_young = clock.now().saturating_duration_since(armed_at) < MAX_ARMED_AGE;
        (is_young && helper.budget() == budget && helper.is_waiting()).then_some((helper, armed_at))
    }
}
