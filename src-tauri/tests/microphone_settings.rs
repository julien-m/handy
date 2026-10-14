//! Feature 001, Step 2: the persisted microphone priority on a real temporary store. Migration of
//! the legacy single selection (absent key only, exact names, idempotent, legacy key kept for a
//! downgrade, original recoverable), whole-list commits through the shared transaction (one
//! revision, no-op repeat, compare-and-set) and the refusal of generic writes.
use handy_app_lib::microphones::priority::{priority_change, selector_change};
use handy_app_lib::settings_service::{
    descriptor::descriptor,
    errors::SettingsErrorCode,
    live::LiveSettings,
    operations::SettingsBackend,
    ownership::LegacyPathProbe,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::SettingsRepository,
    transaction::{PersistObserver, PersistStage},
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn probe(directory: &tempfile::TempDir) -> LegacyPathProbe {
    LegacyPathProbe::new(directory.path().join("absent.sock"))
}
/// A store at revision 3 holding `settings`, as an earlier build wrote it.
fn seeded(directory: &tempfile::TempDir, settings: Value) -> Fallible<SettingsRepository> {
    let document = json!({ "settings_revision": 3, "settings": settings });
    std::fs::write(store(directory), serde_json::to_vec(&document)?)?;
    Ok(SettingsRepository::new(store(directory)))
}
fn stored(directory: &tempfile::TempDir) -> Fallible<Value> {
    Ok(serde_json::from_slice(&std::fs::read(store(directory))?)?)
}
/// Start the instance on the store (normalization persisted once), then release it.
fn startup(directory: &tempfile::TempDir) -> Fallible<Revision> {
    let repository = SettingsRepository::new(store(directory));
    let (read, _lease) = repository.initialize(&probe(directory))?;
    Ok(read.snapshot.revision)
}
fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|name| (*name).to_owned()).collect()
}

// @spec AC-006: Legacy selection migrated once and kept — .specs/features/001-microphone-priority/spec.md#ac-006
#[test]
fn a_legacy_name_becomes_the_only_choice_once_and_stays_for_a_downgrade() -> TestResult {
    let directory = tempfile::tempdir()?;
    let legacy = json!({"selected_microphone": "USB headset", "clamshell_microphone": "Dock",
                        "unknown_field": "CANARY"});
    seeded(&directory, legacy)?;
    let first = startup(&directory)?;
    let after = stored(&directory)?;
    let settings = &after["settings"];
    assert_eq!(settings["microphone_priority"], json!(["USB headset"]));
    assert_eq!(settings["selected_microphone"], "USB headset");
    assert_eq!(settings["clamshell_microphone"], "Dock");
    assert_eq!(settings["unknown_field"], "CANARY");
    let bytes = std::fs::read(store(&directory))?;
    assert_eq!(startup(&directory)?, first);
    assert_eq!(
        std::fs::read(store(&directory))?,
        bytes,
        "second load is a no-op"
    );
    Ok(())
}
#[test]
fn null_absent_and_literal_default_legacy_values_migrate_exactly() -> TestResult {
    for (legacy, expected) in [
        (json!({"selected_microphone": null}), json!([])),
        (json!({}), json!([])),
        (
            json!({"selected_microphone": "default"}),
            json!(["default"]),
        ),
        (
            json!({"selected_microphone": "Default"}),
            json!(["Default"]),
        ),
    ] {
        let directory = tempfile::tempdir()?;
        seeded(&directory, legacy.clone())?;
        startup(&directory)?;
        let after = stored(&directory)?;
        assert_eq!(
            after["settings"]["microphone_priority"], expected,
            "{legacy}"
        );
    }
    Ok(())
}
#[test]
fn an_explicit_list_wins_over_the_legacy_key_even_when_empty() -> TestResult {
    for explicit in [json!([]), json!(["MacBook microphone"])] {
        let directory = tempfile::tempdir()?;
        let settings = json!({"selected_microphone": "USB", "microphone_priority": explicit});
        seeded(&directory, settings)?;
        startup(&directory)?;
        assert_eq!(
            stored(&directory)?["settings"]["microphone_priority"],
            explicit
        );
    }
    Ok(())
}
#[test]
fn a_headless_read_migrates_in_memory_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let repository = seeded(&directory, json!({"selected_microphone": "USB"}))?;
    let before = std::fs::read(store(&directory))?;
    let public = serde_json::to_value(repository.read_snapshot()?.snapshot.public_settings())?;
    assert_eq!(public["microphone_priority"], json!(["USB"]));
    assert_eq!(std::fs::read(store(&directory))?, before);
    Ok(())
}

