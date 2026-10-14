//! Transcription lifecycle coordinator: one thread owns every start/stop
//! decision for the transcription actions, whichever shortcut, signal or
//! CLI flag produced the input.

use crate::actions::action_for;
use crate::managers::audio::AudioRecordingManager;
use crate::settings::ShortcutActivation;
use log::{debug, error, warn};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

mod quiescence;
use quiescence::{InputGate, LeaseRequest};
pub use quiescence::{QuiescenceLease, SessionBusy};

const DEBOUNCE: Duration = Duration::from_millis(30);
const RELEASE_GRACE: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PttAction {
    Passthrough,
    DeferRelease,
    CancelRelease,
}

/// Identity of one physical shortcut: the action it triggers and which of
/// the action's shortcuts it is. Hold modes match releases on this pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShortcutKey<'a> {
    action_id: &'a str,
    shortcut_id: &'a str,
}

/// A key-up deferred by `RELEASE_GRACE` so a synthesized X11 auto-repeat
/// press can cancel it (#1539). When the grace elapses the hold is resolved
/// by [`CoordinatorState::finish_hold`] (recording) or
/// [`CoordinatorState::finish_pending_hold`] (press remembered while busy).
struct PendingRelease {
    binding_id: String,
    shortcut_id: String,
    deadline: Instant,
    /// When the key actually went up. The hold duration is measured to this
    /// instant, not to the grace expiry.
    released_at: Instant,
    /// Holds at least this long stop recording; shorter ones lock it on.
    /// Push-to-talk passes zero so every release stops.
    hold_threshold: Duration,
}

impl PendingRelease {
    fn key(&self) -> ShortcutKey<'_> {
        ShortcutKey {
            action_id: &self.binding_id,
            shortcut_id: &self.shortcut_id,
        }
    }
}

/// A press that arrived while the pipeline was still busy processing the
/// previous transcription. Toggle-style triggers (SIGUSR2, CLI flags, some
/// pedal setups) flip state on every edge, so dropping a busy press desyncs
/// the parity: the next edge starts a recording nobody will ever stop.
struct PendingPress {
    binding_id: String,
    shortcut_id: String,
    hotkey_string: String,
    /// The real key-down time, so a hold that straddles the drain is still
    /// measured from when the user pressed, not from when recording began.
    pressed_at: Instant,
    /// The recording will start locked on when the pipeline drains: set from
    /// the start for toggle, and for hold-or-toggle once the key came back up
    /// within the threshold (a tap). An unlocked pending press is a key we
    /// believe is still held.
    locked: bool,
    /// Remembered in toggle mode: any shortcut of the action owns it.
    owned_by_action: bool,
}

impl PendingPress {
    fn remembered(&self) -> Remembered {
        if self.locked {
            Remembered::Locked
        } else {
            Remembered::Held
        }
    }

    fn key(&self) -> ShortcutKey<'_> {
        ShortcutKey {
            action_id: &self.binding_id,
            shortcut_id: &self.shortcut_id,
        }
    }

    // @spec AC-003: Busy press owned by action (toggle) or shortcut (hold) — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-003
    /// Whether `input` belongs to this remembered press. Business rule: a
    /// press remembered in toggle mode belongs to the action, and a toggle
    /// input (keyboard or external trigger) acts on the action, so any of its
    /// shortcuts flips it; otherwise the press belongs to the pressed
    /// shortcut, as in the hold modes.
    fn claims(&self, input: &InputEvent) -> bool {
        if self.binding_id != input.binding_id {
            return false;
        }
        self.owned_by_action
            || input.mode == ShortcutActivation::Toggle
            || self.shortcut_id == input.shortcut_id
    }
}

/// What kind of press is already waiting for the pipeline to drain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Remembered {
    /// The key is still down as far as we know.
    Held,
    /// A toggle press or a classified tap: it will start a locked session.
    Locked,
}

/// Bookkeeping for the key press that started the current recording.
struct Hold {
    pressed_at: Instant,
    /// Recording outlives the key: the next press stops it, releases are
    /// ignored. Always set for toggle; set for hold-or-toggle once a release
    /// has been classified as a tap.
    locked: bool,
    /// The shortcut that started the session. Only its release ends a held
    /// session.
    shortcut_id: String,
    /// Hotkey text of the starting input, reused by the stop effect.
    hotkey_string: String,
}

/// What to do with an input that arrives while the pipeline is busy
/// (`Stage::Processing`). `remembered` is the press for the same binding
/// already waiting for the pipeline to drain, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BusyAction {
    /// Ignore the input entirely.
    Ignore,
    /// Remember the press; start recording when the pipeline finishes.
    Remember,
    /// This press cancels a previously remembered press: two presses during
    /// one busy window net to no-op, exactly as a press stops a locked
    /// session once recording.
    Forget,
}

fn classify_busy_input(
    is_pressed: bool,
    mode: ShortcutActivation,
    remembered: Option<Remembered>,
) -> BusyAction {
    use ShortcutActivation::*;
    match (mode, is_pressed, remembered) {
        // Toggle: presses alternate remember/forget to preserve parity.
        (Toggle, true, Some(_)) => BusyAction::Forget,
        (Toggle, true, None) => BusyAction::Remember,
        // Toggle mode ignores releases.
        (Toggle, false, _) => BusyAction::Ignore,
        // Hold modes: a press while busy means the user is holding the key —
        // start as soon as the pipeline drains. A press on a queued tap stops
        // it (parity); a press while the key is already down is a repeat.
        (PushToTalk | HoldOrToggle, true, None) => BusyAction::Remember,
        (PushToTalk | HoldOrToggle, true, Some(Remembered::Locked)) => BusyAction::Forget,
        (PushToTalk | HoldOrToggle, true, Some(Remembered::Held)) => BusyAction::Ignore,
        // Releases of a held pending press are deferred by the grace window
        // before reaching here and resolved by `finish_pending_hold`; any
        // other release (no press remembered, or already locked) is noise.
        (PushToTalk | HoldOrToggle, false, _) => BusyAction::Ignore,
    }
}

/// Pipeline lifecycle.
#[derive(Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Recording(String), // binding_id
    Processing,
}

/// A keyboard/signal edge for a transcribe binding.
struct InputEvent {
    binding_id: String,
    /// Which shortcut of the action produced the edge (the stored shortcut
    /// text, or the trigger name for external inputs).
    shortcut_id: String,
    hotkey_string: String,
    is_pressed: bool,
    mode: ShortcutActivation,
    /// Hold-or-toggle: minimum press duration that counts as a hold.
    hold_threshold: Duration,
    /// External triggers (SIGUSR2, CLI flags) rather than physical keys.
    /// They fire on every edge by design and must never be debounced —
    /// dropping one desyncs toggle parity and wedges recording on.
    external: bool,
}

impl InputEvent {
    /// The hold duration at or above which a release stops recording.
    fn effective_hold_threshold(&self) -> Duration {
        match self.mode {
            ShortcutActivation::HoldOrToggle => self.hold_threshold,
            // Every release stops; toggle never defers releases at all.
            ShortcutActivation::PushToTalk | ShortcutActivation::Toggle => Duration::ZERO,
        }
    }

    fn key(&self) -> ShortcutKey<'_> {
        ShortcutKey {
            action_id: &self.binding_id,
            shortcut_id: &self.shortcut_id,
        }
    }
}

/// A side effect decided by [`CoordinatorState`]; the coordinator thread is
/// the only executor. Keeping decisions pure lets tests drive the exact
/// production transitions without a Tauri `AppHandle` or real timers.
#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Start {
        binding_id: String,
        hotkey_string: String,
    },
    Stop {
        binding_id: String,
        hotkey_string: String,
    },
}

/// Commands processed sequentially by the coordinator thread.
enum Command {
    Input(InputEvent),
    Cancel {
        recording_was_active: bool,
    },
    ProcessingFinished,
    /// A shortcut registration change asks for a quiescence lease (see `quiescence`).
    Quiesce(Arc<LeaseRequest>),
    /// A quiescence lease was released.
    Resume,
    /// The recording's input was lost (feature 001): end it through the normal stop.
    InputLost,
}

