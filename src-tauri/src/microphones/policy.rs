//! Ordered microphone resolution (AC-003, AC-005), pure over injected ports: lid probe,
//! enumeration, open attempts and a monotonic clock. Strategy: with a configured clamshell
//! override and a confirmed closed lid the override is the only configured candidate; otherwise
//! the saved order is. Candidates are looked up in one fresh catalog (re-enumerated once if the
//! first enumeration failed) and opened in order; the system default is attempted last. The
//! preferences are only read: no outcome ever changes them.
//!
//! Budget (orchestrator decision 2026-10-14): every open attempt gets its full 2 s; the 6 s total
//! limits how many attempts start. A configured attempt starts only while at least 4 s remain
//! (its 2 s plus 2 s reserved for the default), probes end by t = 4 s, so the default attempt
//! always starts by t = 4 s with its full 2 s.
use super::catalog::{Catalog, Lookup};
use super::types::EffectiveReason;
use std::time::Duration;

/// Each open attempt times out after this (AC-003).
pub const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(2);
/// The whole resolution ends within this (AC-003).
pub const RESOLUTION_DEADLINE: Duration = Duration::from_secs(6);
/// A configured attempt starts only while this much remains: its own timeout plus the system
/// default's.
pub const CONFIGURED_START_RESERVE: Duration = Duration::from_secs(4);
/// Probes (lid, enumeration) end by this elapsed time; the default attempt starts by it.
pub const DEFAULT_START_LATEST: Duration = Duration::from_secs(4);
/// Most time the lid probe may take. `ioreg` answers in 10–20 ms; one second absorbs a loaded
/// machine and still leaves the configured choices their slot when the probe hangs.
pub const LID_PROBE_BUDGET: Duration = Duration::from_secs(1);

/// What an open attempt targets.
#[derive(Debug, PartialEq)]
pub enum OpenTarget<'a, D> {
    /// One device of the current catalog.
    Device(&'a D),
    /// The system default input.
    SystemDefault,
}

/// Why an open attempt failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenFailure {
    /// No stream within the attempt's budget.
    TimedOut,
    /// The backend refused, with its message (kept for the recording error, never logged here).
    Failed(String),
}

/// Native operations the resolver needs; every call is bounded by the budget it receives.
pub trait ResolutionPorts {
    /// Backend device handle.
    type Device;
    /// Opened stream owned by the caller.
    type Stream;
    /// Monotonic time since the resolution started.
    fn elapsed(&self) -> Duration;
    /// Whether the lid is confirmed closed; `None` when unknown or not answered within `budget`.
    fn lid_closed(&mut self, budget: Duration) -> Option<bool>;
    /// One fresh enumeration, or `None` when it failed or did not end within `budget`.
    fn enumerate(&mut self, budget: Duration) -> Option<Catalog<Self::Device>>;
    /// Open `target` within `budget`.
    ///
    /// # Errors
    /// `TimedOut` or `Failed`; the port releases anything opened late.
    fn open(
        &mut self,
        target: OpenTarget<'_, Self::Device>,
        budget: Duration,
    ) -> Result<Self::Stream, OpenFailure>;
}

/// The committed preferences captured when the stream opening starts.
#[derive(Clone, Copy, Debug)]
pub struct Preferences<'a> {
    /// Saved order of normal choices.
    pub priority: &'a [String],
    /// Clamshell override, if configured.
    pub clamshell: Option<&'a str>,
}

/// Why one choice produced no stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttemptResult {
    /// A complete enumeration did not find it (no open call).
    Missing,
    /// Several present devices share its name (no open call).
    Ambiguous,
    /// No enumeration succeeded, so it could not be resolved (no open call).
    Unresolved,
    /// Fewer than 4 s remained when its turn came (no open call).
    Deadline,
    /// Its open failed or timed out.
    Open(OpenFailure),
}

/// One failed attempt: a configured name, or `None` for the system default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attempt {
    pub choice: Option<String>,
    pub result: AttemptResult,
}

/// The stream that opened and why.
#[derive(Debug)]
pub struct Opened<S> {
    pub stream: S,
    /// Configured name that opened, or `None` for the system default.
    pub choice: Option<String>,
    pub reason: EffectiveReason,
}

/// Outcome of one resolution: the opened stream, if any, and every failed attempt in order.
#[derive(Debug)]
pub struct Resolution<S> {
    pub opened: Option<Opened<S>>,
    pub attempts: Vec<Attempt>,
    /// Whether no enumeration succeeded (the UI shows the enumeration warning).
    pub enumeration_failed: bool,
}

/// Configured candidates in order, with the reason each would carry.
type Candidates<'a> = Vec<(&'a str, EffectiveReason)>;

