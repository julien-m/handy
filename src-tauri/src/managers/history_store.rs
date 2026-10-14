//! History storage without the Tauri runtime: schema migrations, row mapping, inserts and the
//! audio file of a row. `HistoryManager` adds the settings-driven cleanup and the UI events on
//! top; tests open the same store on a temporary directory.
//!
//! A row may have no audio (`file_name` NULL): a dictation whose WAV could not be saved keeps
//! its transcript as a text-only row (006 AC-005, decided 2026-10-14; shared with 005 Step 7).
use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use log::{debug, info};
use rusqlite::{params, Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

/// Version 5 (006 Step 6, shared with 005 Step 7): `file_name` becomes nullable for text-only
/// rows. SQLite cannot drop NOT NULL in place, so the table is rebuilt inside the migration
/// transaction, keeping every row, id, saved flag and timestamp and the AUTOINCREMENT counter
/// (ids of deleted rows are never reused).
const NULLABLE_AUDIO: &str = "
CREATE TABLE transcription_history_rebuilt (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name TEXT,
    timestamp INTEGER NOT NULL,
    saved BOOLEAN NOT NULL DEFAULT 0,
    title TEXT NOT NULL,
    transcription_text TEXT NOT NULL,
    post_processed_text TEXT,
    post_process_prompt TEXT,
    post_process_requested BOOLEAN NOT NULL DEFAULT 0
);
INSERT INTO transcription_history_rebuilt (id, file_name, timestamp, saved, title,
    transcription_text, post_processed_text, post_process_prompt, post_process_requested)
SELECT id, file_name, timestamp, saved, title, transcription_text, post_processed_text,
    post_process_prompt, post_process_requested
FROM transcription_history;
DELETE FROM sqlite_sequence WHERE name = 'transcription_history_rebuilt';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'transcription_history_rebuilt', seq FROM sqlite_sequence
WHERE name = 'transcription_history';
DROP TABLE transcription_history;
ALTER TABLE transcription_history_rebuilt RENAME TO transcription_history;
";

/// Database migrations for transcription history, applied in order and tracked with SQLite's
/// `user_version` pragma (see `adopt_tauri_plugin_sql_version` for databases created by
/// tauri-plugin-sql).
static MIGRATIONS: &[M] = &[
    M::up(
        "CREATE TABLE IF NOT EXISTS transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            saved BOOLEAN NOT NULL DEFAULT 0,
            title TEXT NOT NULL,
            transcription_text TEXT NOT NULL
        );",
    ),
    M::up("ALTER TABLE transcription_history ADD COLUMN post_processed_text TEXT;"),
    M::up("ALTER TABLE transcription_history ADD COLUMN post_process_prompt TEXT;"),
    M::up("ALTER TABLE transcription_history ADD COLUMN post_process_requested BOOLEAN NOT NULL DEFAULT 0;"),
    M::up(NULLABLE_AUDIO),
];

/// Columns of a history row, in mapping order.
pub(crate) const ENTRY_COLUMNS: &str = "id, file_name, timestamp, saved, title, \
    transcription_text, post_processed_text, post_process_prompt, post_process_requested";

/// One history row. `file_name` is `None` for a text-only row (no audio recording).
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct HistoryEntry {
    pub id: i64,
    pub file_name: Option<String>,
    pub timestamp: i64,
    pub saved: bool,
    pub title: String,
    pub transcription_text: String,
    pub post_processed_text: Option<String>,
    pub post_process_prompt: Option<String>,
    pub post_process_requested: bool,
}

/// A row to write: `file_name` is the verified WAV of the dictation, or `None` when the audio
/// could not be saved (text-only row).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewHistoryEntry {
    pub file_name: Option<String>,
    pub transcription_text: String,
    pub post_process_requested: bool,
    pub post_processed_text: Option<String>,
    pub post_process_prompt: Option<String>,
}

/// Where a dictation's history row is written: `HistoryManager` in the app (retention cleanup
/// and UI event included), the bare store in tests.
pub trait HistorySink {
    /// Write `entry` and return the stored row.
    ///
    /// # Errors
    /// Any database failure; nothing is claimed saved then.
    fn save_entry(&self, entry: NewHistoryEntry) -> Result<HistoryEntry>;
}

/// The history database and recordings directory.
pub struct HistoryStore {
    db_path: PathBuf,
    recordings_dir: PathBuf,
}

