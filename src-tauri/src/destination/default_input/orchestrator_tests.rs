// LiveSpec traceability: @spec(AC-005) @spec(AC-006) @spec(AC-007)
//! Real orchestrator, manager and guard with a scripted tree, tool and method (013 FR-005 to
//! FR-007): an editable focus inserts as before, a found field is focused before the method's
//! dispatch, a missing field dispatches nothing with its reason, a terminal is typed through the
//! tool without running the configured method and is marked for 009's Enter suppression.
use super::dispatch::AddressedMethod;
#[cfg(unix)]
use super::fake_tree::bundle_with_tool;
use super::fake_tree::{element, FakeElement, FakeTree, Fallible, OneBundle, TestResult};
use super::fingerprint::fingerprint_of;
use super::model::{AddressingTool, DefaultInput, DefaultTarget, TerminalTarget};
use super::services::DefaultInputServices;
use super::terminal::tests::{FakeRunner, WORKSPACE};
use crate::destination::context::NoRecoveryConsumer;
use crate::destination::fake_workspace::{app, FakeWorkspace};
use crate::destination::guard::DispatchGuard;
use crate::destination::insertion::{
    InsertionPort, InsertionReport, InsertionRequest, Orchestrator,
};
use crate::destination::manager::DestinationManager;
use crate::destination::outcome::{DestinationFailure, InsertionStatus, MethodResult};
use crate::destination::workspace::{Foreground, RunningApp};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const PID: u32 = 42;

/// Method double: calls the guard, counts the dispatches that passed it.
#[derive(Default)]
struct Method {
    dispatched: AtomicUsize,
}
impl InsertionPort for Method {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        match guard.check() {
            Ok(_) => {
                self.dispatched.fetch_add(1, Ordering::SeqCst);
                MethodResult::Legacy(Ok(()))
            }
            Err(refusal) => MethodResult::Legacy(Err(refusal.to_string())),
        }
    }
}
/// A method shared with the test through an `Arc`.
struct Shared(Arc<Method>);
impl InsertionPort for Shared {
    fn dispatch(&self, guard: Arc<DispatchGuard>) -> MethodResult {
        self.0.dispatch(guard)
    }
}

fn target() -> RunningApp {
    app(PID, "com.example.composer", "Composer")
}
fn codex_like() -> Vec<FakeElement> {
    vec![
        element("AXWindow", None, vec![1, 2]),
        element("AXGroup", Some("Conversation"), vec![]),
        element("AXTextArea", Some("Work with Codex"), vec![]),
    ]
}
fn input(target_kind: DefaultTarget) -> DefaultInput {
    DefaultInput {
        identity: target().identity,
        display_name: "Composer".to_owned(),
        target: target_kind,
    }
}
fn services(tree: &FakeTree, inputs: Vec<DefaultInput>) -> DefaultInputServices {
    DefaultInputServices {
        supported: true,
        inputs: Box::new(move || inputs.clone()),
        tree: Arc::new(tree.clone()),
        ..DefaultInputServices::none()
    }
}

/// Run one dictation to `target()` (already in front) with `services`, through `method`.
fn dictate(
    tree: &FakeTree,
    services: DefaultInputServices,
    method: &dyn InsertionPort,
) -> InsertionReport {
    let workspace = FakeWorkspace::new(Foreground::Application(target()), vec![target()]);
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = DestinationManager::new(Arc::new(workspace), tree.clock.clone(), platform)
        .with_default_inputs(services);
    let manager = Arc::new(manager);
    let snapshot = manager.commit_start(manager.capture(true));
    manager.mark_processing();
    let orchestrator = Orchestrator {
        manager,
        clock: tree.clock.clone(),
        pause: tree.clock.clone(),
        recovery: Arc::new(NoRecoveryConsumer),
    };
    let request = InsertionRequest {
        dictation_id: snapshot.dictation_id,
        destination_generation: snapshot.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(|| false),
    };
    orchestrator.deliver(request, method)
}

// @spec AC-005: Editable focus keeps today's insertion — .specs/features/013-default-input-per-app/spec.md#ac-005
#[test]
fn an_editable_focus_dispatches_the_method_unchanged() -> TestResult {
    let elements = codex_like();
    let fingerprint =
        fingerprint_of(&elements[2].facts).map_err(|error| format!("fingerprint: {error:?}"))?;
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(2);
    let method = Method::default();
    let report = dictate(
        &tree,
        services(&tree, vec![input(DefaultTarget::Field(fingerprint))]),
        &method,
    );
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(method.dispatched.load(Ordering::SeqCst), 1);
    assert!(!report.addressed);
    assert_eq!(tree.lock().reads, 0);
    Ok(())
}

