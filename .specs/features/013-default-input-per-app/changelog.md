# Changelog - 013-default-input-per-app

## 2026-10-29 — [Feature]: Default input per application (accessibility field or cmux terminal)

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md; 008 CLI contract and checked schema extended)
- **Code modified:** `src-tauri/src/destination/default_input/` (model, fingerprint, tree, search, route, terminal, runner, services, capture, dispatch, instance, macOS adapters), `src-tauri/src/destination/{guard,insertion,delivery,manager,app,context,outcome,mod}.rs`, `src-tauri/src/automatic_enter/{policy,transaction}.rs`, `src-tauri/src/recovery/{types,decision}.rs`, `src-tauri/src/commands/{default_input,mod}.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/` (descriptor, public, protocol, errors, destination domain and CLI), `src-tauri/src/cli_destination.rs`, `src/overlay/` (menu entry, hook, model, bubble reasons), `src/components/settings/` (Remembered fields), `src/bindings.ts`, 26 locales, tests, `README.md`
- **AC impacted:** AC-001 to AC-010 (automated evidence; native proofs pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's request of 2026-10-29: when the destination's active element is not a text field, paste into a default field remembered per application, by type of application; native measurements of Codex (composer fingerprint) and cmux (CLI addressing) given with the request.
- Decisions in `spec.md` Clarifications; Penflow gap and native protocol in `implementation.md`.
