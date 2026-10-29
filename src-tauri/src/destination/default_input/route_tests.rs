// LiveSpec traceability: @spec(AC-005) @spec(AC-006) @spec(AC-007)
//! Route choice and preparation with a scripted tree and tool (013 FR-005, FR-006, FR-007).
use super::*;
use crate::destination::context::ApplicationIdentity;
#[cfg(unix)]
use crate::destination::default_input::fake_tree::bundle_with_tool;
use crate::destination::default_input::fake_tree::{
    element, FakeElement, FakeTree, Fallible, OneBundle, TestResult,
};
use crate::destination::default_input::fingerprint::{fingerprint_of, SECURE_TEXT_FIELD};
use crate::destination::default_input::model::{AddressingTool, DefaultInput};
use crate::destination::default_input::terminal::tests::{FakeRunner, SURFACE, WORKSPACE};
use std::sync::Arc;

const PID: u32 = 42;
const BUNDLE_ID: &str = "com.example.composer";

fn codex_like() -> Vec<FakeElement> {
    vec![
        element("AXWindow", None, vec![1, 2]),
        element("AXGroup", Some("Conversation"), vec![]),
        element("AXTextArea", Some("Work with Codex"), vec![]),
    ]
}
fn app() -> RunningApp {
    RunningApp {
        pid: PID,
        identity: ApplicationIdentity::BundleId(BUNDLE_ID.to_owned()),
        display_name: "Composer".to_owned(),
        icon_ref: None,
    }
}
fn remembered(target: DefaultTarget) -> DefaultInput {
    DefaultInput {
        identity: app().identity,
        display_name: "Composer".to_owned(),
        target,
    }
}
fn field_target(elements: &[FakeElement]) -> Fallible<DefaultTarget> {
    let fingerprint = fingerprint_of(&elements[2].facts).map_err(|error| format!("{error:?}"))?;
    Ok(DefaultTarget::Field(fingerprint))
}
fn terminal_target() -> DefaultTarget {
    DefaultTarget::Terminal(TerminalTarget {
        tool: AddressingTool::Cmux,
        workspace_id: WORKSPACE.to_owned(),
        workspace_name: "roam-radar".to_owned(),
    })
}
fn services(tree: &FakeTree, inputs: Vec<DefaultInput>) -> DefaultInputServices {
    DefaultInputServices {
        supported: true,
        inputs: Box::new(move || inputs.clone()),
        tree: Arc::new(tree.clone()),
        ..DefaultInputServices::none()
    }
}
fn route(services: &DefaultInputServices, tree: &FakeTree) -> DefaultRoute {
    let clock = tree.clock.as_ref();
    prepare(services, Some(&app()), &|| false, (clock, clock))
}

// @spec AC-005: Editable focus keeps today's insertion — .specs/features/013-default-input-per-app/spec.md#ac-005
#[test]
fn the_plan_table_keeps_editable_and_password_focus_unchanged() -> TestResult {
    let field = field_target(&codex_like())?;
    let terminal = terminal_target();
    let unchanged = [
        (FocusClassification::Editable, &field),
        (FocusClassification::Editable, &terminal),
        (
            FocusClassification::Undetermined(Undetermined::SecureField),
            &terminal,
        ),
        (FocusClassification::PermissionMissing, &field),
    ];
    for (classification, target) in unchanged {
        assert_eq!(plan(classification, target), Plan::Unchanged);
    }
    let not_editable = FocusClassification::NotEditable;
    assert!(matches!(plan(not_editable, &field), Plan::SearchField(_)));
    let unknown = FocusClassification::Undetermined(Undetermined::UnknownRole);
    assert!(matches!(plan(unknown, &terminal), Plan::ResolveTerminal(_)));
    let untrusted = FocusClassification::PermissionMissing;
    assert!(matches!(
        plan(untrusted, &terminal),
        Plan::ResolveTerminal(_)
    ));
    Ok(())
}

// @spec AC-006: Unique field focused and verified — .specs/features/013-default-input-per-app/spec.md#ac-006
#[test]
fn a_conversation_focus_moves_to_the_unique_remembered_field() -> TestResult {
    let elements = codex_like();
    let target = field_target(&elements)?;
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(1);
    let services = services(&tree, vec![remembered(target)]);
    assert_eq!(route(&services, &tree), DefaultRoute::Focused);
    assert_eq!(tree.lock().focused, Some(2));
    Ok(())
}

