//! Verified direct insertion through `AXSelectedText` (AC-017, AC-018).
//!
//! Strategy: read the focused element's value and selection, compute the
//! exact expected value (selection replaced by the transcript, in UTF-16),
//! write `AXSelectedText` once, then poll the value. Only an exact match is
//! "inserted, verified". An unchanged value at the end of the window falls
//! back to the clipboard paste; any other change is "uncertain" and never
//! falls back, so the text is never inserted twice. Every call is bounded by
//! a per-call messaging timeout and the whole attempt by a total budget.

use std::fmt;
use std::time::{Duration, Instant};

use super::ax_port::{AxError, AxField, AxTextPort, Utf16Range};
use super::clock::Clock;

// @spec FR-011: Verified direct insertion — .specs/features/007-smart-paste/spec.md#fr-011

/// Total time direct insertion may add before the fallback starts (AC-018).
pub const TOTAL_DIRECT_BUDGET: Duration = Duration::from_millis(250);
/// Read-back window after the write (AC-017).
pub const READBACK_WINDOW: Duration = Duration::from_millis(150);
/// Preparation and write allowance; the reviewed plan splits the 250 ms
/// budget into 100 ms of calls and 150 ms of read-back.
pub const PREPARATION_BUDGET: Duration = Duration::from_millis(100);
/// Messaging timeout requested on every AX call (reviewed plan: 20 ms).
pub const PER_CALL_TIMEOUT: Duration = Duration::from_millis(20);
/// Read-back polling interval: about fifteen reads in the window, frequent
/// enough for asynchronous fields without flooding the target with requests.
pub const READBACK_POLL_INTERVAL: Duration = Duration::from_millis(10);
/// A read is only started when at least this much of the window remains, so
/// that a near-zero messaging timeout never turns into a false "uncertain".
pub const MIN_READ_TIMEOUT: Duration = Duration::from_millis(5);
/// Technical memory/time guard on field values (reviewed plan), not a
/// product limit: larger fields skip direct insertion.
pub const MAX_FIELD_UTF16_UNITS: usize = 1_000_000;

/// Why direct insertion was skipped before any write (clipboard fallback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The destination names no process (006 found no application in front),
    /// so the focused field's owner cannot be checked.
    NoTarget,
    /// No focused element could be read.
    FieldUnavailable,
    /// The focused element belongs to another process than the destination.
    PidMismatch,
    /// The element is a password field.
    SecureField,
    /// Role, value or selection could not be read.
    Unreadable,
    /// The value exceeds [`MAX_FIELD_UTF16_UNITS`].
    TooLarge,
    /// The selection is out of bounds or splits a surrogate pair.
    InvalidRange,
    /// Preparation exceeded [`PREPARATION_BUDGET`].
    PreparationTooSlow,
}

/// Why a possibly applied write leads to "uncertain".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncertainReason {
    /// The value changed but differs from the expected value.
    ChangedUnexpectedly,
    /// The value could not be read back after the write.
    ReadbackUnavailable,
}

/// Result of one direct insertion attempt.
#[derive(Debug)]
pub enum DirectAttempt {
    /// The field holds exactly the expected value.
    Verified,
    /// The write may have applied; no fallback is allowed.
    Uncertain(UncertainReason),
    /// Nothing was written; use the clipboard paste.
    Skipped(SkipReason),
    /// The write was issued but the value stayed unchanged; use the
    /// clipboard paste after checking the guard right before each effect.
    Unchanged(FallbackGuard),
}

/// Verdict of the final check performed just before a fallback effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardVerdict {
    /// Still unchanged: the fallback may proceed.
    StillUnchanged,
    /// The delivered write applied late: the text is already there.
    AppliedLate,
    /// The value changed otherwise.
    ChangedUnexpectedly,
    /// The value cannot be read anymore.
    Unreadable,
}

/// Keeps the field and the values needed to detect a late-applied write.
pub struct FallbackGuard {
    field: Box<dyn AxField>,
    before: Vec<u16>,
    expected: Vec<u16>,
}

impl fmt::Debug for FallbackGuard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Field values are user content and must never reach a log.
        formatter.write_str("FallbackGuard(<field values redacted>)")
    }
}

