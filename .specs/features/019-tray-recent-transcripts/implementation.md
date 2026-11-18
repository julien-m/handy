# Implementation: Recent Transcripts in the Tray

| Requirement | Location |
|---|---|
| FR-001, AC-002 | `src-tauri/src/managers/history.rs` (`recent_completed_entries`, `entry_by_id`) |
| FR-002, AC-001, AC-003 | `src-tauri/src/tray.rs` (`recent_transcripts_submenu`, `recent_transcript_items`, `transcript_preview`, `recent_label`) |
| AC-004 | `src-tauri/src/tray.rs` (`copy_recent_transcript`, `copy_entry_text`), `src-tauri/src/lib.rs` (menu event) |
| FR-003, AC-005 | `src-tauri/src/managers/history.rs` (`update_tray_menu` after save, retry, delete and retention cleanup in `delete_entries_and_files`) |
| Translations | `src/i18n/locales/*/translation.json` (`tray.recentTranscripts`, `tray.noRecentTranscripts`) |

The retention path rechecks eligibility in [the history store](../../../src-tauri/src/managers/history_store.rs) before [the manager](../../../src-tauri/src/managers/history.rs) removes the audio or refreshes the tray. The manager's `retention_skips_a_candidate_saved_after_selection` regression uses two connections to the migrated store.

Tests: `src-tauri/src/managers/history.rs` (`recent_completed_entries_are_newest_first_and_skip_empty_ones`), `src-tauri/src/tray.rs` (tests).
