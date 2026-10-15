//! Shared fixture of the Enter transaction tests: 006's real manager over a fake workspace and
//! clock, a key spy and a pause that can switch the foreground.
use super::*;
use crate::destination::context::{DestinationSource, RecoveryDisposition, ResolvedTarget};
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::workspace::RunningApp;
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::time::Duration;

/// Key spy: records the clock time of every press; `fails` makes the press fail.
pub(super) struct Keys {
    pub(super) clock: Arc<FakeClock>,
    pub(super) pressed: Mutex<Vec<(AutoSubmitKey, Instant)>>,
    pub(super) fails: bool,
}
impl KeyEmitter for Keys {
    fn send(&self, key: AutoSubmitKey, _expected: Option<&ResolvedTarget>) -> KeyOutcome {
        self.pressed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((key, self.clock.now()));
        if self.fails {
            KeyOutcome::Failed("injected input failure".to_owned())
        } else {
            KeyOutcome::Sent
        }
    }
}
/// Pause that switches the foreground half-way, when asked to.
pub(super) struct SwitchingPause {
    pub(super) clock: Arc<FakeClock>,
    pub(super) workspace: Arc<FakeWorkspace>,
    pub(super) switch_to: Option<RunningApp>,
}
impl Pause for SwitchingPause {
    fn pause(&self, duration: Duration) {
        self.clock.advance(duration);
        if let Some(app) = &self.switch_to {
            self.workspace
                .set_foreground(Foreground::Application(app.clone()));
        }
    }
}

pub(super) struct Fixture {
    pub(super) clock: Arc<FakeClock>,
    pub(super) workspace: Arc<FakeWorkspace>,
    pub(super) manager: Arc<DestinationManager>,
    pub(super) keys: Arc<Keys>,
    pub(super) context: DestinationContext,
}
pub(super) fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
pub(super) fn safari() -> RunningApp {
    app(33, "com.apple.Safari", "Safari")
}
/// A dictation of ChatGPT whose guard accepted the dispatch (disposition `Insert`).
pub(super) fn fixture(source: DestinationSource, fails: bool) -> Fixture {
    let clock = Arc::new(FakeClock::new());
    let workspace = Arc::new(FakeWorkspace::new(
        Foreground::Application(chatgpt()),
        vec![chatgpt(), safari()],
    ));
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace.clone(),
        clock.clone(),
        platform,
    ));
    let started = manager.commit_start(manager.capture(true));
    let (id, generation) = (started.dictation_id, started.destination_generation);
    manager.mark_processing();
    assert!(manager.freeze(id, generation).is_ok(), "frozen");
    let insert = manager.set_disposition(id, generation, RecoveryDisposition::Insert);
    assert!(insert.is_ok(), "insert");
    let keys = Arc::new(Keys {
        clock: clock.clone(),
        pressed: Mutex::new(Vec::new()),
        fails,
    });
    let target = (source != DestinationSource::FrontmostAtEnd).then(|| ResolvedTarget {
        pid: chatgpt().pid,
        identity: chatgpt().identity,
    });
    let context = DestinationContext {
        dictation_id: id,
        destination_generation: generation,
        source,
        target,
    };
    Fixture {
        clock,
        workspace,
        manager,
        keys,
        context,
    }
}
pub(super) fn transaction(
    fixture: &Fixture,
    policy: AutomaticEnterPolicy,
    switch_to: Option<RunningApp>,
) -> EnterTransaction {
    let frozen = EnterSnapshot {
        dictation_id: fixture.context.dictation_id,
        destination_generation: fixture.context.destination_generation,
        policy,
        key: AutoSubmitKey::CtrlEnter,
    };
    let pause = Arc::new(SwitchingPause {
        clock: fixture.clock.clone(),
        workspace: fixture.workspace.clone(),
        switch_to,
    });
    let ports = EnterPorts {
        claims: fixture.manager.clone(),
        workspace: fixture.workspace.clone(),
        keys: fixture.keys.clone(),
        clock: fixture.clock.clone(),
        pause,
    };
    EnterTransaction::new(frozen, ports)
}
pub(super) fn presses(fixture: &Fixture) -> Vec<(AutoSubmitKey, Instant)> {
    fixture
        .keys
        .pressed
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}
pub(super) const CLASSIC: Completion = Completion::Classic { succeeded: true };