#[test]
fn an_editable_focus_searches_nothing_and_changes_nothing() -> TestResult {
    let elements = codex_like();
    let target = field_target(&elements)?;
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(2);
    let services = services(&tree, vec![remembered(target)]);
    assert_eq!(route(&services, &tree), DefaultRoute::Unchanged);
    assert_eq!(tree.lock().reads, 0);
    Ok(())
}

#[test]
fn a_focused_password_field_is_left_alone() -> TestResult {
    let mut elements = codex_like();
    let target = field_target(&elements)?;
    elements.push(FakeElement {
        facts: crate::destination::default_input::fingerprint::NodeFacts {
            role: Some(SECURE_TEXT_FIELD.to_owned()),
            ..Default::default()
        },
        ..FakeElement::default()
    });
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(3);
    let services = services(&tree, vec![remembered(target)]);
    assert_eq!(route(&services, &tree), DefaultRoute::Unchanged);
    Ok(())
}

#[test]
fn no_match_or_an_ignored_focus_is_unavailable() -> TestResult {
    let elements = codex_like();
    let mut other =
        fingerprint_of(&elements[2].facts).map_err(|error| format!("fingerprint: {error:?}"))?;
    other.name = Some("Search".to_owned());
    let tree = FakeTree::new(PID, elements.clone());
    tree.lock().focused = Some(1);
    let missing = services(&tree, vec![remembered(DefaultTarget::Field(other))]);
    assert_eq!(
        route(&missing, &tree),
        DefaultRoute::Unavailable(DefaultKind::Field)
    );
    let deaf = FakeTree::new(PID, elements.clone());
    deaf.lock().focused = Some(1);
    deaf.lock().focus_applies = false;
    let ignored = services(&deaf, vec![remembered(field_target(&elements)?)]);
    assert_eq!(
        route(&ignored, &deaf),
        DefaultRoute::Unavailable(DefaultKind::Field)
    );
    Ok(())
}

#[test]
fn nothing_applies_without_an_input_a_target_or_after_a_cancellation() -> TestResult {
    let elements = codex_like();
    let target = field_target(&elements)?;
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(1);
    let empty = services(&tree, Vec::new());
    assert_eq!(route(&empty, &tree), DefaultRoute::Unchanged);
    let full = services(&tree, vec![remembered(target)]);
    let clock = tree.clock.as_ref();
    assert_eq!(
        prepare(&full, None, &|| false, (clock, clock)),
        DefaultRoute::Unchanged
    );
    assert_eq!(
        prepare(&full, Some(&app()), &|| true, (clock, clock)),
        DefaultRoute::Unchanged
    );
    assert_eq!(tree.lock().reads, 0);
    Ok(())
}

// @spec AC-007: Terminal resolved through the bundle's tool — .specs/features/013-default-input-per-app/spec.md#ac-007
#[cfg(unix)]
#[test]
fn a_terminal_target_resolves_through_the_bundles_tool() -> TestResult {
    let bundle = bundle_with_tool(0o755)?;
    let tree = FakeTree::new(PID, vec![element("AXWindow", None, vec![])]);
    tree.lock().focused = Some(0);
    let mut services = services(&tree, vec![remembered(terminal_target())]);
    services.bundles = Arc::new(OneBundle(bundle.path().to_path_buf()));
    services.runner = Arc::new(FakeRunner::with_workspace("roam-radar", "terminal"));
    match route(&services, &tree) {
        DefaultRoute::Terminal(resolved) => {
            assert_eq!(resolved.surface_id, SURFACE);
            assert!(resolved
                .cli
                .ends_with(AddressingTool::Cmux.bundle_relative_path()));
        }
        other => return Err(format!("unexpected route {other:?}").into()),
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_gone_workspace_or_a_missing_tool_is_unavailable() -> TestResult {
    let bundle = bundle_with_tool(0o755)?;
    let tree = FakeTree::new(PID, vec![element("AXWindow", None, vec![])]);
    let mut gone = services(&tree, vec![remembered(terminal_target())]);
    gone.bundles = Arc::new(OneBundle(bundle.path().to_path_buf()));
    gone.runner = Arc::new(FakeRunner::default());
    assert_eq!(
        route(&gone, &tree),
        DefaultRoute::Unavailable(DefaultKind::Terminal)
    );
    let no_tool = services(&tree, vec![remembered(terminal_target())]);
    assert_eq!(
        route(&no_tool, &tree),
        DefaultRoute::Unavailable(DefaultKind::Terminal)
    );
    Ok(())
}
