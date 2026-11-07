//! Live settings commit typed edits once, publish them, and leave disk and snapshot untouched on
//! refusal; offline authority stays fenced by the instance lease and the passive probe.
use super::{LiveSettings, OwnerProbe, OwnerState};
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode, ValidationReason},
    ownership::LegacyPathProbe,
    store::SettingsRepository,
};
use std::path::PathBuf;
type TestResult = Result<(), Box<dyn std::error::Error>>;

struct OccupiedProbe;
impl OwnerProbe for OccupiedProbe {
    fn inspect(&self) -> OwnerState {
        OwnerState::Occupied
    }
}
fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn absent(directory: &tempfile::TempDir) -> LegacyPathProbe {
    LegacyPathProbe::new(directory.path().join("absent.sock"))
}
fn owned(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    LiveSettings::start_owned(
        SettingsRepository::new(store(directory)),
        &absent(directory),
    )
}
#[test]
fn owned_edit_commits_once_publishes_and_reports_changed_keys() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let start = live.snapshot().revision.0;
    let commit = live.commit_edit(
        &|settings| {
            settings.custom_words = vec!["handy".into()];
            settings.audio_feedback = !settings.audio_feedback;
            Ok(())
        },
        None,
    )?;
    assert_eq!(commit.revision.0, start + 1);
    assert_eq!(commit.changed, vec!["audio_feedback", "custom_words"]);
    assert_eq!(commit.before.audio_feedback, !commit.after.audio_feedback);
    assert_eq!(live.snapshot().revision, commit.revision);
    let reread = SettingsRepository::new(store(&directory)).read_snapshot()?;
    assert_eq!(reread.snapshot.revision, commit.revision);
    assert_eq!(reread.snapshot.settings.custom_words, vec!["handy"]);
    Ok(())
}
/// Commit `edit` on a fresh instance; assert the refusal reason and that nothing changed.
fn assert_refused(
    edit: crate::settings_service::edit::SettingsEditFn<'_>,
    reason: ValidationReason,
) -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let bytes = std::fs::read(store(&directory))?;
    let revision = live.snapshot().revision;
    let refused = live
        .commit_edit(edit, None)
        .err()
        .ok_or("refused edit committed")?;
    assert_eq!(refused.reason(), Some(reason));
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    assert_eq!(live.snapshot().revision, revision);
    Ok(())
}
#[test]
fn refused_edit_changes_neither_disk_nor_published_snapshot() -> TestResult {
    assert_refused(
        &|settings| {
            settings.audio_feedback = !settings.audio_feedback;
            Err(SettingsError::validation(
                "post_process_prompts",
                ValidationReason::UnknownPrompt,
            ))
        },
        ValidationReason::UnknownPrompt,
    )
}
#[test]
fn gpu_without_device_edit_is_refused_by_the_shared_rule() -> TestResult {
    assert_refused(
        &|settings| {
            settings.transcribe_accelerator = crate::settings::TranscribeAcceleratorSetting::Gpu;
            settings.transcribe_gpu_device = None;
            Ok(())
        },
        ValidationReason::GpuDeviceRequired,
    )
}
#[test]
fn noop_edit_keeps_revision_and_reports_no_change() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let bytes = std::fs::read(store(&directory))?;
    let commit = live.commit_edit(&|_| Ok(()), None)?;
    assert!(commit.changed.is_empty());
    assert_eq!(commit.revision, live.snapshot().revision);
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    Ok(())
}
#[test]
fn offline_authority_is_refused_by_an_owner_and_commits_without_one() -> TestResult {
    let directory = tempfile::tempdir()?;
    let toggle = |settings: &mut crate::settings::AppSettings| {
        settings.start_hidden = !settings.start_hidden;
        Ok(())
    };
    let repository = || SettingsRepository::new(store(&directory));
    let occupied = LiveSettings::start_offline(repository(), Box::new(OccupiedProbe))?;
    let error = occupied
        .commit_edit(&toggle, None)
        .err()
        .ok_or("offline write accepted while the probe saw an owner")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert!(!store(&directory).exists());
    let instance = owned(&directory)?;
    let offline = LiveSettings::start_offline(repository(), Box::new(absent(&directory)))?;
    let fenced = offline
        .commit_edit(&toggle, None)
        .err()
        .ok_or("offline write accepted while the instance holds the lease")?;
    assert_eq!(fenced.code(), SettingsErrorCode::Unavailable);
    drop(instance);
    let committed = offline.commit_edit(&toggle, None)?;
    assert_eq!(committed.changed, vec!["start_hidden"]);
    Ok(())
}

