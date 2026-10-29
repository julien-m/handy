//! Insertion route of feature 013 (spec AC-005 to AC-007), decided once on 006's insertion
//! worker after the destination's instance is in front and only without a 011 attempt: when a
//! default input is remembered for the instance and its focused element is not editable by 010's
//! rule, focus the unique matching field, or resolve the remembered terminal. The guard reads the
//! route at its first pass; the method wrapper (`dispatch`) types a terminal delivery.
use super::model::{input_for, DefaultTarget, FieldFingerprint, TerminalTarget};
use super::search::{search, SearchOutcome};
use super::services::DefaultInputServices;
use super::terminal::{resolve, ResolvedTerminal};
use super::tree::{TreeAccess, TreeNode};
use crate::destination::field::{FIELD_NOTE_BUDGET, MIN_AX_TIMEOUT};
use crate::destination::workspace::{Clock, Pause, RunningApp};
use crate::recovery::classifier::{classify_focused, Deadline, FocusClassification, Undetermined};
use std::time::Duration;

/// Bound of the focus write and of its verification (spec AC-006).
pub const FOCUS_VERIFY_BUDGET: Duration = Duration::from_millis(200);
/// Period between two reads of the focused element while the focus settles.
const VERIFY_PERIOD: Duration = Duration::from_millis(10);

/// Which kind of remembered target could not be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefaultKind {
    Field,
    Terminal,
}

/// The route handed to the guard.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DefaultRoute {
    /// Nothing remembered applies: the insertion is the one without this feature.
    #[default]
    Unchanged,
    /// The remembered field is focused and verified: the configured method pastes.
    Focused,
    /// Type through the addressing tool into this terminal.
    Terminal(ResolvedTerminal),
    /// The remembered target was not found: nothing is dispatched, 010's bubble shows it.
    Unavailable(DefaultKind),
}

/// What the focused element's classification calls for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Plan {
    Unchanged,
    SearchField(FieldFingerprint),
    ResolveTerminal(TerminalTarget),
}

// @spec AC-005: Editable focus keeps today's insertion — .specs/features/013-default-input-per-app/spec.md#ac-005
/// Business rule (spec AC-005, decision `secure_focus_unchanged`): an editable focus or a
/// password field keeps today's insertion; a field target also needs the accessibility
/// permission (no search is possible without it); otherwise the remembered target applies.
pub fn plan(classification: FocusClassification, target: &DefaultTarget) -> Plan {
    match (classification, target) {
        (FocusClassification::Editable, _)
        | (FocusClassification::Undetermined(Undetermined::SecureField), _)
        | (FocusClassification::PermissionMissing, DefaultTarget::Field(_)) => Plan::Unchanged,
        (_, DefaultTarget::Field(fingerprint)) => Plan::SearchField(fingerprint.clone()),
        (_, DefaultTarget::Terminal(terminal)) => Plan::ResolveTerminal(terminal.clone()),
    }
}

/// Classify the focused element of `pid` with 010's rule within the note budget (a Chromium
/// application brought forward a moment ago may answer its first request late, as in 011).
pub fn classify_focus(access: &dyn TreeAccess, pid: u32, clock: &dyn Clock) -> FocusClassification {
    if !access.is_trusted() {
        return FocusClassification::PermissionMissing;
    }
    let deadline = Deadline::new(clock, FIELD_NOTE_BUDGET, MIN_AX_TIMEOUT);
    let Ok(timeout) = deadline.remaining() else {
        return FocusClassification::Undetermined(Undetermined::BudgetSpent);
    };
    let mut element = match access.focused_element(pid, timeout) {
        Ok(element) => element,
        Err(_) => return FocusClassification::Undetermined(Undetermined::NoFocusedElement),
    };
    classify_focused(element.as_mut(), pid, &deadline)
        .unwrap_or_else(FocusClassification::Undetermined)
}

