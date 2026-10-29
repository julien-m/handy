# Testing Strategy: Handy

## Observed Tools

- @playwright/test

## Observed Commands

- `bun src/lib/utils/keyboard.test.ts`
- `playwright test`
- `playwright test --ui`
- `cargo test --features destination-return,smart-paste-dispatch --test <suite>` (from `src-tauri/`), for example the 014 suites `src-tauri/tests/in_dictation_keys.rs` and `src-tauri/tests/shortcuts_cli.rs`

## Evidence Boundary

Commands above are observed and deduplicated; they were not executed during initialization.
Run relevant unit, integration and native/frontend tests before claiming application success.