impl FallbackGuard {
    /// Reads the value once, bounded by [`PER_CALL_TIMEOUT`]. This narrows,
    /// but cannot eliminate, the race with a write applied after the check.
    pub fn check(&mut self) -> GuardVerdict {
        match self
            .field
            .value_utf16(PER_CALL_TIMEOUT, MAX_FIELD_UTF16_UNITS)
        {
            Ok(value) if value == self.before => GuardVerdict::StillUnchanged,
            Ok(value) if value == self.expected => GuardVerdict::AppliedLate,
            Ok(_) => GuardVerdict::ChangedUnexpectedly,
            Err(_) => GuardVerdict::Unreadable,
        }
    }
}

/// Replaces `range` of `value` with `insertion`, all in UTF-16 code units.
///
/// Returns `None` when the range is out of bounds or splits a surrogate pair.
pub fn expected_value(value: &[u16], range: Utf16Range, insertion: &[u16]) -> Option<Vec<u16>> {
    let end = range.location.checked_add(range.length)?;
    if end > value.len()
        || !is_code_point_boundary(value, range.location)
        || !is_code_point_boundary(value, end)
    {
        return None;
    }
    let mut expected = Vec::with_capacity(value.len() - range.length + insertion.len());
    expected.extend_from_slice(&value[..range.location]);
    expected.extend_from_slice(insertion);
    expected.extend_from_slice(&value[end..]);
    Some(expected)
}

fn is_code_point_boundary(value: &[u16], index: usize) -> bool {
    let is_high_surrogate = |unit: &u16| (0xD800..=0xDBFF).contains(unit);
    let is_low_surrogate = |unit: &u16| (0xDC00..=0xDFFF).contains(unit);
    let previous = index.checked_sub(1).and_then(|i| value.get(i));
    !(previous.is_some_and(is_high_surrogate) && value.get(index).is_some_and(is_low_surrogate))
}

struct Prepared {
    field: Box<dyn AxField>,
    before: Vec<u16>,
    expected: Vec<u16>,
}

/// Runs one bounded direct insertion attempt into the focused element of
/// the destination process `pid`. The caller has already checked
/// accessibility permission.
pub fn attempt_direct_insertion(
    clock: &dyn Clock,
    ax: &mut dyn AxTextPort,
    pid: i32,
    transcript: &[u16],
) -> DirectAttempt {
    let started = clock.now();
    let mut prepared = match prepare(clock, ax, pid, transcript, started) {
        Ok(prepared) => prepared,
        Err(reason) => return DirectAttempt::Skipped(reason),
    };
    let Some(write_timeout) = preparation_timeout(clock, started) else {
        return DirectAttempt::Skipped(SkipReason::PreparationTooSlow);
    };
    // The setter's own status is deliberately ignored: AC-017 decides only
    // from the value read back, because some fields report failure yet apply
    // the change, and others report success without applying it.
    let _ = prepared.field.set_selected_text(transcript, write_timeout);
    let window_end = (clock.now() + READBACK_WINDOW).min(started + TOTAL_DIRECT_BUDGET);
    read_back(clock, prepared, window_end)
}

fn preparation_timeout(clock: &dyn Clock, started: Instant) -> Option<Duration> {
    let remaining = PREPARATION_BUDGET.checked_sub(clock.now() - started)?;
    let timeout = remaining.min(PER_CALL_TIMEOUT);
    (!timeout.is_zero()).then_some(timeout)
}

fn prepare(
    clock: &dyn Clock,
    ax: &mut dyn AxTextPort,
    pid: i32,
    transcript: &[u16],
    started: Instant,
) -> Result<Prepared, SkipReason> {
    let timeout = || preparation_timeout(clock, started).ok_or(SkipReason::PreparationTooSlow);
    let mut field = ax
        .focused_field(pid, timeout()?)
        .map_err(|_| SkipReason::FieldUnavailable)?;
    check_field_identity(field.as_mut(), pid, timeout()?)?;
    let before = field
        .value_utf16(timeout()?, MAX_FIELD_UTF16_UNITS)
        .map_err(skip_reason_for_value_error)?;
    let range = field
        .selected_range(timeout()?)
        .map_err(|_| SkipReason::Unreadable)?;
    let expected = expected_value(&before, range, transcript).ok_or(SkipReason::InvalidRange)?;
    if expected.len() > MAX_FIELD_UTF16_UNITS {
        return Err(SkipReason::TooLarge);
    }
    Ok(Prepared {
        field,
        before,
        expected,
    })
}