/// Decide whether a key-up should be deferred (so auto-repeat can cancel it)
/// or a key-down cancels a deferred release. `hold_to_talk` is whether a
/// release currently ends the session: true for push-to-talk and for an
/// unlocked hold-or-toggle session, false for toggle and a locked session.
/// `held_binding` is the shortcut whose key we believe is down — the one
/// recording, or the one remembered while the pipeline is busy.
fn classify_ptt_event<K: PartialEq + ?Sized>(
    pending_release_binding: Option<&K>,
    is_pressed: bool,
    hold_to_talk: bool,
    binding_id: &K,
    held_binding: Option<&K>,
) -> PttAction {
    if !hold_to_talk {
        return PttAction::Passthrough;
    }

    if is_pressed {
        if pending_release_binding == Some(binding_id) {
            PttAction::CancelRelease
        } else {
            PttAction::Passthrough
        }
    } else if held_binding == Some(binding_id) && pending_release_binding.is_none() {
        PttAction::DeferRelease
    } else {
        PttAction::Passthrough
    }
}

/// Pure lifecycle state machine: owns every transition decision (release
/// grace, hold-vs-tap classification, debounce, busy-pipeline
/// remember/forget, cancel, drain). Produces [`Effect`]s instead of touching
/// the app, so unit tests exercise the real production logic.
///
/// All three activation modes run through one machine. A recording starts on
/// key-down in every mode; what differs is how it ends:
///
/// * push-to-talk — every release stops (hold threshold of zero)
/// * toggle — releases are ignored, the next press stops (locked from the start)
/// * hold-or-toggle — a release after a long hold stops; a release after a
///   short tap locks the session, and the next press stops
///
/// An action can own several shortcuts. A locked session (toggle, or a
/// classified tap) belongs to the action: any of its shortcuts stops it. A
/// held session belongs to the starting shortcut: only its release ends it
/// and the action's other shortcuts are ignored meanwhile.
struct CoordinatorState {
    stage: Stage,
    hold: Option<Hold>,
    last_press: Option<Instant>,
    pending_release: Option<PendingRelease>,
    pending_press: Option<PendingPress>,
}

impl CoordinatorState {
    fn new() -> Self {
        Self {
            stage: Stage::Idle,
            hold: None,
            last_press: None,
            pending_release: None,
            pending_press: None,
        }
    }

    /// Whether no session exists: idle, nothing held, remembered or deferred. Only then may a
    /// shortcut registration change (it could remove the shortcut a session still needs).
    fn is_quiescent(&self) -> bool {
        self.stage == Stage::Idle
            && self.hold.is_none()
            && self.pending_press.is_none()
            && self.pending_release.is_none()
    }

    /// Deadline of the deferred release, if any — drives `recv_timeout`.
    fn grace_deadline(&self) -> Option<Instant> {
        self.pending_release.as_ref().map(|p| p.deadline)
    }

    /// Whether the current session (recording, or remembered for the drain)
    /// outlives the key, so releases are ignored and the next press ends it.
    fn is_locked(&self) -> bool {
        self.hold.as_ref().is_some_and(|h| h.locked)
            || self.pending_press.as_ref().is_some_and(|p| p.locked)
    }

