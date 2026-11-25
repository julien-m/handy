# Changelog — 024 Transcription engine selector

## 2026-11-25 — Bugfix: ChatGPT dictation and recognition settings

- **Type:** Bugfix
- **Spec modified:** Yes (AC-005, AC-011, AC-012, FR-005, FR-010)
- **Code modified:** Shortcut and live WAV replay routing, recognition settings card, indicator and toast notices
- **AC impacted:** AC-005, AC-007, AC-011, AC-012
- **Author:** Codex

## 2026-11-23 — Bugfix: ChatGPT selectable only when signed in

- **Type:** Bugfix + Spec Update (requested: selecting ChatGPT must first check the ChatGPT sign-in, otherwise the selection is not allowed)
- **Spec modified:** Yes (AC-010, FR-009)
- **Code modified:** src-tauri/src/transcription_engine/mod.rs (`is_selectable`), src-tauri/src/settings_service/{validation,errors}.rs (shared rule, `engine_sign_in_required`), src/stores/settings-errors.ts, src/bindings.ts, src/components/settings/transcription-engine/{api-engine-panel,engine-activation}.tsx, locales (`errors.settings.engineSignInRequired`), tests/transcription-engine.spec.ts, tests/harness/transcription-engine-fake.ts
- **AC impacted:** AC-010 added and implemented
- **Author:** claude-code (claude-opus-5-5)

## 2026-11-23 — Feature: Transcription engine setting with a ChatGPT live engine

- **Type:** Feature
- **Spec modified:** Yes (created spec.md, plan.md)
- **Code modified:** src-tauri/src/transcription_engine/** (new), managers/transcription.rs, actions.rs, lib.rs, settings.rs, settings_service/{descriptor_data,public}.rs, shortcut/mod.rs, commands/transcription_engine.rs, src/components/settings/transcription-engine/** (new), ModelsSettings.tsx, settingsStore.ts, bindings.ts, locales, tests/transcription-engine.spec.ts, tests/harness/transcription-engine*, package.json, Cargo.toml, AGENTS.md
- **AC impacted:** AC-001 … AC-007 implemented; AC-008 blocked (mockup tooling unavailable)
- **Author:** claude-code (claude-opus-5-5)
- **Review:** independent final review — CHANGES REQUIRED (unbounded connect/write, misplaced Linux cfg guard, AC-008 missing) → corrected except AC-008; re-review pending.

## 2026-11-23 — Bugfix + Spec Update: Local / API tabs, local model list locked while an API engine is active

- **Type:** Bugfix (reported: with ChatGPT selected, local models stayed selectable and showed "Active") + Spec Update (engine dropdown replaced by Local / API tabs at the user's request, room for Soniox and other APIs)
- **Spec modified:** Yes (US-1 rewritten, US-5, AC-002, AC-009, FR-002, FR-008, Screens)
- **Code modified:** src/components/settings/transcription-engine/** (tabs, panels, activation, sign-in hook; dropdown component removed), ModelsSettings.tsx, onboarding/ModelCard.tsx, model-card-render.test.tsx, tests/transcription-engine.spec.ts, tests/harness/transcription-engine.tsx, locales, package.json
- **AC impacted:** AC-002, AC-009
- **Author:** claude-code (claude-opus-5-5)

## 2026-11-23 — Bugfix: fallback lock on every model-switch path

- **Type:** Bugfix (review finding: auto-select after download, footer selector and tray could still switch the local model with ChatGPT active)
- **Spec modified:** No
- **Code modified:** src-tauri/src/transcription_engine/mod.rs, src-tauri/src/commands/models.rs, src/stores/modelStore.ts, src/components/model-selector/{ModelSelector,ModelDropdown}.tsx, model-dropdown-render.test.tsx, transcription-engine tabs (Home/End), api-engine-panel, use-chatgpt-sign-in
- **AC impacted:** AC-009
- **Author:** claude-code (claude-opus-5-5)

## 2026-11-23 — Bugfix: onboarding exempt from the fallback lock

- **Type:** Bugfix (re-review minor: an engine set to ChatGPT before onboarding would block its completion)
- **Spec modified:** Yes (Edge Cases: onboarding exemption, tray feedback follow-up)
- **Code modified:** src-tauri/src/commands/models.rs, src/stores/modelStore.ts
- **AC impacted:** AC-009
- **Author:** claude-code (claude-opus-5-5)
- **Review:** independent review of the tabs/lock correction — ACCEPT (2 minors: onboarding fixed here; tray feedback documented as follow-up).
