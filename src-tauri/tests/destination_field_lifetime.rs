//! Lifetime of 011's remembered input in 006's session (plan "Lifetime and destination match",
//! spec AC-003, AC-006, AC-008): a badge choice of another application, the cross, a pin of
//! another application or an unpin that changes the instance release it at once; the same
//! application chosen again, or pinned in place, keeps it; a cancellation, a new dictation and
//! the end of the session release it; paste method "none" releases it unused, with no focus,
//! range or verification call. "Released" is observed as the element handle being dropped.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use destination::{comet, notes, textedit, Method};
use handy_app_lib::destination::snapshot::{DestinationSnapshot, MenuChoice};
use rig::FieldRig;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rig() -> FieldRig {
    FieldRig::new(comet(), vec![comet(), notes(), textedit()])
}
/// Badge choice of the application named `name` for `started`.
fn choose(field: &FieldRig, started: &DestinationSnapshot, name: &str) -> TestResult {
    let version = field
        .rig
        .manager
        .snapshot()
        .ok_or("session")?
        .selection_version;
    let menu = field.rig.manager.menu(started.dictation_id, version)?;
    let token = menu
        .entries
        .iter()
        .find(|entry| entry.display_name == name)
        .map(|entry| entry.token)
        .ok_or("menu entry")?;
    field.rig.manager.choose(
        started.dictation_id,
        version,
        MenuChoice::Application(token),
    )?;
    Ok(())
}
/// Whether the input remembered for `started` comes out of the insertion freeze.
fn kept(
    field: &FieldRig,
    started: &DestinationSnapshot,
) -> Result<bool, Box<dyn std::error::Error>> {
    field.rig.manager.mark_processing();
    let (_, remembered) = field
        .rig
        .manager
        .freeze_with_field(started.dictation_id, started.destination_generation)?;
    Ok(remembered.is_some())
}

// @spec AC-003: Dropped by a badge choice or an unpin — .specs/features/011-dictation-field-return/spec.md#ac-003
#[test]
fn another_application_from_the_badge_or_the_cross_releases_the_input() -> TestResult {
    for name in ["TextEdit", "cross"] {
        let field = rig();
        let started = field.start(true);
        assert_eq!(field.world.alive(), 1, "remembered");
        if name == "cross" {
            let version = field
                .rig
                .manager
                .snapshot()
                .ok_or("session")?
                .selection_version;
            field
                .rig
                .manager
                .choose(started.dictation_id, version, MenuChoice::FrontmostAtEnd)?;
        } else {
            choose(&field, &started, name)?;
            choose(&field, &started, "Comet")?;
        }
        assert_eq!(
            field.world.alive(),
            0,
            "{name}: released at once, never resurrected"
        );
        assert!(!kept(&field, &started)?, "{name}");
    }
    Ok(())
}
#[test]
fn the_same_application_chosen_again_or_pinned_in_place_keeps_the_input() -> TestResult {
    let field = rig();
    let started = field.start(true);
    choose(&field, &started, "Comet")?;
    let version = field
        .rig
        .manager
        .snapshot()
        .ok_or("session")?
        .selection_version;
    field
        .rig
        .manager
        .pin_current(started.dictation_id, version)?;
    assert!(kept(&field, &started)?, "same instance, pinned in place");
    Ok(())
}
#[test]
fn a_pin_of_another_application_or_an_unpin_to_another_instance_releases_it() -> TestResult {
    let field = rig();
    field.start(true);
    field.rig.manager.pin_identifier("com.apple.TextEdit")?;
    assert_eq!(field.world.alive(), 0, "pin of another application");
    // Pinned Comet at the start from Notes, then an unpin back to Notes (the start route).
    let field = rig();
    let first = field.rig.start(true);
    field.rig.manager.pin_current(first.dictation_id, 0)?;
    field.rig.workspace.bring(notes());
    let started = field.start(true);
    assert_eq!(field.world.alive(), 1, "pinned Comet's input");
    field.rig.manager.unpin();
    assert_eq!(field.world.alive(), 0, "unpin to another instance");
    assert!(!kept(&field, &started)?);
    Ok(())
}
// @spec AC-006: Cancel or method none drop it unused — .specs/features/011-dictation-field-return/spec.md#ac-006
#[test]
fn a_cancellation_a_new_dictation_or_the_end_releases_the_input() -> TestResult {
    let field = rig();
    field.start(true);
    field.rig.manager.cancel();
    assert_eq!(field.world.alive(), 0, "cancel");
    let field = rig();
    field.start(true);
    field.start(true);
    assert_eq!(field.world.alive(), 1, "only the new dictation's input");
    let field = rig();
    let started = field.start(true);
    let method = Method::succeeding();
    field.rig.deliver(&started, &method, false);
    field
        .rig
        .manager
        .finish(started.dictation_id, started.destination_generation);
    assert_eq!(field.world.alive(), 0, "end of the session");
    Ok(())
}
#[test]
fn paste_method_none_releases_the_input_unused() {
    let field = rig();
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    let calls = field.world.calls().len();
    let method = Method::succeeding();
    field.rig.deliver(&started, &method, true);
    assert_eq!(
        field.world.calls().len(),
        calls,
        "no note, focus, range or verification"
    );
    assert_eq!(field.world.alive(), 0);
    assert!(field.rig.workspace.activations().is_empty());
}
#[test]
fn return_off_with_nothing_pinned_remembers_nothing_and_pastes_at_the_end() {
    let field = rig();
    let started = field.start(false);
    assert!(field.world.calls().is_empty());
    field.rig.workspace.bring(notes());
    let method = Method::succeeding();
    field.rig.deliver(&started, &method, false);
    let target = method
        .dispatched()
        .first()
        .and_then(|context| context.target.clone());
    assert_eq!(
        target.map(|target| target.pid),
        Some(notes().pid),
        "frontmost at the end"
    );
    assert!(field.world.calls().is_empty());
}