// @spec FR-005: Route decided after activation, without 011 — .specs/features/013-default-input-per-app/spec.md#fr-005
/// The route of a dictation whose destination `target` is in front; `Unchanged` without an
/// activated instance, without a remembered input for it, or once `cancelled`.
pub fn prepare(
    services: &DefaultInputServices,
    target: Option<&RunningApp>,
    cancelled: &dyn Fn() -> bool,
    (clock, pause): (&dyn Clock, &dyn Pause),
) -> DefaultRoute {
    let Some(app) = target else {
        return DefaultRoute::Unchanged;
    };
    let inputs = services.remembered();
    let Some(input) = input_for(&inputs, &app.identity) else {
        return DefaultRoute::Unchanged;
    };
    if cancelled() {
        return DefaultRoute::Unchanged;
    }
    let started = clock.now();
    let classification = classify_focus(services.tree.as_ref(), app.pid, clock);
    let route = match plan(classification, &input.target) {
        Plan::Unchanged => DefaultRoute::Unchanged,
        Plan::SearchField(fingerprint) => {
            field_route(services, app.pid, &fingerprint, (clock, pause))
        }
        Plan::ResolveTerminal(terminal) => terminal_route(services, app.pid, &terminal),
    };
    // Kind of route and timing only: never a name, a fingerprint or a workspace.
    log::debug!(
        "default_input route {} after focus {classification:?} in {:?}",
        route_label(&route),
        clock.now().saturating_duration_since(started)
    );
    route
}

fn route_label(route: &DefaultRoute) -> &'static str {
    match route {
        DefaultRoute::Unchanged => "unchanged",
        DefaultRoute::Focused => "field_focused",
        DefaultRoute::Terminal(_) => "terminal",
        DefaultRoute::Unavailable(DefaultKind::Field) => "field_unavailable",
        DefaultRoute::Unavailable(DefaultKind::Terminal) => "terminal_unavailable",
    }
}

// @spec AC-006: Unique field focused and verified — .specs/features/013-default-input-per-app/spec.md#ac-006
/// Search, then focus the unique match and verify it is the application's focused element.
fn field_route(
    services: &DefaultInputServices,
    pid: u32,
    fingerprint: &FieldFingerprint,
    (clock, pause): (&dyn Clock, &dyn Pause),
) -> DefaultRoute {
    let (outcome, stats) = search(services.tree.as_ref(), pid, fingerprint, clock);
    log::debug!(
        "default_input search {outcome:?}: {} elements, {} matches, {:?}",
        stats.visited,
        stats.matches,
        stats.elapsed
    );
    let SearchOutcome::Unique(mut node) = outcome else {
        return DefaultRoute::Unavailable(DefaultKind::Field);
    };
    if focus_verified(services.tree.as_ref(), pid, node.as_mut(), (clock, pause)) {
        DefaultRoute::Focused
    } else {
        DefaultRoute::Unavailable(DefaultKind::Field)
    }
}

/// Give `node` focus, then read the focused element until it is `node` or the budget is spent.
fn focus_verified(
    access: &dyn TreeAccess,
    pid: u32,
    node: &mut dyn TreeNode,
    (clock, pause): (&dyn Clock, &dyn Pause),
) -> bool {
    let deadline = Deadline::new(clock, FOCUS_VERIFY_BUDGET, MIN_AX_TIMEOUT);
    let Ok(timeout) = deadline.remaining() else {
        return false;
    };
    if node.focus(timeout).is_err() {
        return false;
    }
    while let Ok(timeout) = deadline.remaining() {
        let focused = access.focused_element(pid, timeout);
        if focused.is_ok_and(|focused| focused.same_as(node)) {
            return true;
        }
        pause.pause(VERIFY_PERIOD);
    }
    false
}

/// The remembered terminal resolved through the instance's addressing tool.
fn terminal_route(
    services: &DefaultInputServices,
    pid: u32,
    terminal: &TerminalTarget,
) -> DefaultRoute {
    let tool = services
        .tool_for(pid)
        .filter(|(tool, _)| *tool == terminal.tool);
    let Some((_, cli)) = tool else {
        return DefaultRoute::Unavailable(DefaultKind::Terminal);
    };
    match resolve(services.runner.as_ref(), &cli, terminal) {
        Ok(resolved) => DefaultRoute::Terminal(resolved),
        Err(error) => {
            log::debug!("default_input terminal not resolved: {error:?}");
            DefaultRoute::Unavailable(DefaultKind::Terminal)
        }
    }
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
