//! 005 Step 7 on the real history store (temporary SQLite database): migration 6 keeps every
//! row of a version-5 database and adds an empty processing record; an AI failure whose WAV also
//! failed is saved as a text-only row with the exact raw transcript, the prompt name and the
//! failure code; an ordinary dictation keeps an empty record; a re-opened store reads them back.
use handy_app_lib::destination::history::save_dictation;
use handy_app_lib::history_store::{HistoryStore, NewHistoryEntry, ProcessingRecord};
use rusqlite::{params, Connection};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const RAW: &str = "  um, the exact raw words, kept verbatim ";

fn store(directory: &tempfile::TempDir) -> anyhow::Result<HistoryStore> {
    HistoryStore::open(
        directory.path().join("history.db"),
        directory.path().join("recordings"),
    )
}
/// A version-5 database (006: nullable `file_name`) with one audio row and one text-only row.
fn version_five(db: &std::path::Path) -> rusqlite::Result<()> {
    let conn = Connection::open(db)?;
    conn.execute_batch(
        "CREATE TABLE transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT, file_name TEXT, timestamp INTEGER NOT NULL,
            saved BOOLEAN NOT NULL DEFAULT 0, title TEXT NOT NULL, transcription_text TEXT NOT NULL,
            post_processed_text TEXT, post_process_prompt TEXT,
            post_process_requested BOOLEAN NOT NULL DEFAULT 0);
         PRAGMA user_version = 5;",
    )?;
    for (id, file) in [(3, Some("handy-3.wav")), (9, None)] {
        conn.execute(
            "INSERT INTO transcription_history (id, file_name, timestamp, saved, title,
                transcription_text, post_processed_text, post_process_prompt,
                post_process_requested) VALUES (?1, ?2, 100, 1, 'title', 'text', 'done', 'p', 1)",
            params![id, file],
        )?;
    }
    Ok(())
}

#[test]
fn migration_six_keeps_every_row_and_adds_an_empty_processing_record() -> TestResult {
    let directory = tempfile::tempdir()?;
    version_five(&directory.path().join("history.db"))?;
    let store = store(&directory)?;
    let version: i64 = store
        .connection()?
        .pragma_query_value(None, "user_version", |r| r.get(0))?;
    assert_eq!(version, 6);
    for (id, file) in [(3, Some("handy-3.wav")), (9, None)] {
        let row = store.entry(id)?.ok_or("row lost")?;
        assert_eq!(
            (
                row.file_name.as_deref(),
                row.saved,
                row.transcription_text.as_str()
            ),
            (file, true, "text")
        );
        assert_eq!(row.post_processed_text.as_deref(), Some("done"));
        assert_eq!(row.processing, ProcessingRecord::default());
    }
    Ok(())
}
// @spec AC-005: History keeps raw text, prompt and reason — .specs/features/005-custom-ai-prompts/spec.md#ac-005
#[test]
fn an_ai_failure_without_audio_keeps_the_raw_text_prompt_and_reason() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = store(&directory)?;
    let failed = NewHistoryEntry {
        file_name: None,
        transcription_text: RAW.to_owned(),
        post_process_requested: true,
        processing: ProcessingRecord {
            post_process_prompt_id: Some("starter_notes".into()),
            post_process_prompt_name: Some("Notes".into()),
            post_process_provider_id: Some("custom".into()),
            post_process_model: Some("llama3.2".into()),
            post_process_failure: Some("timeout".into()),
        },
        ..NewHistoryEntry::default()
    };
    let record = save_dictation(&store, failed.clone(), &|| false);
    let id = record.entry_id.ok_or("not saved")?;
    let reread = store.entry(id)?.ok_or("row missing")?;
    assert_eq!(
        (reread.file_name, reread.transcription_text.as_str()),
        (None, RAW)
    );
    assert_eq!(reread.post_processed_text, None);
    assert_eq!(reread.processing, failed.processing);
    Ok(())
}
#[test]
fn an_ordinary_dictation_has_no_processing_record() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = store(&directory)?;
    let plain = NewHistoryEntry {
        transcription_text: "plain".to_owned(),
        ..NewHistoryEntry::default()
    };
    let id = save_dictation(&store, plain, &|| false)
        .entry_id
        .ok_or("not saved")?;
    let reopened = HistoryStore::open(
        directory.path().join("history.db"),
        directory.path().join("recordings"),
    )?;
    assert_eq!(
        reopened.entry(id)?.ok_or("row missing")?.processing,
        ProcessingRecord::default()
    );
    Ok(())
}
