# Changelog - 016-smart-paragraphs

## 2026-10-29 — [Refactor]: Smart paragraphs row moved into the Text formatting section

- **Type:** Refactor (integration with 015)
- **Spec modified:** Yes (AC-010, FR-007, decision `text_formatting_row`)
- **Code modified:** `src/components/settings/text-formatting/text-formatting-options.ts` (new `smart_paragraphs_enabled` row with the `Pilcrow` icon), `src/components/settings/text-formatting/text-formatting-row.tsx` (platform-dependent description), `src/components/settings/advanced/AdvancedSettings.tsx` (provisional row removed); `SmartParagraphsToggle.tsx` and `smart-paragraphs-render.test.tsx` removed, their checks moved to `text-formatting-render.test.tsx`; `package.json` (`test:smart-paragraphs`)
- **AC impacted:** AC-010, AC-007 (UI description)
- **Author:** Claude Code (integration of 015, claude-opus-5-5)
- Row order set by Julien: « ␣ Smart spacing » (to come), « Aa Smart capitalization », « ¶ Smart paragraphs ». Pipeline order after the merge: replacements and vocabulary, AI, paragraphs (016), capitalization (015, insertion preparation), insertion.

## 2026-10-29 — [Feature]: Smart paragraphs, local and off by default

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md)
- **Code modified:** `src-tauri/src/smart_paragraphs/` (layout, structure, quotes, segmenter port, UAX #29 and Apple NaturalLanguage segmenters, tests), `src-tauri/src/dictation_output.rs` (+ tests), `src-tauri/src/actions.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/{descriptor_data,public}.rs`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/tests/settings_validation/{scalars,contracts}.rs`, 008 contract schema, `src/bindings.ts`, `src/stores/settingsStore.ts`, `src/components/settings/{SmartParagraphsToggle.tsx,smart-paragraphs-state.ts}` (+ tests), `src/components/settings/advanced/AdvancedSettings.tsx`, 26 locales, `package.json`, `README.md`
- **AC impacted:** AC-001 to AC-010 (automated evidence; native proof pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's request of 2026-10-29: readable long dictations with automatic paragraphs, without a generative model, under a clean-room contract (public sources S1 to S4 only). Coordinator's UI instruction the same day: a standalone « ¶ Paragraphes intelligents (i) » row with its own switch, near the output settings until 015's text-formatting section exists.
- Decisions in `spec.md` Clarifications; provenance table in `spec.md` Provenance.
