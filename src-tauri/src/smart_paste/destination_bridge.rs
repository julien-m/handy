//! Bridge between the destination feature (006) and smart paste: the 007
//! [`DestinationContext`] built from 006's guard context (or, without a 006
//! session, from the live foreground), and the per-dictation destination
//! checks the coordinator calls before its accessibility write and before its
//! keystroke. One serialized actor serves every dictation, so each check is
//! registered under its destination generation while its insertion runs.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use super::ids::DestinationGeneration;
use super::request::{DestinationContext, DestinationGuard};
use crate::destination::context::DestinationContext as SessionContext;
use crate::destination::guard::DispatchGuard;
use crate::destination::workspace::{Foreground, WorkspacePort};

// @spec FR-008: Destination context from 006 — .specs/features/007-smart-paste/spec.md#fr-008
impl DestinationContext {
    /// Context of one insertion from the context 006's pre-dispatch guard
    /// accepted: its target process, bundle identifier and generation.
    pub fn from_session(context: &SessionContext) -> Self {
        let target = context.target.as_ref();
        Self {
            // A process id beyond `i32` is no macOS process: no named target.
            pid: target.and_then(|target| i32::try_from(target.pid).ok()),
            bundle_id: target
                .and_then(|target| target.identity.bundle_id())
                .map(str::to_owned),
            generation: DestinationGeneration(context.destination_generation),
        }
    }

    /// Context read from the live foreground for a dictation without a 006
    /// session: the frontmost application, or no named target when Handy is
    /// in front or the foreground cannot be read (like 006's
    /// frontmost-at-the-end route).
    pub fn from_foreground(foreground: &Foreground, generation: DestinationGeneration) -> Self {
        let app = match foreground {
            Foreground::Application(app) => Some(app),
            Foreground::Handy | Foreground::Unreadable => None,
        };
        Self {
            pid: app.and_then(|app| i32::try_from(app.pid).ok()),
            bundle_id: app
                .and_then(|app| app.identity.bundle_id())
                .map(str::to_owned),
            generation,
        }
    }
}

/// One dictation's destination check, called before each insertion effect.
pub trait RequestCheck: Send + Sync {
    /// Whether `destination` is still where this dictation may insert.
    fn is_current(&self, destination: &DestinationContext) -> bool;
}

/// 006's guard: cancellation, supersession and the foreground are rechecked
/// by 006, and the process in front must still be the one smart paste
/// resolved (no retargeting to another application).
pub struct SessionCheck(pub Arc<DispatchGuard>);

impl RequestCheck for SessionCheck {
    fn is_current(&self, destination: &DestinationContext) -> bool {
        match self.0.check() {
            Ok(context) => DestinationContext::from_session(&context).pid == destination.pid,
            Err(refusal) => {
                log::info!("[smart-paste] destination check refused: {refusal:?}");
                false
            }
        }
    }
}

/// Without a 006 session: the dictation is not cancelled and the same
/// process is still in front (live read, as 006's guard does).
pub struct ForegroundCheck {
    /// Live foreground source shared with 006.
    pub workspace: Arc<dyn WorkspacePort>,
    /// True once the dictation was cancelled.
    pub cancelled: Box<dyn Fn() -> bool + Send + Sync>,
}

impl RequestCheck for ForegroundCheck {
    fn is_current(&self, destination: &DestinationContext) -> bool {
        if (self.cancelled)() {
            return false;
        }
        let front = self.workspace.frontmost_now();
        DestinationContext::from_foreground(&front, destination.generation).pid == destination.pid
    }
}

/// Checks of the insertions in flight, by destination generation.
#[derive(Default)]
pub struct DestinationChecks {
    checks: Mutex<HashMap<u64, Arc<dyn RequestCheck>>>,
}

impl DestinationChecks {
    /// Registers `check` for `generation` until the returned registration
    /// is dropped (side effect: replaces an earlier check of that generation).
    pub fn register(
        self: &Arc<Self>,
        generation: DestinationGeneration,
        check: Arc<dyn RequestCheck>,
    ) -> CheckRegistration {
        self.lock().insert(generation.0, Arc::clone(&check));
        CheckRegistration {
            checks: Arc::clone(self),
            generation: generation.0,
            check,
        }
    }

    fn lookup(&self, generation: DestinationGeneration) -> Option<Arc<dyn RequestCheck>> {
        self.lock().get(&generation.0).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<u64, Arc<dyn RequestCheck>>> {
        self.checks.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Keeps one check registered; dropping it unregisters that check only.
pub struct CheckRegistration {
    checks: Arc<DestinationChecks>,
    generation: u64,
    check: Arc<dyn RequestCheck>,
}

impl Drop for CheckRegistration {
    fn drop(&mut self) {
        let mut checks = self.checks.lock();
        // A later registration of the same generation stays in place.
        if checks
            .get(&self.generation)
            .is_some_and(|current| Arc::ptr_eq(current, &self.check))
        {
            checks.remove(&self.generation);
        }
    }
}

/// The coordinator's destination port over the registered checks. An
/// insertion without a registered check fails closed: nothing is dispatched.
pub struct RegisteredDestinations(pub Arc<DestinationChecks>);

impl DestinationGuard for RegisteredDestinations {
    fn is_current(&mut self, destination: &DestinationContext) -> bool {
        // Looked up first so the registry lock is not held during the check.
        let check = self.0.lookup(destination.generation);
        check.is_some_and(|check| check.is_current(destination))
    }
}

#[cfg(test)]
#[path = "destination_bridge_tests.rs"]
mod tests;