    /// The shortcut that started the live recording, if any.
    fn recording_owner(&self) -> Option<ShortcutKey<'_>> {
        match (&self.stage, &self.hold) {
            (Stage::Recording(id), Some(hold)) => Some(ShortcutKey {
                action_id: id,
                shortcut_id: &hold.shortcut_id,
            }),
            _ => None,
        }
    }

    /// The shortcut whose key we believe is down: the one recording, or the
    /// one remembered while the pipeline is busy.
    fn held_key(&self) -> Option<ShortcutKey<'_>> {
        match &self.stage {
            Stage::Recording(_) => self.recording_owner(),
            Stage::Processing => self.pending_press.as_ref().map(PendingPress::key),
            Stage::Idle => None,
        }
    }

    // @spec FR-002: Mode-dependent session ownership — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
    fn on_input(&mut self, input: InputEvent, now: Instant) -> Option<Effect> {
        let hold_to_talk = input.mode != ShortcutActivation::Toggle && !self.is_locked();
        if self.absorb_release_edge(&input, hold_to_talk, now) {
            return None;
        }
        if self.is_debounced(&input, now) {
            return None;
        }
        // A busy pipeline can't accept lifecycle changes now: classify the
        // input against any already-remembered press instead of dropping it
        // silently.
        if let Stage::Processing = self.stage {
            self.on_busy_input(input, now);
            return None;
        }
        if input.is_pressed {
            return self.on_press(input, now);
        }
        self.on_release(input, hold_to_talk, now)
    }

    /// Release-grace handling for the hold modes (#1539): defers a key-up of
    /// the held shortcut, or lets an auto-repeat press cancel the deferred
    /// one. Returns whether the edge was absorbed.
    fn absorb_release_edge(
        &mut self,
        input: &InputEvent,
        hold_to_talk: bool,
        now: Instant,
    ) -> bool {
        let pending = self.pending_release.as_ref().map(PendingRelease::key);
        let held = self.held_key();
        match classify_ptt_event(
            pending.as_ref(),
            input.is_pressed,
            hold_to_talk,
            &input.key(),
            held.as_ref(),
        ) {
            PttAction::CancelRelease => {
                self.pending_release = None;
                true
            }
            PttAction::DeferRelease => {
                self.defer_release(input, now);
                true
            }
            PttAction::Passthrough => false,
        }
    }

    fn defer_release(&mut self, input: &InputEvent, now: Instant) {
        self.pending_release = Some(PendingRelease {
            hold_threshold: input.effective_hold_threshold(),
            binding_id: input.binding_id.clone(),
            shortcut_id: input.shortcut_id.clone(),
            deadline: now + RELEASE_GRACE,
            released_at: now,
        });
    }

    /// Debounce rapid-fire press events (key repeat / double-tap). The
    /// window is global, not per shortcut. Releases in the hold modes are
    /// deferred before this to absorb X11 auto-repeat. External triggers are
    /// exempt: each one is a deliberate edge from the user's own integration,
    /// and dropping it desyncs toggle parity.
    fn is_debounced(&mut self, input: &InputEvent, now: Instant) -> bool {
        if !input.is_pressed || input.external {
            return false;
        }
        if self
            .last_press
            .is_some_and(|t| now.duration_since(t) < DEBOUNCE)
        {
            debug!("Debounced press for '{}'", input.binding_id);
            return true;
        }
        self.last_press = Some(now);
        false
    }

    fn on_busy_input(&mut self, input: InputEvent, now: Instant) {
        // Only one press can be remembered. Once a shortcut has claimed it,
        // inputs it does not claim are ignored — the same rule as a different
        // binding pressed while recording — rather than silently replacing
        // the remembered press and breaking its parity.
        if let Some(pending) = &self.pending_press {
            if !pending.claims(&input) {
                debug!(
                    "Ignoring input for '{}': '{}' is already pending",
                    input.binding_id, pending.binding_id
                );
                return;
            }
        }
        let remembered = self.pending_press.as_ref().map(|p| p.remembered());
        match classify_busy_input(input.is_pressed, input.mode, remembered) {
            BusyAction::Remember => self.remember_press(input, now),
            BusyAction::Forget => {
                debug!("Forgetting remembered press for '{}'", input.binding_id);
                self.pending_press = None;
            }
            BusyAction::Ignore => {
                debug!("Ignoring input for '{}': pipeline busy", input.binding_id);
            }
        }
    }

    fn remember_press(&mut self, input: InputEvent, now: Instant) {
        debug!(
            "Remembering press for '{}': pipeline busy",
            input.binding_id
        );
        let is_toggle = input.mode == ShortcutActivation::Toggle;
        self.pending_press = Some(PendingPress {
            // Toggle never ends on a release: locked from the start.
            locked: is_toggle,
            owned_by_action: is_toggle,
            binding_id: input.binding_id,
            shortcut_id: input.shortcut_id,
            hotkey_string: input.hotkey_string,
            pressed_at: now,
        });
    }

    fn on_press(&mut self, input: InputEvent, now: Instant) -> Option<Effect> {
        match &self.stage {
            Stage::Idle => {
                // Toggle never ends on a release: locked from the start.
                let locked = input.mode == ShortcutActivation::Toggle;
                Some(self.begin_recording(
                    input.binding_id,
                    input.shortcut_id,
                    input.hotkey_string,
                    now,
                    locked,
                ))
            }
            Stage::Recording(id) if id == &input.binding_id => {
                self.on_press_while_recording(&input)
            }
            _ => {
                debug!(
                    "Ignoring press for '{}': another binding is recording",
                    input.binding_id
                );
                None
            }
        }
    }

    /// A press of the recording action. A locked session ends on the next
    /// press of any shortcut of its action. In toggle mode every press ends
    /// it, even if the recording began under a hold mode (the setting
    /// changed mid-recording) — otherwise nothing but Escape could stop it.
    fn on_press_while_recording(&mut self, input: &InputEvent) -> Option<Effect> {
        if self.is_locked() || input.mode == ShortcutActivation::Toggle {
            return self.stop_recording();
        }
        // The starting key is still held (its release will end this
        // recording): a repeat, or another shortcut of the action.
        debug!("Ignoring press for '{}': key is held", input.binding_id);
        None
    }

    /// A release that was not deferred (one is already pending for this
    /// shortcut): resolve it immediately rather than dropping it. Only the
    /// release of the shortcut that started the session counts.
    fn on_release(
        &mut self,
        input: InputEvent,
        hold_to_talk: bool,
        now: Instant,
    ) -> Option<Effect> {
        if !hold_to_talk || self.recording_owner() != Some(input.key()) {
            return None;
        }
        let threshold = input.effective_hold_threshold();
        self.finish_hold(now, threshold)
    }

    /// The `RELEASE_GRACE` window elapsed with no cancelling press arriving:
    /// resolve the deferred release against whatever that binding's key was
    /// holding — the live recording, or a press remembered while busy.
    fn on_grace_expired(&mut self) -> Option<Effect> {
        let pending = self.pending_release.take()?;
        if self.recording_owner() == Some(pending.key()) {
            return self.finish_hold(pending.released_at, pending.hold_threshold);
        }
        if let Stage::Processing = self.stage {
            self.finish_pending_hold(&pending);
        }
        None
    }

    /// A press remembered while the pipeline was busy has been released for
    /// real, still before the drain. A completed hold has nothing left to
    /// start; a tap queues a locked session so the drain starts it — the
    /// same hold-vs-tap rule as [`CoordinatorState::finish_hold`].
    fn finish_pending_hold(&mut self, release: &PendingRelease) {
        let Some(pending) = self
            .pending_press
            .as_mut()
            .filter(|p| p.key() == release.key())
        else {
            return;
        };
        let held = release
            .released_at
            .saturating_duration_since(pending.pressed_at);
        if held < release.hold_threshold {
            debug!(
                "Tap ({held:?}) for '{}' while busy: will start locked on when the pipeline drains",
                release.binding_id
            );
            pending.locked = true;
        } else {
            debug!(
                "Forgetting remembered press for '{}': released after a {held:?} hold while busy",
                release.binding_id
            );
            self.pending_press = None;
        }
    }

    /// The key that started the current recording has been released for real.
    /// A hold at least `threshold` long stops recording; anything shorter was a
    /// tap, which locks the session on until the next press.
    fn finish_hold(&mut self, released_at: Instant, threshold: Duration) -> Option<Effect> {
        let held = self
            .hold
            .as_ref()
            .map(|h| released_at.saturating_duration_since(h.pressed_at))
            // No hold bookkeeping means we cannot tell a tap from a hold;
            // stopping is the safe reading (it is what push-to-talk always did).
            .unwrap_or(Duration::MAX);
        if held >= threshold {
            return self.stop_recording();
        }
        if let Some(hold) = &mut self.hold {
            debug!("Tap ({held:?}): recording locked on until the next press");
            hold.locked = true;
        }
        None
    }

    fn on_cancel(&mut self, recording_was_active: bool) {
        self.pending_release = None;
        // An explicit cancel abandons any remembered start too — the user
        // asked for silence, not a deferred recording.
        self.pending_press = None;
        // Don't reset during processing — wait for the pipeline to finish.
        if !matches!(self.stage, Stage::Processing)
            && (recording_was_active || matches!(self.stage, Stage::Recording(_)))
        {
            self.stage = Stage::Idle;
            self.hold = None;
        }
    }

    // @spec FR-004: A lost input ends the recording once — .specs/features/001-microphone-priority/spec.md#fr-004
    /// The recording's input was lost: stop it once through the normal stop effect, so its
    /// captured audio is transcribed. The held key's later release finds no session (no-op);
    /// a later press starts a new one.
    fn on_input_lost(&mut self) -> Option<Effect> {
        self.pending_release = None;
        self.stop_recording()
    }

    fn on_processing_finished(&mut self) -> Option<Effect> {
        self.stage = Stage::Idle;
        self.hold = None;
        let pending = self.pending_press.take()?;
        debug!(
            "Pipeline drained; starting remembered press for '{}'",
            pending.binding_id
        );
        Some(self.begin_recording(
            pending.binding_id,
            pending.shortcut_id,
            pending.hotkey_string,
            pending.pressed_at,
            pending.locked,
        ))
    }

    /// Reconcile the optimistic `Stage::Recording` after the executor reports
    /// whether recording actually began (microphone access can be denied).
    fn on_start_result(&mut self, binding_id: &str, started: bool) {
        if !started && matches!(&self.stage, Stage::Recording(id) if id == binding_id) {
            self.stage = Stage::Idle;
            self.hold = None;
        }
    }

    /// Optimistic transition to `Recording`, owned by `shortcut_id`; rolled
    /// back via [`CoordinatorState::on_start_result`] if the effect fails to
    /// start recording for real.
    fn begin_recording(
        &mut self,
        binding_id: String,
        shortcut_id: String,
        hotkey_string: String,
        pressed_at: Instant,
        locked: bool,
    ) -> Effect {
        self.stage = Stage::Recording(binding_id.clone());
        self.hold = Some(Hold {
            pressed_at,
            locked,
            shortcut_id,
            hotkey_string: hotkey_string.clone(),
        });
        Effect::Start {
            binding_id,
            hotkey_string,
        }
    }

    /// Stop the live recording with the metadata of the input that started
    /// it, whichever shortcut of the action ends it.
    fn stop_recording(&mut self) -> Option<Effect> {
        let Stage::Recording(binding_id) = &self.stage else {
            return None;
        };
        let binding_id = binding_id.clone();
        self.stage = Stage::Processing;
        let hotkey_string = self
            .hold
            .take()
            .map(|hold| hold.hotkey_string)
            .unwrap_or_default();
        Some(Effect::Stop {
            binding_id,
            hotkey_string,
        })
    }
}

/// Serialises all transcription lifecycle events through a single thread
/// to eliminate race conditions between keyboard shortcuts, signals, and
/// the async transcribe-paste pipeline. The thread is a thin shell: it
/// transports commands to the pure [`CoordinatorState`] and executes the
/// returned [`Effect`]s.
pub struct TranscriptionCoordinator {
    tx: Sender<Command>,
}

/// Whether `id` is one of the actions driven by the coordinator: the two dictation actions and
/// the prompt shortcuts (005), which follow the same global activation mode.
pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe"
        || id == "transcribe_with_post_process"
        || crate::shortcut::prompt_targets::prompt_of(id).is_some()
}

impl TranscriptionCoordinator {
    /// Spawns the coordinator thread. It runs until the coordinator is
    /// dropped (channel closed).
    pub fn new(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_coordinator_loop(&app, &rx);
            }));
            if let Err(e) = result {
                error!("Transcription coordinator panicked: {e:?}");
            }
        });

        Self { tx }
    }

    /// Send a keyboard input event for one shortcut of a transcribe binding.
    /// `shortcut_id` names which of the action's shortcuts fired;
    /// `hold_threshold` only matters for [`ShortcutActivation::HoldOrToggle`].
    pub fn send_input(
        &self,
        binding_id: &str,
        shortcut_id: &str,
        hotkey_string: &str,
        is_pressed: bool,
        mode: ShortcutActivation,
        hold_threshold: Duration,
    ) {
        self.send(InputEvent {
            binding_id: binding_id.to_string(),
            shortcut_id: shortcut_id.to_string(),
            hotkey_string: hotkey_string.to_string(),
            is_pressed,
            mode,
            hold_threshold,
            external: false,
        });
    }

    /// Send an external trigger (SIGUSR2, CLI flag). Always a toggle press,
    /// always exempt from debounce — see [`InputEvent::external`].
    pub fn send_external_input(&self, binding_id: &str, source: &str) {
        self.send(InputEvent {
            binding_id: binding_id.to_string(),
            shortcut_id: source.to_string(),
            hotkey_string: source.to_string(),
            is_pressed: true,
            mode: ShortcutActivation::Toggle,
            hold_threshold: Duration::ZERO,
            external: true,
        });
    }

    fn send(&self, input: InputEvent) {
        if self.tx.send(Command::Input(input)).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }

    /// Tells the coordinator that the recording was cancelled.
    pub fn notify_cancel(&self, recording_was_active: bool) {
        if self
            .tx
            .send(Command::Cancel {
                recording_was_active,
            })
            .is_err()
        {
            warn!("Transcription coordinator channel closed");
        }
    }

    /// Tells the coordinator that the recording's input was lost.
    pub fn notify_input_lost(&self) {
        if self.tx.send(Command::InputLost).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }

    /// Tells the coordinator that the transcription pipeline drained.
    pub fn notify_processing_finished(&self) {
        if self.tx.send(Command::ProcessingFinished).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }

    // @spec FR-002: Changes wait for a quiescent coordinator — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
    /// Quiescence lease for a shortcut registration change, answered by the coordinator thread
    /// in input order (see `quiescence`). Blocks the caller (never the coordinator) up to
    /// `timeout`.
    ///
    /// # Errors
    /// `SessionBusy` while a session exists, or when the coordinator does not answer in time
    /// (it is then running a start or stop effect).
    pub fn quiesce(&self, timeout: Duration) -> Result<QuiescenceLease, SessionBusy> {
        let request = LeaseRequest::new();
        if self
            .tx
            .send(Command::Quiesce(Arc::clone(&request)))
            .is_err()
        {
            // No coordinator thread, so no session and no input: nothing to protect.
            warn!("Transcription coordinator channel closed");
            return Ok(QuiescenceLease::new(self.tx.clone()));
        }
        if request.wait(timeout) {
            Ok(QuiescenceLease::new(self.tx.clone()))
        } else {
            Err(SessionBusy)
        }
    }
}

