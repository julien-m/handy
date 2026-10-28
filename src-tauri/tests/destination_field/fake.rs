//! Scripted accessibility world for the 011 tests: applications with a focused element and a
//! focused window, elements with the facts of 010's rule, a selection range and scripted write
//! answers, and a log of every call with the timeout it carried. A call advances the manual
//! clock by `cost`; the port can be made to block until released (a port that never answers).
//! No method can return an element's value, selected text or title: the port has none.
#![allow(dead_code)]

use super::destination::ManualClock;
use handy_app_lib::destination::field::{FieldAccess, FieldNode, SelectionRange};
use handy_app_lib::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// How a scripted element answers a focus or range write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Write {
    /// Answered and applied.
    Apply,
    /// Refused at once (element gone, not focusable).
    Refuse,
    /// No answer in time (`kAXErrorCannotComplete`), not applied.
    Unanswered,
    /// No answer in time, but the target applies it anyway.
    UnansweredApplied,
}

/// One scripted element.
#[derive(Clone, Debug)]
pub struct Element {
    pub pid: u32,
    pub role: &'static str,
    pub subrole: Option<&'static str>,
    pub settable: bool,
    pub range: Result<(usize, usize), AxQueryError>,
    pub window: Option<u32>,
    pub focus: Write,
    pub select: Write,
    pub raise_works: bool,
    /// Where an applied focus write really lands (a background tab: another element).
    pub focus_lands_on: Option<u32>,
}
impl Element {
    /// An editable text area of `pid` in window `window`, caret at `range`.
    pub fn input(pid: u32, window: u32, range: (usize, usize)) -> Self {
        Self {
            pid,
            role: "AXTextArea",
            subrole: None,
            settable: true,
            range: Ok(range),
            window: Some(window),
            focus: Write::Apply,
            select: Write::Apply,
            raise_works: true,
            focus_lands_on: None,
        }
    }
    /// A non-editable element of `role` (a list, a button…) of `pid`.
    pub fn control(pid: u32, role: &'static str) -> Self {
        Self {
            settable: false,
            role,
            ..Self::input(pid, 0, (0, 0))
        }
    }
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
        let state = lock(&self.state);
        state.elements.get(&id).and_then(|e| e.range.ok())
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
    fn call(&self, op: &'static str, element: Option<u32>, timeout: Duration) {
        let blocker = lock(&self.state).blocker.take();
        if let Some(blocker) = blocker {
            let _ = blocker.recv();
        }
        let cost = {
            let mut state = lock(&self.state);
            state.calls.push(Call {
                op,
                element,
                timeout,
            });
            state.cost
        };
        self.clock.advance(cost);
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
        self.0.call("focused_element", None, timeout);
        let focused = lock(&self.0.state).focused.get(&pid).copied();
        let id = focused.unwrap_or(Err(AxQueryError::NoValue))?;
        Ok(self.0.node(id))
    }
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        self.0.call("focused_window", None, timeout);
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
        self.world.call("role", Some(self.id), timeout);
        Ok(self.world.get(self.id)?.role.to_owned())
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        self.world.call("subrole", Some(self.id), timeout);
        Ok(self.world.get(self.id)?.subrole.map(str::to_owned))
    }
    fn is_settable(
        &mut self,
        _attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.world.call("is_settable", Some(self.id), timeout);
        Ok(self.world.get(self.id)?.settable)
    }
}
impl FieldNode for Node {
    fn read_range(&mut self, timeout: Duration) -> Result<SelectionRange, AxQueryError> {
        self.world.call("read_range", Some(self.id), timeout);
        let (location, length) = self.world.get(self.id)?.range?;
        Ok(SelectionRange::offsets(location, length))
    }
    fn range_is(
        &mut self,
        range: &SelectionRange,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.world.call("range_is", Some(self.id), timeout);
        Ok(self.world.get(self.id)?.range? == range.as_offsets().ok_or(AxQueryError::Failed)?)
    }
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        self.world.call("focus", Some(self.id), timeout);
        let element = self.world.get(self.id)?;
        let (id, lands) = (self.id, element.focus_lands_on.unwrap_or(self.id));
        self.world.write(id, element.focus, |state, pid| {
            state.focused.insert(pid, Ok(lands));
        })
    }
    fn select(&mut self, range: &SelectionRange, timeout: Duration) -> Result<(), AxQueryError> {
        self.world.call("select", Some(self.id), timeout);
        let answer = self.world.get(self.id)?.select;
        let (id, offsets) = (self.id, range.as_offsets());
        self.world.write(id, answer, |state, _| {
            if let (Some(element), Some(offsets)) = (state.elements.get_mut(&id), offsets) {
                element.range = Ok(offsets);
            }
        })
    }
    fn window(&mut self, timeout: Duration) -> Result<Option<Box<dyn FieldNode>>, AxQueryError> {
        self.world.call("window", Some(self.id), timeout);
        let window = self
            .world
            .get(self.id)?
            .window
            .ok_or(AxQueryError::NoValue)?;
        Ok(Some(self.world.node(window)))
    }
    fn raise(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        self.world.call("raise", Some(self.id), timeout);
        let element = self.world.get(self.id)?;
        if !element.raise_works {
            return Err(AxQueryError::Failed);
        }
        lock(&self.world.state)
            .focused_window
            .insert(element.pid, Ok(self.id));
        Ok(())
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
