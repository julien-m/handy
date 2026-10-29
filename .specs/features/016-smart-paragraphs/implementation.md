---
feature: smart-paragraphs
title: Smart Paragraphs Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-29
---

# Implementation — 016 Smart Paragraphs

**Branch:** `feat/smart-paragraphs` (from `dev` `00f2ec6c`) · **Last verified:** 2026-10-29 · **Final run:** [proofs/final/summary.txt](proofs/final/summary.txt)

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-001: Paragraphs at sentence boundaries](spec.md#fr-001) | `src-tauri/src/smart_paragraphs/mod.rs`, `src-tauri/src/smart_paragraphs/layout.rs` | `@spec FR-001: Paragraphs at sentence boundaries`, `@spec AC-001: 45 words at a sentence boundary` | ✅ Implemented | 2026-10-29 |
| [FR-002: Blank line replaces only the gap](spec.md#fr-002) | `src-tauri/src/smart_paragraphs/layout.rs` | `@spec FR-002: Blank line replaces only the gap` | ✅ Implemented | 2026-10-29 |
| [FR-003: Keep structure, never split a quotation](spec.md#fr-003) | `src-tauri/src/smart_paragraphs/mod.rs`, `src-tauri/src/smart_paragraphs/structure.rs`, `src-tauri/src/smart_paragraphs/quotes.rs`, `src-tauri/src/smart_paragraphs/layout.rs` | `@spec FR-003: Keep structure`, `@spec AC-005: Lists, code and line breaks kept`, `@spec AC-006: Not in a quotation` | ✅ Implemented | 2026-10-29 |
| [FR-004: NaturalLanguage on macOS, UAX #29 elsewhere](spec.md#fr-004) | `src-tauri/src/smart_paragraphs/segmenter.rs`, `src-tauri/src/smart_paragraphs/apple_segmenter.rs`, `src-tauri/src/smart_paragraphs/unicode_segmenter.rs` | `@spec FR-004: NaturalLanguage on macOS, UAX #29 elsewhere`, `@spec AC-007: CJK unchanged outside macOS` | ✅ Implemented (both segmenters tested on macOS; Windows/Linux builds not run here) | 2026-10-29 |
| [FR-005: Step of the dictation output stages](spec.md#fr-005) | `src-tauri/src/smart_paragraphs/mod.rs`, `src-tauri/src/dictation_output.rs`, `src-tauri/src/actions.rs` | `@spec FR-005: Step of the dictation output stages`, `@spec AC-008: After AI and history, before insertion` | ✅ Implemented | 2026-10-29 |
| [FR-006: Setting through the settings service](spec.md#fr-006) | `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/descriptor_data.rs`, `src-tauri/src/settings_service/public.rs`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs` | `@spec FR-006: Off by default, settings service`, `@spec FR-006: Same validation as the CLI` | ✅ Implemented | 2026-10-29 |
| [FR-007: Standalone translated row](spec.md#fr-007) | `src/components/settings/SmartParagraphsToggle.tsx`, `src/components/settings/smart-paragraphs-state.ts`, `src/components/settings/advanced/AdvancedSettings.tsx`, `src/i18n/locales/en/translation.json` (and the 25 other locales) | `@spec AC-010: Standalone row with icon, info, switch, FR-007: Translated row`, `@spec AC-007: Limitation shown outside macOS` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| AC-001 | `src-tauri/src/smart_paragraphs/layout_tests.rs`, `src-tauri/src/smart_paragraphs/arrange_tests.rs` — tests `below_the_word_limit_nothing_breaks`, `exactly_the_word_limit_breaks_after_that_sentence`, `the_break_falls_at_the_next_sentence_boundary_past_the_limit`, `a_french_dictation_breaks_at_the_word_limit` | ✅ |
| AC-002 | `src-tauri/src/smart_paragraphs/layout_tests.rs` — tests `four_substantial_sentences_close_a_paragraph`, `short_interjections_add_words_but_do_not_count_as_sentences`, `a_three_word_sentence_is_not_substantial_but_a_four_word_one_is`, `several_paragraphs_each_close_at_their_own_limit` | ✅ |
| AC-003 | `src-tauri/src/smart_paragraphs/layout_tests.rs`, `src-tauri/src/smart_paragraphs/arrange_tests.rs`, `src-tauri/src/smart_paragraphs/apple_segmenter_tests.rs` — tests `a_break_is_one_blank_line_and_every_other_character_is_kept`, `spacing_inside_a_paragraph_is_kept`, `japanese_sentences_are_joined_without_any_added_space`, `japanese_paragraphs_add_no_space`, `a_french_dictation_keeps_every_character` | ✅ |
| AC-004 | `src-tauri/src/smart_paragraphs/layout_tests.rs`, `src-tauri/src/smart_paragraphs/arrange_tests.rs` — tests `a_lone_long_sentence_stays_whole`, `a_short_dictation_is_unchanged`, `the_last_sentence_never_opens_an_empty_paragraph`, `empty_and_blank_texts_are_unchanged` | ✅ |
| AC-005 | `src-tauri/src/smart_paragraphs/structure_tests.rs`, `src-tauri/src/smart_paragraphs/arrange_tests.rs` — test `a_structured_text_is_never_handed_to_the_rules` | ✅ |
| AC-006 | `src-tauri/src/smart_paragraphs/quotes_tests.rs`, `src-tauri/src/smart_paragraphs/layout_tests.rs` — tests `a_quotation_is_never_split`, `french_guillemets_are_quotations_too`, `an_unclosed_quotation_keeps_the_rest_in_one_paragraph` | ✅ |
| AC-007 | `src-tauri/src/smart_paragraphs/apple_segmenter_tests.rs`, `src-tauri/src/smart_paragraphs/unicode_segmenter_tests.rs`, `src-tauri/src/smart_paragraphs/arrange_tests.rs`, `src/components/settings/smart-paragraphs-state.test.ts` | ✅ automated (Windows/Linux runtime not exercised) |
| AC-008 | `src-tauri/src/dictation_output_tests.rs` — tests `smart_paragraphs_arrange_the_ai_text_after_history_and_before_insertion`, `with_smart_paragraphs_off_insertion_receives_the_final_text_unchanged` | ✅ |
| AC-009 | `src-tauri/src/settings.rs`, plus the cargo integration targets settings_live and settings_validation (scalars, contracts) — fresh-install default, old store default, UI/CLI parity accepted and refused, checked contract | ✅ |
| AC-010 | `src/components/settings/smart-paragraphs-render.test.tsx` (and `bun run check:translations`) | ✅ automated (visual check in the app pending) |

## Files Created/Modified

- `src-tauri/src/smart_paragraphs/mod.rs`, `src-tauri/src/smart_paragraphs/layout.rs`, `src-tauri/src/smart_paragraphs/structure.rs`, `src-tauri/src/smart_paragraphs/quotes.rs`, `src-tauri/src/smart_paragraphs/segmenter.rs`, `src-tauri/src/smart_paragraphs/unicode_segmenter.rs`, `src-tauri/src/smart_paragraphs/apple_segmenter.rs` (new): arrange and preservation reasons, limits from S2 and break plan, structure, quotation balance, segmenter port, UAX #29 with the CJK rule, NaturalLanguage through `objc2` with UTF-16 to byte offsets and a UAX #29 fallback; tests beside each file and `src-tauri/src/smart_paragraphs/test_text.rs`.
- `src-tauri/src/dictation_output.rs` — port `arrange_paragraphs` between history and insertion; `src-tauri/src/dictation_output_tests.rs` — order and history tests.
- `src-tauri/src/actions.rs` — the real port reads `smart_paragraphs_enabled` per dictation.
- `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/descriptor_data.rs`, `src-tauri/src/settings_service/public.rs`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs` — setting, descriptor (`general`, boolean), public field, UI command.
- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` — direct dependency `unicode-segmentation` 1.12 (already locked as a transitive dependency).
- `src-tauri/tests/settings_live.rs`, `src-tauri/tests/settings_validation/scalars.rs`, `src-tauri/tests/settings_validation/inventory.rs`, `src-tauri/tests/settings_validation/contracts.rs`, `src-tauri/tests/settings_validation/request_contracts.rs`, `.specs/features/008-settings-cli/contracts/settings-cli.schema.json` — new key in the inventories and the checked contract.
- `src/bindings.ts` (regenerated), `src/stores/settingsStore.ts`, `src/components/settings/SmartParagraphsToggle.tsx`, `src/components/settings/smart-paragraphs-state.ts`, `src/components/settings/smart-paragraphs-state.test.ts`, `src/components/settings/smart-paragraphs-render.test.tsx`, `src/components/settings/advanced/AdvancedSettings.tsx`, the 26 `translation.json` files, `package.json` (`test:smart-paragraphs`).
- `README.md` — "Smart paragraphs" section.

## Evidence levels

- **Transformation:** Rust unit tests over UAX #29 (every platform) and Apple NaturalLanguage (run on this Mac).
- **Compilation:** macOS only (`cargo clippy --all-targets`, `cargo test` with `destination-return,smart-paste-dispatch`). Windows and Linux builds of the `cfg(not(target_os = "macos"))` path were not compiled here; that path is the same `UnicodeSegmenter` tested on macOS.
- **Real use:** not exercised (no dictation run in the app). Protocol below.

## Native proof protocol (Julien)

1. Build and launch the branch (`bun run tauri dev`), Settings > Advanced > Output: the row « ¶ Paragraphes intelligents (i) » is present, off; the (i) shows the description; on Windows/Linux it ends with the Chinese/Japanese limitation.
2. `handy settings get smart_paragraphs_enabled` → `false`. Turn the row on; `get` → `true`. `handy settings set smart_paragraphs_enabled maybe` → refused with the same error as an invalid boolean from the UI.
3. In TextEdit, dictate about 120 words in 8 French sentences: the text arrives in 2 or 3 paragraphs separated by one blank line, no sentence cut, no word changed. History shows the text without blank lines.
4. Dictate "Oui. D'accord." only: unchanged. Dictate one 60-word sentence: unchanged.
5. Dictate a text with « à la ligne » (or use an AI prompt returning a bulleted list): unchanged layout.
6. With an AI prompt on, dictate a long text: the AI result is arranged; with 015 merged, capitalization applies after.
7. Japanese or Chinese dictation on macOS (long): paragraphs, no added space. Same on Windows/Linux: unchanged.
8. Turn the row off: the next long dictation has no blank lines.

## Known limits

- NaturalLanguage and UAX #29 may disagree on abbreviations ("Mr. Smith"); a wrong boundary can only move a break.
- Chinese and Japanese are left unchanged outside macOS (documented, shown in the UI).
- A text with any line break gets no automatic paragraphs (decision `line_breaks_preserved`).
- No claim of parity with Spokenly's implementation: only the public behaviour of S2 is reproduced.
- Penflow forward contract and independent review gates of `/spec-feature` were not run in this autonomous pass; the row is provisional until 015's shared section is merged.
