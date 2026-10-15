//! Windows UI Automation adapter of 011's field port (spec AC-007, plan "Windows adapter"). One
//! COM MTA thread, created on first use, owns Handy's own `IUIAutomation` client and every
//! element and text range it obtained (`windows_uia`); callers hold opaque ids. Each call waits
//! at most the remaining budget (`BoundedWorker`): an expired wait is `Timeout` (for `SetFocus`
//! or `Select`, the write may still apply: unsettled), a call while the thread is still occupied
//! is refused at once, and the client's connection and transaction timeouts are set to the same
//! budget (never 0). The range is the first selection range, cloned and kept as endpoints.
//! Element identity is the runtime id and process. UI Automation exposes only the system focus,
//! so a pinned application that is not in front at the start remembers nothing. Native behavior
//! is pending release evidence; this file is compile-checked for `x86_64-pc-windows-msvc`.
use super::bounded_worker::BoundedWorker;
use super::field::{FieldAccess, FieldNode, SelectionRange};
use super::uia_mapping::FactsView;
use super::windows_uia::{
    call_error, create_client, first_selection, focused, selection_is, Uia, Worker,
};
use crate::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

/// Production [`FieldAccess`] of Windows.
pub struct WindowsFieldAccess {
    worker: Worker,
}
impl WindowsFieldAccess {
    /// The adapter and its UI Automation thread.
    pub fn new() -> Self {
        Self {
            worker: Arc::new(BoundedWorker::spawn("handy-uia-field", create_client)),
        }
    }
}
impl Default for WindowsFieldAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldAccess for WindowsFieldAccess {
    fn is_trusted(&self) -> bool {
        // UI Automation needs no permission on Windows.
        true
    }
    fn focused_element(
        &self,
        _pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        // `_pid` is unused: UI Automation exposes the system focus only; 010's rule then
        // refuses an element of another process.
        let seen = self
            .worker
            .call(timeout, move |uia: &mut Uia| focused(uia, timeout))
            .map_err(call_error)??;
        Ok(Box::new(WindowsFieldNode {
            id: seen.id,
            view: FactsView {
                pid: seen.pid,
                facts: seen.facts,
            },
            runtime_id: seen.runtime_id,
            worker: Arc::clone(&self.worker),
        }))
    }
    fn focused_window(
        &self,
        _pid: u32,
        _timeout: Duration,
    ) -> Result<Box<dyn FieldNode>, AxQueryError> {
        // Never asked: `SetFocus` brings the element's window (`window` answers `None`).
        Err(AxQueryError::Unsupported)
    }
}

/// A kept text range, released on the thread when the last copy goes.
struct RangeHandle {
    id: u64,
    worker: Worker,
}
impl Drop for RangeHandle {
    fn drop(&mut self) {
        let id = self.id;
        self.worker.post(move |uia: &mut Uia| {
            uia.ranges.remove(&id);
        });
    }
}

/// One element kept on the UI Automation thread.
pub struct WindowsFieldNode {
    id: u64,
    view: FactsView,
    runtime_id: Vec<i32>,
    worker: Worker,
}
impl Drop for WindowsFieldNode {
    fn drop(&mut self) {
        let id = self.id;
        self.worker.post(move |uia: &mut Uia| {
            uia.elements.remove(&id);
        });
    }
}
impl WindowsFieldNode {
    /// Run `job` on this element within `timeout`.
    fn on_element<T: Send + 'static>(
        &self,
        timeout: Duration,
        job: impl FnOnce(&IUIAutomationElement, &mut Uia) -> Result<T, AxQueryError> + Send + 'static,
    ) -> Result<T, AxQueryError> {
        let id = self.id;
        self.worker
            .call(timeout, move |uia: &mut Uia| {
                uia.bound(timeout);
                let element = uia
                    .elements
                    .get(&id)
                    .cloned()
                    .ok_or(AxQueryError::NoValue)?;
                job(&element, uia)
            })
            .map_err(call_error)?
    }
    fn range_id(range: &SelectionRange) -> Result<u64, AxQueryError> {
        range
            .as_endpoints()
            .and_then(|handle| handle.downcast_ref::<RangeHandle>())
            .map(|handle| handle.id)
            .ok_or(AxQueryError::Unsupported)
    }
}

impl FocusedElement for WindowsFieldNode {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        self.view.pid()
    }
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError> {
        self.view.role(timeout)
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        self.view.subrole(timeout)
    }
    fn is_settable(
        &mut self,
        attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        self.view.is_settable(attribute, timeout)
    }
}

impl FieldNode for WindowsFieldNode {
    fn read_range(&mut self, timeout: Duration) -> Result<SelectionRange, AxQueryError> {
        let id = self.on_element(timeout, |element, uia| {
            // SAFETY: COM calls on the thread's own objects; `GetText` is never called.
            let kept =
                unsafe { first_selection(element)?.Clone() }.map_err(|_| AxQueryError::Failed)?;
            Ok(uia.keep_range(kept))
        })?;
        let handle = RangeHandle {
            id,
            worker: Arc::clone(&self.worker),
        };
        Ok(SelectionRange::endpoints(Arc::new(handle)))
    }
    fn range_is(
        &mut self,
        range: &SelectionRange,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        let range_id = Self::range_id(range)?;
        self.on_element(timeout, move |element, uia| {
            let kept = uia
                .ranges
                .get(&range_id)
                .cloned()
                .ok_or(AxQueryError::NoValue)?;
            // SAFETY: COM calls on the thread's own objects; endpoints only.
            unsafe { selection_is(element, &kept) }
        })
    }
    fn focus(&mut self, timeout: Duration) -> Result<(), AxQueryError> {
        self.on_element(timeout, |element, _| {
            // SAFETY: COM call on the thread's own element.
            unsafe { element.SetFocus() }.map_err(|_| AxQueryError::Failed)
        })
    }
    fn select(&mut self, range: &SelectionRange, timeout: Duration) -> Result<(), AxQueryError> {
        let range_id = Self::range_id(range)?;
        self.on_element(timeout, move |_, uia| {
            let kept = uia
                .ranges
                .get(&range_id)
                .cloned()
                .ok_or(AxQueryError::NoValue)?;
            // SAFETY: COM call on the thread's own range.
            unsafe { kept.Select() }.map_err(|_| AxQueryError::Failed)
        })
    }
    fn window(&mut self, _timeout: Duration) -> Result<Option<Box<dyn FieldNode>>, AxQueryError> {
        Ok(None)
    }
    fn raise(&mut self, _timeout: Duration) -> Result<(), AxQueryError> {
        Err(AxQueryError::Unsupported)
    }
    fn same_as(&self, other: &dyn FieldNode) -> bool {
        other
            .as_any()
            .downcast_ref::<WindowsFieldNode>()
            .is_some_and(|other| {
                !self.runtime_id.is_empty()
                    && self.runtime_id == other.runtime_id
                    && self.view.pid == other.view.pid
            })
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
