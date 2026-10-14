//! Monotonic time source for smart paste.
//!
//! Every timer of the feature (no-read decision, quiet period, layout settle
//! and restore, AX budgets) reads time through [`Clock`] so tests can drive
//! boundaries such as 299/300/301 ms deterministically.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Monotonic clock with a blocking sleep.
pub trait Clock: Send + Sync {
    /// Returns the current monotonic instant.
    fn now(&self) -> Instant;
    /// Blocks the calling thread for `duration`. Never call on the UI thread.
    fn sleep(&self, duration: Duration);
}

/// Real monotonic clock backed by [`Instant`] and [`std::thread::sleep`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Virtual clock for deterministic tests: `sleep` advances time instantly.
#[derive(Debug, Clone)]
pub struct ManualClock {
    origin: Instant,
    elapsed: Arc<Mutex<Duration>>,
}

impl ManualClock {
    /// Creates a clock whose virtual time starts at an arbitrary origin.
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            elapsed: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    /// Returns the instant `offset` after the clock origin.
    pub fn at(&self, offset: Duration) -> Instant {
        self.origin + offset
    }

    /// Returns the virtual time elapsed since the origin.
    pub fn elapsed(&self) -> Duration {
        // A poisoned lock only means another test thread panicked; the
        // stored duration is still a valid value to read.
        *self.elapsed.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Advances virtual time by `duration`.
    pub fn advance(&self, duration: Duration) {
        let mut elapsed = self.elapsed.lock().unwrap_or_else(PoisonError::into_inner);
        *elapsed += duration;
    }

    /// Moves virtual time forward to `instant`; earlier instants are ignored
    /// because monotonic time never goes back.
    pub fn advance_to(&self, instant: Instant) {
        let mut elapsed = self.elapsed.lock().unwrap_or_else(PoisonError::into_inner);
        let target = instant.saturating_duration_since(self.origin);
        if target > *elapsed {
            *elapsed = target;
        }
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.origin + self.elapsed()
    }

    fn sleep(&self, duration: Duration) {
        self.advance(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_sleep_advances_virtual_time_only() {
        let clock = ManualClock::new();
        let start = clock.now();
        clock.sleep(Duration::from_secs(60));
        assert_eq!(clock.now() - start, Duration::from_secs(60));
    }

    #[test]
    fn manual_clock_never_goes_back() {
        let clock = ManualClock::new();
        clock.advance(Duration::from_millis(500));
        clock.advance_to(clock.at(Duration::from_millis(100)));
        assert_eq!(clock.elapsed(), Duration::from_millis(500));
    }
}