struct FailAt(PersistStage);
impl PersistObserver for FailAt {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == self.0 {
            return Err(std::io::Error::other("injected filesystem failure"));
        }
        Ok(())
    }
}
fn context() -> ValidationContext<'static> {
    ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    }
}
#[test]
fn a_failed_write_leaves_the_legacy_store_recoverable() -> TestResult {
    let directory = tempfile::tempdir()?;
    let repository = seeded(&directory, json!({"selected_microphone": "USB"}))?;
    let before = std::fs::read(store(&directory))?;
    let mutation = validate_cli("audio_feedback", "true", &context())?;
    let failure = FailAt(PersistStage::BeforeReplace);
    let refused = repository.commit_with_probe(mutation, None, &probe(&directory), &failure);
    assert!(refused.is_err());
    assert_eq!(std::fs::read(store(&directory))?, before);
    Ok(())
}

/// The CLI's offline backend on `directory`.
fn offline(directory: &tempfile::TempDir) -> Fallible<LiveSettings> {
    let repository = SettingsRepository::new(store(directory));
    Ok(LiveSettings::start_offline(
        repository,
        Box::new(probe(directory)),
    )?)
}
// @spec AC-007: One complete-order commit, no partial state — .specs/features/001-microphone-priority/spec.md#ac-007
#[test]
fn a_valid_complete_order_commits_once_and_a_repeat_is_a_no_op() -> TestResult {
    let directory = tempfile::tempdir()?;
    seeded(
        &directory,
        json!({"microphone_priority": ["USB"], "clamshell_microphone": "Dock"}),
    )?;
    let backend = offline(&directory)?;
    let change = priority_change(names(&["MacBook microphone", "USB"]))?;
    let committed = backend.commit_change(&change, Some(Revision(3)))?;
    assert_eq!(committed.snapshot.revision, Revision(4));
    let repeat = backend.commit_change(&change, None)?;
    assert_eq!(repeat.snapshot.revision, Revision(4));
    let settings = &stored(&directory)?["settings"];
    assert_eq!(
        settings["microphone_priority"],
        json!(["MacBook microphone", "USB"])
    );
    assert_eq!(settings["clamshell_microphone"], "Dock");
    Ok(())
}
#[test]
fn invalid_or_stale_orders_change_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    seeded(&directory, json!({"microphone_priority": ["USB"]}))?;
    let before = std::fs::read(store(&directory))?;
    let duplicate = priority_change(names(&["Mac", "Mac"]))
        .err()
        .ok_or("accepted")?;
    assert_eq!(duplicate.code(), SettingsErrorCode::ValidationFailed);
    let backend = offline(&directory)?;
    let stale = backend
        .commit_change(&priority_change(names(&["Mac"]))?, Some(Revision(2)))
        .err()
        .ok_or("stale revision accepted")?;
    assert_eq!(stale.code(), SettingsErrorCode::RevisionConflict);
    assert_eq!(stale.current_revision(), Some(Revision(3)));
    assert_eq!(std::fs::read(store(&directory))?, before);
    Ok(())
}
#[test]
fn generic_set_of_either_microphone_key_is_refused_with_the_domain_reason() -> TestResult {
    for (key, value) in [
        ("selected_microphone", "USB"),
        ("microphone_priority", "[]"),
    ] {
        let refused = validate_cli(key, value, &context())
            .err()
            .ok_or("accepted")?;
        let wire = serde_json::to_value(&refused)?;
        assert_eq!(wire["code"], "validation_failed", "{key}");
        assert_eq!(wire["details"]["field"], key);
        assert_eq!(wire["details"]["reason"], "microphones_domain");
        assert!(wire["message"].is_string() && !wire.to_string().contains("USB"));
    }
    let legacy = descriptor("selected_microphone").ok_or("missing descriptor")?;
    assert!(legacy.read_only);
    Ok(())
}
/// Review r3, warning 1: until the priority editor ships, the legacy single selector writes the
/// priority through the same transaction instead of the now read-only legacy key.
#[test]
fn the_legacy_selector_moves_its_choice_first_and_default_empties_the_list() -> TestResult {
    let directory = tempfile::tempdir()?;
    seeded(
        &directory,
        json!({"microphone_priority": ["USB", "Mac"], "selected_microphone": "USB"}),
    )?;
    let backend = offline(&directory)?;
    backend.commit_change(&selector_change(Some("Dock".to_owned()))?, None)?;
    let settings = stored(&directory)?["settings"].clone();
    assert_eq!(
        settings["microphone_priority"],
        json!(["Dock", "USB", "Mac"])
    );
    assert_eq!(
        settings["selected_microphone"], "USB",
        "legacy key never written"
    );
    backend.commit_change(&selector_change(None)?, None)?;
    assert_eq!(
        stored(&directory)?["settings"]["microphone_priority"],
        json!([])
    );
    assert!(selector_change(Some("  ".to_owned())).is_err());
    Ok(())
}
