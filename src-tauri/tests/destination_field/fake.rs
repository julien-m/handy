//! Scripted accessibility world for the 011 tests: applications with a focused element and a
//! focused window, elements with the facts of 010's rule, a selection range and scripted write
//! answers, and a log of every call with the timeout it carried. A call advances the manual
//! clock by `cost`; the port can be made to block until released (a port that never answers),
//! or to answer late for a while (an application brought forward a moment ago): a call whose
//! timeout ends before the application answers spends its timeout and gets `Timeout`, and a
//! write sent then is applied anyway, as the target does once it answers.
//! No method can return an element's value, selected text or title: the port has none.
#![allow(dead_code)]

use super::destination::ManualClock;
#[path = "element.rs"]
mod element;
use element::late;
pub use element::{Element, Write};
use handy_app_lib::destination::field::{FieldAccess, FieldNode, SelectionRange};
use handy_app_lib::destination::workspace::Clock;
use handy_app_lib::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One accessibility call and the timeout it carried.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Call {
    pub op: &'static str,
    pub element: Option<u32>,
    pub timeout: Duration,
}

/// An effect run once after the call named `op`.
type After = (&'static str, Box<dyn FnOnce() + Send>);

#[derive(Default)]
struct State {
    trusted: bool,
    focused: HashMap<u32, Result<u32, AxQueryError>>,
    focused_window: HashMap<u32, Result<u32, AxQueryError>>,
    elements: HashMap<u32, Element>,
    cost: Duration,
    /// The next call starts a period of this length during which nothing answers.
    late_for: Option<Duration>,
    /// Range writes applied but not yet visible: element, visible from, range.
    pending_ranges: Vec<(u32, Instant, (usize, usize))>,
    /// Nothing answers before this instant.
    answers_from: Option<Instant>,
    blocker: Option<Receiver<()>>,
    after: Option<After>,
    calls: Vec<Call>,
    nodes_alive: i64,
}

/// The scripted world shared by the port and its elements.
pub struct World {
    state: Mutex<State>,
    clock: Arc<ManualClock>,
}
impl World {
    /// A trusted world on `clock` where every call costs 1 ms.
    pub fn new(clock: &Arc<ManualClock>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                trusted: true,
                cost: Duration::from_millis(1),
                ..State::default()
            }),
            clock: Arc::clone(clock),
        })
    }
    /// Grant or withdraw the Accessibility permission.
    pub fn trusted(&self, trusted: bool) {
        lock(&self.state).trusted = trusted;
    }
    /// Time each call takes on the manual clock.
    pub fn cost(&self, cost: Duration) {
        lock(&self.state).cost = cost;
    }
    /// Element `id` is `element`.
    pub fn element(&self, id: u32, element: Element) {
        lock(&self.state).elements.insert(id, element);
    }
    /// Change element `id` in place.
    pub fn edit(&self, id: u32, change: impl FnOnce(&mut Element)) {
        if let Some(element) = lock(&self.state).elements.get_mut(&id) {
            change(element);
        }
    }
    /// Application `pid` has `focused` as its focused element (or that error).
    pub fn focus(&self, pid: u32, focused: Result<u32, AxQueryError>) {
        lock(&self.state).focused.insert(pid, focused);
    }
    /// Application `pid` has window `window` focused.
    pub fn focus_window(&self, pid: u32, window: Result<u32, AxQueryError>) {
        lock(&self.state).focused_window.insert(pid, window);
    }
    /// The focused element of `pid` now.
    pub fn focused(&self, pid: u32) -> Option<u32> {
        lock(&self.state).focused.get(&pid).and_then(|f| f.ok())
    }
    /// The current range of element `id`.
    pub fn range(&self, id: u32) -> Option<(usize, usize)> {
        self.flush_ranges();
        let state = lock(&self.state);
        state.elements.get(&id).and_then(|e| e.range.ok())
    }
    /// From the next call on, the application answers nothing for `delay` (N18).
    pub fn answers_late(&self, delay: Duration) {
        lock(&self.state).late_for = Some(delay);
    }
    /// Block every call until `release` is dropped or sent to.
    pub fn block_until(&self, release: Receiver<()>) {
        lock(&self.state).blocker = Some(release);
    }
    /// Run `effect` once, right after the next call named `op`.
    pub fn after(&self, op: &'static str, effect: impl FnOnce() + Send + 'static) {
        lock(&self.state).after = Some((op, Box::new(effect)));
    }
    /// Every call so far.
    pub fn calls(&self) -> Vec<Call> {
        lock(&self.state).calls.clone()
    }
    /// Calls named `op`.
    pub fn count(&self, op: &str) -> usize {
        self.calls().iter().filter(|call| call.op == op).count()
    }
    /// Element handles not yet released.
    pub fn alive(&self) -> i64 {
        lock(&self.state).nodes_alive
    }
    /// Record one call and spend its time; `false` when it got no answer within `timeout`.
    fn call(&self, op: &'static str, element: Option<u32>, timeout: Duration) -> bool {
        let blocker = lock(&self.state).blocker.take();
        if let Some(blocker) = blocker {
            let _ = blocker.recv();
        }
        let (cost, wait) = self.record(Call {
            op,
            element,
            timeout,
        });
        // Without a late period the scripted cost never exceeds a timeout (as before N18).
        let answered = wait.is_zero() || wait + cost <= timeout;
        self.clock
            .advance(if answered { wait + cost } else { timeout });
        self.run_after(op);
        answered
    }
    /// A read: `Timeout` when it got no answer in time.
    fn answer(
        &self,
        op: &'static str,
        element: u32,
        timeout: Duration,
    ) -> Result<(), AxQueryError> {
        self.call(op, Some(element), timeout)
            .then_some(())
            .ok_or(AxQueryError::Timeout)
    }
    /// Log `call`; its cost and how long until the application answers.
    fn record(&self, call: Call) -> (Duration, Duration) {
        let now = self.clock.now();
        let mut state = lock(&self.state);
        state.calls.push(call);
        if let Some(delay) = state.late_for.take() {
            state.answers_from = Some(now + delay);
        }
        let wait = state
            .answers_from
            .map_or(Duration::ZERO, |from| from.saturating_duration_since(now));
        (state.cost, wait)
    }
    /// The effect registered for `op`, once.
    fn run_after(&self, op: &'static str) {
        let effect = {
            let mut state = lock(&self.state);
            match state.after.take() {
                Some((name, effect)) if name == op => Some(effect),
                other => {
                    state.after = other;
                    None
                }
            }
        };
        if let Some(effect) = effect {
            effect();
        }
    }
    fn node(self: &Arc<Self>, id: u32) -> Box<dyn FieldNode> {
        lock(&self.state).nodes_alive += 1;
        Box::new(Node {
            id,
            world: Arc::clone(self),
        })
    }
    /// Range writes whose lag has passed become visible.
    fn flush_ranges(&self) {
        let now = self.clock.now();
        let mut state = lock(&self.state);
        let pending = std::mem::take(&mut state.pending_ranges);
        for (id, from, range) in pending {
            match state.elements.get_mut(&id) {
                Some(element) if from <= now => element.range = Ok(range),
                _ => state.pending_ranges.push((id, from, range)),
            }
        }
    }
    fn get(&self, id: u32) -> Result<Element, AxQueryError> {
        lock(&self.state)
            .elements
            .get(&id)
            .cloned()
            .ok_or(AxQueryError::Failed)
    }
    fn write(
        &self,
        id: u32,
        answer: Write,
        apply: impl FnOnce(&mut State, u32),
    ) -> Result<(), AxQueryError> {
        let mut state = lock(&self.state);
        if matches!(answer, Write::Apply | Write::UnansweredApplied) {
            let pid = state.elements.get(&id).map_or(0, |e| e.pid);
            apply(&mut state, pid);
        }
        match answer {
            Write::Apply => Ok(()),
            Write::Refuse => Err(AxQueryError::Failed),
            Write::Unanswered | Write::UnansweredApplied => Err(AxQueryError::Timeout),
        }
    }
}