/// Coordinator thread body: executes every command until the channel
/// closes.
fn run_coordinator_loop(app: &AppHandle, rx: &mpsc::Receiver<Command>) {
    let mut state = CoordinatorState::new();
    let mut gate = InputGate::default();
    while let Some(cmd) = next_command(app, &mut state, rx) {
        if let Some(effect) = dispatch(&mut state, &mut gate, cmd, Instant::now()) {
            run_effect(app, &mut state, effect);
        }
    }
    debug!("Transcription coordinator exited");
}

/// One command applied to the state machine and the input gate, in arrival order.
fn dispatch(
    state: &mut CoordinatorState,
    gate: &mut InputGate,
    cmd: Command,
    now: Instant,
) -> Option<Effect> {
    match cmd {
        Command::Input(input) if !gate.admits(input.external) => {
            debug!("Shortcut input gated during a registration change");
            None
        }
        Command::Input(input) => state.on_input(input, now),
        Command::Cancel {
            recording_was_active,
        } => {
            state.on_cancel(recording_was_active);
            None
        }
        Command::ProcessingFinished => state.on_processing_finished(),
        Command::Quiesce(request) => {
            gate.answer(&request, state.is_quiescent());
            None
        }
        Command::Resume => {
            gate.release();
            None
        }
        Command::InputLost => state.on_input_lost(),
    }
}

/// Waits for the next command. While a release is deferred, the wait ends
/// at its grace deadline and the release is resolved first. Returns `None`
/// once the channel is closed.
fn next_command(
    app: &AppHandle,
    state: &mut CoordinatorState,
    rx: &mpsc::Receiver<Command>,
) -> Option<Command> {
    loop {
        let Some(deadline) = state.grace_deadline() else {
            return rx.recv().ok();
        };
        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(cmd) => return Some(cmd),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(effect) = state.on_grace_expired() {
                    run_effect(app, state, effect);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return None,
        }
    }
}

fn run_effect(app: &AppHandle, state: &mut CoordinatorState, effect: Effect) {
    match effect {
        Effect::Start {
            binding_id,
            hotkey_string,
        } => {
            let started = start(app, &binding_id, &hotkey_string);
            state.on_start_result(&binding_id, started);
        }
        Effect::Stop {
            binding_id,
            hotkey_string,
        } => stop(app, &binding_id, &hotkey_string),
    }
}

/// Execute a start effect; returns whether recording actually began, so the
/// state machine can roll back its optimistic transition on failure.
fn start(app: &AppHandle, binding_id: &str, hotkey_string: &str) -> bool {
    let Some(action) = action_for(binding_id) else {
        warn!("No action in ACTION_MAP for '{binding_id}'");
        return false;
    };
    action.start(app, binding_id, hotkey_string);
    let recording = app
        .try_state::<Arc<AudioRecordingManager>>()
        .is_some_and(|a| a.is_recording());
    if !recording {
        debug!("Start for '{binding_id}' did not begin recording; staying idle");
    }
    recording
}

fn stop(app: &AppHandle, binding_id: &str, hotkey_string: &str) {
    let Some(action) = action_for(binding_id) else {
        warn!("No action in ACTION_MAP for '{binding_id}'");
        return;
    };
    action.stop(app, binding_id, hotkey_string);
}

