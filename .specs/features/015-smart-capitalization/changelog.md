# Changelog — 015-smart-capitalization

## 2026-10-29 — [Feature]: Smart capitalization

- **Type:** Feature
- **Spec modified:** Yes (created: stories, AC-001 to AC-011, FR-001 to FR-005, clarifications, provenance, QE analysis)
- **Code modified:** `src-tauri/src/smart_capitalization/*`, `src-tauri/src/destination/{field.rs,field_context.rs,macos_field.rs,mod.rs,delivery.rs}`, `src-tauri/src/{dictation_output.rs,dictation_output_tests.rs,actions.rs,settings.rs,lib.rs}`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,domains/output.rs}`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/tests/{smart_capitalization_context.rs,destination_field/fake.rs,settings_validation/*}`, `src/bindings.ts`, `src/stores/settingsStore.ts`, `src/components/settings/text-formatting/*`, `src/components/ui/{SettingContainer.tsx,ToggleSwitch.tsx}`, `src/components/settings/advanced/AdvancedSettings.tsx`, 26 locales, `package.json`, `README.md`, 008 contract
- **AC impacted:** AC-001 to AC-011
- **Author:** Claude Code (claude-opus-5-5)

The first letter of a dictation follows the text before the cursor (capital at a field, line, sentence or list start; lower case in continuation; unchanged when the context is absent or undecidable), with protections for acronyms, inner capitals, "I", 002 replacements, 003 custom words, a repeated capitalized word and AI-decided text. Context read on macOS through `AXStringForRange` (64 UTF-16 units, 50 ms, never a password field, never logged), from 011's remembered caret or the frontmost editable field. Setting on for new installs, off on upgrade. New "Text formatting" section of the Advanced page.

## 2026-10-29 — [Bugfix]: Independent review corrections

- **Type:** Bugfix
- **Spec modified:** Yes (decisions `title_case_limits`, `protection_limits`, `marker_needs_line_start`)
- **Code modified:** `decision.rs` (multi-byte line breaks never split, marker needs its line start), `decision_tests.rs`, `destination/field_context.rs` (read outside the manager lock; put back only while it still serves the selection), `tests/smart_capitalization_context.rs`, `SettingContainer.tsx` (inline layout unchanged), 008 `settings-cli.md` (12 output settings)
- **AC impacted:** AC-002, AC-004, AC-006, AC-007, AC-008
- **Author:** Claude Code (claude-opus-5-5), from the native independent review in `checks/2026-10-29-review.md`
