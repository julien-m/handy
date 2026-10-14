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
| 4 CLI contract and native routing | Done | `settings_service/domains/{vocabulary,vocabulary_tests,mod}.rs`, `settings_service/{cli,render}.rs`, `src/cli.rs`, `tests/{vocabulary_cli,settings_domains}.rs`, `README.md`, 008 contract doc | `cargo test --lib -- settings_service recognition_vocabulary`; `cargo test --test vocabulary_cli --test settings_domains --test settings_cli --test settings_validation --test vocabulary_transactions --test shortcuts_cli` | Pass (132 + 70) | 2026-10-14 |
| 5 Settings screen | Todo | — | — | Not started: waits for the Penflow pass (code-ir, semantic and test IDs, design certification) | 2026-10-14 |
| 6 Acceptance, performance, regressions | In Progress | `proofs/claude-impl/final/` | `cargo test` (all targets); `bun run test:keyboard`, `test:shortcut-list`, `test:settings-errors`, `test:settings-store` | Automated suites pass (749 Rust tests, 21 ignored); native proof, SC-003, performance and UI not run | 2026-10-14 |

## Sync with dev (2026-10-14)

Merge `bc1143ee` of `dev` (`825598e9`: smart paste 007 behind `smart-paste-dispatch`, clipboard
test fix, docs). No textual conflict; the files changed on both sides were checked: one
`[features]` table in `Cargo.toml` (`smart-paste-dispatch`, `destination-return`) beside the
`caseless` pin, 65 public keys with 003's `custom_words` and `custom_filler_words` refusals, 007's
`ValidationReason` variants beside 003's closed `ErrorReason` list, README and changelogs (dev's
versions; 003 did not touch them). The 008 checked schema (contract DTO generator) and
`bindings.ts` (debug specta export, isolated HOME, `--list-models`) were regenerated and came out
byte-identical to the merged files. Evidence in `proofs/claude-sync/` (`summary.txt`):
full `cargo test` 954 passed, 0 failed, 25 ignored; clippy `--lib --bins -D warnings` exit 0;
clippy `--all-targets` exit 0 with the 5 pre-existing upstream warnings only; `cargo fmt --check`,
`bun run lint`, `bun run format:check`, `tsc`, the frontend unit tests and the translation check
exit 0; repository conventions PASS, new 0, baseline unchanged. `livespec validate
--progression implement --model claude-opus-5-5 --review-max-chars 180000` after the sync:
`Progression: READY`, exit 0 (`proofs/claude-sync/validate-implement.*`); no review was rerun.

## Resume point

Resume at Step 0a and Step 5 once the Penflow pass for `recognition-vocabulary` / `vocabulary-settings`
is certified, then run Step 6. The light-lane delivery review (one `combined` report) runs on the
complete diff after that, by other agents.

## What remains

1. **UI (Steps 0a and 5).**
   - Turn `CustomWords.tsx` into the "Vocabulary" editor: ordered rows with their legacy flags,
     add and per-row remove, paste box with preview and Apply, one status, terms sent and not sent,
     the error `context_budget_unverified`, and the vocabulary versus replacements explanation.
   - Supporting code: `use-vocabulary.ts` hook, revision-aware store, en/fr keys, keyboard and
     Behavioral AC tests (Playwright).
   - Already available: the typed commands `getVocabulary`, `previewVocabularyImport`,
     `applyVocabularyImport`, `addVocabularyTerm`, `removeVocabularyOccurrence`,
     `getVocabularyStatus`, and `updateCustomWords(words, expectedRevision)`.
2. **Generic mechanisms.** Generic vocabulary and context prompt (Qwen3-ASR, Fun-ASR-Nano, Granite
   Speech, MOSS) are disabled (`context_budget_unverified`). Under the plan this blocks AC-004 for
   these families until the engine exposes the room or the number of terms kept
   (`contracts/context-budget.md`).
3. **macOS native evidence (Step 6).** None of it has been executed:
   - `scripts/vocabulary-native-proof.ts`;
   - Whisper tokenizer measurement of a real prompt;
   - the request actually sent;
   - SC-003 on a "Claude Code" sample;
   - preparation time under 5 ms;
   - the capabilities read from catalog models (Granite GGUF);
   - live `vocabulary status` in the running app (model loaded and not loaded);
   - no model reload after an edit.

   Windows and Linux evidence is recorded as pending release evidence.
4. **Closure.** Light-lane delivery review, changelogs, `finalize` (spec status) and the README
   activity row. Spec status not changed in this run.

## Notes

- The pre-existing `cargo clippy --tests` errors on `dev` (lib.rs, supervisor.rs, gguf_meta.rs,
  transcription.rs, portable.rs) are unchanged. The step gate is `clippy --lib --bins -D warnings`
  (PASS).
- The conventions baseline was never rewritten. Every step's conventions verdict is PASS (receipts
  under `.specs/conventions/runs/`, ignored by git, and their stdout in `proofs/claude-impl/<step>/`).