impl HistoryStore {
    /// Open the history at `db_path`, creating `recordings_dir` and applying pending migrations.
    ///
    /// # Errors
    /// The directory cannot be created, or the database cannot be opened or migrated.
    pub fn open(db_path: PathBuf, recordings_dir: PathBuf) -> Result<Self> {
        if !recordings_dir.exists() {
            fs::create_dir_all(&recordings_dir)?;
            debug!("Created recordings directory: {:?}", recordings_dir);
        }
        let store = Self {
            db_path,
            recordings_dir,
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        info!("Initializing database at {:?}", self.db_path);
        let mut conn = Connection::open(&self.db_path)?;
        adopt_tauri_plugin_sql_version(&conn)?;
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        #[cfg(debug_assertions)]
        migrations.validate()?;
        let before = user_version(&conn)?;
        migrations.to_latest(&mut conn)?;
        let after = user_version(&conn)?;
        if after > before {
            info!("Database migrated from version {before} to {after}");
        } else {
            debug!("Database already at latest version {after}");
        }
        Ok(())
    }

    /// A new connection to the history database.
    ///
    /// # Errors
    /// The database cannot be opened.
    pub fn connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Directory of the WAV recordings.
    pub fn recordings_dir(&self) -> &Path {
        &self.recordings_dir
    }

    /// Map one row selected with `ENTRY_COLUMNS`.
    pub(crate) fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
        Ok(HistoryEntry {
            id: row.get("id")?,
            file_name: row.get("file_name")?,
            timestamp: row.get("timestamp")?,
            saved: row.get("saved")?,
            title: row.get("title")?,
            transcription_text: row.get("transcription_text")?,
            post_processed_text: row.get("post_processed_text")?,
            post_process_prompt: row.get("post_process_prompt")?,
            post_process_requested: row.get("post_process_requested")?,
        })
    }

    /// Insert `entry` now, unsaved, titled with the local time.
    ///
    /// # Errors
    /// The database cannot be opened or written.
    pub fn insert(&self, entry: NewHistoryEntry) -> Result<HistoryEntry> {
        let timestamp = Utc::now().timestamp();
        let title = format_timestamp_title(timestamp);
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO transcription_history (file_name, timestamp, saved, title,
                transcription_text, post_processed_text, post_process_prompt,
                post_process_requested)
             VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.file_name,
                timestamp,
                title,
                entry.transcription_text,
                entry.post_processed_text,
                entry.post_process_prompt,
                entry.post_process_requested,
            ],
        )?;
        let id = conn.last_insert_rowid();
        debug!("Saved history entry with id {id}");
        self.entry_with(&conn, id)?
            .ok_or_else(|| anyhow::anyhow!("History entry {id} vanished after insert"))
    }

    /// The row `id`, if it exists.
    ///
    /// # Errors
    /// The database cannot be opened or read.
    pub fn entry(&self, id: i64) -> Result<Option<HistoryEntry>> {
        self.entry_with(&self.connection()?, id)
    }

    fn entry_with(&self, conn: &Connection, id: i64) -> Result<Option<HistoryEntry>> {
        let sql = format!("SELECT {ENTRY_COLUMNS} FROM transcription_history WHERE id = ?1");
        Ok(conn
            .query_row(&sql, params![id], Self::map_entry)
            .optional()?)
    }

    /// Path of the audio recording `file_name` of a row, or `None` for a text-only row or a name
    /// that is not a plain file name (never the recordings directory itself or a path outside).
    pub fn audio_path(&self, file_name: Option<&str>) -> Option<PathBuf> {
        let name = file_name?;
        let plain = Path::new(name)
            .file_name()
            .is_some_and(|component| component == name);
        plain.then(|| self.recordings_dir.join(name))
    }

    /// Delete the audio recording of a row, if it has one and it is a file.
    ///
    /// # Errors
    /// The file exists but cannot be removed.
    pub fn remove_audio(&self, file_name: Option<&str>) -> std::io::Result<bool> {
        match self.audio_path(file_name) {
            Some(path) if path.is_file() => fs::remove_file(path).map(|()| true),
            _ => Ok(false),
        }
    }
}

impl HistorySink for HistoryStore {
    fn save_entry(&self, entry: NewHistoryEntry) -> Result<HistoryEntry> {
        self.insert(entry)
    }
}

fn user_version(conn: &Connection) -> Result<i32> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// A database created by tauri-plugin-sql tracks its migrations in `_sqlx_migrations`; adopt
/// that version as `user_version` once so that rusqlite_migration does not re-run them.
fn adopt_tauri_plugin_sql_version(conn: &Connection) -> Result<()> {
    let legacy: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if !legacy || user_version(conn)? > 0 {
        return Ok(());
    }
    let old_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if old_version > 0 {
        info!("Migrating from tauri-plugin-sql (version {old_version}) to rusqlite_migration");
        conn.pragma_update(None, "user_version", old_version)?;
    }
    Ok(())
}

/// Local-time title of a row created at `timestamp` (seconds since the Unix epoch).
fn format_timestamp_title(timestamp: i64) -> String {
    match DateTime::from_timestamp(timestamp, 0) {
        Some(utc) => utc
            .with_timezone(&Local)
            .format("%B %e, %Y - %l:%M%p")
            .to_string(),
        None => format!("Recording {timestamp}"),
    }
}
