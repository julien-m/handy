# Changelog — 018 Quick Add from the Tray

## 2026-11-02 — review of PR #4

- A saved addition clears only the fields of its tab, and keeps text typed during the commit (AC-007).
- The Enter that confirms an input-method candidate (Chinese, Japanese) no longer submits (AC-008).

## 2026-10-31 — alphabetical lists / Show the list

- Fix (Julien): the green confirmation stayed after closing and reopening the window, because every window close was turned into a hide. The Quick Add window now really closes (and no longer hides the dock icon while the settings window is open), and the confirmation goes away after 4 s (AC-007).
- "Show the list" opens the settings on Advanced, scrolled to the list of the current tab, and closes the Quick Add window (AC-009, FR-006).

## 2026-10-31

- Created: tray items "Add a Replacement…" and "Add to Vocabulary…" open a Quick Add window that commits one rule or one term through the settings service (shared 002/003 validation). Penflow mockup and native proof pending.