/// Direct insertion only targets a non-secure field of the destination
/// process itself (AC-017); anything else uses the clipboard paste.
fn check_field_identity(
    field: &mut dyn AxField,
    pid: i32,
    timeout: Duration,
) -> Result<(), SkipReason> {
    if field.pid().map_err(|_| SkipReason::Unreadable)? != pid {
        return Err(SkipReason::PidMismatch);
    }
    if field
        .is_secure(timeout)
        .map_err(|_| SkipReason::Unreadable)?
    {
        return Err(SkipReason::SecureField);
    }
    Ok(())
}

fn skip_reason_for_value_error(error: AxError) -> SkipReason {
    match error {
        AxError::TooLarge { .. } => SkipReason::TooLarge,
        _ => SkipReason::Unreadable,
    }
}

/// Last observation of the field during the read-back window.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Observed {
    Unchanged,
    Changed,
    Unreadable,
}

fn read_back(clock: &dyn Clock, mut prepared: Prepared, window_end: Instant) -> DirectAttempt {
    let mut last = Observed::Unchanged;
    // Polls until the window closes: an exact match ends early. A read
    // error is only remembered, never answered with a second write: the
    // next read may still show the value (busy or asynchronous fields).
    // The verdict comes from the last read of the window.
    while let Some(timeout) = readback_timeout(clock, window_end) {
        last = match prepared.field.value_utf16(timeout, MAX_FIELD_UTF16_UNITS) {
            Ok(value) if value == prepared.expected => return DirectAttempt::Verified,
            Ok(value) if value == prepared.before => Observed::Unchanged,
            Ok(_) => Observed::Changed,
            Err(_) => Observed::Unreadable,
        };
        let Some(pause) = readback_pause(clock, window_end) else {
            break;
        };
        clock.sleep(pause);
    }
    match last {
        Observed::Unchanged => DirectAttempt::Unchanged(FallbackGuard {
            field: prepared.field,
            before: prepared.before,
            expected: prepared.expected,
        }),
        Observed::Changed => DirectAttempt::Uncertain(UncertainReason::ChangedUnexpectedly),
        Observed::Unreadable => DirectAttempt::Uncertain(UncertainReason::ReadbackUnavailable),
    }
}

/// Pause before the next read; `None` once the last read of the window was
/// taken (a read only starts with [`MIN_READ_TIMEOUT`] left).
fn readback_pause(clock: &dyn Clock, window_end: Instant) -> Option<Duration> {
    let last_read_start = window_end
        .checked_sub(MIN_READ_TIMEOUT)
        .unwrap_or(window_end);
    let pause = READBACK_POLL_INTERVAL.min(last_read_start.saturating_duration_since(clock.now()));
    (!pause.is_zero()).then_some(pause)
}

fn readback_timeout(clock: &dyn Clock, window_end: Instant) -> Option<Duration> {
    let remaining = window_end.saturating_duration_since(clock.now());
    (remaining >= MIN_READ_TIMEOUT).then(|| remaining.min(PER_CALL_TIMEOUT))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn replaces_the_selected_range_with_the_transcript() {
        let range = Utf16Range {
            location: 1,
            length: 2,
        };
        assert_eq!(
            expected_value(&utf16("A😀Z"), range, &utf16("hi")),
            Some(utf16("AhiZ"))
        );
    }

    #[test]
    fn inserts_at_the_caret_when_the_selection_is_empty() {
        let caret = Utf16Range {
            location: 3,
            length: 0,
        };
        assert_eq!(
            expected_value(&utf16("abc"), caret, &utf16("!")),
            Some(utf16("abc!"))
        );
    }

    #[test]
    fn rejects_ranges_out_of_bounds_or_inside_a_surrogate_pair() {
        let out_of_bounds = Utf16Range {
            location: 2,
            length: 5,
        };
        let split_pair = Utf16Range {
            location: 2,
            length: 0,
        };
        let overflowing = Utf16Range {
            location: usize::MAX,
            length: 1,
        };
        assert_eq!(expected_value(&utf16("abc"), out_of_bounds, &[]), None);
        assert_eq!(expected_value(&utf16("A😀Z"), split_pair, &[]), None);
        assert_eq!(expected_value(&utf16("abc"), overflowing, &[]), None);
    }
}
