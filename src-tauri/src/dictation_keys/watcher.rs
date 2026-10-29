//! Observer of the two in-dictation keys (014): one HandyKeys keyboard listener per dictation, in
//! observe-only mode (no key is blocked, so Option, Shift or Control keep working everywhere),
//! started on its own thread once the recording started and stopped as soon as the dictation no
//! longer accepts changes (insertion began, cancelled, finished, or superseded). A lone press of a
//! configured key (`tap.rs`) toggles the dictation's state in the destination manager and the UI
//! is told. Nothing is logged about other keys.
use super::tap::{is_configured_key, KeyInput, ModifierState, Tap, TapDetector};
use super::{parse_key, RETARGET_ACTION, SKIP_ENTER_ACTION};
use crate::destination::context::DictationId;
use crate::destination::manager::DestinationManager;
use handy_keys::{KeyEvent, KeyboardListener, Modifiers};
use log::{debug, warn};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;

/// Count of Handy shortcut events (any action), read by the tap detector.
static SHORTCUT_EVENTS: AtomicU64 = AtomicU64::new(0);
/// How long the observer waits for an event before checking the dictation and pending taps.
const POLL: Duration = Duration::from_millis(10);
/// Wait before confirming a lone press: the backend thread drains its shortcut events every
/// 10 ms, so a shortcut that hid its keys from this observer (Option+Space) is counted by then.
const SHORTCUT_SETTLE: Duration = Duration::from_millis(30);

/// A shortcut of Handy fired (called by the shared shortcut handler).
pub fn note_shortcut_event() {
    SHORTCUT_EVENTS.fetch_add(1, Ordering::SeqCst);
}

/// The keys configured for the two actions, read at the dictation start.
#[derive(Clone, Copy, Debug)]
struct Keys {
    retarget: Option<Modifiers>,
    skip_enter: Option<Modifiers>,
}

/// One dictation's observer.
struct Observer {
    app: AppHandle,
    manager: Arc<DestinationManager>,
    keys: Keys,
    dictation: (DictationId, u64),
}

/// Start the observer of the dictation `dictation_id`/`generation`; returns at once.
pub fn start(app: &AppHandle, dictation_id: DictationId, generation: u64) {
    let Some(manager) = crate::destination::app::manager(app) else {
        return;
    };
    let keys = configured_keys(app);
    if keys.retarget.is_none() && keys.skip_enter.is_none() {
        return;
    }
    let observer = Observer {
        app: app.clone(),
        manager,
        keys,
        dictation: (dictation_id, generation),
    };
    std::thread::spawn(move || observer.run());
}

/// The configured keys; a removed or unreadable one is off (unreadable is logged), the other
/// still works.
fn configured_keys(app: &AppHandle) -> Keys {
    let settings = crate::settings::get_settings(app);
    let key_of = |id: &str| {
        // No key (removed by the user) means the action is off.
        let key = settings.bindings.get(id)?.shortcuts().into_iter().next()?;
        parse_key(&key)
            .map_err(|error| warn!("In-dictation key '{id}' is off: {error}"))
            .ok()
    };
    Keys {
        retarget: key_of(RETARGET_ACTION),
        skip_enter: key_of(SKIP_ENTER_ACTION),
    }
}

impl Observer {
    fn run(self) {
        let listener = match KeyboardListener::new() {
            Ok(listener) => listener,
            Err(error) => {
                warn!("In-dictation keys unavailable for this dictation: {error}");
                return;
            }
        };
        let (id, generation) = self.dictation;
        let mut modifiers = initial_modifier_state();
        let mut detector = TapDetector::default();
        let mut waiting: Option<(Tap, Instant)> = None;
        while self.manager.accepts_keys(id, generation) {
            if waiting.is_some_and(|(_, at)| at.elapsed() >= SHORTCUT_SETTLE) {
                self.confirm(waiting.take());
            }
            let Ok(input) = next_input(&listener, &mut modifiers) else {
                break;
            };
            let seen = SHORTCUT_EVENTS.load(Ordering::SeqCst);
            if let Some(tap) = input.and_then(|input| detector.feed(input, seen)) {
                waiting = Some((tap, Instant::now()));
            }
        }
        debug!("In-dictation keys of dictation {} stopped", id.0);
    }

