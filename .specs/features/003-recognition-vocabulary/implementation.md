---
feature: recognition-vocabulary
title: Recognition Vocabulary Implementation
spec_ref: spec.md
updated: 2026-10-14
---

# Implementation — Recognition Vocabulary

This run covers the non-UI steps (1 to 4) of [the plan](plan.md) for [the spec](spec.md), on
`feature/003-recognition-vocabulary` from `dev` (008, 006 step 0 and 004 included). The settings
screen (Step 5, with the Behavioral AC tests of Step 0a) waits for the Penflow pass. The native
proof (Step 6) has not been run. Inspect [progress](progress.md) for steps, evidence and what
remains. No AC is certified. Nothing on the screen shows the vocabulary status yet.

## FR mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-001](spec.md#fr-001) | [validation](../../../src-tauri/src/recognition_vocabulary/validation.rs), [occurrences](../../../src-tauri/src/recognition_vocabulary/occurrences.rs), [preview](../../../src-tauri/src/recognition_vocabulary/preview.rs), [edits](../../../src-tauri/src/recognition_vocabulary/edits.rs), [UI commands](../../../src-tauri/src/commands/vocabulary.rs), [`update_custom_words`](../../../src-tauri/src/shortcut/mod.rs), [`tauri_adapter::write_change`](../../../src-tauri/src/settings_service/tauri_adapter.rs) | `FR-001: Shared term validation and keys`, `FR-001: Legacy terms kept and flagged`, `FR-001: Bulk paste preview and atomic candidate`, `FR-001: One validated atomic vocabulary edit`, `FR-001: Paste preview command`, `FR-001: Per-occurrence removal command`, `FR-001: UI list commit through the shared change` | ⚠️ Partial. The backend is implemented: validation, keys, legacy flags, preview, atomic commit, typed UI commands. The settings editor (Step 5) is not built. | 2026-10-14 |
| [FR-002](spec.md#fr-002) | [capability](../../../src-tauri/src/recognition_vocabulary/capability.rs), [status](../../../src-tauri/src/recognition_vocabulary/status.rs), [manager](../../../src-tauri/src/managers/transcription_vocabulary.rs), [UI command](../../../src-tauri/src/commands/vocabulary.rs) | `FR-002: Loaded capability decides context support`, `FR-002: Five truthful status outcomes`, `FR-002: Status from the loaded model and mode`, `FR-002: Status command for the settings UI` | ⚠️ Partial. The status is computed and served to the UI and the CLI. Nothing renders it yet (Step 5). A model that declares a generic mechanism gets the technical error `context_budget_unverified`, not one of the five statuses. | 2026-10-14 |
| [FR-003](spec.md#fr-003) | [budget](../../../src-tauri/src/recognition_vocabulary/budget.rs), [context](../../../src-tauri/src/recognition_vocabulary/context.rs), [worker](../../../src-tauri/src/engine_supervisor/worker.rs), [supervisor query](../../../src-tauri/src/engine_supervisor/supervisor_context.rs), [manager](../../../src-tauri/src/managers/transcription_vocabulary.rs), [budget contract](contracts/context-budget.md) | `FR-003: Whole terms in list order within budget`, `FR-003: Exact request within documented budget`, `FR-003: Batch request carries the fitted terms` | ⚠️ Partial. Only the Whisper initial prompt is enabled (223 tokens, counted by the worker with the model's tokenizer). The generic mechanisms (Qwen3-ASR, Fun-ASR, Granite, MOSS) stay disabled: see the budget contract. Native request evidence is pending. | 2026-10-14 |
| [FR-004](spec.md#fr-004) | [manager](../../../src-tauri/src/managers/transcription.rs) (`post_process_transcription_text`, `RunOutcome::context_applied`) | `FR-004: Correction only when no context was sent` | Implemented, certification pending. Correction is skipped only when at least one term reached the engine. Correction stays off for an empty list and still runs once on streamed text. | 2026-10-14 |
| [FR-005](spec.md#fr-005) | [domain](../../../src-tauri/src/settings_service/domains/vocabulary.rs), [CLI wiring](../../../src-tauri/src/settings_service/cli.rs), [clap types](../../../src-tauri/src/cli.rs), [renderer](../../../src-tauri/src/settings_service/render.rs), [README](../../../README.md), [008 contract](../008-settings-cli/contracts/settings-cli.md) | `FR-005: Vocabulary CLI operations`, `FR-005: Live vocabulary status`, `FR-005: Vocabulary CLI commands` | Implemented, certification pending. `status` inside a real running Handy with a loaded model has not been observed. | 2026-10-14 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| [AC-001](spec.md#ac-001) | [validation tests](../../../src-tauri/src/recognition_vocabulary/validation.rs), [preview tests](../../../src-tauri/src/recognition_vocabulary/preview.rs), [edit tests](../../../src-tauri/src/recognition_vocabulary/edits_tests.rs), [transactions](../../../src-tauri/tests/vocabulary_transactions.rs) | ⚠️ Partial. Rules covered: 1–100 scalars, line breaks, `<`, `>`, `"` and NUL, full case-fold keys, 500 terms, no partial write. The UI list, add, remove and paste box with preview are not built. |
| [AC-002](spec.md#ac-002) | [transactions](../../../src-tauri/tests/vocabulary_transactions.rs), [occurrence tests](../../../src-tauri/src/recognition_vocabulary/occurrences.rs) | ⚠️ Partial. Stored terms keep their order and bytes, and the threshold is unchanged across unrelated writes (SC-002). The "Vocabulary" label and the explanation in the UI are not done. |
| [AC-003](spec.md#ac-003) | [status matrix](../../../src-tauri/src/recognition_vocabulary/status_tests.rs), [capability tests](../../../src-tauri/src/recognition_vocabulary/capability.rs) | ⚠️ Partial. The five statuses come from the loaded capabilities and are never "used" without a term in the request. Display is pending (Step 5). Generic-mechanism models get the error. |
| [AC-004](spec.md#ac-004) | [budget tests](../../../src-tauri/src/recognition_vocabulary/budget.rs), [context tests](../../../src-tauri/src/recognition_vocabulary/context.rs), [protocol tests](../../../src-tauri/src/engine_supervisor/protocol.rs), [status matrix](../../../src-tauri/src/recognition_vocabulary/status_tests.rs) | ⚠️ Partial. Whisper only, with terms sent and not sent reported. Generic mechanisms are disabled, which blocks release for those families under the plan. No measurement on a real tokenizer, no request observed. |
| [AC-005](spec.md#ac-005) | [correction routing test](../../../src-tauri/src/managers/transcription_vocabulary.rs), [status matrix](../../../src-tauri/src/recognition_vocabulary/status_tests.rs), [context tests](../../../src-tauri/src/recognition_vocabulary/context.rs) | Implemented, certification pending. Native evidence is pending. |
| [AC-006](spec.md#ac-006) | [correction routing test](../../../src-tauri/src/managers/transcription_vocabulary.rs) | Implemented, certification pending. Each batch request uses its own settings snapshot and the worker refits it, with no model reload. A real-app edit between two dictations has not been observed. |
| [AC-007](spec.md#ac-007) | [CLI process tests](../../../src-tauri/tests/vocabulary_cli.rs), [domain tests](../../../src-tauri/src/settings_service/domains/vocabulary_tests.rs) | Implemented, certification pending. Covered: the six operations, file and stdin import, dry run, `details.items`, exit codes, and offline `unavailable` for `status`. A live `status` with a loaded model is pending. |

| SC | Evidence | Status |
|---|---|---|
| [SC-001](spec.md#sc-001) | [status matrix](../../../src-tauri/src/recognition_vocabulary/status_tests.rs) | ⚠️ Partial. Every engine family is covered with simulated capabilities. Real engine requests are not observed. |
| [SC-002](spec.md#sc-002) | [transactions](../../../src-tauri/tests/vocabulary_transactions.rs) | Implemented, certification pending |
| [SC-003](spec.md#sc-003) | — | Pending: no model or recorded sample on the implementation machine |

## Decisions and deviations

- **Wrapper `tauri_adapter::write_change(app, &DomainChange, expected)`.** `dev` and the 001 branch did not have it, so it was created here with the signature from plans 001 and 005. If 001 merges first, keep a single copy.
- **`context_budget_unverified` versus the five statuses.** The spec does not allow a model that declares a mechanism without a documented budget to show one of the five statuses:
  - "not supported" would contradict the declared capability;
  - "no term sent" only covers an empty list or a full budget;
  - "unknown" would claim the model is not loaded.

  So it stays a technical error: in the UI, the command returns `SettingsCommandError` with `detail_reason`; in the CLI, `internal_error`, exit 1. Dictations run without context and use approximate correction. This applies to any model, in or out of the catalog. In this build it also applies to the catalog families with a generic mechanism.
- **Engine query unanswered.** A loaded model whose engine does not answer within 2 s gives `unknown_until_model_loads` with its own reason, `capability_query_unanswered` (review r3, item 5).
- **Empty list on a model without context.** The status stays `not_supported`: it describes the model. "No term sent, empty vocabulary" only applies to a model that supports context.
- **Terms sent are not persisted.** The status is the exact preparation of the next request (`evidence_kind: prepared`). The last request actually sent (`dispatched`) is not recorded, because the spec does not require it. Logs keep only counts.
- **Not built in this run:**
  - no cache of the context preparation: one worker call per dictation, performance not measured;
  - no new invalidation event: the UI will refresh on `settings-changed` and `model-state-changed`.
- **Generic `set` refusal.** Only `custom_words` gets `VocabularyDomain` ("Use handy settings vocabulary to change this list."); `custom_filler_words` gets `UiOnly` ("This setting is changed in the settings UI."), since the vocabulary commands do not manage filler words and the CLI must not point to them. `settings_cli` `list_secret_and_read_only_keys_point_to_their_owner` fixes both.

## Files Created/Modified

- `src-tauri/src/recognition_vocabulary/`: new module.
  - capability, budget and context: Step 1;
  - validation, occurrences, preview and edits: Step 2;
  - status: Step 3.
- `src-tauri/src/engine_supervisor/`:
  - `protocol.rs`: `PrepareContext` and `ContextPrepared`, `LoadedInfo.context`;
  - `worker.rs`: probing and fitting;
  - `supervisor.rs` and new `supervisor_context.rs`: bounded query.
- `src-tauri/src/managers/`:
  - `transcription.rs`: `context_applied` replaces `model_is_whisper`;
  - new `transcription_vocabulary.rs`: request prompt and status.
- `src-tauri/src/settings_service/`:
  - `error_reason.rs`: 6 reasons;
  - `errors.rs`;
  - `command_error.rs`: `detail_reason`, `items`;
  - `tauri_adapter.rs`: `write_change`;
  - `domains/{mod,vocabulary,vocabulary_tests}.rs`;
  - `cli.rs`, `render.rs`.
- `src-tauri/src/cli.rs`: `vocabulary` subcommand.
- `src-tauri/src/commands/{mod,vocabulary}.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/shortcut/mod.rs`.
- `src-tauri/Cargo.toml` and `Cargo.lock`: `caseless = "=0.2.2"`.
- Tests:
  - `src-tauri/tests/vocabulary_transactions.rs` and `vocabulary_cli.rs` (new);
  - `settings_domains.rs`, `settings_cli/contract.rs` and `settings_validation/error_details.rs` (updated).
- Frontend: `src/bindings.ts` (regenerated); `src/stores/settingsStore.ts` (`updateCustomWords(words, null)`).
- Docs and contracts:
  - `README.md`;
  - `.specs/features/008-settings-cli/contracts/settings-cli.{md,schema.json}`;
  - `contracts/context-budget.md`.
