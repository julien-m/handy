# Progress — 003-recognition-vocabulary

`/spec-implement`, light lane (`proofs/claude-review/delivery-lane-r3.json`). Branch
`feature/003-recognition-vocabulary`, worktree from `dev`. Implementer: Claude (`claude-opus-5-5`).
Scope of this run: every non-UI step of the plan. The settings screen (Step 5) waits for the
Penflow pass and is not started. Per-step evidence lives in `proofs/claude-impl/<step>/`
(`*.stdout`, `*.stderr`, `*.exit`).

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a Behavioral TDD | Todo | — (UI tests of the Behavioral AC, written with Step 5) | — | Deferred with the UI | 2026-10-14 |
| 1 Capability and budget | Done | `recognition_vocabulary/{mod,capability,budget,context}.rs`, `engine_supervisor/{protocol,worker,supervisor,supervisor_context}.rs`, `contracts/context-budget.md` | `cargo test --lib -- recognition_vocabulary engine_supervisor::protocol engine_supervisor::supervisor::context` | Pass (18) | 2026-10-14 |
| 2 Validation and transactions | Done | `recognition_vocabulary/{validation,occurrences,preview,edits,edits_tests}.rs`, `settings_service/{error_reason,errors,command_error,tauri_adapter}.rs` (`write_change`), `commands/vocabulary.rs`, `shortcut/mod.rs` (`update_custom_words`), `Cargo.toml` (`caseless = "=0.2.2"`), `tests/vocabulary_transactions.rs`, `src/bindings.ts`, `src/stores/settingsStore.ts`, 008 contract schema and doc | `cargo test --lib -- recognition_vocabulary settings_service`; `cargo test --test vocabulary_transactions --test settings_validation --test settings_cli`; `bun run test:settings-errors`, `test:settings-store`, `test:shortcut-list` | Pass | 2026-10-14 |
| 3 Request snapshot, status and correction routing | Done | `recognition_vocabulary/{status,status_tests}.rs`, `managers/{transcription,transcription_vocabulary}.rs`, `commands/vocabulary.rs` (`get_vocabulary_status`), `src/bindings.ts`; refactor of `edits.rs`, `preview.rs`, `budget.rs`, `occurrences.rs` to functions of at most 30 lines | `cargo test --lib -- recognition_vocabulary managers::transcription audio_toolkit::text engine_supervisor::protocol engine_supervisor::supervisor::context`; `cargo test --test vocabulary_transactions` | Pass (111 + 4) | 2026-10-14 |
