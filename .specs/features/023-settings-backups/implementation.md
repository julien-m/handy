# Implementation: Settings Survive Another Handy Version

| Requirement | Location |
|---|---|
| FR-001, AC-001 | `src-tauri/src/settings_service/backups.rs` (`keep`, `prune`, `write_private`), `src-tauri/src/settings_service/transaction.rs` (`persist_locked`, `keep_first_backup`) |
| FR-002, AC-002 | `src-tauri/src/settings_service/backups.rs` (`latest`, `restoration`, `set_aside`), `src-tauri/src/settings_service/transaction.rs` (`initialize`, `restore_if_rewritten`) |
| FR-003, AC-003 | `src-tauri/src/settings_service/store.rs` (`document`), `src-tauri/src/settings.rs` (`CURRENT_SETTINGS_SCHEMA_VERSION`) |

Tests: `src-tauri/src/settings_service/backups.rs` (tests), `src-tauri/src/settings_service/live_tests.rs` (`settings_dropped_by_an_older_build_are_restored_at_the_next_start`, `a_store_from_a_newer_build_is_read_but_never_rewritten`).
