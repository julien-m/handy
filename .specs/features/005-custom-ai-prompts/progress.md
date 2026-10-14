# Progress — 005 Custom AI Prompts

`/spec-implement` run (Claude `claude-opus-5-5`, full lane), branch `feature/005-custom-ai-prompts`, worktree `handy-wt/005-custom-ai-prompts`, build directory `handy-wt/.target-005`. Scope of this run (orchestrator decision 2026-10-14): every non-UI step of the plan; the visual UI (prompt management, indicator menu, on-screen failure notice) waits for the reworked Penflow pass. Execution order of the plan: 1 → 2 → 3 → 4 → 7 → 5 → 6 → 8.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 — Native menu spike (macOS NSPanel) | Todo (UI, out of this run) | — | — | Not started: native indicator menu belongs to the UI pass | 2026-10-14 |
| 2 — Library validation and migration | Done | `prompts/{mod,types,library,library_tests,migration,view}.rs` (new), `commands/prompts.rs` (new), `settings.rs`, `settings_service/{errors,error_reason,platform_capability,public,protocol}.rs`, `shortcut/mod.rs` (old prompt commands moved out), `commands/mod.rs`, `lib.rs`; `tests/prompt_library.rs` (new), `tests/settings_transactions/ownership.rs`, `tests/settings_validation/error_details.rs`; checked 008 schema; `bindings.ts`; `settings-errors.ts` (+test); 26 locales | [lib prompts + settings](proofs/claude-impl/step2/test-lib-prompts-settings.stdout), [prompt_library](proofs/claude-impl/step2/test-prompt-library.stdout), [settings targets](proofs/claude-impl/step2/test-settings-targets.stdout), [clippy](proofs/claude-impl/step2/clippy.exit), [fmt](proofs/claude-impl/step2/cargo-fmt-check.exit), [lint](proofs/claude-impl/step2/bun-lint.exit), [format:check](proofs/claude-impl/step2/bun-format-check.exit), [tsc](proofs/claude-impl/step2/tsc.exit), [settings-errors](proofs/claude-impl/step2/settings-errors-test.exit), [translations](proofs/claude-impl/step2/check-translations.exit), [conventions](proofs/claude-impl/step2/conventions-repo-summary.json) | lib 127 passed/1 ignored (17 `prompts::`); prompt_library 4; settings cli 19, domains 8, live 10, platform 33, transactions 42/1 ignored, validation 25, writers 7, shortcuts_cli 8, destination_cli 6; all checks exit 0; conventions PASS | 2026-10-14 11:56 CEST |
| 3 — CLI domain and prompt shortcuts | Todo | | | | |
| 4 — Bounded processor and test operation | Todo | | | | |
| 7 — History metadata (migration 6) | Todo | | | | |
| 5 — Session snapshot and fallback events | Todo | | | | |
| 6 — Settings and indicator UI | Todo (UI, out of this run) | | | | |
| 8 — Acceptance and native evidence | Todo | | | | |

## Notes

- **Step 2, deviations from the plan text:**
  - The starter marker is the settings schema version (2 → 3), not a new `prompt_starters_migration_version` field: it is already the persisted migration marker of `apply_settings_migrations`, and a new stored field would have needed a reviewed descriptor row for an internal value. Fresh installs get the starters from the defaults with version 3.
  - UI commands commit through the existing `tauri_adapter::edit` (typed edit on the locked latest settings in the write order) instead of a new `tauri_adapter::write_change`: the UI commands carry no base revision, and the CLI domain (Step 3) runs the same pure `prompts::library` edits, so both surfaces refuse identically.
  - The prompt UI commands moved from `shortcut/mod.rs` to the new `commands/prompts.rs` (same command names, so the frontend calls are unchanged); `set_post_process_selected_prompt` now takes `Option<String>` and refuses a prompt with issues (`prompt_not_usable`).
  - The Apple Intelligence platform capability and the provider override edit (plan Step 3) are delivered here, because the override UI command needs them.
- **AC-001 precision to carry into the spec (orchestrator decision 4):** a prompt text made only of whitespace is refused (`prompt_text_empty`): the text must be non-empty after trimming, 1–8000 characters; the stored text stays verbatim.
