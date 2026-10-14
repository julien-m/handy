//! Production ports of the smart-paste actor on macOS: the system clock,
//! accessibility, the general pasteboard with its snapshot helper (this
//! executable, `--smart-paste-snapshot`), Text Input Sources, key events,
//! the registered destination checks (006) and the late-read sink (010).

use std::sync::Arc;

use super::actor::SmartPasteActor;
use super::clock::{Clock, SystemClock};
use super::coordinator::SmartPastePorts;
use super::destination_bridge::{DestinationChecks, RegisteredDestinations};
use super::macos_ax::MacAxTextPort;
use super::macos_keyboard::{MacInputSources, MacKeyEvents};
use super::macos_pasteboard::MacPasteboard;
use super::request::DiagnosticSink;
use super::snapshot_helper::HelperCommand;

/// Starts the actor over the production ports. Side effects: one thread;
/// no pasteboard access and no helper process before the first insertion.
///
/// # Errors
/// The executable path or the thread cannot be obtained.
pub fn spawn_actor(
    checks: Arc<DestinationChecks>,
    diagnostics: Box<dyn DiagnosticSink>,
) -> std::io::Result<SmartPasteActor> {
    let executable = std::env::current_exe()?;
    SmartPasteActor::spawn(move |sink| {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        // `None` names the general pasteboard, the one dictation pastes from.
        let helper = HelperCommand {
            executable,
            pasteboard_name: None,
        };
        let pasteboard = MacPasteboard::new(None, helper, sink, Arc::clone(&clock));
        SmartPastePorts {
            clock,
            ax: Box::new(MacAxTextPort),
            pasteboard: Box::new(pasteboard),
            input_sources: Box::new(MacInputSources),
            keys: Box::new(MacKeyEvents),
            destination: Box::new(RegisteredDestinations(checks)),
            diagnostics,
        }
    })
}
