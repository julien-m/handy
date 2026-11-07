# Changelog — 023 Settings Survive Another Handy Version

## 2026-11-07

- Created: every committed settings generation is copied privately to `settings_backups/` (20 kept); at startup, a store rewritten by another program is restored from the latest copy and the rewritten file is kept; a store from a newer schema is never rewritten.
- Fixed (review): a rewritten store is restored only after it was set aside; when the backup folder is not writable, the store is left untouched instead of being replaced.
- Fixed (review): when the rewritten store cannot be set aside, startup also skips normalization and reports a recovery, so the damaged store never becomes authoritative.
- Fixed (review): each set-aside gets a new file that never replaces an earlier one, and the backup folder is synced after every copy and set-aside before a restore proceeds.
- Fixed (review): while a rewritten store waits for a restore it could not set aside, every commit is refused, so an automatic repair at startup cannot make it authoritative.
- Fixed (CI): the pending-restore refusal applies only to the running instance; an offline hand or CLI edit while Handy is stopped is written as before.