    /// A lone press is applied only if no Handy shortcut fired since its key went down.
    fn confirm(&self, waiting: Option<(Tap, Instant)>) {
        let Some((tap, _)) = waiting else {
            return;
        };
        if SHORTCUT_EVENTS.load(Ordering::SeqCst) == tap.shortcuts_at_press {
            self.on_tap(tap.key);
        }
    }

    fn on_tap(&self, tapped: Modifiers) {
        // While Secure Input is on, key presses do not reach the observer: a lone modifier
        // cannot be told from Shift used for a capital letter in a password field.
        if crate::secure_input::is_enabled_now() {
            debug!("In-dictation key ignored: Secure Input is on");
            return;
        }
        let (id, generation) = self.dictation;
        let is = |key: Option<Modifiers>| key.is_some_and(|key| is_configured_key(key, tapped));
        let snapshot = if is(self.keys.retarget) {
            self.manager
                .toggle_active_app(id, generation)
                .ok()
                .flatten()
        } else if is(self.keys.skip_enter) {
            self.manager.toggle_skip_enter(id, generation).ok()
        } else {
            None
        };
        if let Some(snapshot) = snapshot {
            debug!(
                "In-dictation key applied to dictation {}: route {:?}, Enter skipped {}",
                id.0, snapshot.source, snapshot.enter_skipped
            );
            crate::destination::app::emit_changed(&self.app, &snapshot);
        }
    }
}

/// The next input within `POLL` (`None` on a timeout or an event without key).
///
/// # Errors
/// The listener stopped (its thread ended); logged.
fn next_input(
    listener: &KeyboardListener,
    modifiers: &mut ModifierState,
) -> Result<Option<KeyInput>, ()> {
    match listener.recv_timeout(POLL) {
        Ok(event) => Ok(input_of(&event, modifiers)),
        Err(handy_keys::Error::Timeout) => Ok(None),
        Err(error) => {
            warn!("In-dictation keys stopped: {error}");
            Err(())
        }
    }
}

/// The detector's view of a listener event, with the modifier direction corrected.
fn input_of(event: &KeyEvent, modifiers: &mut ModifierState) -> Option<KeyInput> {
    match (event.changed_modifier, event.key) {
        (Some(key), _) => Some(KeyInput::Modifier {
            key,
            down: modifiers.is_press(key, event.is_key_down),
        }),
        (None, Some(key)) => Some(KeyInput::Other {
            key,
            down: event.is_key_down,
        }),
        (None, None) => None,
    }
}

/// The modifiers held when the observer starts. macOS: read from the system, and the direction
/// of each change is derived from it (see `ModifierState`); elsewhere the listener reports the
/// direction of each change itself.
fn initial_modifier_state() -> ModifierState {
    #[cfg(target_os = "macos")]
    return ModifierState::new(macos::held_modifiers(), true);
    #[cfg(not(target_os = "macos"))]
    return ModifierState::new(Modifiers::empty(), false);
}

#[cfg(target_os = "macos")]
mod macos {
    use handy_keys::Modifiers;

    /// `kCGEventSourceStateCombinedSessionState`.
    const COMBINED_SESSION_STATE: i32 = 0;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
    }

    /// Device-dependent side bits of the event flags (IOKit `NX_DEVICE*KEYMASK`), and FN.
    const SIDES: [(u64, Modifiers); 9] = [
        (0x0000_0001, Modifiers::CTRL_LEFT),
        (0x0000_2000, Modifiers::CTRL_RIGHT),
        (0x0000_0002, Modifiers::SHIFT_LEFT),
        (0x0000_0004, Modifiers::SHIFT_RIGHT),
        (0x0000_0008, Modifiers::CMD_LEFT),
        (0x0000_0010, Modifiers::CMD_RIGHT),
        (0x0000_0020, Modifiers::OPT_LEFT),
        (0x0000_0040, Modifiers::OPT_RIGHT),
        (0x0080_0000, Modifiers::FN),
    ];

    /// The modifier keys held right now, by side.
    pub fn held_modifiers() -> Modifiers {
        // SAFETY: a pure read of the session's modifier flags, valid on any thread.
        let flags = unsafe { CGEventSourceFlagsState(COMBINED_SESSION_STATE) };
        SIDES
            .iter()
            .filter(|(mask, _)| flags & mask != 0)
            .fold(Modifiers::empty(), |held, (_, side)| held | *side)
    }
}
