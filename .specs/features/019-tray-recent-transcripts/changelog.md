# Changelog — 019 Recent Transcripts in the Tray

## 2026-11-18 — Preserve saved history during retention cleanup

- Recheck the saved flag in the deletion itself. Remove recording audio only after its row was deleted. Toggling a saved flag uses a single write and reports a missing entry.

## 2026-11-02 — review of PR #4

- The retention cleanup (lower history limit, shorter retention period) now refreshes the submenu, which kept showing deleted transcripts (AC-005, FR-003).

## 2026-11-02

- Created: a « Recent Transcripts » tray submenu lists the 5 newest transcripts (time and start of the text); a click copies the full text. Native proof pending.