/// Linux probe of decision 008: the session-bus answer classified with the lease-backed scope.
struct SessionBus(Result<bool, std::io::ErrorKind>);
impl OwnerProbe for SessionBus {
    fn inspect(&self) -> OwnerState {
        use crate::settings_service::ownership::probe::{classify_owner, ProbeScope};
        classify_owner(
            self.0.map_err(std::io::Error::from),
            ProbeScope::LeaseBackedSession,
        )
    }
}
fn toggle_start_hidden(settings: &mut crate::settings::AppSettings) -> Result<(), SettingsError> {
    settings.start_hidden = !settings.start_hidden;
    Ok(())
}
/// Offline commit through `bus`; the store bytes before and the result.
fn offline_through(
    directory: &tempfile::TempDir,
    bus: Result<bool, std::io::ErrorKind>,
) -> Result<Result<(), SettingsError>, Box<dyn std::error::Error>> {
    let offline = LiveSettings::start_offline(
        SettingsRepository::new(store(directory)),
        Box::new(SessionBus(bus)),
    )?;
    Ok(offline.commit_edit(&toggle_start_hidden, None).map(|_| ()))
}
#[test]
fn linux_free_session_bus_and_free_lease_allow_the_offline_write() -> TestResult {
    let directory = tempfile::tempdir()?;
    drop(owned(&directory)?);
    let before = std::fs::read(store(&directory))?;
    offline_through(&directory, Ok(false))??;
    assert_ne!(std::fs::read(store(&directory))?, before);
    Ok(())
}
#[test]
fn linux_free_session_bus_never_bypasses_a_held_lease_or_a_bus_owner() -> TestResult {
    let directory = tempfile::tempdir()?;
    let instance = owned(&directory)?;
    let bytes = std::fs::read(store(&directory))?;
    for bus in [Ok(false), Ok(true), Err(std::io::ErrorKind::NotConnected)] {
        let refused = offline_through(&directory, bus)?
            .err()
            .ok_or("offline write accepted while an instance holds the lease")?;
        assert_eq!(refused.code(), SettingsErrorCode::Unavailable);
    }
    drop(instance);
    for bus in [Ok(true), Err(std::io::ErrorKind::NotConnected)] {
        let refused = offline_through(&directory, bus)?
            .err()
            .ok_or("offline write accepted while the bus shows an owner or fails")?;
        assert_eq!(refused.code(), SettingsErrorCode::Unavailable);
    }
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    Ok(())
}
/// The lease is the authority of the Linux decision: the instance must hold it for its whole
/// life, through commits, refusals and reads by other handles, and release it only when dropped.
#[test]
fn instance_keeps_its_lease_until_it_is_dropped() -> TestResult {
    use crate::settings_service::route::instance_running;
    let directory = tempfile::tempdir()?;
    let instance = owned(&directory)?;
    for round in 0..3 {
        instance.commit_edit(&toggle_start_hidden, None)?;
        let _ = instance.commit_ui_value("paste_method", serde_json::json!("not_a_method"));
        SettingsRepository::new(store(&directory)).read_snapshot()?;
        assert!(
            instance_running(&store(&directory))?,
            "lease lost at {round}"
        );
    }
    drop(instance);
    assert!(!instance_running(&store(&directory))?);
    Ok(())
}

// @spec FR-002: A foreign rewrite is detected at startup — .specs/features/023-settings-backups/spec.md#fr-002
#[test]
fn settings_dropped_by_an_older_build_are_restored_at_the_next_start() -> TestResult {
    let directory = tempfile::tempdir()?;
    let revision = {
        let live = owned(&directory)?;
        live.commit_edit(
            &|settings| {
                settings.custom_words = vec!["kept".into()];
                settings.smart_spacing = true;
                Ok(())
            },
            None,
        )?;
        live.snapshot().revision.0
    };
    // An older build rewrites the store without the settings it does not know.
    let mut raw: serde_json::Value = serde_json::from_slice(&std::fs::read(store(&directory))?)?;
    if let Some(settings) = raw["settings"].as_object_mut() {
        settings.remove("custom_words");
        settings.remove("smart_spacing");
    }
    std::fs::write(store(&directory), serde_json::to_vec(&raw)?)?;

    let live = owned(&directory)?;
    let restored = live.snapshot();
    assert_eq!(restored.settings.custom_words, vec!["kept".to_string()]);
    assert!(restored.settings.smart_spacing);
    assert!(restored.revision.0 > revision);
    let set_aside = std::fs::read_dir(directory.path().join("settings_backups"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("overwritten-"))
        .count();
    assert_eq!(set_aside, 1);
    Ok(())
}

// @spec FR-003: A newer store is never rewritten — .specs/features/023-settings-backups/spec.md#fr-003
#[test]
fn a_store_from_a_newer_build_is_read_but_never_rewritten() -> TestResult {
    let directory = tempfile::tempdir()?;
    let newer = serde_json::json!({
        "settings_revision": 4,
        "settings": { "settings_schema_version": 999, "custom_words": ["future"] }
    });
    let bytes = serde_json::to_vec(&newer)?;
    std::fs::write(store(&directory), &bytes)?;
    let live = owned(&directory)?;
    let refused = live.commit_edit(
        &|settings| {
            settings.custom_words = vec![];
            Ok(())
        },
        None,
    );
    assert!(refused.is_err());
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    Ok(())
}
