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

Tests: `src-tauri/src/quick_add.rs` (tests), `src-tauri/src/commands/quick_add.rs` (tests), `src/overlay/quick-add-model.test.ts` (`bun run test:quick-add`).
