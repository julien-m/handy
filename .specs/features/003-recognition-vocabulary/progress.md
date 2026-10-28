# Progress — 003-recognition-vocabulary

`/spec-implement`, light lane (`proofs/claude-review/delivery-lane-r3.json`). Branch
`feature/003-recognition-vocabulary`, worktree from `dev`. Implementer: Claude (`claude-opus-5-5`).
The non-UI steps (1 to 4) were built first; the settings screen (Steps 0a and 5) was then built on
`feature/ui-003-vocabulary` from the approved Penflow mockups and corrected in UI fix round 1 (see
"UI pass"). Per-step evidence lives in `proofs/claude-impl/<step>/`, the UI evidence in
`proofs/claude-ui/` and `proofs/claude-ui-fix1/` (`*.stdout`, `*.stderr`, `*.exit`, `summary.txt`).

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a Behavioral TDD | Done (unit level) | `src/stores/vocabulary-store.test.ts`, `src/components/settings/vocabulary/vocabulary-model.test.ts`, `vocabulary-notices.test.tsx`, `src/hooks/use-vocabulary.test.ts` | `bun run test:vocabulary` | Pass (45 after fix round 1): visual states of the editor, term input and Apply, busy commit, refusals, conflicts, durable outcomes, stale answers, Apply at the preview's revision, list loading and read failure, focus ring and removal focus rules, hook bindings, contract test IDs. No Playwright spec (see "What remains") | 2026-10-14 |
| 1 Capability and budget | Done | `recognition_vocabulary/{mod,capability,budget,context}.rs`, `engine_supervisor/{protocol,worker,supervisor,supervisor_context}.rs`, `contracts/context-budget.md` | `cargo test --lib -- recognition_vocabulary engine_supervisor::protocol engine_supervisor::supervisor::context` | Pass (18) | 2026-10-14 |
| 2 Validation and transactions | Done | `recognition_vocabulary/{validation,occurrences,preview,edits,edits_tests}.rs`, `settings_service/{error_reason,errors,command_error,tauri_adapter}.rs` (`write_change`), `commands/vocabulary.rs`, `shortcut/mod.rs` (`update_custom_words`), `Cargo.toml` (`caseless = "=0.2.2"`), `tests/vocabulary_transactions.rs`, `src/bindings.ts`, `src/stores/settingsStore.ts`, 008 contract schema and doc | `cargo test --lib -- recognition_vocabulary settings_service`; `cargo test --test vocabulary_transactions --test settings_validation --test settings_cli`; `bun run test:settings-errors`, `test:settings-store`, `test:shortcut-list` | Pass | 2026-10-14 |
| 3 Request snapshot, status and correction routing | Done | `recognition_vocabulary/{status,status_tests}.rs`, `managers/{transcription,transcription_vocabulary}.rs`, `commands/vocabulary.rs` (`get_vocabulary_status`), `src/bindings.ts`; refactor of `edits.rs`, `preview.rs`, `budget.rs`, `occurrences.rs` to functions of at most 30 lines | `cargo test --lib -- recognition_vocabulary managers::transcription audio_toolkit::text engine_supervisor::protocol engine_supervisor::supervisor::context`; `cargo test --test vocabulary_transactions` | Pass (111 + 4) | 2026-10-14 |
| 4 CLI contract and native routing | Done | `settings_service/domains/{vocabulary,vocabulary_tests,mod}.rs`, `settings_service/{cli,render}.rs`, `src/cli.rs`, `tests/{vocabulary_cli,settings_domains}.rs`, `README.md`, 008 contract doc | `cargo test --lib -- settings_service recognition_vocabulary`; `cargo test --test vocabulary_cli --test settings_domains --test settings_cli --test settings_validation --test vocabulary_transactions --test shortcuts_cli` | Pass (132 + 70) | 2026-10-14 |
| 5 Settings screen | Done, native proof pending | `src/components/settings/CustomWords.tsx` (Vocabulary group), `src/components/settings/vocabulary/*`, `src/stores/vocabulary-store.ts`, `src/hooks/use-vocabulary.ts`, `advanced/AdvancedSettings.tsx`, 26 locales, `package.json` (`test:vocabulary`) | lint, format:check, tsc, check:translations, build, `test:vocabulary` and the existing frontend suites; conventions verify | Pass (`proofs/claude-ui/summary.txt`, then `proofs/claude-ui-fix1/summary.txt`). Browser checks with mocked IPC only (`proofs/claude-ui/harness/`, `proofs/claude-ui-fix1/harness/`); real window and native evidence pending, with Julien | 2026-10-14 |
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

