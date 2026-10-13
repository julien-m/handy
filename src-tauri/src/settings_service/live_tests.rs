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
