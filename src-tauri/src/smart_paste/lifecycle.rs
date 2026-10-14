//! Pure read lifecycle of one clipboard paste transaction (AC-008, AC-009).
//!
//! The lifecycle only consumes timestamps: the key-down instant recorded
//! immediately before the first key event is posted, and the instants at
//! which the on-demand item served its plain-text type. It never touches the
//! pasteboard; the coordinator applies its decisions.
//!
//! Rules encoded here:
//! - reads before key-down are served but never count;
//! - the first qualifying read is latched once and makes the outcome
//!   "read observed";
//! - restoration becomes due `quiet` after the last qualifying read, capped
//!   at `cap` after the first one;
//! - without any qualifying read at `key_down + no_read`, the outcome is
//!   "not read" and nothing is ever restored for this transaction;
//! - a read stamped exactly at a deadline is processed before that timer.

use std::time::Instant;

use super::timing::ClipboardTimings;

// @spec FR-005: Item-scoped read events — .specs/features/007-smart-paste/spec.md#fr-005
// @spec FR-006: Guarded restoration timing — .specs/features/007-smart-paste/spec.md#fr-006

/// Phase of the read lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadPhase {
    /// The item is published; the paste keystroke is not posted yet.
    Published,
    /// Key-down recorded; waiting for the first qualifying read.
    AwaitingRead {
        /// Instant recorded just before the first key-down was posted.
        key_down_at: Instant,
    },
    /// At least one qualifying read was served.
    ReadObserved {
        /// Instant recorded just before the first key-down was posted.
        key_down_at: Instant,
        /// First qualifying read, latched once.
        first_read_at: Instant,
        /// Latest qualifying read.
        last_read_at: Instant,
    },
    /// The no-read deadline passed without a qualifying read.
    NotRead,
    /// The clipboard was settled; later reads have no effect.
    Settled,
}

/// How one served read was interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadClassification {
    /// Served before the keystroke (clipboard manager); does not count.
    BeforeKeyDown,
    /// First read after the keystroke: the outcome is "read observed".
    FirstQualifying,
    /// Another read after the keystroke: restarts the quiet period.
    LaterQualifying,
    /// Read after "not read" was declared; diagnostic only.
    LateAfterNotRead {
        /// True only for the first such read, so the diagnostic is emitted once.
        first_late_read: bool,
    },
    /// Read after settlement; ignored.
    AfterSettlement,
}

/// Result of recording one read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadUpdate {
    /// The no-read deadline had already passed when this read was recorded,
    /// so "not read" was declared first.
    pub declared_not_read: bool,
    /// Interpretation of the read itself.
    pub classification: ReadClassification,
}

/// Decision produced when a deadline is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineOutcome {
    /// No deadline reached.
    Pending,
    /// "Not read" was declared now; the snapshot must be discarded.
    DeclaredNotRead,
    /// Reads went quiet (or the cap passed); the clipboard must be settled.
    SettlementDue,
}

/// Read lifecycle of one published on-demand item.
#[derive(Debug, Clone)]
pub struct ReadLifecycle {
    timings: ClipboardTimings,
    phase: ReadPhase,
    late_read_seen: bool,
}

impl ReadLifecycle {
    /// Creates the lifecycle of a freshly published item.
    pub fn new(timings: ClipboardTimings) -> Self {
        Self {
            timings,
            phase: ReadPhase::Published,
            late_read_seen: false,
        }
    }

    /// Returns the current phase.
    pub fn phase(&self) -> ReadPhase {
        self.phase
    }

    /// Returns the timings used by this lifecycle.
    pub fn timings(&self) -> ClipboardTimings {
        self.timings
    }

    /// Records the instant taken immediately before the first key-down is
    /// posted. Only the first call has an effect.
    pub fn record_key_down(&mut self, key_down_at: Instant) {
        if self.phase == ReadPhase::Published {
            self.phase = ReadPhase::AwaitingRead { key_down_at };
        }
    }

