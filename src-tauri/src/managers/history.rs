use super::history_store::{HistoryStore, NewHistoryEntry, RetriedText, ENTRY_COLUMNS};
use anyhow::{anyhow, Result};
use chrono::Utc;
use log::{debug, error};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_specta::Event;

pub use super::history_store::{HistoryEntry, HistorySink};

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct PaginatedHistory {
    pub entries: Vec<HistoryEntry>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(tag = "action")]
pub enum HistoryUpdatePayload {
    #[serde(rename = "added")]
    Added { entry: HistoryEntry },
    #[serde(rename = "updated")]
    Updated { entry: HistoryEntry },
    #[serde(rename = "deleted")]
    Deleted { id: i64 },
    #[serde(rename = "toggled")]
    Toggled { id: i64 },
}

/// History of the app: the store plus the settings-driven retention cleanup and the
/// `history-updated` events of the settings window.
pub struct HistoryManager {
    app_handle: AppHandle,
    store: HistoryStore,
}

impl HistoryManager {
    /// Open the history of the app data directory (migrating it) for `app_handle`.
    ///
    /// # Errors
    /// The data directory cannot be resolved, or the store cannot be opened or migrated.
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data_dir = crate::portable::app_data_dir(app_handle)?;
        let store = HistoryStore::open(
            app_data_dir.join("history.db"),
            app_data_dir.join("recordings"),
        )?;
        Ok(Self {
            app_handle: app_handle.clone(),
            store,
        })
    }

    fn get_connection(&self) -> Result<Connection> {
        self.store.connection()
    }

    /// Directory of the WAV recordings.
    pub fn recordings_dir(&self) -> &std::path::Path {
        self.store.recordings_dir()
    }

    // @spec FR-004: Text-only history row when the WAV fails — .specs/features/006-destination-application/spec.md#fr-004
    /// Save a new history entry, with its WAV (`entry.file_name`, already written to the
    /// recordings directory) or text-only, then apply the retention cleanup and notify the UI.
    ///
    /// # Errors
    /// The row could not be written (the cleanup and the event never fail the save).
    pub fn save_entry(&self, entry: NewHistoryEntry) -> Result<HistoryEntry> {
        let entry = self.store.insert(entry)?;
        if let Err(e) = self.cleanup_old_entries() {
            error!("Failed to clean up old history entries: {}", e);
        }
        let added = HistoryUpdatePayload::Added {
            entry: entry.clone(),
        };
        if let Err(e) = added.emit(&self.app_handle) {
            error!("Failed to emit history-updated event: {}", e);
        }
        Ok(entry)
    }

    /// Update an existing history entry with new transcription results (used by retry).
    pub fn update_transcription(&self, id: i64, text: RetriedText) -> Result<HistoryEntry> {
        let entry = self
            .store
            .update_text(id, text)?
            .ok_or_else(|| anyhow!("History entry {} not found", id))?;
        debug!("Updated transcription for history entry {}", id);
        if let Err(e) = (HistoryUpdatePayload::Updated {
            entry: entry.clone(),
        })
        .emit(&self.app_handle)
        {
            error!("Failed to emit history-updated event: {}", e);
        }
        Ok(entry)
    }

    pub fn cleanup_old_entries(&self) -> Result<()> {
        let retention_period = crate::settings::get_recording_retention_period(&self.app_handle);

        match retention_period {
            crate::settings::RecordingRetentionPeriod::Never => {
                // Don't delete anything
                Ok(())
            }
            crate::settings::RecordingRetentionPeriod::PreserveLimit => {
                // Use the old count-based logic with history_limit
                let limit = crate::settings::get_history_limit(&self.app_handle);
                self.cleanup_by_count(limit)
            }
            _ => {
                // Use time-based logic
                self.cleanup_by_time(retention_period)
            }
        }
    }

    fn delete_entries_and_files(&self, entries: &[(i64, Option<String>)]) -> Result<usize> {
        if entries.is_empty() {
            return Ok(0);
        }

        let conn = self.get_connection()?;
        let mut deleted_count = 0;

        for (id, file_name) in entries {
            // Delete database entry
            conn.execute(
                "DELETE FROM transcription_history WHERE id = ?1",
                params![id],
            )?;

            // Delete the WAV file; a text-only row has none, and the recordings directory itself
            // is never removed.
            match self.store.remove_audio(file_name.as_deref()) {
                Ok(true) => {
                    debug!("Deleted old WAV file: {:?}", file_name);
                    deleted_count += 1;
                }
                Ok(false) => {}
                Err(e) => error!("Failed to delete WAV file {:?}: {}", file_name, e),
            }
        }

        Ok(deleted_count)
    }

    fn cleanup_by_count(&self, limit: usize) -> Result<()> {
        let conn = self.get_connection()?;

        // Get all entries that are not saved, ordered by timestamp desc
        let mut stmt = conn.prepare(
            "SELECT id, file_name FROM transcription_history WHERE saved = 0 ORDER BY timestamp DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>("id")?,
                row.get::<_, Option<String>>("file_name")?,
            ))
        })?;

        let mut entries: Vec<(i64, Option<String>)> = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        if entries.len() > limit {
            let entries_to_delete = &entries[limit..];
            let deleted_count = self.delete_entries_and_files(entries_to_delete)?;

            if deleted_count > 0 {
                debug!("Cleaned up {} old history entries by count", deleted_count);
            }
        }

        Ok(())
    }

    fn cleanup_by_time(
        &self,
        retention_period: crate::settings::RecordingRetentionPeriod,
    ) -> Result<()> {
        let conn = self.get_connection()?;

        // Calculate cutoff timestamp (current time minus retention period)
        let now = Utc::now().timestamp();
        let cutoff_timestamp = match retention_period {
            crate::settings::RecordingRetentionPeriod::Days3 => now - (3 * 24 * 60 * 60), // 3 days in seconds
            crate::settings::RecordingRetentionPeriod::Weeks2 => now - (2 * 7 * 24 * 60 * 60), // 2 weeks in seconds
            crate::settings::RecordingRetentionPeriod::Months3 => now - (3 * 30 * 24 * 60 * 60), // 3 months in seconds (approximate)
            _ => unreachable!("Should not reach here"),
        };

        // Get all unsaved entries older than the cutoff timestamp
        let mut stmt = conn.prepare(
            "SELECT id, file_name FROM transcription_history WHERE saved = 0 AND timestamp < ?1",
        )?;

        let rows = stmt.query_map(params![cutoff_timestamp], |row| {
            Ok((
                row.get::<_, i64>("id")?,
                row.get::<_, Option<String>>("file_name")?,
            ))
        })?;

        let mut entries_to_delete: Vec<(i64, Option<String>)> = Vec::new();
        for row in rows {
            entries_to_delete.push(row?);
        }

        let deleted_count = self.delete_entries_and_files(&entries_to_delete)?;

        if deleted_count > 0 {
            debug!(
                "Cleaned up {} old history entries based on retention period",
                deleted_count
            );
        }

        Ok(())
    }

    pub async fn get_history_entries(
        &self,
        cursor: Option<i64>,
        limit: Option<usize>,
    ) -> Result<PaginatedHistory> {
        let conn = self.get_connection()?;
        let limit = limit.map(|l| l.min(100));

        let mut entries: Vec<HistoryEntry> = match (cursor, limit) {
            (Some(cursor_id), Some(lim)) => {
                let fetch_count = (lim + 1) as i64;
                let mut stmt = conn.prepare(&format!(
                    "SELECT {ENTRY_COLUMNS}
                     FROM transcription_history
                     WHERE id < ?1
                     ORDER BY id DESC
                     LIMIT ?2"
                ))?;
                let result = stmt
                    .query_map(params![cursor_id, fetch_count], HistoryStore::map_entry)?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                result
            }
            (None, Some(lim)) => {
                let fetch_count = (lim + 1) as i64;
                let mut stmt = conn.prepare(&format!(
                    "SELECT {ENTRY_COLUMNS}
                     FROM transcription_history
                     ORDER BY id DESC
                     LIMIT ?1"
                ))?;
                let result = stmt
                    .query_map(params![fetch_count], HistoryStore::map_entry)?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                result
            }
            (_, None) => {
                let mut stmt = conn.prepare(&format!(
                    "SELECT {ENTRY_COLUMNS}
                     FROM transcription_history
                     ORDER BY id DESC"
                ))?;
                let result = stmt
                    .query_map([], HistoryStore::map_entry)?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                result
            }
        };

        let has_more = limit.is_some_and(|lim| entries.len() > lim);
        if has_more {
            entries.pop();
        }

        Ok(PaginatedHistory { entries, has_more })
    }

    #[cfg(test)]
    fn get_latest_entry_with_conn(conn: &Connection) -> Result<Option<HistoryEntry>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {ENTRY_COLUMNS}
             FROM transcription_history
             ORDER BY timestamp DESC
             LIMIT 1"
        ))?;

        let entry = stmt.query_row([], HistoryStore::map_entry).optional()?;
        Ok(entry)
    }

    /// Get the latest entry with non-empty transcription text.
    pub fn get_latest_completed_entry(&self) -> Result<Option<HistoryEntry>> {
        let conn = self.get_connection()?;
        Self::get_latest_completed_entry_with_conn(&conn)
    }

    fn get_latest_completed_entry_with_conn(conn: &Connection) -> Result<Option<HistoryEntry>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {ENTRY_COLUMNS}
             FROM transcription_history
             WHERE transcription_text != ''
             ORDER BY timestamp DESC
             LIMIT 1"
        ))?;

        let entry = stmt.query_row([], HistoryStore::map_entry).optional()?;
        Ok(entry)
    }

    pub async fn toggle_saved_status(&self, id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        // Get current saved status
        let current_saved: bool = conn.query_row(
            "SELECT saved FROM transcription_history WHERE id = ?1",
            params![id],
            |row| row.get("saved"),
        )?;

        let new_saved = !current_saved;

        conn.execute(
            "UPDATE transcription_history SET saved = ?1 WHERE id = ?2",
            params![new_saved, id],
        )?;

        debug!("Toggled saved status for entry {}: {}", id, new_saved);

        // Emit history updated event
        if let Err(e) = (HistoryUpdatePayload::Toggled { id }).emit(&self.app_handle) {
            error!("Failed to emit history-updated event: {}", e);
        }

        Ok(())
    }

    pub fn get_audio_file_path(&self, file_name: &str) -> PathBuf {
        self.store.recordings_dir().join(file_name)
    }

    /// Audio recording of `entry`, or `None` for a text-only row.
    pub fn audio_path(&self, entry: &HistoryEntry) -> Option<PathBuf> {
        self.store.audio_path(entry.file_name.as_deref())
    }

    pub async fn get_entry_by_id(&self, id: i64) -> Result<Option<HistoryEntry>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {ENTRY_COLUMNS}
             FROM transcription_history
             WHERE id = ?1"
        ))?;

        let entry = stmt.query_row([id], HistoryStore::map_entry).optional()?;

        Ok(entry)
    }

    pub async fn delete_entry(&self, id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        // Get the entry to find the file name
        if let Some(entry) = self.get_entry_by_id(id).await? {
            // Delete the audio file first (none for a text-only row); continue with the
            // database deletion even if the file deletion fails.
            if let Err(e) = self.store.remove_audio(entry.file_name.as_deref()) {
                error!("Failed to delete audio file {:?}: {}", entry.file_name, e);
            }
        }

        // Delete from database
        conn.execute(
            "DELETE FROM transcription_history WHERE id = ?1",
            params![id],
        )?;

        debug!("Deleted history entry with id: {}", id);

        // Emit history updated event
        if let Err(e) = (HistoryUpdatePayload::Deleted { id }).emit(&self.app_handle) {
            error!("Failed to emit history-updated event: {}", e);
        }

        Ok(())
    }
}

