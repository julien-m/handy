# Progress — 014-in-dictation-keys

Run of 2026-10-29 (Claude Code, `/spec-feature` in autonomous mode, model claude-opus-5-5), branch `feat/dictate-to-active-app` from `dev` `b0de2fe1`. Conventions loaded from `.conventions/index.md` (sub-domain `code`: general, javascript, cli). Native agents only (two read-only reviews); no Codex, no `audit` skill. The target changed twice during the run (start shortcut → key during the dictation; « Insert without Enter » added; defaults Option left and Control left); the start-shortcut implementation was reverted before any commit.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | `dictation_keys/mod.rs`, `settings.rs`, `lib.rs` | `cargo test --lib -- dictation_keys` | Pass (defaults on the left, one-modifier syntax) | 2026-10-29 |
| 2 | Done | `dictation_keys/tap.rs` | `cargo test --lib -- dictation_keys::tap` | Pass (8 tests: lone press against combination, repeats, held modifier, shortcut fired, lost release, macOS direction) | 2026-10-29 |
| 3 | Done | `destination/{dictation_toggles,session,snapshot,manager}.rs` | `cargo test --lib -- destination::dictation_toggles` | Pass (7 tests) | 2026-10-29 |
| 4 | Done | `automatic_enter/{policy,transaction}.rs` | `cargo test --test in_dictation_keys` | Pass (Enter matrix) | 2026-10-29 |
| 5 | Done | `dictation_keys/watcher.rs`, `shortcut/handler.rs`, `destination/app.rs`, `actions.rs` | `cargo clippy --all-targets` | Clean (runtime proof is native) | 2026-10-29 |
| 6 | Done | `shortcut/{mod,edits,action_shortcuts,conflicts}.rs` | `cargo test --lib -- dictation_keys::tests::conflicts`, `--test shortcuts_cli` | Pass | 2026-10-29 |
| 7 | Done | settings list, overlay mark, `bindings.ts`, 26 locales | `bun run test:shortcut-states`, `test:indicator`, `lint`, `tsc`, `check:translations` | Pass | 2026-10-29 |
| 8 | Done | review fixes (6 findings), docs, registry | full check (see `implementation.md`) | Pass | 2026-10-29 |
| 9 | Pending (Julien) | — | Native protocol in `implementation.md` | Not run | 2026-10-29 |
| 10 | Done | `destination/dictation_toggles.rs`, `dictation_keys/watcher.rs`, `src/overlay/{notice-model,use-overlay-notices,overlay-notices,indicator-screen}.ts*`, 26 locales | `cargo test --features destination-return,smart-paste-dispatch --lib -- dictation_keys destination committed_bindings_match_the_registry`, `--test in_dictation_keys`, `bun run test:indicator`, `lint`, `format:check`, `check:translations` | Pass (spec-fix 2026-10-30: AC-007 no-pin notice) | 2026-10-30 |