// @spec FR-003: Ordered bounded resolution then default, FR-005: Closed-lid override first — .specs/features/001-microphone-priority/spec.md#fr-003
/// Resolve `preferences` against `ports` (see the module strategy).
pub fn resolve<P: ResolutionPorts>(
    preferences: &Preferences<'_>,
    ports: &mut P,
) -> Resolution<P::Stream> {
    let candidates = candidates(preferences, ports);
    let mut resolution = Resolution {
        opened: None,
        attempts: Vec::new(),
        enumeration_failed: false,
    };
    if !candidates.is_empty() {
        resolution.opened = configured(ports, candidates, &mut resolution);
        if resolution.opened.is_some() {
            return resolution;
        }
    }
    match ports.open(OpenTarget::SystemDefault, ATTEMPT_TIMEOUT) {
        Ok(stream) => {
            resolution.opened = Some(opened(stream, None, EffectiveReason::SystemDefault))
        }
        Err(failure) => resolution
            .attempts
            .push(failed(None, AttemptResult::Open(failure))),
    }
    resolution
}
/// The first configured candidate that opens, from one fresh catalog; each failure is recorded
/// in `resolution`.
fn configured<P: ResolutionPorts>(
    ports: &mut P,
    candidates: Candidates<'_>,
    resolution: &mut Resolution<P::Stream>,
) -> Option<Opened<P::Stream>> {
    let catalog = observe(ports);
    resolution.enumeration_failed = catalog.is_none();
    for (name, reason) in candidates {
        match attempt(ports, catalog.as_ref(), name) {
            // Early exit: the first choice that opens wins; later ones are never tried.
            Ok(stream) => return Some(opened(stream, Some(name), reason)),
            Err(result) => resolution.attempts.push(failed(Some(name), result)),
        }
    }
    None
}
/// Business rule (AC-005): a configured override with a confirmed closed lid is the only
/// configured candidate (then the default, as before 001); an open or unknown lid, or no
/// override, uses the saved order.
fn candidates<'a, P: ResolutionPorts>(
    preferences: &Preferences<'a>,
    ports: &mut P,
) -> Candidates<'a> {
    if let Some(clamshell) = preferences.clamshell {
        let budget = DEFAULT_START_LATEST
            .saturating_sub(ports.elapsed())
            .min(LID_PROBE_BUDGET);
        if ports.lid_closed(budget) == Some(true) {
            return vec![(clamshell, EffectiveReason::Clamshell)];
        }
    }
    preferences
        .priority
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let reason = match index {
                0 => EffectiveReason::FirstChoice,
                _ => EffectiveReason::Fallback,
            };
            (name.as_str(), reason)
        })
        .collect()
}
/// One fresh catalog; after a failed or timed-out enumeration, exactly one more fresh attempt,
/// both ending by `DEFAULT_START_LATEST`.
fn observe<P: ResolutionPorts>(ports: &mut P) -> Option<Catalog<P::Device>> {
    for _ in 0..2 {
        let budget = DEFAULT_START_LATEST.saturating_sub(ports.elapsed());
        if budget.is_zero() {
            // Early exit: no time is left before the default's slot.
            return None;
        }
        if let Some(catalog) = ports.enumerate(budget) {
            return Some(catalog);
        }
    }
    None
}
/// One configured choice: skipped without an open call when time, presence or uniqueness
/// rules it out, otherwise opened with the full attempt timeout.
fn attempt<P: ResolutionPorts>(
    ports: &mut P,
    catalog: Option<&Catalog<P::Device>>,
    name: &str,
) -> Result<P::Stream, AttemptResult> {
    if RESOLUTION_DEADLINE.saturating_sub(ports.elapsed()) < CONFIGURED_START_RESERVE {
        return Err(AttemptResult::Deadline);
    }
    let catalog = catalog.ok_or(AttemptResult::Unresolved)?;
    match catalog.lookup(name) {
        Lookup::Missing => Err(AttemptResult::Missing),
        Lookup::Ambiguous => Err(AttemptResult::Ambiguous),
        Lookup::Unique(device) => ports
            .open(OpenTarget::Device(device), ATTEMPT_TIMEOUT)
            .map_err(AttemptResult::Open),
    }
}
fn opened<S>(stream: S, choice: Option<&str>, reason: EffectiveReason) -> Opened<S> {
    Opened {
        stream,
        choice: choice.map(str::to_owned),
        reason,
    }
}
fn failed(choice: Option<&str>, result: AttemptResult) -> Attempt {
    Attempt {
        choice: choice.map(str::to_owned),
        result,
    }
}