impl HistorySink for HistoryManager {
    fn save_entry(&self, entry: NewHistoryEntry) -> Result<HistoryEntry> {
        HistoryManager::save_entry(self, entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "CREATE TABLE transcription_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                saved BOOLEAN NOT NULL DEFAULT 0,
                title TEXT NOT NULL,
                transcription_text TEXT NOT NULL,
                post_processed_text TEXT,
                post_process_prompt TEXT,
                post_process_requested BOOLEAN NOT NULL DEFAULT 0,
                post_process_prompt_id TEXT,
                post_process_prompt_name TEXT,
                post_process_provider_id TEXT,
                post_process_model TEXT,
                post_process_failure TEXT
            );",
        )
        .expect("create transcription_history table");
        conn
    }

    fn insert_entry(conn: &Connection, timestamp: i64, text: &str, post_processed: Option<&str>) {
        conn.execute(
            "INSERT INTO transcription_history (
                file_name,
                timestamp,
                saved,
                title,
                transcription_text,
                post_processed_text,
                post_process_prompt,
                post_process_requested
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                format!("handy-{}.wav", timestamp),
                timestamp,
                false,
                format!("Recording {}", timestamp),
                text,
                post_processed,
                Option::<String>::None,
                false,
            ],
        )
        .expect("insert history entry");
    }

    #[test]
    fn get_latest_entry_returns_none_when_empty() {
        let conn = setup_conn();
        let entry = HistoryManager::get_latest_entry_with_conn(&conn).expect("fetch latest entry");
        assert!(entry.is_none());
    }

    #[test]
    fn get_latest_entry_returns_newest_entry() {
        let conn = setup_conn();
        insert_entry(&conn, 100, "first", None);
        insert_entry(&conn, 200, "second", Some("processed"));

        let entry = HistoryManager::get_latest_entry_with_conn(&conn)
            .expect("fetch latest entry")
            .expect("entry exists");

        assert_eq!(entry.timestamp, 200);
        assert_eq!(entry.transcription_text, "second");
        assert_eq!(entry.post_processed_text.as_deref(), Some("processed"));
    }

    #[test]
    fn get_latest_completed_entry_skips_empty_entries() {
        let conn = setup_conn();
        insert_entry(&conn, 100, "completed", None);
        insert_entry(&conn, 200, "", None);

        let entry = HistoryManager::get_latest_completed_entry_with_conn(&conn)
            .expect("fetch latest completed entry")
            .expect("completed entry exists");

        assert_eq!(entry.timestamp, 100);
        assert_eq!(entry.transcription_text, "completed");
    }

    // @spec AC-005: Text-only rows are read like any other — .specs/features/006-destination-application/spec.md#ac-005
    #[test]
    fn latest_completed_entry_reads_a_text_only_row() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = HistoryStore::open(
            directory.path().join("history.db"),
            directory.path().join("recordings"),
        )?;
        store.insert(NewHistoryEntry {
            transcription_text: "text only".to_owned(),
            ..NewHistoryEntry::default()
        })?;
        let entry = HistoryManager::get_latest_completed_entry_with_conn(&store.connection()?)?
            .ok_or_else(|| anyhow!("no completed entry"))?;
        assert_eq!(entry.file_name, None);
        assert_eq!(entry.transcription_text, "text only");
        Ok(())
    }
}