## UI pass (2026-10-14, branch `feature/ui-003-vocabulary`)

The editor follows the approved mockups (`vocabulary-settings`, `-paste`, `-unsupported`, `-empty`) and every
contract test ID of these four screens (a unit test checks the source emits them). The "Custom Words" row of
Advanced > Transcription is replaced by a "Vocabulary" settings group under it; its keys
`settings.advanced.customWords.*` are removed. Evidence in `proofs/claude-ui/` (`summary.txt`). The screen
was checked in a browser with mocked IPC: the harness source is committed as
`proofs/claude-ui/harness/vocab-harness.tsx.txt` with its captures, and was not left in `src/`. No real
Handy window, no native capture, no keyboard check in the app yet: pending, with Julien.

**UI fix round 1** (delivery review `combined`, BLOCKED with 2 blocking and 7 warnings). Fixed:
- visible keyboard focus on Cancel and on both Retry buttons (status and commit failure);
- the commit failure says "Your saved vocabulary is unchanged" only without a durable outcome; an
  `unknown` outcome rereads the list and keeps Retry; a `committed` one rereads the list, has its own
  message and no Retry;
- Apply commits at the revision its preview was computed on;
- the saved list shows a loading line before its first answer and an error with Retry when the read
  fails, never "No terms yet";
- focus returns to "Paste a list…" after Cancel or Apply, stays on the term input after Add, and stays
  on the same term after a failed removal (also when the list is reread);
- `use-vocabulary.test.ts` tests the hook's bindings (the effect bodies; no DOM renderer is installed);
- the 2 MB `proofs/claude-ui/conventions.stdout` is replaced by `conventions-summary.txt`.

Evidence in `proofs/claude-ui-fix1/` (`summary.txt`, `harness/checks.txt` and captures, mocked IPC).

**UI hardening pass** (2026-10-14, branch `fix/ui-hardening-a`; warnings of the UI delivery review
round 2, `combined` PASS). Fixed:
- Apply is made inactive with `aria-disabled` (never `disabled`), so a focused Apply keeps the focus
  while it sends and after a failed Apply; activating it while inactive does nothing;
- after a successful Retry the focus goes to a stable point: the term input (Add or removal) or
  "Paste a list…" (Apply, the paste box closes);
- Retry of a failed Apply follows the Apply rule on the preview reread after the failure: a preview that
  cannot be applied sends nothing and shows its note in a toast, a merge that adds nothing is never sent
  (also refused by the store's `apply`), a success shows the same "Terms added" toast as Apply;
- `AddButton` and `CustomWords` are 30 lines or fewer (`addFromButton`, `AddHint`, `vocabularyScreen`,
  `StatusView`).

Evidence in `proofs/claude-hardening/summary.txt`. Not changed: Apply's focus ring (`primary` variant),
to check in the real-window keyboard proof with Julien.

## Resume point

The UI is built and its delivery-review findings of round 1 are fixed. Next: the delivery review reruns
on the fixed diff (other agents), then the real-window, keyboard-in-app and native evidence with Julien
(Step 6).

## What remains

1. **UI evidence (Steps 0a and 5).** The editor is built and unit-tested. What remains:
   - real Handy window captures of the four screens and their states, keyboard traversal and focus
     in the running app, the status with a loaded and an unloaded model: pending, with Julien;
   - the Playwright spec `tests/vocabulary-settings.spec.ts` declared by the plan is not written; the
     mocked-IPC harness shows it is feasible. Recorded as pending evidence, with Julien;
   - the Penflow contract still has to record the deviations listed in `implementation.md`
     ("Design deviations").
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