// @spec AC-006: Unique field focused and verified — .specs/features/013-default-input-per-app/spec.md#ac-006
#[test]
fn a_found_field_is_focused_before_the_method_pastes() -> TestResult {
    let elements = codex_like();
    let fingerprint =
        fingerprint_of(&elements[2].facts).map_err(|error| format!("fingerprint: {error:?}"))?;
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(1);
    let method = Method::default();
    let report = dictate(
        &tree,
        services(&tree, vec![input(DefaultTarget::Field(fingerprint))]),
        &method,
    );
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(method.dispatched.load(Ordering::SeqCst), 1);
    assert_eq!(tree.lock().focused, Some(2));
    Ok(())
}

#[test]
fn a_missing_field_dispatches_nothing_and_names_its_reason() -> TestResult {
    let elements = codex_like();
    let mut fingerprint =
        fingerprint_of(&elements[2].facts).map_err(|error| format!("fingerprint: {error:?}"))?;
    fingerprint.name = Some("Search".to_owned());
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(1);
    let method = Method::default();
    let report = dictate(
        &tree,
        services(&tree, vec![input(DefaultTarget::Field(fingerprint))]),
        &method,
    );
    assert_eq!(report.status, InsertionStatus::Failed);
    assert_eq!(
        report.reason,
        Some(DestinationFailure::DefaultFieldNotFound)
    );
    assert_eq!(method.dispatched.load(Ordering::SeqCst), 0);
    assert_eq!(tree.lock().focused, Some(1));
    Ok(())
}

#[cfg(unix)]
fn terminal_services(
    tree: &FakeTree,
    runner: Arc<FakeRunner>,
) -> Fallible<(DefaultInputServices, tempfile::TempDir)> {
    let bundle = bundle_with_tool(0o755)?;
    let terminal = DefaultTarget::Terminal(TerminalTarget {
        tool: AddressingTool::Cmux,
        workspace_id: WORKSPACE.to_owned(),
        workspace_name: "roam-radar".to_owned(),
    });
    let mut services = services(tree, vec![input(terminal)]);
    services.bundles = Arc::new(OneBundle(bundle.path().to_path_buf()));
    services.runner = runner;
    Ok((services, bundle))
}

// @spec AC-007: Typed through the tool, no method, no Enter — .specs/features/013-default-input-per-app/spec.md#ac-007
#[cfg(unix)]
#[test]
fn a_terminal_is_typed_through_the_tool_instead_of_the_method() -> TestResult {
    let tree = FakeTree::new(PID, vec![element("AXWindow", None, vec![])]);
    tree.lock().focused = Some(0);
    let runner = Arc::new(FakeRunner::with_workspace("roam-radar", "terminal"));
    let (services, _bundle) = terminal_services(&tree, Arc::clone(&runner))?;
    let inner = Arc::new(Method::default());
    let wrapper = AddressedMethod::new(
        Box::new(Shared(Arc::clone(&inner))),
        "ls -la",
        runner.clone(),
    );
    let report = dictate(&tree, services, &wrapper);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert!(report.addressed);
    assert_eq!(inner.dispatched.load(Ordering::SeqCst), 0);
    let sends: Vec<_> = runner
        .calls()
        .into_iter()
        .filter(|arguments| arguments.first().map(String::as_str) == Some("send"))
        .collect();
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].last().map(String::as_str), Some("ls -la"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_gone_terminal_types_nothing_and_names_its_reason() -> TestResult {
    let tree = FakeTree::new(PID, vec![element("AXWindow", None, vec![])]);
    let runner = Arc::new(FakeRunner::default());
    let (services, _bundle) = terminal_services(&tree, Arc::clone(&runner))?;
    let inner = Arc::new(Method::default());
    let wrapper = AddressedMethod::new(Box::new(Shared(Arc::clone(&inner))), "ls", runner.clone());
    let report = dictate(&tree, services, &wrapper);
    assert_eq!(report.reason, Some(DestinationFailure::DefaultTerminalGone));
    assert!(!report.addressed);
    assert_eq!(inner.dispatched.load(Ordering::SeqCst), 0);
    assert!(runner
        .calls()
        .iter()
        .all(|arguments| arguments.first().map(String::as_str) != Some("send")));
    Ok(())
}
