//! Vocabulary edits through the real settings service on temporary stores: legacy terms, order and
//! correction threshold survive the upgrade and unrelated writes (SC-002), refusals and stale
//! revisions leave the file untouched, a bulk paste commits atomically once, and no-ops write
//! nothing.
use handy_app_lib::recognition_vocabulary::{
    edits::VocabularyEdit, preview::ImportMode, validation::MAX_TERMS,
};
use handy_app_lib::settings_service::{
    errors::{ErrorReason, SettingsErrorCode},
    live::LiveSettings,
    operations::SettingsBackend,
    ownership::LegacyPathProbe,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Store {
    directory: tempfile::TempDir,
}
impl Store {
    fn with(settings: Value, revision: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let raw = json!({ "settings_revision": revision, "settings": settings });
        std::fs::write(
            directory.path().join("settings_store.json"),
            serde_json::to_vec(&raw)?,
        )?;
        Ok(Self { directory })
    }
    fn path(&self) -> PathBuf {
        self.directory.path().join("settings_store.json")
    }
    fn service(&self) -> Result<LiveSettings, Box<dyn std::error::Error>> {
        let probe = LegacyPathProbe::new(self.directory.path().join("absent.sock"));
        Ok(LiveSettings::start_offline(
            SettingsRepository::new(self.path()),
            Box::new(probe),
        )?)
    }
    fn disk(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(&std::fs::read(self.path())?)?)
    }
    fn bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(std::fs::read(self.path())?)
    }
}
fn legacy_words() -> Value {
    let mut words = vec![
        json!("Handy"),
        json!("handy"),
        json!("<legacy>"),
        json!("  spaced  "),
        json!(""),
    ];
    // Five legacy entries and `MAX_TERMS - 2` terms: three occurrences over the limit.
    words.extend((0..MAX_TERMS - 2).map(|index| json!(format!("term {index}"))));
    Value::Array(words)
}
fn commit(
    service: &LiveSettings,
    edit: VocabularyEdit,
    expected: u64,
) -> Result<u64, handy_app_lib::settings_service::errors::SettingsError> {
    let read = service.commit_change(&edit.into_change(), Some(Revision(expected)))?;
    Ok(read.snapshot.revision.0)
}

#[test]
fn legacy_terms_order_and_threshold_survive_upgrade_and_unrelated_writes() -> TestResult {
    let store = Store::with(
        json!({ "custom_words": legacy_words(), "word_correction_threshold": 0.27, "audio_feedback": true }),
        4,
    )?;
    let service = store.service()?;
    let mutation = validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
            platform: PlatformCapabilities::current(),
        },
    )?;
    let read = service.commit(mutation, Some(Revision(4)))?;
    assert_eq!(read.snapshot.revision, Revision(5));
    let disk = store.disk()?;
    assert_eq!(disk["settings"]["custom_words"], legacy_words());
    assert_eq!(disk["settings"]["word_correction_threshold"], json!(0.27));
    Ok(())
}

#[test]
fn removal_shrinks_an_over_capacity_legacy_list_one_occurrence_at_a_time() -> TestResult {
    let store = Store::with(json!({ "custom_words": legacy_words() }), 1)?;
    let service = store.service()?;
    let full = commit(&service, VocabularyEdit::Add("New".into()), 1);
    assert_eq!(
        full.err().map(|e| e.code()),
        Some(SettingsErrorCode::LimitExceeded)
    );
    let edit = VocabularyEdit::RemoveOccurrence {
        index: 1,
        term: "handy".into(),
    };
    assert_eq!(commit(&service, edit, 1)?, 2);
    let words = store.disk()?["settings"]["custom_words"].clone();
    assert_eq!(words[0], "Handy");
    assert_eq!(words[1], "<legacy>");
    assert_eq!(words.as_array().map(Vec::len), Some(MAX_TERMS + 2));
    Ok(())
}

#[test]
fn deduplicated_paste_commits_once_and_invalid_paste_writes_nothing() -> TestResult {
    let store = Store::with(json!({ "custom_words": [] }), 8)?;
    let service = store.service()?;
    let before = store.bytes()?;
    let invalid = VocabularyEdit::Import {
        input: "Claude Code\n<b>\n".into(),
        mode: ImportMode::Merge,
    };
    let refused = commit(&service, invalid, 8)
        .err()
        .ok_or("invalid paste committed")?;
    assert_eq!(refused.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(
        refused.items()[0].code(),
        ErrorReason::TermForbiddenCharacter
    );
    assert_eq!(store.bytes()?, before);
    let paste = VocabularyEdit::Import {
        input: "Claude Code\nClaude Sonnet\nclaude code\nOpenClaw\n".into(),
        mode: ImportMode::Merge,
    };
    assert_eq!(commit(&service, paste, 8)?, 9);
    assert_eq!(
        store.disk()?["settings"]["custom_words"],
        json!(["Claude Code", "Claude Sonnet", "OpenClaw"])
    );
    Ok(())
}

#[test]
fn stale_revision_is_refused_and_a_no_op_writes_nothing() -> TestResult {
    let store = Store::with(json!({ "custom_words": ["OpenClaw"] }), 3)?;
    let service = store.service()?;
    let before = store.bytes()?;
    let stale = commit(&service, VocabularyEdit::Add("Claude".into()), 2);
    let stale = stale.err().ok_or("stale revision committed")?;
    assert_eq!(stale.code(), SettingsErrorCode::RevisionConflict);
    assert_eq!(stale.current_revision(), Some(Revision(3)));
    let same = VocabularyEdit::Replace(vec!["OpenClaw".into()]);
    assert_eq!(commit(&service, same, 3)?, 3);
    let empty_merge = VocabularyEdit::Import {
        input: String::new(),
        mode: ImportMode::Merge,
    };
    assert_eq!(commit(&service, empty_merge, 3)?, 3);
    assert_eq!(store.bytes()?, before);
    Ok(())
}
