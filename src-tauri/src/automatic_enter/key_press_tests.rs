//! The guarded key press with a scripted input thread and a fake workspace: the destination is
//! checked on the input thread right before the key, a focus switch after the worker's check
//! cancels the key, a start past the bound never presses it, and the reported outcome is always
//! the actual one (no failure reported for a key that was pressed).
use super::*;
use crate::destination::fake_workspace::{app, FakeWorkspace};
use crate::destination::workspace::RunningApp;
use std::sync::Mutex;

/// Input thread that runs each task on its own thread after `delay`, optionally switching the
/// foreground first.
struct ScriptedThread {
    delay: Duration,
    switch_to: Option<(Arc<FakeWorkspace>, RunningApp)>,
}
impl MainThreadRunner for ScriptedThread {
    fn run(&self, task: Box<dyn FnOnce() + Send>) -> Result<(), String> {
        let (delay, switch_to) = (self.delay, self.switch_to.clone());
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            if let Some((workspace, front)) = switch_to {
                workspace.set_foreground(Foreground::Application(front));
            }
            task();
        });
        Ok(())
    }
}
fn chatgpt() -> RunningApp {
    app(11, "com.openai.chat", "ChatGPT")
}
fn target() -> ResolvedTarget {
    ResolvedTarget {
        pid: chatgpt().pid,
        identity: chatgpt().identity,
    }
}
/// Emitter over `runner` that records every real press.
fn emitter(
    runner: ScriptedThread,
    workspace: Arc<FakeWorkspace>,
) -> (GuardedKeyPress, Arc<Mutex<u32>>) {
    let presses = Arc::new(Mutex::new(0));
    let counter = Arc::clone(&presses);
    let press: PressFn = Arc::new(move |_| {
        *counter.lock().map_err(|_| "poisoned".to_owned())? += 1;
        Ok(())
    });
    let keys = GuardedKeyPress {
        runner: Arc::new(runner),
        workspace,
        press,
        bound: Duration::from_millis(100),
    };
    (keys, presses)
}
fn count(presses: &Mutex<u32>) -> u32 {
    presses.lock().map(|count| *count).unwrap_or(u32::MAX)
}
fn workspace() -> Arc<FakeWorkspace> {
    Arc::new(FakeWorkspace::new(
        Foreground::Application(chatgpt()),
        vec![chatgpt()],
    ))
}

#[test]
fn the_destination_still_in_front_on_the_input_thread_gets_one_key() {
    let runner = ScriptedThread {
        delay: Duration::ZERO,
        switch_to: None,
    };
    let (keys, presses) = emitter(runner, workspace());
    assert_eq!(
        keys.send(AutoSubmitKey::Enter, Some(&target())),
        KeyOutcome::Sent
    );
    assert_eq!(count(&presses), 1);
}
#[test]
fn a_switch_after_the_worker_check_cancels_the_key_on_the_input_thread() {
    let workspace = workspace();
    let safari = app(33, "com.apple.Safari", "Safari");
    let runner = ScriptedThread {
        delay: Duration::from_millis(5),
        switch_to: Some((Arc::clone(&workspace), safari)),
    };
    let (keys, presses) = emitter(runner, workspace);
    let outcome = keys.send(AutoSubmitKey::Enter, Some(&target()));
    assert_eq!(outcome, KeyOutcome::FocusLost);
    assert_eq!(count(&presses), 0);
}
#[test]
fn a_frontmost_at_end_route_has_no_expectation_to_check() {
    let workspace = workspace();
    let safari = app(33, "com.apple.Safari", "Safari");
    let runner = ScriptedThread {
        delay: Duration::ZERO,
        switch_to: Some((Arc::clone(&workspace), safari)),
    };
    let (keys, presses) = emitter(runner, workspace);
    assert_eq!(keys.send(AutoSubmitKey::Enter, None), KeyOutcome::Sent);
    assert_eq!(count(&presses), 1);
}
#[test]
fn a_late_input_thread_never_presses_and_reports_late() {
    let runner = ScriptedThread {
        delay: Duration::from_millis(250),
        switch_to: None,
    };
    let (keys, presses) = emitter(runner, workspace());
    assert_eq!(
        keys.send(AutoSubmitKey::Enter, Some(&target())),
        KeyOutcome::Late
    );
    // Let the late task run: it must find the press abandoned.
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(count(&presses), 0);
}
#[test]
fn a_task_taken_just_in_time_reports_its_actual_outcome() {
    // The task starts inside the bound and takes 150 ms to press: the worker waits for it.
    let workspace = workspace();
    let presses = Arc::new(Mutex::new(0));
    let counter = Arc::clone(&presses);
    let press: PressFn = Arc::new(move |_| {
        std::thread::sleep(Duration::from_millis(150));
        *counter.lock().map_err(|_| "poisoned".to_owned())? += 1;
        Ok(())
    });
    let keys = GuardedKeyPress {
        runner: Arc::new(ScriptedThread {
            delay: Duration::from_millis(10),
            switch_to: None,
        }),
        workspace,
        press,
        bound: Duration::from_millis(100),
    };
    assert_eq!(
        keys.send(AutoSubmitKey::Enter, Some(&target())),
        KeyOutcome::Sent
    );
    assert_eq!(count(&presses), 1);
}
#[test]
fn a_press_that_cannot_be_scheduled_is_a_failure_without_key() {
    struct Refusing;
    impl MainThreadRunner for Refusing {
        fn run(&self, _task: Box<dyn FnOnce() + Send>) -> Result<(), String> {
            Err("main thread gone".to_owned())
        }
    }
    let keys = GuardedKeyPress {
        runner: Arc::new(Refusing),
        workspace: workspace(),
        press: Arc::new(|_| Ok(())),
        bound: Duration::from_millis(100),
    };
    assert_eq!(
        keys.send(AutoSubmitKey::Enter, Some(&target())),
        KeyOutcome::Failed("main thread gone".to_owned())
    );
}
