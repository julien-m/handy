//! Scripted accessibility world for the tests of feature 013: elements with identity facts,
//! children and editability, one focused element and one focused window per application, and a
//! fake clock advanced by every read. Like the real adapter it exposes no value or text.
use super::fingerprint::NodeFacts;
use super::tree::{NodeRead, TreeAccess, TreeNode};
use crate::destination::fake_workspace::FakeClock;
use crate::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use std::any::Any;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// One scripted element.
#[derive(Clone, Default)]
pub(crate) struct FakeElement {
    pub facts: NodeFacts,
    pub children: Vec<usize>,
    pub editable: bool,
    /// The read answers this error instead of the facts.
    pub read_error: Option<AxQueryError>,
    /// The focus write is refused with this error.
    pub focus_error: Option<AxQueryError>,
}

/// The world: elements by index, the focused element and window, and the read cost.
pub(crate) struct World {
    pub pid: u32,
    pub trusted: bool,
    pub elements: Vec<FakeElement>,
    pub focused: Option<usize>,
    pub window: Option<usize>,
    /// Time each element read takes on the fake clock.
    pub read_cost: Duration,
    pub reads: usize,
    /// A focus write is applied only when true (an app that ignores `AXFocused`).
    pub focus_applies: bool,
}

/// The shared world and its clock.
#[derive(Clone)]
pub(crate) struct FakeTree {
    pub world: Arc<Mutex<World>>,
    pub clock: Arc<FakeClock>,
}
impl FakeTree {
    /// A trusted world of application `pid` whose element 0 is the focused window.
    pub(crate) fn new(pid: u32, elements: Vec<FakeElement>) -> Self {
        let world = World {
            pid,
            trusted: true,
            elements,
            focused: None,
            window: Some(0),
            read_cost: Duration::from_micros(50),
            reads: 0,
            focus_applies: true,
        };
        Self {
            world: Arc::new(Mutex::new(world)),
            clock: Arc::new(FakeClock::new()),
        }
    }
    /// The world, recovered after a poisoning panic.
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, World> {
        self.world.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn node(&self, id: usize) -> Box<dyn TreeNode> {
        Box::new(FakeNode {
            id,
            tree: self.clone(),
        })
    }
}

/// An element with the given role, name and children.
pub(crate) fn element(role: &str, name: Option<&str>, children: Vec<usize>) -> FakeElement {
    FakeElement {
        facts: NodeFacts {
            role: Some(role.to_owned()),
            title: name.map(str::to_owned),
            ..NodeFacts::default()
        },
        children,
        editable: matches!(role, "AXTextArea" | "AXTextField"),
        ..FakeElement::default()
    }
}

struct FakeNode {
    id: usize,
    tree: FakeTree,
}
impl FakeNode {
    fn with<T>(&self, read: impl FnOnce(&FakeElement) -> T) -> Result<T, AxQueryError> {
        let world = self.tree.lock();
        world
            .elements
            .get(self.id)
            .map(read)
            .ok_or(AxQueryError::Failed)
    }
}
impl FocusedElement for FakeNode {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.tree.lock().pid)
    }
    fn role(&mut self, _: Duration) -> Result<String, AxQueryError> {
        self.with(|element| element.facts.role.clone())?
            .ok_or(AxQueryError::NoValue)
    }
    fn subrole(&mut self, _: Duration) -> Result<Option<String>, AxQueryError> {
        self.with(|element| element.facts.subrole.clone())
    }
    fn is_settable(&mut self, attribute: EditAttribute, _: Duration) -> Result<bool, AxQueryError> {
        // Every edit attribute answers the element's editability.
        let _ = attribute;
        self.with(|element| element.editable)
    }
}
impl TreeNode for FakeNode {
    fn read(&mut self, _: Duration) -> Result<NodeRead, AxQueryError> {
        let cost = {
            let mut world = self.tree.lock();
            world.reads += 1;
            world.read_cost
        };
        self.tree.clock.advance(cost);
        let (facts, children, error) = self.with(|element| {
            (
                element.facts.clone(),
                element.children.clone(),
                element.read_error,
            )
        })?;
        if let Some(error) = error {
            return Err(error);
        }
        let children = children.into_iter().map(|id| self.tree.node(id)).collect();
        Ok(NodeRead { facts, children })
    }
    fn focus(&mut self, _: Duration) -> Result<(), AxQueryError> {
        if let Some(error) = self.with(|element| element.focus_error)? {
            return Err(error);
        }
        let mut world = self.tree.lock();
        if world.focus_applies {
            world.focused = Some(self.id);
        }
        Ok(())
    }
    fn same_as(&self, other: &dyn TreeNode) -> bool {
        other
            .as_any()
            .downcast_ref::<FakeNode>()
            .is_some_and(|other| other.id == self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TreeAccess for FakeTree {
    fn is_trusted(&self) -> bool {
        self.lock().trusted
    }
    fn focused_element(&self, pid: u32, _: Duration) -> Result<Box<dyn TreeNode>, AxQueryError> {
        let focused = {
            let world = self.lock();
            (world.pid == pid).then_some(world.focused).flatten()
        };
        focused.map(|id| self.node(id)).ok_or(AxQueryError::NoValue)
    }
    fn focused_window(&self, pid: u32, _: Duration) -> Result<Box<dyn TreeNode>, AxQueryError> {
        let window = {
            let world = self.lock();
            (world.pid == pid).then_some(world.window).flatten()
        };
        window.map(|id| self.node(id)).ok_or(AxQueryError::NoValue)
    }
}

/// Result of a test or a fallible fixture.
pub(crate) type TestResult = Result<(), Box<dyn std::error::Error>>;
/// A fixture value or why it could not be built.
pub(crate) type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

/// A bundle locator that answers one directory for every process.
pub(crate) struct OneBundle(pub std::path::PathBuf);
impl super::services::BundleLocator for OneBundle {
    fn bundle_path(&self, _pid: u32) -> Option<std::path::PathBuf> {
        Some(self.0.clone())
    }
}

#[cfg(unix)]
/// A temporary bundle holding cmux's tool with permission `mode`.
pub(crate) fn bundle_with_tool(mode: u32) -> Fallible<tempfile::TempDir> {
    use std::os::unix::fs::PermissionsExt;
    let bundle = tempfile::tempdir()?;
    let tool = bundle
        .path()
        .join(super::model::AddressingTool::Cmux.bundle_relative_path());
    std::fs::create_dir_all(tool.parent().ok_or("tool without a parent")?)?;
    std::fs::write(&tool, b"#!/bin/sh\n")?;
    std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(mode))?;
    Ok(bundle)
}
