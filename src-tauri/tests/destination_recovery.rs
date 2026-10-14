//! 006 history rule (AC-005 as refined on 2026-10-14, plan Step 6) on the real history store:
//! a dictation whose WAV was saved gets a row with its audio, one whose WAV failed gets a
//! text-only row that is reported `saved`, a failed `save_entry` is reported `failed` and writes
//! nothing, cancellation before the write is `not_saved`; the migration to a nullable
//! `file_name` keeps every existing row, id, saved flag and the id counter; rows without audio
//! never lead to a file or directory removal.
use handy_app_lib::destination::{
    delivery::HistoryRecord, history::save_dictation, outcome::HistoryStatus,
};
use handy_app_lib::history_store::{HistoryEntry, HistoryStore, NewHistoryEntry};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const PHRASE: &str = "safe test phrase for the history matrix";

struct Fixture {
    _directory: tempfile::TempDir,
    db: PathBuf,
    recordings: PathBuf,
}
impl Fixture {
    fn new() -> Result<Self, std::io::Error> {
        let directory = tempfile::tempdir()?;
        Ok(Self {
            db: directory.path().join("history.db"),
            recordings: directory.path().join("recordings"),
            _directory: directory,
        })
    }
    fn open(&self) -> anyhow::Result<HistoryStore> {
        HistoryStore::open(self.db.clone(), self.recordings.clone())
    }
    fn rows(&self) -> rusqlite::Result<i64> {
        Connection::open(&self.db)?.query_row(
            "SELECT COUNT(*) FROM transcription_history",
            [],
            |row| row.get(0),
        )
    }
}

fn dictation(file_name: Option<&str>) -> NewHistoryEntry {
    NewHistoryEntry {
        file_name: file_name.map(str::to_owned),
        transcription_text: PHRASE.to_owned(),
        post_process_requested: true,
        post_processed_text: Some("processed phrase".to_owned()),
        post_process_prompt: Some("prompt".to_owned()),
    }
}
fn reread(store: &HistoryStore, record: HistoryRecord) -> Result<HistoryEntry, String> {
    let id = record.entry_id.ok_or("no entry id")?;
    store
        .entry(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("entry {id} not found"))
}
fn not_cancelled() -> bool {
    false
}

// @spec AC-005: History row with audio when the WAV was saved — .specs/features/006-destination-application/spec.md#ac-005
#[test]
fn a_saved_wav_gives_a_row_with_its_audio() -> TestResult {
    let fixture = Fixture::new()?;
    let store = fixture.open()?;
    std::fs::write(fixture.recordings.join("handy-1.wav"), b"RIFF")?;
    let record = save_dictation(&store, dictation(Some("handy-1.wav")), &not_cancelled);
    assert_eq!(record.status, HistoryStatus::Saved);
    let entry = reread(&store, record)?;
    assert_eq!(entry.file_name.as_deref(), Some("handy-1.wav"));
    assert_eq!(entry.transcription_text, PHRASE);
    let audio = store
        .audio_path(entry.file_name.as_deref())
        .ok_or("no audio path")?;
    assert!(audio.is_file());
    Ok(())
}

// @spec AC-005: Text-only history entry when the audio could not be saved — .specs/features/006-destination-application/spec.md#ac-005
#[test]
fn a_failed_wav_still_saves_a_text_only_row_reported_saved() -> TestResult {
    let fixture = Fixture::new()?;
    let store = fixture.open()?;
    let record = save_dictation(&store, dictation(None), &not_cancelled);
    assert_eq!(record.status, HistoryStatus::Saved);
    let entry = reread(&store, record)?;
    assert_eq!(entry.file_name, None);
    assert_eq!(entry.transcription_text, PHRASE);
    assert_eq!(
        entry.post_processed_text.as_deref(),
        Some("processed phrase")
    );
    assert!(entry.post_process_requested);
    assert_eq!(store.audio_path(entry.file_name.as_deref()), None);
    Ok(())
}

