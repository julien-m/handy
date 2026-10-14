//! History of a finished dictation (AC-005 as refined on 2026-10-14): the transcript of every
//! non-cancelled dictation goes through `save_entry`, with its audio when the WAV was saved and
//! text-only otherwise, and the history status of the insertion outcome is the actual result of
//! that write, never derived from the WAV.
use super::delivery::HistoryRecord;
use crate::managers::history_store::{HistorySink, NewHistoryEntry};
use log::error;

// @spec FR-004: History status from the actual save result — .specs/features/006-destination-application/spec.md#fr-004
/// Save `entry` (`file_name` is the verified WAV, or `None` for a text-only row) and report
/// what happened: `not_saved` when `cancelled` is observed before the write (no row, as
/// today), `saved` with the row id when `save_entry` committed it, `failed` when it did not.
pub fn save_dictation(
    sink: &dyn HistorySink,
    entry: NewHistoryEntry,
    cancelled: &dyn Fn() -> bool,
) -> HistoryRecord {
    if cancelled() {
        return HistoryRecord::NOT_SAVED;
    }
    match sink.save_entry(entry) {
        Ok(saved) => HistoryRecord::saved(saved.id),
        Err(err) => {
            error!("Failed to save history entry: {}", err);
            HistoryRecord::FAILED
        }
    }
}
