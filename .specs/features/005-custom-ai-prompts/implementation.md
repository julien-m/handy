# Implementation — 005 Custom AI Prompts

Status: **In Progress** — every non-UI step of the [plan](plan.md) is implemented on `feature/005-custom-ai-prompts` (Steps 2, 3, 4, 7 and the backend of Step 5); the settings UI of Step 6 is implemented on `feature/ui-005-prompts`; the indicator UI (Step 1 and the indicator part of Step 6), real-window visual proofs, the native macOS evidence and the acceptance run (Step 8), and the end-of-feature delivery review (full lane) remain. Step-by-step evidence and every deviation from the plan text are in [progress.md](progress.md).

## Commits

| Step | Commit | Subject |
|---|---|---|
| 2 | `aeedb3f7` | validate the prompt library and add the starter prompts once |
| 3 | `7ed149a7` | manage prompts and their shortcuts from the settings CLI |
| 4 | `a3775ca7` | bound the AI step to 30 seconds and add a side-effect-free prompt test |
| 7 | `17a2c11f` | record the prompt and AI failure of each dictation in history |
| 5 (backend) | `02d654f3` | give each dictation its own prompt, chosen by its trigger |

## Requirements

| Requirement | Source | `@spec` anchor | Status | Updated |
|---|---|---|---|---|
| [FR-001: Prompt library, migration, starters](spec.md#fr-001) | `settings.rs` (`LLMPrompt`, `PromptProviderOverride`, schema version 3), `prompts/{library,migration,types,view}.rs`, `commands/prompts.rs` | `@spec FR-001: Validated prompt create`, `Migrate prompts and add starters once`, `Delete resets the default to None`, `Prompt library with stable ids and override`, `Prompt library read for the settings` | ✅ Backend and settings UI implemented (`feature/ui-005-prompts`); real-window and native evidence pending | 2026-10-14 |
| [FR-002: Prompt of each dictation](spec.md#fr-002) | `prompts/session.rs`, `prompts/dictation.rs`, `actions.rs`, `transcription_coordinator.rs` (`is_transcribe_binding`), `actions.rs` (`action_for`), `commands/prompts.rs` (`select_dictation_prompt`) | `@spec FR-002: Prompt of a dictation from its trigger`, `Per-dictation choice without changing the default`, `Indicator choice for this dictation only` | 🟡 Backend implemented; indicator menu (native, ≤200 ms, no focus steal) pending | 2026-10-14 |
| [FR-003: Provider, readiness, 30 s budget](spec.md#fr-003) | `prompts/{provider,processor,native,formatting}.rs` | `@spec FR-003: Resolve provider and report readiness`, `Whole AI step bounded to 30 seconds` | ✅ Implemented (Apple Intelligence evidence on a real Apple Silicon run pending) | 2026-10-14 |
| [FR-004: Raw transcript and failure on any AI failure](spec.md#fr-004) | `prompts/{processor,output,dictation}.rs`, `managers/history_store.rs` (migration 6), `actions.rs` | `@spec FR-004: Failure and cancellation outcomes`, `Original transcript on any AI failure`, `Raw transcript kept when the AI step fails`, `History keeps the prompt and the failure` | 🟡 Backend implemented; on-screen notice pending | 2026-10-14 |
| [FR-005: Test without insertion or history](spec.md#fr-005) | `prompts/test_run.rs`, `commands/prompts.rs` (`test_post_process_prompt`, `cancel_post_process_prompt_test`) | `@spec FR-005: Test operation without insertion or history`, `Prompt test without insertion or history` | 🟡 Backend and test panel implemented (`feature/ui-005-prompts`); real-window evidence pending | 2026-10-14 |
| [FR-006: Prompts through the settings CLI](spec.md#fr-006) | `settings_service/domains/prompts.rs`, `settings_service/{cli,render,tauri_adapter}.rs`, `cli.rs`, `prompts/types.rs`, `prompts/library.rs` | `@spec FR-006: Prompts CLI list, create, update, delete, default, override`, `Shared prompt mutation validation`, `Set or clear the provider override`, `Per-prompt provider override` | ✅ Implemented | 2026-10-14 |

## Acceptance criteria

| AC | Evidence today (automated) | Missing for acceptance |
|---|---|---|
| [AC-001](spec.md#ac-001) | `prompts/library_tests.rs`, `settings_service/domains/prompts_tests.rs`, `tests/prompt_cli.rs`; UI: `post-processing/prompt-settings-model.test.ts`, `prompt-commands.test.ts` (incl. `committed`/`unknown` outcomes), `prompt-editor-sync.test.ts` (own save vs a library one revision behind, failed reload, typing during a save, Cancel during a creation, invalid prompt opened) | Real-window evidence of the list, editor, confirmation and Behavioral AC states (browser-mock screenshots only) |
| [AC-002](spec.md#ac-002) | `prompts/migration.rs` tests, `tests/prompt_library.rs` (real store, byte-identical prompts and selection) | Real-window evidence of the invalid flag (implemented: list badge, default option disabled) |
| [AC-003](spec.md#ac-003) | `prompts/session_tests.rs`, `shortcut/prompt_shortcuts_tests.rs`, [indicator models](../../../src/overlay/indicator-models.test.ts), [indicator markup](../../../src/overlay/indicator-render.test.tsx), [menu timers](../../../src/overlay/menu-timer.test.ts) | Indicator pill and menu built (browser-tested, `feature/ui-overlay`; hover close cancelled on unmount and at each new dictation, hardening 2026-10-14); native surface, 200 ms, no focus change pending (macOS evidence) |
| [AC-004](spec.md#ac-004) | `prompts/provider_tests.rs`, `prompts/processing_http_tests.rs` (override routed to a loopback endpoint); UI: `prompt-editor-rules.test.ts` (inline fix targets the resolved provider) | Real-window evidence of the "not ready" display (implemented in the list and editor) |
| [AC-005](spec.md#ac-005) | `prompts/processor_tests.rs`, `prompts/output.rs` tests, `tests/prompt_history.rs`, notice wording test in `prompts/dictation.rs`, [indicator models](../../../src/overlay/indicator-models.test.ts) | Notice ≥4 s on the indicator held after the hide (browser-tested), toast with style none; macOS end-to-end raw-text and notice evidence pending |
| [AC-006](spec.md#ac-006) | `shortcut/prompt_shortcuts_tests.rs`, `prompts/session_tests.rs` (deletion takes the shortcut lease) | Native registration evidence (shortcut rows implemented through 004's list, `shortcut-list-model.test.ts`) |
| [AC-007](spec.md#ac-007) | `prompts/test_run_tests.rs`; UI: `prompt-editor-rules.test.ts` (result outdated after a text, provider, model or sample change) | Real-window evidence of the test panel (implemented: sample prefill, busy, retry; `prompt-settings-model.test.ts`) |
| [AC-008](spec.md#ac-008) | `tests/prompt_cli.rs` (real binary, offline and routed, key sentinel), `settings_service/domains/prompts_tests.rs` | — |

**Settings UI, delivery-review fix round 1 (2026-10-14, `feature/ui-005-prompts`):**

- **Deviation from the contract — readiness fix inside the editor.** The contract action `prompt-settings-invalid.form.fix` navigates to `prompt-settings` (the page's provider settings). Orchestrator decision: the fix happens in the editor, for the provider the prompt resolves to. "Add API key" / "Set the endpoint" (`<prefix>-form-fix`) opens an inline field (`<prefix>-form-fix-field`) that reuses the existing `ApiKeyField` / `BaseUrlField` and the store's `updatePostProcessApiKey` / `updatePostProcessBaseUrl`, i.e. the backend commands `change_post_process_api_key_setting` / `change_post_process_base_url_setting`. Both store per provider (`post_process_api_keys[provider_id]`, the custom provider's `base_url`) and never touch `post_process_provider_id`, so the default provider does not change and the draft is never unmounted. The page's provider group still configures the default provider as before. Penflow should align the contract (fix as an inline field, no navigation).
- **Test-ID prefixes of the editor.** The editor's prefix follows its mode: a saved prompt uses `prompt-settings-editor-*`, a new prompt `prompt-settings-invalid-*` (the contract's "New prompt" screen). Consequences to align in the Penflow contract later: `prompt-settings.new` leads to `prompt-settings-invalid-screen` (the contract says `prompt-settings-editor`); once a new prompt is saved the editor switches to `prompt-settings-editor-*`; the error and not-ready nodes (`-form-name-error`, `-form-not-ready`, `-form-fix`) exist under both prefixes. New IDs of this round: `<prefix>-reload-error` / `-reload-retry`, `<prefix>-form-fix-field`, `<prefix>-form-shortcuts-hint`, `<prefix>-test-outdated`, `prompt-settings-reload-error` / `-reload-retry`.
- **Other changes:** the editor ignores a library older than its own save (`prompt-editor-sync.ts`: values or revision), shows a failed reload with Retry and never restores older values; keystrokes typed during a save are kept; a creation answered after Cancel never reopens the editor; a `committed`/`unknown` outcome is resolved by rereading the library (`prompt-commands.ts`); the default-prompt select catches a failed command (translated toast); a test result is marked outdated once the text, provider, model or sample changes; an invalid saved prompt shows its issues at once; a hint says shortcut changes are saved immediately; the model select has an accessible name, the name field takes the focus when the editor opens and the edit button (or New prompt) takes it back when it closes.
- **Pending:** the Playwright spec `tests/custom-ai-prompts.spec.ts` named by plan Step 6 does not exist; the hooks are covered through their extracted pure functions (no DOM test tool in the project) and a browser-mock pass. The Playwright run and the real-window evidence stay pending, with Julien.

**AC-001 precision to carry into the spec (orchestrator decision 2026-10-14):** a prompt text made only of whitespace is refused (`prompt_text_empty`); the text must be non-empty after trimming and have 1–8000 characters. The UI already disabled Save in that case.

## Files

- **Created:** `src-tauri/src/prompts/{mod,types,library,library_tests,migration,view,provider,provider_tests,formatting,processor,processor_tests,processing_http_tests,native,output,test_run,test_run_tests,session,session_tests,dictation}.rs`, `src-tauri/src/commands/prompts.rs`, `src-tauri/src/settings_service/domains/{prompts,prompts_tests}.rs`, `src-tauri/src/shortcut/{prompt_targets,prompt_shortcuts_tests,gated}.rs`, `src-tauri/tests/{prompt_library,prompt_cli,prompt_history}.rs`.
- **Modified:** `src-tauri/src/{settings,actions,lib,main,cli,utils,secure_input,transcription_coordinator,llm_client,tray}.rs`, `src-tauri/src/settings_service/{errors,error_reason,platform_capability,public,protocol,cli,render,tauri_adapter}.rs`, `src-tauri/src/settings_service/domains/{mod,output,shortcuts,fixture_tests}.rs`, `src-tauri/src/shortcut/{mod,edits,edits/tests,action_shortcuts,bindings_change,keyboard_switch}.rs`, `src-tauri/src/managers/{history,history_store}.rs`, `src-tauri/src/commands/{mod,history}.rs`; tests `settings_transactions/ownership.rs`, `settings_validation/error_details.rs`, `settings_domains.rs`, `destination_recovery.rs`; `src/bindings.ts`, `src/stores/settings-errors.ts` (+test), 26 locales, `README.md`, 008 `contracts/settings-cli.{md,schema.json}`.

## UI hardening pass (2026-10-14, branch `fix/ui-hardening-a`)

- `prompt-provider-fix.tsx`: `providerFixFieldOf` renders `ProviderFixField` with `key={fixFieldKeyOf(fix, providerId)}`; `useSaveFix` writes through `updatePostProcessSetting(fixSettingOf(fix), providerId, value)`, so the endpoint fix keeps the model (the API group keeps `updatePostProcessBaseUrl` and its model reset, from upstream `f1516d92`).
- `prompt-draft-model.ts`: `actionNoteKeyOf` (note "fixFields" only with a field error shown), used by `prompt-editor-actions.tsx`.
- `ui/Alert.tsx`: `hasAlertAction` (undefined, null, false and "" are no action), `AlertBody`.
- Tests: `prompt-editor-rules.test.ts` (fix setting, field key, action note), `prompt-settings-render.test.tsx` (field key per provider, empty action). Proofs: [summary](proofs/claude-hardening/summary.txt).

## Remaining work

1. **UI (Steps 1 and 6):** the Playwright spec `tests/custom-ai-prompts.spec.ts` (pending, with Julien); the settings part (prompt list/editor with confirmation and readiness, default selector, provider override, prompt shortcut rows, test panel) is done on `feature/ui-005-prompts` (see progress.md, Step 6), its real-window evidence pending with Julien; remaining: indicator prompt chip and menu (`get_dictation_prompt`, `select_dictation_prompt`, event `dictation-prompt-changed`), the ≥4 s failure notice (event `prompt-processing-failed`, key `overlay.fallbackNotice`, including the overlay style `none` case), history display of the processing record; native spike on macOS (NSPanel, 200 ms, no focus change).
2. **Proofs (Step 8):** macOS release-binary evidence (menu latency/focus, notice, raw text handed to insertion, prompt shortcuts registered/released, Apple Intelligence deadline and busy worker), acceptance mapping (`livespec test --derive-mapping` from the `@spec AC-xxx` test anchors), Windows/Linux as pending release evidence.
3. **Delivery review (full lane):** one `review` and one `audit` of the complete diff, recorded with `livespec delivery review`.
4. **Spec:** report the AC-001 whitespace precision through `spec-refine`.

## Penflow test ID alignment (2026-10-17, branch `fix/ui-test-ids`)

Decisions of Julien of 2026-10-17 (inventory 005-09, 005-11 and D1 for 005-12). No Rust change.

- **Instance ids (005-11):** each prompt row of the library carries `data-instance-id="prompt-<id>"` (`prompt-{id}` of `prompt-settings-list`, the prompt's technical id, as on the indicator) in [the prompt list](../../../src/components/settings/post-processing/prompt-list.tsx); each stored shortcut row of the editor carries `data-instance-id="shortcut-<keys>"` (`shortcut-{keys}` of `-editor-form-shortcuts`) in [the shared shortcut row](../../../src/components/settings/shortcut-row.tsx). `data-key` is kept.
- **AI notice text (005-09):** verified unchanged, `overlay.fallbackNotice` is already « AI processing failed. The original text was used. »; nothing changed.
- **Tests:** `prompt-settings-render.test.tsx` (both instance ids).
- **D1 — indicator prompt menu (005-12):** while the prompt menu is open, the bar, the pill, the prompt, its shortcut and the destination badge carry `recording-indicator-prompt-menu-…` (screen chosen by [indicator-screen](../../../src/overlay/indicator-screen.ts)); details in the 006 and 009 implementation notes. Test: `indicator-render.test.tsx`.

## Native session fixes (2026-10-28, branch `fix/native-session-ui`)

- **N5 (keyboard focus, checked in 005).** The list already gives the focus back to the button of the closed editor; [the editor](../../../src/components/settings/post-processing/prompt-editor.tsx) now uses `useFocusRescue` of [focus-return](../../../src/components/ui/focus-return.ts) with its group title as the stable point, for a focus lost while the editor stays open. The prompt shortcuts ([editor](../../../src/components/settings/shortcut-list-editor.tsx)) add a capture at once, without the draft of 004, so N8 does not apply to them.
- **N5, review correction (2026-10-28).** The editor root took an inline ref callback, so React detached and attached the focus rescue on every render and it forgot that the focus was in the editor: after Save, the title never took the focus back in WebKit. The ref is now a stable `useCallback` in [the editor](../../../src/components/settings/post-processing/prompt-editor.tsx), and `useFocusRescue` ([focus-return](../../../src/components/ui/focus-return.ts)) keeps what it knew when the same root is attached again and rescues a focus lost in between. Tests: [prompt-editor-focus.spec.ts](../../../tests/prompt-editor-focus.spec.ts) (harness [prompt-editor](../../../tests/harness/prompt-editor.tsx)) and [focus-rescue.spec.ts](../../../tests/focus-rescue.spec.ts), each also with `?webkitFocus` (no focusout on removal, as in WebKit), which fails on the previous code.
