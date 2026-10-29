// LiveSpec traceability: @spec(AC-001) @spec(AC-002) @spec(AC-003)
//! Menu offer and capture with a scripted tree and tool (013 FR-001 to FR-003).
use super::*;
#[cfg(unix)]
use crate::destination::default_input::fake_tree::bundle_with_tool;
use crate::destination::default_input::fake_tree::{
    element, FakeElement, FakeTree, Fallible, OneBundle, TestResult,
};
use crate::destination::default_input::fingerprint::{NodeFacts, SECURE_TEXT_FIELD};
use crate::destination::default_input::terminal::tests::{FakeRunner, WORKSPACE};
use std::sync::Arc;

const PID: u32 = 7;

fn instance() -> DestinationInstance {
    DestinationInstance {
        identity: ApplicationIdentity::BundleId("com.example.composer".to_owned()),
        display_name: "Composer".to_owned(),
        pid: PID,
    }
}
fn tree(focused: usize, elements: Vec<FakeElement>) -> FakeTree {
    let tree = FakeTree::new(PID, elements);
    tree.lock().focused = Some(focused);
    tree
}
fn services(tree: &FakeTree) -> DefaultInputServices {
    DefaultInputServices {
        supported: true,
        tree: Arc::new(tree.clone()),
        ..DefaultInputServices::none()
    }
}
fn composer() -> Vec<FakeElement> {
    let mut composer = element("AXTextArea", None, vec![]);
    composer.facts.description = Some("Work with Codex".to_owned());
    composer.facts.dom_id = Some("_ComposerLayoutBody_1q7o3_2".to_owned());
    vec![
        element("AXWindow", None, vec![1, 2]),
        element("AXGroup", Some("Conversation"), vec![]),
        composer,
    ]
}

// @spec AC-002: Editable, named, non-password field only — .specs/features/013-default-input-per-app/spec.md#ac-002
#[test]
fn the_focused_composer_is_remembered_without_its_generated_id() -> TestResult {
    let tree = tree(2, composer());
    let input = capture(&services(&tree), &instance(), tree.clock.as_ref())
        .map_err(|error| format!("captured: {error:?}"))?;
    let DefaultTarget::Field(fingerprint) = input.target else {
        return Err("field expected".into());
    };
    assert_eq!(fingerprint.role, "AXTextArea");
    assert_eq!(fingerprint.name.as_deref(), Some("Work with Codex"));
    assert_eq!(fingerprint.dom_id, None);
    assert_eq!(input.identity, instance().identity);
    Ok(())
}

#[test]
fn a_conversation_focus_or_a_password_field_is_refused() -> TestResult {
    let conversation = tree(1, composer());
    assert_eq!(
        capture(
            &services(&conversation),
            &instance(),
            conversation.clock.as_ref()
        ),
        Err(RememberRefusal::NotTextField)
    );
    let mut elements = composer();
    elements[2].facts.subrole = Some(SECURE_TEXT_FIELD.to_owned());
    let password = tree(2, elements);
    assert_eq!(
        capture(&services(&password), &instance(), password.clock.as_ref()),
        Err(RememberRefusal::PasswordField)
    );
    Ok(())
}

#[test]
fn an_unnamed_field_or_an_untrusted_process_is_refused() -> TestResult {
    let mut elements = composer();
    elements[2].facts = NodeFacts {
        role: Some("AXTextArea".to_owned()),
        ..NodeFacts::default()
    };
    let unnamed = tree(2, elements);
    assert_eq!(
        capture(&services(&unnamed), &instance(), unnamed.clock.as_ref()),
        Err(RememberRefusal::Unrecognizable)
    );
    let untrusted = tree(2, composer());
    untrusted.lock().trusted = false;
    assert_eq!(
        capture(&services(&untrusted), &instance(), untrusted.clock.as_ref()),
        Err(RememberRefusal::NotTextField)
    );
    Ok(())
}

#[cfg(unix)]
fn terminal_services(
    tree: &FakeTree,
    runner: FakeRunner,
) -> Fallible<(DefaultInputServices, tempfile::TempDir)> {
    let bundle = bundle_with_tool(0o755)?;
    let mut services = services(tree);
    services.bundles = Arc::new(OneBundle(bundle.path().to_path_buf()));
    services.runner = Arc::new(runner);
    Ok((services, bundle))
}

// @spec AC-001: Terminal entry for a bundle with its tool — .specs/features/013-default-input-per-app/spec.md#ac-001
#[cfg(unix)]
#[test]
fn the_offer_follows_the_bundle_capability() -> TestResult {
    let field_tree = tree(2, composer());
    let field_offer = offer(&services(&field_tree), &instance()).ok_or("offer")?;
    assert_eq!(field_offer.kind, DefaultInputKind::Field);
    assert!(!field_offer.remembered);
    let terminal_tree = tree(0, vec![element("AXWindow", None, vec![])]);
    let (services, _bundle) = terminal_services(&terminal_tree, FakeRunner::default())?;
    let terminal_offer = offer(&services, &instance()).ok_or("offer")?;
    assert_eq!(terminal_offer.kind, DefaultInputKind::Terminal);
    assert!(offer(&DefaultInputServices::none(), &instance()).is_none());
    Ok(())
}

// @spec AC-003: Workspace UUID and name, never a ref — .specs/features/013-default-input-per-app/spec.md#ac-003
#[cfg(unix)]
#[test]
fn a_terminal_is_remembered_by_its_workspace() -> TestResult {
    let terminal_tree = tree(0, vec![element("AXWindow", None, vec![])]);
    let runner = FakeRunner::with_workspace("roam-radar", "terminal");
    let (services, _bundle) = terminal_services(&terminal_tree, runner)?;
    let input = capture(&services, &instance(), terminal_tree.clock.as_ref())
        .map_err(|error| format!("captured: {error:?}"))?;
    let DefaultTarget::Terminal(target) = input.target else {
        return Err("terminal expected".into());
    };
    assert_eq!(target.workspace_id, WORKSPACE);
    assert_eq!(target.workspace_name, "roam-radar");
    let (silent, _bundle) = terminal_services(&terminal_tree, FakeRunner::default())?;
    assert_eq!(
        capture(&silent, &instance(), terminal_tree.clock.as_ref()),
        Err(RememberRefusal::TerminalUnreadable)
    );
    Ok(())
}
