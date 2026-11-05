# Implementation: Quick Add from the Tray

| Requirement | Location |
|---|---|
| FR-001, AC-001 | `src-tauri/src/tray.rs` (`quick_add_replacement`, `quick_add_vocabulary` items) |
| FR-002, AC-002, AC-003 | `src-tauri/src/quick_add.rs` (`open`) ; `src-tauri/src/lib.rs` (menu event) |
| FR-003, AC-004, AC-005 | `src-tauri/src/commands/quick_add.rs` (`quick_add_replacement`) |
| FR-004, AC-006 | `src-tauri/src/commands/quick_add.rs` (`quick_add_vocabulary_term`) |
| FR-005 | `src/overlay/quick-add-main.tsx`, `src-tauri/capabilities/quick-add.json` |
| AC-005, AC-006, AC-008 | `src/overlay/quick-add-model.ts` |
| AC-007 | `src/overlay/quick-add-window.tsx` |
| FR-006, AC-009 | `src-tauri/src/quick_add.rs` (`reveal_in_settings`), `src-tauri/src/commands/quick_add.rs` (`open_settings_list`), `src/components/settings/settings-reveal.ts`, `src/overlay/quick-add-window.tsx` (Show the list) |
| 002/003 `alphabetical_display` | `src/lib/utils/alphabetical.ts`, `src/components/settings/word-replacements/rule-list.tsx`, `src/components/settings/vocabulary/vocabulary-terms.tsx`, `src/components/ui/bounded-list.ts` |

Tests: `src-tauri/src/quick_add.rs` (tests), `src-tauri/src/commands/quick_add.rs` (tests), `src/overlay/quick-add-model.test.ts` (`bun run test:quick-add`).
