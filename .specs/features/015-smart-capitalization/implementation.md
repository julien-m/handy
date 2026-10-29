---
feature: smart-capitalization
title: Smart Capitalization Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-29
---

# Implementation — 015-smart-capitalization

Status: In Progress (automated evidence complete; macOS native proof pending with Julien, protocol below). Branch `feat/smart-capitalization` from `dev` `00f2ec6c`. The setting, the pure steps and the reader exist in every build and every rollout-gate state; the reader is macOS only (Windows and Linux: Absent, no change).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Pure context decision](spec.md#fr-001) | `src-tauri/src/smart_capitalization/decision.rs` | `@spec AC-002: Context decision table` | ✅ Implemented | 2026-10-29 |
| [FR-002: Pure first-letter transformation](spec.md#fr-002) | `src-tauri/src/smart_capitalization/transform.rs` | `@spec AC-001: Only the first letter changes, AC-003: Protections, AC-004: Unicode mappings`, `@spec AC-003: First word protections` | ✅ Implemented | 2026-10-29 |
| [FR-003: Bounded macOS context read](spec.md#fr-003) | `src-tauri/src/smart_capitalization/{reader.rs,macos.rs}`, `src-tauri/src/destination/{field.rs,macos_field.rs,field_context.rs}` | `@spec AC-005: Absent unless an editable field answers, AC-006: Bounded read`, `@spec AC-007: Context of 011's remembered caret` | ✅ Implemented (native pending) | 2026-10-29 |
| [FR-004: Final preparation on every insertion path](spec.md#fr-004) | `src-tauri/src/smart_capitalization/{mod.rs,app.rs}`, `src-tauri/src/destination/delivery.rs`, `src-tauri/src/dictation_output.rs`, `src-tauri/src/actions.rs` | `@spec FR-004: Final preparation on every insertion path`, `@spec AC-009: Preparation order and smart spacing slot`, `@spec AC-003: AI-decided case is kept`, `@spec AC-007: Context source` | ✅ Implemented | 2026-10-29 |
| [FR-005: Setting, UI section, CLI, i18n](spec.md#fr-005) | `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,domains/output.rs}`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs`, `src/bindings.ts`, `src/stores/settingsStore.ts`, `src/components/settings/text-formatting/*`, `src/components/ui/{SettingContainer.tsx,ToggleSwitch.tsx}`, `src/components/settings/advanced/AdvancedSettings.tsx`, `src/i18n/locales/*/translation.json` (26) | `@spec FR-005: Settings UI write of smart capitalization`, `@spec AC-008: Text formatting switch row`, `@spec AC-010: New install on, upgrade off` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| AC-001 | `src-tauri/src/smart_capitalization/transform_tests.rs` (`only_first_letter_changes`, every case asserts the tail), `src-tauri/src/smart_capitalization/mod.rs` (`absent_context_changes_nothing`) | ✅ |
| AC-002 | `src-tauri/src/smart_capitalization/decision_tests.rs` | ✅ |
| AC-003 | `src-tauri/src/smart_capitalization/transform_tests.rs` (acronym, inner capital, I, replacement, custom word, repeated proper noun, AI-decided), `src-tauri/src/dictation_output_tests.rs` (AI flag), `src-tauri/src/smart_capitalization/app.rs` tests (`protected_spellings_hold_replacements_and_custom_words`) | ✅ |
| AC-004 | `src-tauri/src/smart_capitalization/transform_tests.rs` (ß, ﬁ, ǆ/ǉ, İ, ΐ, combining marks, CJK, leading punctuation) | ✅ |
| AC-005 | `src-tauri/src/smart_capitalization/decision_tests.rs` (`absent_and_empty_differ`, `empty_context_capitalizes`), `src-tauri/src/smart_capitalization/reader_tests.rs` (untrusted, secure, non-editable, foreign process, budget), `src-tauri/src/smart_capitalization/app.rs` tests (`foreign_destination_is_absent`, Handy or unreadable foreground) | ✅ (native pending) |
| AC-006 | `src-tauri/src/smart_capitalization/reader_tests.rs` (`reads_at_most_window_units`, `budget_spent_is_absent`, `secure_field_is_never_read`, `oversized_answer_is_absent`), `src-tauri/src/smart_capitalization/decision_tests.rs` (`context_debug_never_prints_the_text`) | ✅ (native log inspection pending) |
| AC-007 | `src-tauri/tests/smart_capitalization_context.rs` (`remembered_caret_is_used`, `no_remembered_input_reads_nothing`, `insertion_target_names_the_destination_or_stale`, `a_destination_change_during_the_read_releases_the_input`), `src-tauri/src/smart_capitalization/reader_tests.rs` (`selection_start_is_used`) | ✅ |
| AC-008 | `src/components/settings/text-formatting/text-formatting-render.test.tsx`, `src-tauri/tests/settings_validation/*`, `src-tauri/tests/settings_domains.rs`, `bun run check:translations` | ✅ |
| AC-009 | `src-tauri/src/dictation_output_tests.rs`, `src-tauri/src/smart_capitalization/mod.rs` tests; wiring reviewed in `.specs/features/015-smart-capitalization/checks/2026-10-29-review.md` | ✅ |
| AC-010 | `src-tauri/src/settings.rs` tests (`fresh_install_enables_smart_capitalization`, `upgrade_without_key_disables_it`, `stored_value_is_kept`) | ✅ |
| AC-011 | `src-tauri/src/smart_capitalization/transform_tests.rs` (`reference_example_je_souhaite`) | ✅ |

## Files Created/Modified

- Created: `src-tauri/src/smart_capitalization/{mod.rs,decision.rs,decision_tests.rs,transform.rs,transform_tests.rs,reader.rs,reader_tests.rs,macos.rs,app.rs}`, `src-tauri/src/destination/field_context.rs`, `src-tauri/tests/smart_capitalization_context.rs`, `src/components/settings/text-formatting/{text-formatting-options.ts,text-formatting-row.tsx,text-formatting-section.tsx,text-formatting-render.test.tsx}`.
- Modified: `OutputPorts::insert` takes `case_decided` (`dictation_output.rs`, its tests, `actions.rs`); `insert_final_text` prepares the text before every insertion path (`destination/delivery.rs`); `FieldNode::text_in` (default `Unsupported`, macOS `AXStringForRange`) and `cf_range` shared (`destination/{field.rs,macos_field.rs,mod.rs}`); setting `smart_capitalization` (field, default on for a fresh install, migration off for an upgrade, descriptor, public projection, output domain keys, UI command, bindings, store updater); optional decorative `icon` in `SettingContainer`/`ToggleSwitch` (tooltip layout only); the Text formatting section on the Advanced page; 26 locales; `README.md` CLI example; `package.json` script `test:text-formatting`; 008 contract (`settings-cli.md`, `settings-cli.schema.json`) and the settings validation integration tests (71 keys, 54 scalars); the 011 test fake answers `text_in` only for text a test scripts.

## Pipeline position (AC-009)

Recognition → script conversion → replacements (002) → custom-word correction (003) → fillers → normalization → AI post-processing (005) → history (native and AI texts, unchanged) → **final preparation (`insert_final_text`): smart capitalization, then the reserved smart spacing slot (`smart_capitalization::prepare_text`)** → 006 session / smart paste / legacy paste (trailing space appended by the method as before).

## Gaps

- Penflow: `penflow/ui.pen` and the `output-settings` mockup are not regenerated for the new section (one switch row); recorded, not claimed.
- Windows and Linux readers are not built (Absent by design, spec decision `macos_only_reader`): pending release evidence is only that the option has no effect there.
- The goal-runner ceremony of `/spec-feature` (goal render/prove/archive, nested phase agents, Penflow forward chain) was not run in this autonomous pass; the evidence is the command output listed in `progress.md` and the independent review in `checks/`. `pipeline.md` records Specify Done only: `livespec pipeline update --phase plan` answers `BLOCKED: clarify_not_ready:missing_current_spec_review`, because no LiveSpec spec-review receipt (livespec-verifier) was produced; the later phases stay Pending rather than being forced.

## Native proof protocol (macOS, pending Julien)

Build with both gates (`bun run tauri dev --features destination-return,smart-paste-dispatch` or the release build), Accessibility granted, Advanced > Text formatting > "Smart capitalization" on, Debug log level. For each line, place the caret as described, dictate the phrase, and compare the inserted text. Record the result and attach the log excerpt (it must show only `Smart capitalization: <Decision>, first letter changed: <bool>`, never field text or the transcript).

| # | App and field | Text before the caret | Dictate | Expected insertion |
| --- | --- | --- | --- | --- |
| 1 | TextEdit, new document | (empty) | "hello there" (if the engine capitalizes, check it stays "Hello there") | `Hello there` |
| 2 | TextEdit | `I bought apples, ` | "Pears and plums" | `pears and plums` |
| 3 | TextEdit | `It works. ` | "now test it" | `Now test it` |
| 4 | TextEdit | `Groceries:` then Return then `- ` | "buy milk" | `Buy milk` |
| 5 | TextEdit | `Je souhaite ` | "Vérifier ce point." | `vérifier ce point.` |
| 6 | TextEdit | `I just bought an ` | "iPhone" / "API keys" | unchanged |
| 7 | Chrome, a page `<textarea>` | `Note: ` | "Tomorrow at noon" | `tomorrow at noon` |
| 8 | Chrome, a password field | anything | any phrase | unchanged, and no read: the log shows `Absent` |
| 9 | Codex (desktop app composer) | `We will meet ` | "Tomorrow" | `tomorrow` if Codex answers `AXStringForRange`; otherwise unchanged with `Absent` in the log (both acceptable; record which) |
| 10 | cmux (terminal) | a shell prompt | "Hello there" | **unchanged** (`Absent`: not editable by 010's rule) |
| 11 | Any app, option off | `I bought apples, ` | "Pears" | `Pears` (byte-for-byte as before, no accessibility query) |
| 12 | Destination return: start in TextEdit after `I bought apples, `, switch to Safari before stopping | — | "Pears" | inserted in TextEdit as `pears` (011's remembered caret) |
| 13 | Fresh install vs upgrade | — | — | a new settings store has the switch on; an existing store without the key shows it off |
