# Changelog — 023 Settings Survive Another Handy Version

## 2026-11-07

- Created: every committed settings generation is copied privately to `settings_backups/` (20 kept); at startup, a store rewritten by another program is restored from the latest copy and the rewritten file is kept; a store from a newer schema is never rewritten.
