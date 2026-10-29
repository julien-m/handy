# Testing Strategy: Handy

## Observed Tools

- @playwright/test

## Observed Commands

- `bun src/lib/utils/keyboard.test.ts`
- `playwright test`
- `playwright test --ui`
- `cargo test --lib --features destination-return,smart-paste-dispatch` (Rust unit tests, `src-tauri/src`)
- `cargo test --features destination-return,smart-paste-dispatch --test <suite>` (Rust integration suites in `src-tauri/tests`, one binary per file, for example `smart_capitalization_context` and `settings_domains`)

## Evidence Boundary

Commands above are observed and deduplicated; they were not executed during initialization.
Run relevant unit, integration and native/frontend tests before claiming application success.