    /// Records one served plain-text read stamped `at`.
    ///
    /// A read stamped exactly at the no-read deadline still qualifies; a read
    /// stamped after it means the deadline should have fired first, so "not
    /// read" is declared before the read is classified as late.
    pub fn record_read(&mut self, at: Instant) -> ReadUpdate {
        let is_past_no_read_deadline = matches!(self.phase, ReadPhase::AwaitingRead { .. })
            && self.next_deadline().is_some_and(|deadline| at > deadline);
        if is_past_no_read_deadline {
            self.phase = ReadPhase::NotRead;
        }
        ReadUpdate {
            declared_not_read: is_past_no_read_deadline,
            classification: self.classify_read(at),
        }
    }

    fn classify_read(&mut self, at: Instant) -> ReadClassification {
        match self.phase {
            ReadPhase::Published => ReadClassification::BeforeKeyDown,
            ReadPhase::AwaitingRead { key_down_at }
            | ReadPhase::ReadObserved { key_down_at, .. }
                if at < key_down_at =>
            {
                ReadClassification::BeforeKeyDown
            }
            ReadPhase::AwaitingRead { key_down_at } => {
                self.phase = ReadPhase::ReadObserved {
                    key_down_at,
                    first_read_at: at,
                    last_read_at: at,
                };
                ReadClassification::FirstQualifying
            }
            ReadPhase::ReadObserved { .. } => self.extend_quiet_period(at),
            ReadPhase::NotRead => {
                let first_late_read = !self.late_read_seen;
                self.late_read_seen = true;
                ReadClassification::LateAfterNotRead { first_late_read }
            }
            ReadPhase::Settled => ReadClassification::AfterSettlement,
        }
    }

    fn extend_quiet_period(&mut self, at: Instant) -> ReadClassification {
        if let ReadPhase::ReadObserved { last_read_at, .. } = &mut self.phase {
            *last_read_at = (*last_read_at).max(at);
        }
        ReadClassification::LaterQualifying
    }

    /// Returns the next instant at which [`Self::poll_deadline`] can change
    /// the lifecycle, if any.
    pub fn next_deadline(&self) -> Option<Instant> {
        match self.phase {
            ReadPhase::AwaitingRead { key_down_at } => {
                Some(key_down_at + self.timings.no_read_decision)
            }
            ReadPhase::ReadObserved {
                first_read_at,
                last_read_at,
                ..
            } => Some(self.settlement_due_at(first_read_at, last_read_at)),
            _ => None,
        }
    }

    fn settlement_due_at(&self, first_read_at: Instant, last_read_at: Instant) -> Instant {
        let quiet = last_read_at + self.timings.quiet_after_last_read;
        let cap = first_read_at + self.timings.restore_cap_after_first_read;
        quiet.min(cap)
    }

    /// Applies any deadline reached at `now`.
    ///
    /// Callers record every read stamped at or before `now` first, so
    /// reaching the no-read deadline here is a definitive "not read".
    pub fn poll_deadline(&mut self, now: Instant) -> DeadlineOutcome {
        match (self.phase, self.next_deadline()) {
            (ReadPhase::AwaitingRead { .. }, Some(deadline)) if now >= deadline => {
                self.phase = ReadPhase::NotRead;
                DeadlineOutcome::DeclaredNotRead
            }
            (ReadPhase::ReadObserved { .. }, Some(due)) if now >= due => {
                DeadlineOutcome::SettlementDue
            }
            _ => DeadlineOutcome::Pending,
        }
    }

    /// Marks the clipboard as settled (restored, materialized or abandoned).
    pub fn mark_settled(&mut self) {
        self.phase = ReadPhase::Settled;
    }

    /// Whether a qualifying read was observed (before settlement).
    pub fn is_read_observed(&self) -> bool {
        matches!(self.phase, ReadPhase::ReadObserved { .. })
    }

    /// Whether restoration is due at `now` (read observed and quiet or cap
    /// elapsed). Used on normal exit, which must never bypass the quiet period.
    pub fn is_restoration_due(&self, now: Instant) -> bool {
        match self.phase {
            ReadPhase::ReadObserved {
                first_read_at,
                last_read_at,
                ..
            } => now >= self.settlement_due_at(first_read_at, last_read_at),
            _ => false,
        }
    }
}
