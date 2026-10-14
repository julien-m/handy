//! Native changes of the shortcut registrations, prepared before their settings commit and
//! undone when the commit is refused, shared by the settings UI and the CLI writes routed to the
//! running instance (`settings_service::native_changes`), so both reach the same decision:
//!
//! - `bindings`: the shortcut lists (list edits, `handy settings shortcuts`);
//! - `post_process_enabled`: the post-processing toggle (its shortcuts registered or released);
//! - `keyboard_implementation`: the switch, with the lists repaired for the destination.
//!
//! Every change first takes the coordinator's quiescence lease: during a dictation session it
//! is refused with `unavailable` (`shortcut_session_busy`) before anything is registered or
//! written. The lease and a Secure Input reconciliation are held by a [`ChangeWindow`] until the
//! commit path ends, committed or not.

use log::warn;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use super::bindings_change::{changed_actions, compensate_changes, prepare_changes, ActionChange};
use super::edits::{self, BackendRegistry, RefusedShortcut};
use super::keyboard_switch;
use crate::settings::AppSettings;
use crate::settings_service::errors::{
    ErrorReason, SettingsError, SettingsErrorCode, ShortcutDetails,
};
use crate::settings_service::sequence::PreparedChange;
use crate::transcription_coordinator::{QuiescenceLease, TranscriptionCoordinator};

/// How long a change waits for the coordinator's answer. It answers at once between inputs;
/// longer means it is running a start or stop effect (opening the microphone), so a session is
/// beginning or ending and the change is refused as busy.
const LEASE_TIMEOUT: Duration = Duration::from_secs(1);

/// Shortcut refusals the settings UI shows with their system reason (in English); the CLI gets
/// the stable code only.
pub(crate) type RefusalSink = Rc<RefCell<Vec<RefusedShortcut>>>;

/// What a preparation hands to its compensation: the open window and what it changed.
type Held<T> = Rc<RefCell<Option<(ChangeWindow, T)>>>;

/// Open while a registration change is prepared and committed: the coordinator is quiescent
/// and shortcut input gated. Dropping it reconciles the Secure Input shadows with the published
/// settings (committed or, after a refusal, unchanged), then releases the lease.
pub(crate) struct ChangeWindow {
    app: AppHandle,
    _lease: Option<QuiescenceLease>,
}

impl Drop for ChangeWindow {
    fn drop(&mut self) {
        crate::secure_input::reconcile_fallback(&self.app);
    }
}

// @spec FR-002: Changes wait for a quiescent coordinator — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
/// Open a change window for setting `key`.
///
/// # Errors
/// `unavailable` with reason `shortcut_session_busy` while a dictation session exists.
pub(crate) fn open(app: &AppHandle, key: &str) -> Result<ChangeWindow, SettingsError> {
    // Without a coordinator (startup, headless) no session and no shortcut input exist.
    let lease = match app.try_state::<TranscriptionCoordinator>() {
        Some(coordinator) => Some(
            coordinator
                .quiesce(LEASE_TIMEOUT)
                .map_err(|_| SettingsError::session_busy(key))?,
        ),
        None => None,
    };
    Ok(ChangeWindow {
        app: app.clone(),
        _lease: lease,
    })
}

/// Registration refusal of a list change: `validation_failed` naming the action; the system
/// reason goes to the log and to `sink` (the settings UI), never to the CLI envelope.
fn registration_refused(
    action_id: &str,
    reason: &str,
    sink: Option<&RefCell<Option<String>>>,
) -> SettingsError {
    warn!("Shortcut change of '{action_id}' refused by the system: {reason}");
    if let Some(sink) = sink {
        *sink.borrow_mut() = Some(reason.to_string());
    }
    let details = ShortcutDetails {
        action_id: Some(action_id.to_string()),
        ..ShortcutDetails::default()
    };
    SettingsError::shortcut(
        SettingsErrorCode::ValidationFailed,
        Some(ErrorReason::RegistrationRefused),
        details,
    )
}