// @spec AC-005: Never claim history when save_entry fails — .specs/features/006-destination-application/spec.md#ac-005
#[test]
fn a_failed_save_entry_is_reported_failed_and_writes_nothing() -> TestResult {
    let fixture = Fixture::new()?;
    let store = fixture.open()?;
    let mut permissions = std::fs::metadata(&fixture.db)?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&fixture.db, permissions)?;
    let record = save_dictation(&store, dictation(None), &not_cancelled);
    assert_eq!(record, HistoryRecord::FAILED);
    assert_eq!(fixture.rows()?, 0);
    Ok(())
}

#[test]
fn cancellation_before_the_write_is_not_saved_and_writes_nothing() -> TestResult {
    let fixture = Fixture::new()?;
    let store = fixture.open()?;
    let record = save_dictation(&store, dictation(None), &|| true);
    assert_eq!(record, HistoryRecord::NOT_SAVED);
    assert_eq!(fixture.rows()?, 0);
    Ok(())
}

/// A history database at version 4 (before 006): `file_name` NOT NULL, ids 1, 2 and 5 kept,
/// the counter at 7 because rows 6 and 7 were deleted.
fn version_four(db: &Path) -> rusqlite::Result<()> {
    let conn = Connection::open(db)?;
    conn.execute_batch(
        "CREATE TABLE transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT, file_name TEXT NOT NULL,
            timestamp INTEGER NOT NULL, saved BOOLEAN NOT NULL DEFAULT 0, title TEXT NOT NULL,
            transcription_text TEXT NOT NULL, post_processed_text TEXT, post_process_prompt TEXT,
            post_process_requested BOOLEAN NOT NULL DEFAULT 0);
         PRAGMA user_version = 4;",
    )?;
    for (id, saved) in [(1, true), (2, false), (5, true), (6, false), (7, false)] {
        conn.execute(
            "INSERT INTO transcription_history (id, file_name, timestamp, saved, title,
                transcription_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                format!("handy-{id}.wav"),
                100 + id,
                saved,
                "title",
                "text"
            ],
        )?;
    }
    conn.execute("DELETE FROM transcription_history WHERE id >= 6", [])?;
    Ok(())
}
/// One history row as read back: id, file name (NULL for text only), timestamp, saved flag.
type HistoryRow = (i64, Option<String>, i64, bool);
fn rows_of(db: &Path) -> rusqlite::Result<Vec<HistoryRow>> {
    let conn = Connection::open(db)?;
    let mut statement = conn
        .prepare("SELECT id, file_name, timestamp, saved FROM transcription_history ORDER BY id")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    rows.collect()
}

#[test]
fn migration_keeps_rows_ids_saved_flags_and_the_id_counter() -> TestResult {
    let fixture = Fixture::new()?;
    version_four(&fixture.db)?;
    let before = rows_of(&fixture.db)?;
    let store = fixture.open()?;
    assert_eq!(rows_of(&fixture.db)?, before);
    assert_eq!(
        before.iter().map(|row| row.0).collect::<Vec<_>>(),
        [1, 2, 5]
    );
    let conn = store.connection()?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    assert_eq!(version, 5);
    let not_null: i64 = conn.query_row(
        "SELECT \"notnull\" FROM pragma_table_info('transcription_history')
         WHERE name = 'file_name'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(not_null, 0);
    let record = save_dictation(&store, dictation(None), &not_cancelled);
    assert_eq!(record.entry_id, Some(8));
    let reopened = fixture.open()?;
    assert_eq!(reopened.entry(8)?.ok_or("missing")?.file_name, None);
    Ok(())
}

#[test]
fn rows_without_audio_never_remove_a_file_or_the_recordings_directory() -> TestResult {
    let fixture = Fixture::new()?;
    let store = fixture.open()?;
    std::fs::write(fixture.recordings.join("handy-2.wav"), b"RIFF")?;
    for name in [
        None,
        Some(""),
        Some(".."),
        Some("."),
        Some("nested/handy-2.wav"),
    ] {
        assert_eq!(store.audio_path(name), None, "{name:?}");
        assert!(!store.remove_audio(name)?, "{name:?}");
    }
    assert!(fixture.recordings.is_dir());
    assert!(store.remove_audio(Some("handy-2.wav"))?);
    assert!(!fixture.recordings.join("handy-2.wav").exists());
    assert!(fixture.recordings.is_dir());
    Ok(())
}
