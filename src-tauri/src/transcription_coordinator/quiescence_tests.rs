//! Quiescence lease: answered in input order by the production state machine, gating shortcut
//! input while held, never granted to a requester that gave up.
use super::super::{dispatch, Command, CoordinatorState, Effect, InputEvent, Stage};
use super::*;
use crate::settings::ShortcutActivation;
use std::thread;

fn key(mode: ShortcutActivation, is_pressed: bool, external: bool) -> InputEvent {
    InputEvent {
        binding_id: "transcribe".to_string(),
        shortcut_id: "option+space".to_string(),
        hotkey_string: "option+space".to_string(),
        is_pressed,
        mode,
        hold_threshold: Duration::from_millis(300),
        external,
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Sends a lease request through `dispatch`; returns whether it was granted.
fn quiesce(state: &mut CoordinatorState, gate: &mut InputGate, now: Instant) -> bool {
    let request = LeaseRequest::new();
    dispatch(state, gate, Command::Quiesce(Arc::clone(&request)), now);
    request.wait(Duration::ZERO)
}

#[test]
fn an_idle_coordinator_grants_the_lease() {
    let (mut state, mut gate) = (CoordinatorState::new(), InputGate::default());
    assert!(quiesce(&mut state, &mut gate, Instant::now()));
}

// @spec FR-002: Changes wait for a quiescent coordinator — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
#[test]
fn a_key_down_handled_before_the_request_refuses_the_lease() {
    let t0 = Instant::now();
    let (mut state, mut gate) = (CoordinatorState::new(), InputGate::default());
    let press = key(ShortcutActivation::PushToTalk, true, false);
    let effect = dispatch(&mut state, &mut gate, Command::Input(press), t0);
    assert!(matches!(effect, Some(Effect::Start { .. })));
    assert!(!quiesce(&mut state, &mut gate, t0 + ms(1)));
    // The owner's release still ends the session: nothing was gated.
    let release = key(ShortcutActivation::PushToTalk, false, false);
    dispatch(&mut state, &mut gate, Command::Input(release), t0 + ms(100));
    assert!(state.pending_release.is_some());
}

#[test]
fn a_key_down_during_the_lease_is_gated_and_its_release_ignored() {
    let t0 = Instant::now();
    let (mut state, mut gate) = (CoordinatorState::new(), InputGate::default());
    assert!(quiesce(&mut state, &mut gate, t0));
    let press = key(ShortcutActivation::PushToTalk, true, false);
    assert_eq!(
        dispatch(&mut state, &mut gate, Command::Input(press), t0),
        None
    );
    assert!(state.is_quiescent());
    dispatch(&mut state, &mut gate, Command::Resume, t0 + ms(5));
    let release = key(ShortcutActivation::PushToTalk, false, false);
    assert_eq!(
        dispatch(&mut state, &mut gate, Command::Input(release), t0 + ms(40)),
        None
    );
    assert_eq!(state.stage, Stage::Idle);
}

#[test]
fn external_triggers_pass_the_gate_and_input_resumes_after_release() {
    let t0 = Instant::now();
    let (mut state, mut gate) = (CoordinatorState::new(), InputGate::default());
    assert!(quiesce(&mut state, &mut gate, t0));
    let external = key(ShortcutActivation::Toggle, true, true);
    let effect = dispatch(&mut state, &mut gate, Command::Input(external), t0);
    assert!(matches!(effect, Some(Effect::Start { .. })));
    dispatch(&mut state, &mut gate, Command::Resume, t0 + ms(1));
    assert!(gate.admits(false));
}

#[test]
fn every_session_state_refuses_the_lease() {
    let t0 = Instant::now();
    let mut processing = CoordinatorState::new();
    processing.stage = Stage::Processing;
    let mut gate = InputGate::default();
    assert!(!quiesce(&mut processing, &mut gate, t0));
    let mut remembered = CoordinatorState::new();
    let toggle = key(ShortcutActivation::Toggle, true, false);
    remembered.stage = Stage::Processing;
    remembered.on_input(toggle, t0);
    remembered.stage = Stage::Idle;
    assert!(remembered.pending_press.is_some());
    assert!(!quiesce(&mut remembered, &mut gate, t0));
    assert!(
        gate.admits(false),
        "a refused request never closes the gate"
    );
}

#[test]
fn a_late_answer_never_grants_an_abandoned_request() {
    let request = LeaseRequest::new();
    assert!(!request.wait(ms(1)));
    let mut gate = InputGate::default();
    gate.answer(&request, true);
    assert!(gate.admits(false));
}

#[test]
fn the_requester_receives_an_answer_given_on_another_thread() {
    let request = LeaseRequest::new();
    let answering = Arc::clone(&request);
    let coordinator = thread::spawn(move || answering.answer(true));
    assert!(request.wait(Duration::from_secs(5)));
    assert_eq!(coordinator.join().ok(), Some(true));
}

#[test]
fn dropping_the_lease_reopens_the_gate() {
    let (tx, rx) = std::sync::mpsc::channel();
    drop(QuiescenceLease::new(tx));
    assert!(matches!(rx.try_recv(), Ok(Command::Resume)));
}
