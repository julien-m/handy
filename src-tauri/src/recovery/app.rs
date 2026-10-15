//! Tauri glue of the recovery bubble: installation with the `destination-return` gate open
//! (service, 006 pre-insertion check and outcome consumer, 007 late-read listener), the hooks
//! of a starting dictation, and the production ports (006 slot, session facts, Copy, timers,
//! `paste-error`). Gate closed, nothing is installed and `dev`'s path runs unchanged.
use super::platform;
use super::ports::{CopyError, CopyPort, FallbackPort, SessionFacts, TimerPort, TranscriptSource};
use super::service::{RecoveryPorts, RecoveryService};
use super::surface::TauriSurface;
use super::types::RecoveryPlatform;
use crate::destination::context::{ApplicationIdentity, DictationId};
use crate::destination::outcome::{HistoryStatus, InsertionOutcome, TranscriptRef};
use crate::destination::recovery::RecoverySlot;
use crate::destination::snapshot::SelectionView;
use crate::destination::workspace::SystemClock;
use crate::smart_paste::app::{DeliberateCopyError, LateReadListener};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

// @spec FR-002: Installed only with the destination-return gate open — .specs/features/010-recovery-bubble/spec.md#fr-002
/// Install the recovery bubble when the rollout gate opens sessions; after 006's install.
pub fn install(app: &AppHandle) {
    if !crate::destination::rollout::Rollout::CURRENT.opens_sessions() {
        return;
    }
    let service = RecoveryService::new(ports(app), RecoveryPlatform::current());
    app.manage(Arc::clone(&service));
    crate::destination::app::install_recovery(app, service.clone(), service.clone());
    let listener = Arc::downgrade(&service);
    app.manage(LateReadListener(Box::new(move |diagnostic| {
        if let Some(service) = listener.upgrade() {
            let dictation = DictationId(diagnostic.dictation_id.0);
            service.late_read(dictation, diagnostic.transaction_id.0);
        }
    })));
}

fn ports(app: &AppHandle) -> RecoveryPorts {
    RecoveryPorts {
        surface: Arc::new(TauriSurface::new(app.clone())),
        transcripts: Arc::new(SlotTranscripts(app.clone())),
        facts: Arc::new(DestinationFacts(app.clone())),
        copy: Arc::new(DeliberateCopy(app.clone())),
        timers: Arc::new(RuntimeTimers),
        fallback: Arc::new(PasteError(app.clone())),
        focus: Mutex::new(platform::focused_element_port()),
        clock: Arc::new(SystemClock),
    }
}

/// The installed service, if any.
pub fn service(app: &AppHandle) -> Option<Arc<RecoveryService>> {
    app.try_state::<Arc<RecoveryService>>()
        .map(|state| Arc::clone(state.inner()))
}

// @spec AC-005: New dictation closes the bubble before the indicator — .specs/features/010-recovery-bubble/spec.md#ac-005
/// A dictation is starting: close the bubble before the indicator shows, and freeze "Show the
/// bubble when no text field is focused" from the settings read at this start.
pub fn dictation_starting(app: &AppHandle, settings: &crate::settings::AppSettings) {
    if let Some(service) = service(app) {
        service.dictation_starting(super::setting::enabled(settings));
    }
}
/// The recording started: bind the frozen setting to its dictation.
pub fn dictation_started(app: &AppHandle) {
    let snapshot = crate::destination::app::manager(app).and_then(|manager| manager.snapshot());
    if let (Some(service), Some(snapshot)) = (service(app), snapshot) {
        service.dictation_started(snapshot.dictation_id);
    }
}

/// 006's recovery slot.
struct SlotTranscripts(AppHandle);
impl TranscriptSource for SlotTranscripts {
    fn read(&self, reference: TranscriptRef) -> Option<String> {
        let slot = self.0.try_state::<Arc<RecoverySlot>>()?;
        slot.read(reference).ok()
    }
    fn release(&self, reference: TranscriptRef) {
        if let Some(slot) = self.0.try_state::<Arc<RecoverySlot>>() {
            slot.release(reference);
        }
    }
}

/// Names from 006's session (still current while consumers run), then the workspace.
struct DestinationFacts(AppHandle);
impl SessionFacts for DestinationFacts {
    fn app_name(&self, outcome: &InsertionOutcome) -> Option<String> {
        let manager = crate::destination::app::manager(&self.0)?;
        let session = manager.snapshot().filter(|snapshot| {
            snapshot.dictation_id == outcome.dictation_id
                && snapshot.destination_generation == outcome.destination_generation
        });
        if let Some(SelectionView::Application { display_name, .. }) =
            session.map(|snapshot| snapshot.selection)
        {
            return Some(display_name);
        }
        let identity: &ApplicationIdentity = outcome.target.as_ref()?;
        let instances = manager.workspace().instances(identity);
        instances.into_iter().next().map(|app| app.display_name)
    }
    fn decided_at(&self, outcome: &InsertionOutcome) -> Option<Instant> {
        crate::destination::app::manager(&self.0)?.recovery_decided_at(outcome.dictation_id)
    }
}

// @spec AC-005: Copy through 007's actor, else dev's clipboard write — .specs/features/010-recovery-bubble/spec.md#ac-005
/// 007's actor when it runs, otherwise the clipboard write of `dev`'s "copy to clipboard".
struct DeliberateCopy(AppHandle);
impl CopyPort for DeliberateCopy {
    fn write(&self, transcript: &str) -> Result<(), CopyError> {
        match crate::smart_paste::app::deliberate_copy(&self.0, transcript) {
            Some(written) => written.map_err(|error| match error {
                DeliberateCopyError::Pasteboard(_) => CopyError::Pasteboard,
                DeliberateCopyError::ActorStopped => CopyError::ActorStopped,
                DeliberateCopyError::TimedOut => CopyError::TimedOut,
            }),
            None => crate::clipboard::write_text_to_clipboard(&self.0, transcript)
                .map_err(|_| CopyError::Clipboard),
        }
    }
}

/// Timers on the async runtime (never on the main thread).
struct RuntimeTimers;
impl TimerPort for RuntimeTimers {
    fn after(&self, delay: Duration, task: Box<dyn FnOnce() + Send>) {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            task();
        });
    }
}

/// 006's existing toast.
struct PasteError(AppHandle);
impl FallbackPort for PasteError {
    fn paste_error(&self, history: HistoryStatus) {
        crate::destination::delivery::emit_paste_error(&self.0, history);
    }
}