/// Prepared change of the shortcut lists. A system refusal is also written to `sink`.
pub(crate) fn bindings(
    app: AppHandle,
    sink: Option<Rc<RefCell<Option<String>>>>,
) -> PreparedChange<'static> {
    let held: Held<Vec<ActionChange>> = Rc::default();
    let prepared = Rc::clone(&held);
    let undo_app = app.clone();
    PreparedChange {
        prepare: Box::new(move |before, requested| {
            let window = open(&app, "bindings")?;
            let changes = prepare_bindings(&app, before, requested, sink.as_deref())?;
            *prepared.borrow_mut() = Some((window, changes));
            Ok(())
        }),
        compensate: Box::new(move |before| {
            if let Some((_window, changes)) = held.borrow_mut().take() {
                let preparation = |id: &str| edits::preparation_of(before, id);
                let mut port = BackendRegistry { app: &undo_app };
                compensate_changes(&mut port, before, &changes, &preparation);
            }
        }),
        extend: None,
    }
}

/// Register the difference of every changed list from `before` to `requested`.
///
/// # Errors
/// `validation_failed` (`registration_refused`) naming the refused action; nothing stays
/// changed.
fn prepare_bindings(
    app: &AppHandle,
    before: &AppSettings,
    requested: &AppSettings,
    sink: Option<&RefCell<Option<String>>>,
) -> Result<Vec<ActionChange>, SettingsError> {
    let changes = changed_actions(before, requested);
    let preparation = |id: &str| edits::preparation_of(before, id);
    let mut port = BackendRegistry { app };
    prepare_changes(&mut port, before, &changes, &preparation)
        .map_err(|(action, reason)| registration_refused(&action, &reason, sink))?;
    Ok(changes)
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Prepared change of `post_process_enabled`: its shortcuts are registered or released (see
/// [`edits::toggle_post_process`]); the shortcuts the system refuses stay stored, are logged and
/// written to `sink`. The toggle itself is never refused, except during a session.
pub(crate) fn post_process(app: AppHandle, sink: Option<RefusalSink>) -> PreparedChange<'static> {
    let held: Held<AppSettings> = Rc::default();
    let prepared = Rc::clone(&held);
    let undo_app = app.clone();
    PreparedChange {
        prepare: Box::new(move |before, requested| {
            let window = open(&app, "post_process_enabled")?;
            let enabled = requested.post_process_enabled;
            let (after, refused) = edits::toggle_post_process_on_backend(&app, before, enabled);
            edits::log_refusals(&refused);
            if let Some(sink) = &sink {
                sink.borrow_mut().extend(refused);
            }
            *prepared.borrow_mut() = Some((window, after));
            Ok(())
        }),
        compensate: Box::new(move |before| {
            if let Some((_window, after)) = held.borrow_mut().take() {
                let enabled = before.post_process_enabled;
                edits::toggle_post_process_on_backend(&undo_app, &after, enabled);
            }
        }),
        extend: None,
    }
}

/// Prepared change of `keyboard_implementation`: the lists repaired for the destination are
/// committed with it (`extend`), and every registration moves to the destination before the
/// commit (back again on refusal).
pub(crate) fn keyboard_implementation(app: AppHandle) -> PreparedChange<'static> {
    let held: Held<AppSettings> = Rc::default();
    let prepared = Rc::clone(&held);
    let undo_app = app.clone();
    PreparedChange {
        prepare: Box::new(move |before, requested| {
            let window = open(&app, "keyboard_implementation")?;
            keyboard_switch::switch_registrations(&app, before, requested).map_err(|reason| {
                warn!("Keyboard implementation switch refused: {reason}");
                SettingsError::live_state_refused("keyboard_implementation")
            })?;
            *prepared.borrow_mut() = Some((window, requested.clone()));
            Ok(())
        }),
        compensate: Box::new(move |before| {
            if let Some((_window, requested)) = held.borrow_mut().take() {
                if let Err(reason) =
                    keyboard_switch::switch_registrations(&undo_app, &requested, before)
                {
                    warn!("Could not restore the keyboard implementation: {reason}");
                }
            }
        }),
        extend: Some(Box::new(|settings| {
            keyboard_switch::repair_lists(settings);
            Ok(())
        })),
    }
}
