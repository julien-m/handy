//! Context source of smart capitalization (015, spec AC-007) on 006's real manager with 011's
//! remembered input: the text before the remembered caret is read once, bounded, from that
//! element; without a remembered input nothing is read from it; a stale dictation is Stale.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use handy_app_lib::destination::field_context::{InsertionTarget, RememberedText};
use handy_app_lib::destination::snapshot::{DestinationSnapshot, MenuChoice};
use handy_app_lib::smart_capitalization::reader::{
    context_from_units, window_before, CONTEXT_BUDGET,
};
use handy_app_lib::smart_capitalization::{decide, CaseDecision, CursorContext};
use rig::{FieldRig, CARET, INPUT};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The remembered-caret read of `started`.
fn read(field: &FieldRig, started: &DestinationSnapshot) -> Option<RememberedText> {
    let key = (started.dictation_id, started.destination_generation);
    field
        .rig
        .manager
        .read_remembered_before_caret(key, window_before, CONTEXT_BUDGET)
}

/// Whether 011's input of `started` still comes out of the insertion freeze.
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

/// The badge-menu token of the application named `name` during `started`, and its version.
fn token(
    field: &FieldRig,
    started: &DestinationSnapshot,
    name: &str,
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
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
    Ok((token, version))
}

// @spec AC-007: Context of 011's remembered caret — .specs/features/015-smart-capitalization/spec.md#ac-007
#[test]
fn remembered_caret_is_used() -> TestResult {
    let field = FieldRig::comet();
    // CARET is 12: the twelve units before it end with a comma.
    field.world.text(INPUT, "Hello world, here we go");
    let started = field.start(true);
    let answer = read(&field, &started).ok_or("a remembered input")?;
    let (units, window) = answer.map_err(|error| format!("{error:?}"))?;
    assert_eq!(window, (0, CARET.0));
    let context = context_from_units(&units, window);
    assert_eq!(
        context,
        CursorContext::Before {
            text: "Hello world,".to_owned(),
            reaches_start: true,
        }
    );
    assert_eq!(decide(&context), CaseDecision::Lower);
    assert_eq!(field.calls("text_in"), 1);
    // The input was read outside the lock and put back: 011 still returns to it.
    assert!(kept(&field, &started)?);
    Ok(())
}

#[test]
fn an_element_without_a_bounded_read_is_an_error() -> TestResult {
    let field = FieldRig::comet();
    let started = field.start(true);
    let answer = read(&field, &started).ok_or("a remembered input")?;
    assert!(answer.is_err(), "the caller treats it as Absent");
    Ok(())
}

#[test]
fn no_remembered_input_reads_nothing() -> TestResult {
    let field = FieldRig::comet();
    field.world.text(INPUT, "Hello world, here we go");
    let started = field.start(false);
    assert!(read(&field, &started).is_none());
    assert_eq!(field.calls("text_in"), 0);
    Ok(())
}

#[test]
fn insertion_target_names_the_destination_or_stale() -> TestResult {
    let field = FieldRig::comet();
    let started = field.start(true);
    let (id, generation) = (started.dictation_id, started.destination_generation);
    assert!(matches!(
        field.rig.manager.insertion_target(id, generation),
        InsertionTarget::Application { .. } | InsertionTarget::Frontmost
    ));
    assert_eq!(
        field.rig.manager.insertion_target(id, generation + 1),
        InsertionTarget::Stale
    );
    Ok(())
}

#[test]
fn a_destination_change_during_the_read_releases_the_input() -> TestResult {
    let field = FieldRig::comet();
    field.world.text(INPUT, "Hello world, here we go");
    let started = field.start(true);
    let (notes, version) = token(&field, &started, "Notes")?;
    // A badge choice of Notes lands while the input is out of the manager for its read.
    let manager = std::sync::Arc::clone(&field.rig.manager);
    let id = started.dictation_id;
    field.world.after("text_in", move || {
        let _ = manager.choose(id, version, MenuChoice::Application(notes));
    });
    let alive = field.world.alive();
    assert!(read(&field, &started).is_some());
    // Released by the read itself, not only filtered later at the freeze.
    assert_eq!(field.world.alive(), alive - 1);
    assert!(!kept(&field, &started)?, "released at once, 011 AC-003");
    Ok(())
}