/// The port of the world.
pub struct Access(pub Arc<World>);
impl FieldAccess for Access {
    fn is_trusted(&self) -> bool {
        lock(&self.0.state).trusted
    }
    fn focused_element(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        if !self.0.call("focused_element", None, timeout) {
            return Err(AxQueryError::Timeout);
        }
        let focused = lock(&self.0.state).focused.get(&pid).copied();
        let id = focused.unwrap_or(Err(AxQueryError::NoValue))?;
        Ok(self.0.node(id))
    }
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        if !self.0.call("focused_window", None, timeout) {
            return Err(AxQueryError::Timeout);
        }
        let focused = lock(&self.0.state).focused_window.get(&pid).copied();
        let id = focused.unwrap_or(Err(AxQueryError::NoValue))?;
        Ok(self.0.node(id))
    }
}

/// One element handle.
pub struct Node {
    pub id: u32,
    world: Arc<World>,
}
impl Drop for Node {
    fn drop(&mut self) {
        lock(&self.world.state).nodes_alive -= 1;
    }
}
impl FocusedElement for Node {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.world.get(self.id)?.pid)
    }
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError> {
        self.world.answer("role", self.id, timeout)?;
        Ok(self.world.get(self.id)?.role.to_owned())
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        self.world.answer("subrole", self.id, timeout)?;
        Ok(self.world.get(self.id)?.subrole.map(str::to_owned))
    }
    fn is_settable(
        &mut self,
        _attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.world.answer("is_settable", self.id, timeout)?;
        Ok(self.world.get(self.id)?.settable)
    }
}
impl FieldNode for Node {
    fn read_range(&mut self, timeout: Duration) -> Result<SelectionRange, AxQueryError> {
        self.world.answer("read_range", self.id, timeout)?;
        self.world.flush_ranges();
        let (location, length) = self.world.get(self.id)?.range?;
        Ok(SelectionRange::offsets(location, length))
    }
    fn range_is(
        &mut self,
        range: &SelectionRange,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.world.answer("range_is", self.id, timeout)?;
        self.world.flush_ranges();
        Ok(self.world.get(self.id)?.range? == range.as_offsets().ok_or(AxQueryError::Failed)?)
    }
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        let answered = self.world.call("focus", Some(self.id), timeout);
        let element = self.world.get(self.id)?;
        let (id, lands) = (self.id, element.focus_lands_on.unwrap_or(self.id));
        let answer = late(answered, element.focus);
        self.world.write(id, answer, |state, pid| {
            state.focused.insert(pid, Ok(lands));
        })
    }
    fn select(&mut self, range: &SelectionRange, timeout: Duration) -> Result<(), AxQueryError> {
        let answered = self.world.call("select", Some(self.id), timeout);
        let element = self.world.get(self.id)?;
        let answer = late(answered, element.select);
        let offsets = element.select_lands_at.or(range.as_offsets());
        let (id, visible_from) = (self.id, self.world.clock.now() + element.select_lag);
        self.world.write(id, answer, |state, _| {
            if let Some(offsets) = offsets {
                state.pending_ranges.push((id, visible_from, offsets));
            }
        })?;
        self.world.flush_ranges();
        Ok(())
    }
    fn window(&mut self, timeout: Duration) -> Result<Option<Box<dyn FieldNode>>, AxQueryError> {
        self.world.answer("window", self.id, timeout)?;
        let window = self
            .world
            .get(self.id)?
            .window
            .ok_or(AxQueryError::NoValue)?;
        Ok(Some(self.world.node(window)))
    }
    fn raise(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        let answered = self.world.call("raise", Some(self.id), timeout);
        let element = self.world.get(self.id)?;
        if !element.raise_works {
            return Err(AxQueryError::Failed);
        }
        lock(&self.world.state)
            .focused_window
            .insert(element.pid, Ok(self.id));
        if answered {
            Ok(())
        } else {
            Err(AxQueryError::Timeout)
        }
    }
    fn same_as(&self, other: &dyn FieldNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Node>()
            .is_some_and(|other| other.id == self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
