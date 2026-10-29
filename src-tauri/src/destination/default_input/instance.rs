//! The destination manager's side of feature 013: its ports, and the running instance of the
//! current dictation's destination that a "Remember" applies to.
use super::capture::DestinationInstance;
use super::services::DefaultInputServices;
use crate::destination::context::DictationId;
use crate::destination::manager::DestinationManager;
use crate::destination::session::{Selection, SelectionRefusal};
use crate::destination::workspace::{resolve_instance, Resolution};
use std::sync::Arc;

impl DestinationManager {
    /// 013's ports.
    pub fn default_inputs(&self) -> Arc<DefaultInputServices> {
        Arc::clone(&self.default_inputs)
    }
    #[must_use]
    /// The same manager with 013's ports (installed at startup on macOS).
    pub fn with_default_inputs(mut self, services: DefaultInputServices) -> Self {
        self.default_inputs = Arc::new(services);
        self
    }
    // @spec AC-001: Remember applies to the destination instance — .specs/features/013-default-input-per-app/spec.md#ac-001
    /// The running instance of the destination of `dictation_id` at `version`, while the menu
    /// still accepts choices. Native reads run outside the session lock.
    ///
    /// # Errors
    /// `Stale` or `Frozen` as for a menu choice; `Unavailable` without an application
    /// destination or when its instance is not running (or ambiguous).
    pub fn destination_instance(
        &self,
        dictation_id: DictationId,
        version: u64,
    ) -> Result<DestinationInstance, SelectionRefusal> {
        let selection = crate::destination::menu::still_open(&self.lock(), dictation_id, version)?;
        let Selection::Application(destination) = selection else {
            return Err(SelectionRefusal::Unavailable);
        };
        let instances = self.workspace.instances(&destination.identity);
        match resolve_instance(&destination.identity, destination.captured_pid, &instances) {
            Resolution::Running(app) => Ok(DestinationInstance {
                identity: app.identity,
                display_name: destination.display_name,
                pid: app.pid,
            }),
            Resolution::Closed | Resolution::Ambiguous => Err(SelectionRefusal::Unavailable),
        }
    }
}
