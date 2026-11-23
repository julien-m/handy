# Implementation: Transcription engine selector

Last verified: 2026-11-23 (branch `experiment/chatgpt-dictation`).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| FR-001 | `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/descriptor_data.rs`, `src-tauri/src/settings_service/public.rs`, `src-tauri/src/shortcut/mod.rs` | `@spec FR-001` | ✅ Implemented | 2026-11-23 |
| FR-002 | `src/components/settings/transcription-engine/transcription-engine-tabs.tsx`, `src/components/settings/transcription-engine/api-engine-panel.tsx`, `src/components/settings/transcription-engine/local-engine-panel.tsx`, `src/components/settings/transcription-engine/engine-activation.tsx`, `src/components/settings/transcription-engine/transcription-engine-options.ts`, `src/components/settings/transcription-engine/use-chatgpt-sign-in.ts`, `src/components/settings/models/ModelsSettings.tsx`, `src-tauri/src/commands/transcription_engine.rs`, `src-tauri/src/transcription_engine/chatgpt/auth.rs` | `@spec FR-002` | ✅ Implemented | 2026-11-23 |
| FR-003 | `src-tauri/src/transcription_engine/chatgpt/mod.rs`, `src-tauri/src/transcription_engine/chatgpt/protocol.rs`, `src-tauri/src/transcription_engine/chatgpt/session.rs`, `src-tauri/src/transcription_engine/chatgpt/transcript.rs`, `src-tauri/src/managers/transcription.rs`, `src-tauri/src/actions.rs`, `src-tauri/src/lib.rs` | `@spec FR-003` | ✅ Implemented | 2026-11-23 |
| FR-004 | `src-tauri/src/transcription_engine/mod.rs`, `src-tauri/src/managers/transcription.rs` | `@spec FR-006` (shared) | ✅ Implemented | 2026-11-23 |
| FR-005 | `src-tauri/src/managers/transcription.rs`, `src-tauri/src/transcription_engine/chatgpt/session.rs` | `@spec FR-003` (FR-005 in the same anchor) | ✅ Implemented | 2026-11-23 |
| FR-006 | `src-tauri/src/transcription_engine/mod.rs`, `src-tauri/src/transcription_engine/live.rs`, `src-tauri/src/transcription_engine/chatgpt/mod.rs` | `@spec FR-006` | ✅ Implemented | 2026-11-23 |
| FR-008 | `src-tauri/src/transcription_engine/mod.rs`, `src-tauri/src/commands/models.rs`, `src/stores/modelStore.ts`, `src/components/settings/models/ModelsSettings.tsx`, `src/components/onboarding/ModelCard.tsx`, `src/components/model-selector/ModelSelector.tsx`, `src/components/model-selector/ModelDropdown.tsx`, `src/components/settings/transcription-engine/local-engine-panel.tsx` | `@spec FR-008` | ✅ Implemented | 2026-11-23 |
| FR-007 | `.specs/design/screens/024-transcription-engine-selector/` | — | ❌ Missing (tooling blocked) | 2026-11-23 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| AC-001 | `src-tauri/src/settings.rs` (test `transcription_engine_defaults_to_local_and_reads_chatgpt`); CLI `handy settings set transcription_engine remote` → exit 2 | ✅ |
| AC-002 | `tests/transcription-engine.spec.ts`, `src/components/settings/transcription-engine/transcription-engine-options.test.ts`; screenshots in `.specs/features/024-transcription-engine/baselines/` | ✅ (native window not observed by the agent) |
| AC-003 | CLI replay with chatgpt (session 441 ms, final 1166 ms); user's live dictations (751/803 ms) | ✅ |
| AC-004 | CLI replay with local; `src-tauri/src/transcription_engine/mod.rs` test `local_engine_opens_no_live_session` | ✅ |
| AC-005 | CLI replay with empty `CODEX_HOME` → local text; `src-tauri/src/transcription_engine/chatgpt/session.rs` tests (silent server, closed port); `src-tauri/src/transcription_engine/chatgpt/auth.rs` redaction test | ✅ |
| AC-006 | Module layout under `src-tauri/src/transcription_engine/` | ✅ |
| AC-007 | CLI `--live` path in `src-tauri/src/lib.rs` | ✅ |
| AC-009 | `src-tauri/src/transcription_engine/mod.rs` test `local_model_switch_is_refused_while_an_api_engine_is_active`; `src/components/onboarding/model-card-render.test.tsx`; `src/components/model-selector/model-dropdown-render.test.tsx`; Playwright fallback test. The `ModelsSettings` wiring has no dedicated render test; store and backend guards enforce the rule. | ✅ |
| AC-008 | Claude Design mockups | ❌ Blocked (automation tools unavailable) |

## Checks (2026-11-23)

- `cargo clippy --all-targets`: no warning; `cargo fmt --check`: OK.
- `cargo test --lib -- transcription_engine settings live_engine`: 198 passed, 0 failed.
- `bun run test:transcription-engine`: 3 pass; `tsc --noEmit`: OK; eslint (touched sources): 0 problems; `check:translations`: complete; prettier: OK.
- Playwright `tests/transcription-engine.spec.ts`: 3 passed with system Google Chrome (temporary config, `channel: "chrome"`) because the project's headless Chromium download stalled.

## Files Created/Modified

```
M AGENTS.md
 M package.json
 M src-tauri/Cargo.lock
 M src-tauri/Cargo.toml
 M src-tauri/src/actions.rs
 M src-tauri/src/commands/mod.rs
 M src-tauri/src/lib.rs
 M src-tauri/src/managers/transcription.rs
 M src-tauri/src/settings.rs
 M src-tauri/src/settings_service/descriptor_data.rs
 M src-tauri/src/settings_service/public.rs
 M src-tauri/src/shortcut/mod.rs
 M src/bindings.ts
 M src/components/settings/models/ModelsSettings.tsx
 M src/i18n/locales/ar/translation.json
 M src/i18n/locales/bg/translation.json
 M src/i18n/locales/ca/translation.json
 M src/i18n/locales/cs/translation.json
 M src/i18n/locales/da/translation.json
 M src/i18n/locales/de/translation.json
 M src/i18n/locales/en/translation.json
 M src/i18n/locales/es/translation.json
 M src/i18n/locales/fr/translation.json
 M src/i18n/locales/he/translation.json
 M src/i18n/locales/hi/translation.json
 M src/i18n/locales/id/translation.json
 M src/i18n/locales/it/translation.json
 M src/i18n/locales/ja/translation.json
 M src/i18n/locales/ko/translation.json
 M src/i18n/locales/ne/translation.json
 M src/i18n/locales/nl/translation.json
 M src/i18n/locales/pl/translation.json
 M src/i18n/locales/pt/translation.json
 M src/i18n/locales/ru/translation.json
 M src/i18n/locales/sv/translation.json
 M src/i18n/locales/tr/translation.json
 M src/i18n/locales/uk/translation.json
 M src/i18n/locales/vi/translation.json
 M src/i18n/locales/zh-TW/translation.json
 M src/i18n/locales/zh/translation.json
 M src/stores/settingsStore.ts
?? .specs/features/024-transcription-engine/
?? src-tauri/src/commands/transcription_engine.rs
?? src-tauri/src/transcription_engine/
?? src/components/settings/transcription-engine/
?? tests/harness/transcription-engine-fake.ts
?? tests/harness/transcription-engine.html
?? tests/harness/transcription-engine.tsx
?? tests/transcription-engine.spec.ts
```

Removed: `src-tauri/src/chatgpt_dictation.rs` (experiment, replaced by `transcription_engine/chatgpt/`).