#[cfg(test)]
#[path = "transcription_coordinator/input_lost_tests.rs"]
mod input_lost_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_to_talk_release_while_recording_defers_release() {
        assert_eq!(
            classify_ptt_event(None, false, true, "transcribe", Some("transcribe")),
            PttAction::DeferRelease
        );
    }

    #[test]
    fn push_to_talk_press_matching_pending_release_cancels_release() {
        assert_eq!(
            classify_ptt_event(
                Some("transcribe"),
                true,
                true,
                "transcribe",
                Some("transcribe")
            ),
            PttAction::CancelRelease
        );
    }

    #[test]
    fn toggle_mode_press_and_release_pass_through() {
        assert_eq!(
            classify_ptt_event(
                Some("transcribe"),
                true,
                false,
                "transcribe",
                Some("transcribe")
            ),
            PttAction::Passthrough
        );
        assert_eq!(
            classify_ptt_event(None, false, false, "transcribe", Some("transcribe")),
            PttAction::Passthrough
        );
    }

    #[test]
    fn press_for_different_binding_than_pending_release_passes_through() {
        assert_eq!(
            classify_ptt_event(
                Some("transcribe"),
                true,
                true,
                "transcribe_with_post_process",
                Some("transcribe")
            ),
            PttAction::Passthrough
        );
    }

    #[test]
    fn press_matching_pending_release_cancels_without_recording_state() {
        assert_eq!(
            classify_ptt_event(Some("transcribe"), true, true, "transcribe", None),
            PttAction::CancelRelease
        );
    }

    // ---------------------------------------------------------------------
    // Busy-pipeline input classification.
    //
    // Toggle-style triggers (SIGUSR2, CLI flags, pedals that signal on both
    // edges) flip state on every edge. Dropping a press that arrives while
    // the previous pipeline is still processing desyncs the parity: the next
    // edge then starts a recording no one will stop, leaving the overlay
    // waiting for input with the button long released.
    // ---------------------------------------------------------------------

    #[test]
    fn toggle_press_during_processing_remembers_start() {
        assert_eq!(
            classify_busy_input(true, ShortcutActivation::Toggle, None),
            BusyAction::Remember
        );
    }

    #[test]
    fn second_toggle_press_during_processing_forgets_press() {
        assert_eq!(
            classify_busy_input(true, ShortcutActivation::Toggle, Some(Remembered::Locked)),
            BusyAction::Forget
        );
    }

    #[test]
    fn toggle_release_during_processing_is_ignored() {
        assert_eq!(
            classify_busy_input(false, ShortcutActivation::Toggle, None),
            BusyAction::Ignore
        );
        assert_eq!(
            classify_busy_input(false, ShortcutActivation::Toggle, Some(Remembered::Locked)),
            BusyAction::Ignore
        );
    }

    #[test]
    fn hold_modes_classify_busy_inputs_by_pending_state() {
        let cases = [
            (true, None, BusyAction::Remember),
            (true, Some(Remembered::Held), BusyAction::Ignore),
            (true, Some(Remembered::Locked), BusyAction::Forget),
            (false, None, BusyAction::Ignore),
            (false, Some(Remembered::Held), BusyAction::Ignore),
            (false, Some(Remembered::Locked), BusyAction::Ignore),
        ];

        for mode in [
            ShortcutActivation::PushToTalk,
            ShortcutActivation::HoldOrToggle,
        ] {
            for (is_pressed, remembered, expected) in cases {
                assert_eq!(classify_busy_input(is_pressed, mode, remembered), expected);
            }
        }
    }

    /// Toggle parity across a busy window: an odd number of presses remembers
    /// one start, each further press flips the remembered press off/on again.
    #[test]
    fn toggle_presses_alternate_remember_and_forget_while_busy() {
        let mut remembered = None;
        for expected in [
            BusyAction::Remember,
            BusyAction::Forget,
            BusyAction::Remember,
        ] {
            let action = classify_busy_input(true, ShortcutActivation::Toggle, remembered);
            assert_eq!(action, expected);
            remembered = (action == BusyAction::Remember).then_some(Remembered::Locked);
        }
        assert!(remembered.is_some());
    }

    // ---------------------------------------------------------------------
    // Sequence-level regression coverage for issue #1539.
    //
    // Under X11 key auto-repeat, holding a push-to-talk key does not emit one
    // long press. It emits the initial press followed by a stream of
    // synthesized release/press pairs, then a single genuine release on key-up.
    // Before the fix, every synthesized release passed straight through and
    // stopped recording, so holding the key "rapidly toggled" recording on and
    // off. The fix defers each release for a short grace window and cancels it
    // when the matching auto-repeat press arrives.
    //
    // The unit tests above assert the classifiers in isolation. The harness
    // below drives the real `CoordinatorState` through whole event sequences
    // — the same `on_input` / `on_grace_expired` handlers the coordinator
    // thread runs — so a burst can be exercised deterministically without a
    // Tauri AppHandle or real timers, and the tests can never drift from the
    // production transitions.
    // ---------------------------------------------------------------------

    const BINDING: &str = "transcribe";

    #[derive(Clone, Copy)]
    enum Ev {
        /// A key-down event (real initial press or a synthesized auto-repeat press).
        Press,
        /// A key-up event (synthesized auto-repeat release or the genuine key-up).
        Release,
        /// The `RELEASE_GRACE` window elapsed with no cancelling press arriving.
        Grace,
    }

    struct DriveResult {
        starts: u32,
        stops: u32,
        stage: Stage,
    }

    fn ptt_input(is_pressed: bool) -> InputEvent {
        InputEvent {
            binding_id: BINDING.to_string(),
            shortcut_id: BINDING.to_string(),
            hotkey_string: BINDING.to_string(),
            is_pressed,
            mode: ShortcutActivation::PushToTalk,
            hold_threshold: Duration::ZERO,
            external: false,
        }
    }

    /// Feeds an event sequence to a real [`CoordinatorState`] the way the
    /// coordinator thread would; effects are counted instead of executed.
    fn drive(events: &[Ev]) -> DriveResult {
        let mut state = CoordinatorState::new();
        let mut clock = Instant::now();
        let mut starts = 0u32;
        let mut stops = 0u32;

        for ev in events {
            // Auto-repeat events arrive a few ms apart, well inside DEBOUNCE.
            clock += Duration::from_millis(5);

            let effect = match ev {
                Ev::Grace => state.on_grace_expired(),
                Ev::Press | Ev::Release => {
                    state.on_input(ptt_input(matches!(ev, Ev::Press)), clock)
                }
            };
            match effect {
                Some(Effect::Start { .. }) => starts += 1,
                Some(Effect::Stop { .. }) => stops += 1,
                None => {}
            }
        }

        DriveResult {
            starts,
            stops,
            stage: state.stage,
        }
    }

    /// Initial press plus several synthesized release/press pairs, as X11 emits
    /// while a push-to-talk key is held down.
    fn autorepeat_burst() -> Vec<Ev> {
        let mut events = vec![Ev::Press];
        for _ in 0..6 {
            events.push(Ev::Release);
            events.push(Ev::Press);
        }
        events
    }

    /// Regression for #1539: a burst of X11 auto-repeat release/press pairs must
    /// not stop recording. Before the fix the first synthesized release stopped
    /// recording immediately (stops == 1, stage left Recording), which produced
    /// the rapid on/off toggling. With the fix the releases are coalesced and
    /// recording stays continuously active for the whole burst.
    #[test]
    fn x11_autorepeat_burst_does_not_toggle_recording() {
        let result = drive(&autorepeat_burst());
        assert_eq!(result.starts, 1, "recording should start exactly once");
        assert_eq!(
            result.stops, 0,
            "synthesized auto-repeat releases must not stop recording mid-burst"
        );
        assert_eq!(
            result.stage,
            Stage::Recording(BINDING.to_string()),
            "recording must remain active across the entire auto-repeat burst"
        );
    }

    /// Complements the burst test: once the key is genuinely released and the
    /// grace window elapses with no re-press, recording stops exactly once. This
    /// proves the debounce only coalesces synthesized releases and does not wedge
    /// the coordinator or swallow the real key-up.
    #[test]
    fn genuine_release_after_grace_stops_recording_once() {
        let mut events = autorepeat_burst();
        events.push(Ev::Release); // genuine key-up
        events.push(Ev::Grace); // grace window elapses, no cancelling press
        let result = drive(&events);
        assert_eq!(result.starts, 1, "recording should start exactly once");
        assert_eq!(
            result.stops, 1,
            "a genuine release should stop recording exactly once"
        );
        assert_eq!(result.stage, Stage::Processing);
    }

    // ---------------------------------------------------------------------
    // Sequence-level coverage of the busy-pipeline and cancel paths, driven
    // through the real machine.
    // ---------------------------------------------------------------------

    /// PTT press while the pipeline is busy is remembered and starts recording
    /// once the pipeline drains.
    #[test]
    fn press_during_processing_starts_after_drain() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();

        let effect = state.on_input(ptt_input(true), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));

        let effect = state.on_input(ptt_input(false), now + Duration::from_millis(100));
        assert!(effect.is_none(), "release should be deferred, not fired");

        let effect = state.on_grace_expired();
        assert!(matches!(effect, Some(Effect::Stop { .. })));

        let effect = state.on_input(ptt_input(true), now + Duration::from_millis(200));
        assert!(effect.is_none(), "busy pipeline must remember, not start");

        let effect = state.on_processing_finished();
        assert!(
            matches!(effect, Some(Effect::Start { .. })),
            "remembered press should start once the pipeline drains"
        );
    }

    /// Two toggle presses inside one busy window net to no-op: nothing starts
    /// when the pipeline drains (toggle parity).
    #[test]
    fn toggle_presses_during_processing_net_noop_after_drain() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();

        let effect = state.on_input(ptt_input(true), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));
        let effect = state.on_input(ptt_input(false), now + Duration::from_millis(100));
        assert!(effect.is_none());
        let effect = state.on_grace_expired();
        assert!(matches!(effect, Some(Effect::Stop { .. })));

        let toggle = |state: &mut CoordinatorState, at: Instant| {
            state.on_input(
                InputEvent {
                    binding_id: BINDING.to_string(),
                    shortcut_id: BINDING.to_string(),
                    hotkey_string: BINDING.to_string(),
                    is_pressed: true,
                    mode: ShortcutActivation::Toggle,
                    hold_threshold: Duration::ZERO,
                    external: true,
                },
                at,
            )
        };

        let effect = toggle(&mut state, now + Duration::from_millis(200));
        assert!(effect.is_none());
        let effect = toggle(&mut state, now + Duration::from_millis(300));
        assert!(effect.is_none());

        let effect = state.on_processing_finished();
        assert!(
            effect.is_none(),
            "even number of busy toggle presses must not start recording"
        );
        assert_eq!(state.stage, Stage::Idle);
    }

    /// Cancel while processing abandons a remembered press: the pipeline drains
    /// to idle and nothing starts.
    #[test]
    fn cancel_during_processing_drops_remembered_press() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();

        let effect = state.on_input(ptt_input(true), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));
        let effect = state.on_input(ptt_input(false), now + Duration::from_millis(100));
        assert!(effect.is_none());
        let effect = state.on_grace_expired();
        assert!(matches!(effect, Some(Effect::Stop { .. })));

        let effect = state.on_input(ptt_input(true), now + Duration::from_millis(200));
        assert!(effect.is_none());

        state.on_cancel(false);
        assert_eq!(
            state.stage,
            Stage::Processing,
            "cancel must not reset mid-processing — the pipeline still finishes"
        );

        let effect = state.on_processing_finished();
        assert!(
            effect.is_none(),
            "cancelled session must not spawn a deferred recording"
        );
        assert_eq!(state.stage, Stage::Idle);
    }

    fn toggle_input(external: bool) -> InputEvent {
        toggle_input_for(BINDING, external)
    }

    fn toggle_input_for(binding_id: &str, external: bool) -> InputEvent {
        InputEvent {
            binding_id: binding_id.to_string(),
            shortcut_id: binding_id.to_string(),
            hotkey_string: binding_id.to_string(),
            is_pressed: true,
            mode: ShortcutActivation::Toggle,
            hold_threshold: Duration::ZERO,
            external,
        }
    }

    /// Start and stop one toggle recording so the machine sits in `Processing`.
    fn drive_into_processing(state: &mut CoordinatorState, now: Instant) {
        let effect = state.on_input(toggle_input(true), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));
        let effect = state.on_input(toggle_input(true), now + Duration::from_millis(100));
        assert!(matches!(effect, Some(Effect::Stop { .. })));
        assert_eq!(state.stage, Stage::Processing);
    }

    const OTHER_BINDING: &str = "transcribe_with_post_process";

    /// Only one press can be pending. Once a binding has claimed it, a toggle
    /// for a different binding is ignored (as it is while recording) instead of
    /// replacing the remembered press, so the pending binding's parity holds:
    /// two transcribe toggles still net to no-op.
    #[test]
    fn different_binding_does_not_replace_pending_press() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();
        drive_into_processing(&mut state, now);

        let at = |ms| now + Duration::from_millis(ms);
        assert!(state.on_input(toggle_input(true), at(200)).is_none());
        assert!(state
            .on_input(toggle_input_for(OTHER_BINDING, true), at(300))
            .is_none());
        assert!(state.on_input(toggle_input(true), at(400)).is_none());

        let effect = state.on_processing_finished();
        assert!(
            effect.is_none(),
            "two transcribe toggles net to no-op; the ignored post-process toggle must not start"
        );
        assert_eq!(state.stage, Stage::Idle);
    }

    /// The binding that claimed the pending press is the one that starts on
    /// drain, regardless of other bindings toggled in between.
    #[test]
    fn drain_starts_the_pending_binding_not_a_later_one() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();
        drive_into_processing(&mut state, now);

        let at = |ms| now + Duration::from_millis(ms);
        assert!(state.on_input(toggle_input(true), at(200)).is_none());
        assert!(state
            .on_input(toggle_input_for(OTHER_BINDING, true), at(300))
            .is_none());

        match state.on_processing_finished() {
            Some(Effect::Start { binding_id, .. }) => assert_eq!(binding_id, BINDING),
            other => panic!("expected Start for '{BINDING}', got {other:?}"),
        }
    }

    /// External triggers fire on every edge by design (e.g. SIGUSR2 sent on
    /// both key press and release). Two edges inside the debounce window must
    /// both be honoured, or the parity desyncs and recording wedges on.
    #[test]
    fn external_edges_inside_debounce_window_are_not_dropped() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();

        let effect = state.on_input(toggle_input(true), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));

        let effect = state.on_input(toggle_input(true), now + Duration::from_millis(5));
        assert!(
            matches!(effect, Some(Effect::Stop { .. })),
            "second external edge inside DEBOUNCE must stop the recording"
        );
        assert_eq!(state.stage, Stage::Processing);
    }

    /// Physical keyboard presses keep the debounce: a repeat inside the window
    /// is still dropped and recording stays active.
    #[test]
    fn keyboard_press_inside_debounce_window_is_still_dropped() {
        let mut state = CoordinatorState::new();
        let now = Instant::now();

        let effect = state.on_input(toggle_input(false), now);
        assert!(matches!(effect, Some(Effect::Start { .. })));

        let effect = state.on_input(toggle_input(false), now + Duration::from_millis(5));
        assert!(
            effect.is_none(),
            "keyboard repeat inside DEBOUNCE must be debounced"
        );
        assert_eq!(state.stage, Stage::Recording(BINDING.to_string()));
    }

    /// If the start effect fails to begin recording (e.g. microphone access
    /// denied), the optimistic transition rolls back to idle.
    #[test]
    fn failed_start_rolls_back_to_idle() {
        let mut state = CoordinatorState::new();

        let effect = state.on_input(ptt_input(true), Instant::now());
        assert!(matches!(effect, Some(Effect::Start { .. })));

        state.on_start_result(BINDING, false);
        assert_eq!(state.stage, Stage::Idle);
    }

    // ---------------------------------------------------------------------
    // Hold-or-toggle (the combined mode from #147) and the two legacy modes,
    // driven through the real machine on a synthetic clock. Recording starts
    // on key-down in every mode; the tests pin how each mode ends it.
    // ---------------------------------------------------------------------

    const HOLD_THRESHOLD: Duration = Duration::from_millis(300);

    fn input(mode: ShortcutActivation, is_pressed: bool) -> InputEvent {
        InputEvent {
            binding_id: BINDING.to_string(),
            shortcut_id: BINDING.to_string(),
            hotkey_string: BINDING.to_string(),
            is_pressed,
            mode,
            hold_threshold: HOLD_THRESHOLD,
            external: false,
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// Hold-or-toggle: a key held past the threshold is push-to-talk — the
    /// (deferred) release stops recording.
    #[test]
    fn hold_or_toggle_long_hold_stops_on_release() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        assert!(matches!(
            state.on_input(input(mode, true), t0),
            Some(Effect::Start { .. })
        ));
        assert!(state.on_input(input(mode, false), t0 + ms(800)).is_none());
        assert!(
            matches!(state.on_grace_expired(), Some(Effect::Stop { .. })),
            "an 800ms hold must stop when its release grace elapses"
        );
        assert_eq!(state.stage, Stage::Processing);
    }

    /// Hold-or-toggle: a tap keeps recording (locked on); the next press stops.
    #[test]
    fn hold_or_toggle_tap_locks_recording_until_next_press() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        assert!(matches!(
            state.on_input(input(mode, true), t0),
            Some(Effect::Start { .. })
        ));
        assert!(state.on_input(input(mode, false), t0 + ms(120)).is_none());
        assert!(
            state.on_grace_expired().is_none(),
            "a 120ms tap must not stop recording"
        );
        assert_eq!(state.stage, Stage::Recording(BINDING.to_string()));
        assert!(state.is_locked());

        // Seconds later the user presses again to finish.
        assert!(matches!(
            state.on_input(input(mode, true), t0 + ms(5000)),
            Some(Effect::Stop { .. })
        ));
        assert_eq!(state.stage, Stage::Processing);
        // The release of that stopping press lands in the busy window and is
        // ignored, so nothing is remembered for the drain.
        assert!(state.on_input(input(mode, false), t0 + ms(5080)).is_none());
        assert!(state.on_processing_finished().is_none());
        assert_eq!(state.stage, Stage::Idle);
    }

    /// Hold-or-toggle: a locked session ignores stray releases — only a press
    /// ends it.
    #[test]
    fn hold_or_toggle_locked_session_ignores_release() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        state.on_input(input(mode, true), t0);
        state.on_input(input(mode, false), t0 + ms(100));
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked());

        assert!(state.on_input(input(mode, false), t0 + ms(900)).is_none());
        assert!(
            state.grace_deadline().is_none(),
            "no release may be deferred once locked"
        );
        assert_eq!(state.stage, Stage::Recording(BINDING.to_string()));
    }

    /// Hold-or-toggle: while the key is genuinely held, extra presses do not
    /// stop the recording (that is the release's job).
    #[test]
    fn hold_or_toggle_press_while_held_is_ignored() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        state.on_input(input(mode, true), t0);
        assert!(state.on_input(input(mode, true), t0 + ms(400)).is_none());
        assert_eq!(state.stage, Stage::Recording(BINDING.to_string()));
        assert!(!state.is_locked());
    }

    /// Hold-or-toggle under X11 auto-repeat: the synthesized release/press
    /// pairs must not be misread as taps. The hold is measured from the
    /// original key-down to the genuine key-up.
    #[test]
    fn hold_or_toggle_autorepeat_burst_is_one_long_hold() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        let mut clock = t0;

        assert!(matches!(
            state.on_input(input(mode, true), clock),
            Some(Effect::Start { .. })
        ));
        // ~600ms of auto-repeat pairs a few ms apart.
        for _ in 0..60 {
            clock += ms(5);
            assert!(state.on_input(input(mode, false), clock).is_none());
            clock += ms(5);
            assert!(state.on_input(input(mode, true), clock).is_none());
            assert!(
                state.grace_deadline().is_none(),
                "auto-repeat press must cancel the deferred release"
            );
        }
        assert!(!state.is_locked(), "no tap may be classified mid-burst");

        clock += ms(5);
        assert!(state.on_input(input(mode, false), clock).is_none());
        assert!(
            matches!(state.on_grace_expired(), Some(Effect::Stop { .. })),
            "the genuine release after a ~600ms hold must stop recording"
        );
    }

    /// Hold-or-toggle: a press remembered during the busy window is measured
    /// from the real key-down, so a hold that straddles the drain still counts
    /// as a hold when it is released shortly after recording actually starts.
    #[test]
    fn hold_or_toggle_remembered_press_measures_hold_from_real_key_down() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        // Previous session: hold, release, stop → Processing.
        state.on_input(input(mode, true), t0);
        state.on_input(input(mode, false), t0 + ms(800));
        assert!(matches!(
            state.on_grace_expired(),
            Some(Effect::Stop { .. })
        ));

        // Pressed again while busy; still held when the pipeline drains 700ms later.
        assert!(state.on_input(input(mode, true), t0 + ms(1000)).is_none());
        assert!(matches!(
            state.on_processing_finished(),
            Some(Effect::Start { .. })
        ));
        // Released 100ms after recording began — but 800ms after key-down.
        assert!(state.on_input(input(mode, false), t0 + ms(1800)).is_none());
        assert!(
            matches!(state.on_grace_expired(), Some(Effect::Stop { .. })),
            "held 800ms overall: must stop, not lock"
        );
    }

    /// Toggle: releases never stop, the next press does. (Toggle is the
    /// combined machine with the session locked from the start.)
    #[test]
    fn toggle_mode_ignores_release_and_stops_on_next_press() {
        let mode = ShortcutActivation::Toggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        assert!(matches!(
            state.on_input(input(mode, true), t0),
            Some(Effect::Start { .. })
        ));
        assert!(state.is_locked());
        assert!(state.on_input(input(mode, false), t0 + ms(100)).is_none());
        assert!(
            state.grace_deadline().is_none(),
            "toggle never defers releases"
        );
        assert!(state.on_input(input(mode, false), t0 + ms(3000)).is_none());
        assert!(matches!(
            state.on_input(input(mode, true), t0 + ms(4000)),
            Some(Effect::Stop { .. })
        ));
    }

    /// Push-to-talk: even a very short press stops on release — there is no
    /// tap-to-lock in this mode (hold threshold of zero).
    #[test]
    fn push_to_talk_short_press_still_stops_on_release() {
        let mode = ShortcutActivation::PushToTalk;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        assert!(matches!(
            state.on_input(input(mode, true), t0),
            Some(Effect::Start { .. })
        ));
        assert!(state.on_input(input(mode, false), t0 + ms(40)).is_none());
        assert!(matches!(
            state.on_grace_expired(),
            Some(Effect::Stop { .. })
        ));
    }

    /// Cancel (Escape) during a locked hold-or-toggle session resets cleanly so
    /// the next press starts a fresh recording rather than stopping a dead one.
    #[test]
    fn hold_or_toggle_cancel_clears_locked_session() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        state.on_input(input(mode, true), t0);
        state.on_input(input(mode, false), t0 + ms(100));
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked());

        state.on_cancel(true);
        assert_eq!(state.stage, Stage::Idle);
        assert!(!state.is_locked());
        assert!(matches!(
            state.on_input(input(mode, true), t0 + ms(2000)),
            Some(Effect::Start { .. })
        ));
    }

    /// Switching to toggle while an unlocked hold recording is running must not
    /// strand it: in toggle mode a press always stops.
    #[test]
    fn toggle_press_stops_recording_started_as_hold() {
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();

        state.on_input(input(ShortcutActivation::HoldOrToggle, true), t0);
        assert!(!state.is_locked());
        assert!(matches!(
            state.on_input(input(ShortcutActivation::Toggle, true), t0 + ms(2000)),
            Some(Effect::Stop { .. })
        ));
    }

    // Hold-vs-tap classification while the previous transcription is busy.
    fn hold_or_toggle_into_processing(state: &mut CoordinatorState, t0: Instant) {
        let mode = ShortcutActivation::HoldOrToggle;
        assert!(matches!(
            state.on_input(input(mode, true), t0),
            Some(Effect::Start { .. })
        ));
        assert!(state.on_input(input(mode, false), t0 + ms(800)).is_none());
        assert!(matches!(
            state.on_grace_expired(),
            Some(Effect::Stop { .. })
        ));
        assert_eq!(state.stage, Stage::Processing);
    }

    #[test]
    fn hold_or_toggle_tap_during_processing_queues_locked_start() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        assert!(state.on_input(input(mode, true), t0 + ms(1000)).is_none());
        assert!(state.on_input(input(mode, false), t0 + ms(1100)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked(), "a busy tap should queue a locked start");
        assert!(matches!(
            state.on_processing_finished(),
            Some(Effect::Start { .. })
        ));
        assert!(state.is_locked());
    }

    #[test]
    fn hold_or_toggle_completed_hold_during_processing_nets_noop() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        assert!(state.on_input(input(mode, true), t0 + ms(1000)).is_none());
        assert!(state.on_input(input(mode, false), t0 + ms(1600)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(!state.is_locked());

        assert!(
            state.on_processing_finished().is_none(),
            "a 600ms hold that ended before the drain has nothing left to start"
        );
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn hold_or_toggle_two_taps_during_processing_net_noop() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        assert!(state.on_input(input(mode, true), t0 + ms(1000)).is_none());
        assert!(state.on_input(input(mode, false), t0 + ms(1100)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked());

        assert!(state.on_input(input(mode, true), t0 + ms(1500)).is_none());
        assert!(
            !state.is_locked(),
            "the second tap's press forgets the queued tap"
        );
        assert!(state.on_input(input(mode, false), t0 + ms(1600)).is_none());
        assert!(state.grace_deadline().is_none());

        assert!(state.on_processing_finished().is_none());
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn ptt_tap_inside_busy_window_nets_noop() {
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        assert!(state.on_input(ptt_input(true), t0 + ms(1000)).is_none());
        assert!(state.on_input(ptt_input(false), t0 + ms(1040)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(state.on_processing_finished().is_none());
    }

    /// The pipeline drains inside the 50ms grace of a busy tap: recording
    /// starts first (unlocked, from the real key-down), and the grace then
    /// resolves against the live recording, locking it as the tap it was.
    #[test]
    fn hold_or_toggle_drain_inside_busy_release_grace_still_classifies_tap() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        assert!(state.on_input(input(mode, true), t0 + ms(1000)).is_none());
        assert!(state.on_input(input(mode, false), t0 + ms(1100)).is_none());
        assert!(matches!(
            state.on_processing_finished(),
            Some(Effect::Start { .. })
        ));
        assert!(!state.is_locked());

        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked(), "the deferred 100ms release is a tap");
    }

    /// X11 auto-repeat while busy, key still held at the drain: recording
    /// starts measured from the first press, not from the last synthesized
    /// press before the drain. Released 400ms after the real key-down but
    /// only ~100ms after the drain — a hold, so it must stop rather than lock.
    #[test]
    fn hold_or_toggle_autorepeat_burst_straddling_drain_measures_from_first_press() {
        let mode = ShortcutActivation::HoldOrToggle;
        let mut state = CoordinatorState::new();
        let t0 = Instant::now();
        hold_or_toggle_into_processing(&mut state, t0);

        let mut clock = t0 + ms(1000);
        assert!(state.on_input(input(mode, true), clock).is_none());
        for _ in 0..30 {
            clock += ms(5);
            assert!(state.on_input(input(mode, false), clock).is_none());
            clock += ms(5);
            assert!(state.on_input(input(mode, true), clock).is_none());
            assert!(state.grace_deadline().is_none());
        }

        // Drain at ~t0 + 1300ms with the key still down.
        assert!(matches!(
            state.on_processing_finished(),
            Some(Effect::Start { .. })
        ));
        assert!(!state.is_locked());

        for _ in 0..10 {
            clock += ms(5);
            assert!(state.on_input(input(mode, false), clock).is_none());
            clock += ms(5);
            assert!(state.on_input(input(mode, true), clock).is_none());
        }
        assert_eq!(clock, t0 + ms(1400));
        assert!(state.on_input(input(mode, false), clock).is_none());
        assert!(
            matches!(state.on_grace_expired(), Some(Effect::Stop { .. })),
            "held 400ms since the real key-down: must stop, not lock"
        );
        assert_eq!(state.stage, Stage::Processing);
    }
}

/// Several shortcuts bound to one action (feature 004): A starts the session,
/// B is another shortcut of the same action.
#[cfg(test)]
mod multi_shortcut_tests {
    use super::*;

    const ACTION: &str = "transcribe";
    const A: &str = "option+space";
    const B: &str = "ctrl+option+d";
    const HOLD_THRESHOLD: Duration = Duration::from_millis(300);

    fn key(mode: ShortcutActivation, shortcut: &str, is_pressed: bool) -> InputEvent {
        InputEvent {
            binding_id: ACTION.to_string(),
            shortcut_id: shortcut.to_string(),
            hotkey_string: shortcut.to_string(),
            is_pressed,
            mode,
            hold_threshold: HOLD_THRESHOLD,
            external: false,
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn is_start(effect: &Option<Effect>) -> bool {
        matches!(effect, Some(Effect::Start { .. }))
    }

    fn is_stop(effect: &Option<Effect>) -> bool {
        matches!(effect, Some(Effect::Stop { .. }))
    }

    /// Starts a session with A in `mode` at `t0`.
    fn started_by_a(mode: ShortcutActivation, t0: Instant) -> CoordinatorState {
        let mut state = CoordinatorState::new();
        let effect = state.on_input(key(mode, A, true), t0);
        assert!(is_start(&effect), "A must start the session");
        state
    }

    // @spec AC-003: Toggle stops with any shortcut of the action — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-003
    #[test]
    fn toggle_b_stops_a_session_with_the_starting_metadata() {
        let mode = ShortcutActivation::Toggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        let effect = state.on_input(key(mode, B, true), t0 + ms(500));
        assert_eq!(
            effect,
            Some(Effect::Stop {
                binding_id: ACTION.to_string(),
                hotkey_string: A.to_string(),
            })
        );
        assert_eq!(state.stage, Stage::Processing);
    }

    #[test]
    fn toggle_b_inside_the_debounce_window_is_dropped() {
        let mode = ShortcutActivation::Toggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        assert!(state.on_input(key(mode, B, true), t0 + ms(29)).is_none());
        assert_eq!(state.stage, Stage::Recording(ACTION.to_string()));
        assert!(is_stop(&state.on_input(key(mode, B, true), t0 + ms(60))));
    }

    #[test]
    fn simultaneous_presses_start_only_once() {
        let mode = ShortcutActivation::PushToTalk;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        assert!(state.on_input(key(mode, B, true), t0 + ms(2)).is_none());
        assert_eq!(state.stage, Stage::Recording(ACTION.to_string()));
    }

    // @spec AC-003: Hold modes end only on the starting shortcut's release — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-003
    #[test]
    fn push_to_talk_ignores_b_while_a_is_held() {
        let mode = ShortcutActivation::PushToTalk;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        assert!(state.on_input(key(mode, B, true), t0 + ms(200)).is_none());
        assert!(state.on_input(key(mode, B, false), t0 + ms(300)).is_none());
        assert!(state.grace_deadline().is_none(), "B release must not defer");
        assert_eq!(state.stage, Stage::Recording(ACTION.to_string()));

        assert!(state.on_input(key(mode, A, false), t0 + ms(400)).is_none());
        assert!(is_stop(&state.on_grace_expired()));
    }

    #[test]
    fn push_to_talk_b_press_does_not_cancel_a_deferred_release() {
        let mode = ShortcutActivation::PushToTalk;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        assert!(state.on_input(key(mode, A, false), t0 + ms(400)).is_none());
        assert!(state.on_input(key(mode, B, true), t0 + ms(420)).is_none());
        assert!(
            state.grace_deadline().is_some(),
            "A's release stays pending"
        );
        assert!(is_stop(&state.on_grace_expired()));
    }

    #[test]
    fn hold_or_toggle_held_phase_ignores_b() {
        let mode = ShortcutActivation::HoldOrToggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        assert!(state.on_input(key(mode, B, true), t0 + ms(100)).is_none());
        assert!(state.on_input(key(mode, B, false), t0 + ms(150)).is_none());
        assert!(!state.is_locked(), "B's short press is not A's tap");

        assert!(state.on_input(key(mode, A, false), t0 + ms(800)).is_none());
        assert!(is_stop(&state.on_grace_expired()));
    }

    #[test]
    fn hold_or_toggle_locked_phase_stops_on_b() {
        let mode = ShortcutActivation::HoldOrToggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);

        assert!(state.on_input(key(mode, A, false), t0 + ms(120)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked());

        assert!(is_stop(&state.on_input(key(mode, B, true), t0 + ms(3000))));
    }

    #[test]
    fn toggle_presses_of_a_and_b_while_busy_net_to_no_op() {
        let mode = ShortcutActivation::Toggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        assert!(is_stop(&state.on_input(key(mode, A, true), t0 + ms(500))));

        assert!(state.on_input(key(mode, A, true), t0 + ms(600)).is_none());
        assert!(state.on_input(key(mode, B, true), t0 + ms(700)).is_none());
        assert!(state.on_processing_finished().is_none());
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn hold_mode_busy_press_belongs_to_the_pressed_shortcut() {
        let mode = ShortcutActivation::PushToTalk;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        assert!(state.on_input(key(mode, A, false), t0 + ms(400)).is_none());
        assert!(is_stop(&state.on_grace_expired()));

        assert!(state.on_input(key(mode, A, true), t0 + ms(600)).is_none());
        assert!(state.on_input(key(mode, B, true), t0 + ms(700)).is_none());
        assert!(state.on_input(key(mode, B, false), t0 + ms(750)).is_none());
        assert!(is_start(&state.on_processing_finished()));

        assert!(state.on_input(key(mode, B, false), t0 + ms(900)).is_none());
        assert_eq!(state.stage, Stage::Recording(ACTION.to_string()));
        assert!(state.on_input(key(mode, A, false), t0 + ms(950)).is_none());
        assert!(is_stop(&state.on_grace_expired()));
    }

    #[test]
    fn hold_or_toggle_busy_tap_keeps_shortcut_ownership_until_the_drain() {
        let mode = ShortcutActivation::HoldOrToggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        assert!(state.on_input(key(mode, A, false), t0 + ms(800)).is_none());
        assert!(is_stop(&state.on_grace_expired()));

        assert!(state.on_input(key(mode, A, true), t0 + ms(1000)).is_none());
        assert!(state.on_input(key(mode, A, false), t0 + ms(1100)).is_none());
        assert!(state.on_grace_expired().is_none());
        assert!(state.is_locked(), "A's busy tap is queued locked");

        assert!(state.on_input(key(mode, B, true), t0 + ms(1200)).is_none());
        assert!(state.is_locked(), "B must not forget A's queued tap");
        assert!(is_start(&state.on_processing_finished()));
        assert!(is_stop(&state.on_input(key(mode, B, true), t0 + ms(2000))));
    }

    fn external_toggle() -> InputEvent {
        InputEvent {
            external: true,
            ..key(ShortcutActivation::Toggle, "SIGUSR2", true)
        }
    }

    /// Drives a push-to-talk session started by A into `Processing`.
    fn busy_after_a(t0: Instant) -> CoordinatorState {
        let mode = ShortcutActivation::PushToTalk;
        let mut state = started_by_a(mode, t0);
        assert!(state.on_input(key(mode, A, false), t0 + ms(400)).is_none());
        assert!(is_stop(&state.on_grace_expired()));
        state
    }

    #[test]
    fn toggle_remembered_press_belongs_to_the_action_for_hold_inputs() {
        let t0 = Instant::now();
        let mut state = busy_after_a(t0);
        assert!(state.on_input(external_toggle(), t0 + ms(500)).is_none());
        let press = key(ShortcutActivation::PushToTalk, B, true);
        assert!(state.on_input(press, t0 + ms(600)).is_none());
        assert!(
            state.on_processing_finished().is_none(),
            "B forgets the action-owned queued toggle"
        );
    }

    #[test]
    fn external_toggle_flips_a_press_remembered_by_a_shortcut() {
        let t0 = Instant::now();
        let mut state = busy_after_a(t0);
        let press = key(ShortcutActivation::PushToTalk, A, true);
        assert!(state.on_input(press, t0 + ms(500)).is_none());
        assert!(state.on_input(external_toggle(), t0 + ms(510)).is_none());
        assert!(
            state.on_processing_finished().is_none(),
            "an external toggle acts on the action, as before"
        );
    }

    #[test]
    fn other_action_stays_ignored_while_a_records() {
        let mode = ShortcutActivation::Toggle;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        let mut other = key(mode, "option+shift+space", true);
        other.binding_id = "transcribe_with_post_process".to_string();
        assert!(state.on_input(other, t0 + ms(500)).is_none());
        assert_eq!(state.stage, Stage::Recording(ACTION.to_string()));
    }

    #[test]
    fn failed_start_from_a_lets_b_start_a_new_session() {
        let mode = ShortcutActivation::PushToTalk;
        let t0 = Instant::now();
        let mut state = started_by_a(mode, t0);
        state.on_start_result(ACTION, false);
        assert_eq!(state.stage, Stage::Idle);
        assert!(is_start(&state.on_input(key(mode, B, true), t0 + ms(100))));
    }
}
