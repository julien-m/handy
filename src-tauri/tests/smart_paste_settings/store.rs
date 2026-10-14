//! The macOS migration on real stores: startup persists it once with its marker under one
//! revision, replays change nothing, read-only reads never write it, and a later Cmd+V choice
//! survives every reload. With the gate closed, no method moves and the beta switch stays.
use super::active;
use handy_app_lib::settings_service::{
    ownership::LegacyPathProbe,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
type TestResult = Result<(), Box<dyn std::error::Error>>;
type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

/// Store of an installation upgraded from a build without smart paste (no marker), revision 4.
fn upgraded_store(
    directory: &tempfile::TempDir,
    method: &str,
    beta: bool,
) -> Fallible<SettingsRepository> {
    let path = directory.path().join("settings_store.json");
    let stored = json!({
        "settings_revision": 4,
        "settings": {"paste_method": method, "reliable_paste": beta, "unknown_payload": "KEEP"}
    });
    std::fs::write(&path, serde_json::to_vec(&stored)?)?;
    Ok(SettingsRepository::new(path))
}
fn disk(directory: &tempfile::TempDir) -> Fallible<Value> {
    let bytes = std::fs::read(directory.path().join("settings_store.json"))?;
    Ok(serde_json::from_slice(&bytes)?)
}
fn probe(directory: &tempfile::TempDir) -> LegacyPathProbe {
    LegacyPathProbe::new(directory.path().join("absent.sock"))
}
/// Start the running instance once (startup normalization) and release its lease.
fn start(repository: &SettingsRepository, directory: &tempfile::TempDir) -> Fallible<Revision> {
    let (read, _lease) = repository.initialize(&probe(directory))?;
    Ok(read.snapshot.revision)
}

// @spec FR-009: One-time macOS migration persisted at startup — .specs/features/007-smart-paste/spec.md#fr-009
#[test]
fn startup_moves_cmd_v_to_smart_paste_once_with_its_marker() -> TestResult {
    for beta in [true, false] {
        let directory = tempfile::tempdir()?;
        let repository = upgraded_store(&directory, "ctrl_v", beta)?;
        assert_eq!(start(&repository, &directory)?, Revision(5));
        let stored = disk(&directory)?;
        let (method, marker, kept_beta) = if active() {
            ("smart_paste", 1, false)
        } else {
            ("ctrl_v", 0, beta)
        };
        assert_eq!(stored["settings"]["paste_method"], method);
        assert_eq!(stored["settings"]["smart_paste_migration_version"], marker);
        assert_eq!(stored["settings"]["reliable_paste"], kept_beta);
        assert_eq!(stored["settings"]["unknown_payload"], "KEEP");
        assert_eq!(start(&repository, &directory)?, Revision(5));
    }
    Ok(())
}

#[test]
fn startup_keeps_every_other_method() -> TestResult {
    for method in [
        "direct",
        "none",
        "shift_insert",
        "ctrl_shift_v",
        "external_script",
    ] {
        let directory = tempfile::tempdir()?;
        let repository = upgraded_store(&directory, method, true)?;
        start(&repository, &directory)?;
        let stored = disk(&directory)?;
        assert_eq!(stored["settings"]["paste_method"], method);
        assert_eq!(stored["settings"]["reliable_paste"], !active());
        let marker = u32::from(active());
        assert_eq!(stored["settings"]["smart_paste_migration_version"], marker);
    }
    Ok(())
}

#[test]
fn a_later_cmd_v_choice_survives_every_reload() -> TestResult {
    let directory = tempfile::tempdir()?;
    let repository = upgraded_store(&directory, "ctrl_v", false)?;
    let (_, lease) = repository.initialize(&probe(&directory))?;
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    let chosen = repository.commit_owned(
        &lease,
        validate_cli("paste_method", "ctrl_v", &context)?,
        None,
    )?;
    let expected_revision = if active() { Revision(6) } else { Revision(5) };
    assert_eq!(chosen.snapshot.revision, expected_revision);
    drop(lease);
    assert_eq!(start(&repository, &directory)?, expected_revision);
    let reread = repository.read_snapshot()?.snapshot;
    assert_eq!(
        serde_json::to_value(reread.public_settings())?["paste_method"],
        "ctrl_v"
    );
    Ok(())
}

#[test]
fn read_only_reads_normalize_in_memory_without_writing_the_marker() -> TestResult {
    let directory = tempfile::tempdir()?;
    let repository = upgraded_store(&directory, "ctrl_v", true)?;
    let before = std::fs::read(directory.path().join("settings_store.json"))?;
    let read = repository.read_snapshot()?.snapshot;
    let public = serde_json::to_value(read.public_settings())?;
    let method = if active() { "smart_paste" } else { "ctrl_v" };
    assert_eq!(public["paste_method"], method);
    assert_eq!(public["smart_paste_migration_version"], u32::from(active()));
    assert_eq!(read.revision, Revision(4));
    assert_eq!(
        std::fs::read(directory.path().join("settings_store.json"))?,
        before
    );
    Ok(())
}

#[test]
fn a_new_installation_defaults_to_smart_paste_only_when_active() -> TestResult {
    let directory = tempfile::tempdir()?;
    let absent = SettingsRepository::new(directory.path().join("new").join("settings_store.json"));
    let public = serde_json::to_value(absent.read_snapshot()?.snapshot.public_settings())?;
    let method = if active() {
        "smart_paste"
    } else if cfg!(target_os = "linux") {
        "direct"
    } else {
        "ctrl_v"
    };
    assert_eq!(public["paste_method"], method);
    assert_eq!(public["smart_paste_migration_version"], u32::from(active()));
    assert_eq!(public["reliable_paste"], false);
    Ok(())
}
