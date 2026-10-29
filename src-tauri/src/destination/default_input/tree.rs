//! Accessibility port of feature 013: the focused element and focused window of one application
//! (never the system-wide element), one bounded read per element returning its identity facts
//! and its children, a focus write, and identity between two elements. No method returns a
//! value, a selected text or DOM classes. The macOS adapter is `macos_tree`; tests use fakes.
use super::fingerprint::NodeFacts;
use crate::recovery::classifier::{AxQueryError, FocusedElement};
use std::any::Any;
use std::time::Duration;

/// One element read: its facts and its children (empty for a leaf).
pub struct NodeRead {
    pub facts: NodeFacts,
    pub children: Vec<Box<dyn TreeNode>>,
}

/// One accessibility element obtained by 013.
pub trait TreeNode: FocusedElement + Send {
    /// Role, subrole, title, description, placeholder, DOM id and children in one bounded read.
    fn read(&mut self, timeout: Duration) -> Result<NodeRead, AxQueryError>;
    /// Give the element keyboard focus (`AXFocused` true).
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError>;
    /// Same element as `other` (a local comparison).
    fn same_as(&self, other: &dyn TreeNode) -> bool;
    /// The platform object, for `same_as` between two elements of one adapter.
    fn as_any(&self) -> &dyn Any;
}

/// Accessibility of this process for 013, per application element.
pub trait TreeAccess: Send + Sync {
    /// Accessibility permission, checked without prompting and before any query.
    fn is_trusted(&self) -> bool;
    /// `AXFocusedUIElement` of application `pid`.
    fn focused_element(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn TreeNode>, AxQueryError>;
    /// `AXFocusedWindow` of application `pid`.
    fn focused_window(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn TreeNode>, AxQueryError>;
}

/// A platform without the tree search (Windows, Linux): never trusted, so nothing is queried.
pub struct UnsupportedTree;
impl TreeAccess for UnsupportedTree {
    fn is_trusted(&self) -> bool {
        false
    }
    fn focused_element(&self, _pid: u32, _: Duration) -> Result<Box<dyn TreeNode>, AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
    fn focused_window(&self, _pid: u32, _: Duration) -> Result<Box<dyn TreeNode>, AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
}
