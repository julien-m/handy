//! Insertion method of feature 013's terminal route (spec AC-007): wraps the configured method
//! (classic paste or smart paste). When the guard holds a terminal route, it calls the guard
//! exactly like a method before its first dispatch, then types the text through the addressing
//! tool instead of pasting: no clipboard, no key event, no Enter. Any other route runs the
//! configured method unchanged.
use super::runner::CliRunner;
use super::terminal;
use crate::destination::guard::DispatchGuard;
use crate::destination::insertion::InsertionPort;
use crate::destination::outcome::MethodResult;
use std::sync::Arc;

/// The configured method, or the addressing tool for a terminal route.
pub struct AddressedMethod {
    inner: Box<dyn InsertionPort>,
    text: String,
    runner: Arc<dyn CliRunner>,
}
impl AddressedMethod {
    /// Wrap `inner`, typing `text` with `runner` when the route is a terminal.
    pub fn new(inner: Box<dyn InsertionPort>, text: &str, runner: Arc<dyn CliRunner>) -> Self {
        Self {
            inner,
            text: text.to_owned(),
            runner,
        }
    }
}

impl InsertionPort for AddressedMethod {
    // @spec FR-007: Typed through the tool, without Enter — .specs/features/013-default-input-per-app/spec.md#fr-007
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        let Some(terminal) = guard.terminal_route() else {
            return self.inner.dispatch(guard);
        };
        // Same contract as every method: the guard decides right before the first dispatch.
        if let Err(refusal) = guard.check() {
            return MethodResult::Legacy(Err(refusal.to_string()));
        }
        // Marked before the first call: a send that fails half-way still gets no Enter.
        guard.mark_addressed();
        MethodResult::Legacy(terminal::send(self.runner.as_ref(), &terminal, &self.text))
    }
}
