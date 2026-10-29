---
feature: smart-spacing
title: Smart Spacing Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-29
---

# Implementation — 017-smart-spacing

Status: In Progress (transformation and compilation evidence complete; macOS real-use proof pending with Julien, protocol below). Branch `feat/smart-spacing` from `dev` `07b298ad`. The setting, the pure steps and the extended reader exist in every build and every rollout-gate state; the reader is macOS only (Windows and Linux: every context Unavailable, so only the fallback final space applies).

Evidence kinds, kept apart: **transformation** (pure unit tests of the junction and edges, every platform), **compilation** (clippy, `tsc`, ESLint, both gate states for the settings contract), **real use** (the native protocol below, not run). No parity with Spokenly is claimed.

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Pure junction rule](spec.md#fr-001) | `src-tauri/src/smart_spacing/junction.rs` | `@spec FR-001: Pure junction rule, AC-003: Junction rule` | ✅ Implemented | 2026-10-29 |
| [FR-002: Edges fitted to the context](spec.md#fr-002) | `src-tauri/src/smart_spacing/mod.rs` | `@spec FR-002: Edges fitted to the context`, `@spec AC-002: Leading edge, AC-007: Empty field`, `@spec AC-005: Trailing edge`, `@spec AC-006: Unavailable context fallback` | ✅ Implemented | 2026-10-29 |
| [FR-003: Reader extended after the selection](spec.md#fr-003) | `src-tauri/src/smart_capitalization/{reader.rs,macos.rs,context.rs}`, `src-tauri/src/destination/{field.rs,macos_field.rs,field_context.rs}` | `@spec FR-003: Reader extended after the selection, AC-009: Bounded read after the selection`, `@spec AC-010: Not a text field`, `@spec AC-009: Bounded read after the selection` (macOS `character_count`) | ✅ Implemented (native pending) | 2026-10-29 |
| [FR-004: Same preparation step and read](spec.md#fr-004) | `src-tauri/src/smart_capitalization/{mod.rs,app.rs}`, `src-tauri/src/destination/delivery.rs` | `@spec AC-009: After capitalization, same context`, `@spec FR-004: Same preparation step and read`, `@spec AC-005: One trailing space, never two` | ✅ Implemented | 2026-10-29 |
| [FR-005: Setting, UI row, CLI, i18n](spec.md#fr-005) | `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,domains/output.rs}`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs`, `src/bindings.ts`, `src/stores/settingsStore.ts`, `src/components/settings/text-formatting/text-formatting-options.ts`, `src/i18n/locales/*/translation.json` (26) | `@spec FR-005: Settings UI write of smart spacing`, `@spec AC-013: New install on, upgrade off`, `@spec AC-014: First row of Text formatting` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| AC-001 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`only_edge_whitespace_changes`, `empty_or_blank_text_is_unchanged`), `src-tauri/src/smart_capitalization/preparation_tests.rs` (`spacing_off_never_decides_the_trailing_space`), `src-tauri/src/smart_capitalization/reader_tests.rs` (`spacing_off_reads_nothing_after`) | ✅ |
| AC-002 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`middle_of_a_field_between_two_spaces_adds_nothing`, `spaces_of_the_dictation_never_pile_up`, `nothing_read_before_that_misses_the_field_start_keeps_the_leading_edge`), `src-tauri/src/smart_spacing/junction_tests.rs` (`whitespace_already_present_is_respected`) | ✅ |
| AC-003 | `src-tauri/src/smart_spacing/junction_tests.rs` (words, closing and opening punctuation, quotes, elision, hyphen and slash, decimals) | ✅ |
| AC-004 | `src-tauri/src/smart_spacing/junction_tests.rs` (`french_high_punctuation_gets_no_invented_space`), `src-tauri/src/smart_spacing/spacing_tests.rs` (`french_typography_is_never_invented_nor_removed`) | ✅ |
| AC-005 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`end_of_a_field_follows_append_trailing_space`, `the_trailing_space_is_decided_once`, `selection_replacement_uses_both_sides_of_the_selection`), `src-tauri/src/destination/delivery_tests.rs` (`a_trailing_space_decided_by_smart_spacing_turns_the_method_space_off`) | ✅ |
| AC-006 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`unavailable_context_secures_one_final_space_only`, `partial_context_fits_the_known_side_and_secures_the_end`), `src-tauri/src/smart_capitalization/reader_tests.rs` (`untrusted_is_unavailable`, `unsupported_port_is_unavailable`, `budget_spent_is_unavailable`, `foreign_process_is_unavailable`, `unknown_length_leaves_the_side_after_absent`, `a_short_answer_before_the_caret_is_unavailable`) | ✅ |
| AC-007 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`empty_field_gets_no_leading_space`, `absent_and_empty_differ`) | ✅ |
| AC-008 | `src-tauri/src/smart_spacing/junction_tests.rs` (`scripts_without_spaces_join`, `trailing_space_suits_spaced_scripts_only`), `src-tauri/src/smart_spacing/spacing_tests.rs` (`cjk_and_thai_get_no_spaces`, `korean_is_spaced`) | ✅ |
| AC-009 | `src-tauri/src/smart_capitalization/reader_tests.rs` (`reads_after_the_selection_end`, `reads_at_most_after_window_units`, `caret_at_the_end_is_the_field_end_without_reading_after`, `spent_budget_after_the_before_read_is_partial`, `a_short_answer_after_the_selection_is_kept_from_its_start`, `windows_after_are_bounded_by_the_field`), `src-tauri/tests/smart_capitalization_context.rs` (`remembered_selection_is_read_on_both_sides_for_smart_spacing`), `src-tauri/src/smart_capitalization/preparation_tests.rs` (`capitalization_decides_before_the_space_is_added`), `src-tauri/src/smart_capitalization/context.rs` tests (`debug_never_prints_the_field_text`) | ✅ (native log inspection pending) |
| AC-010 | `src-tauri/src/smart_capitalization/reader_tests.rs` (`non_editable_is_not_text`, `secure_field_is_never_read`), `src-tauri/src/smart_spacing/spacing_tests.rs` (`not_a_text_field_changes_nothing_and_leaves_the_method_space`), `src-tauri/src/smart_capitalization/preparation_tests.rs` (`not_a_text_field_is_left_to_the_method`) | ✅ (native cmux pending) |
| AC-011 | `src-tauri/src/smart_spacing/spacing_tests.rs` (`reference_example_bonjour_demain`), `src-tauri/src/smart_capitalization/preparation_tests.rs` (`reference_example_bonjour_demain_with_both_steps`) | ✅ |
| AC-012 | `src-tauri/src/smart_capitalization/preparation_tests.rs` (`reference_example_je_souhaite_with_both_steps`) | ✅ |
| AC-013 | `src-tauri/src/settings.rs` tests (`fresh_install_enables_smart_spacing`, `upgrade_without_smart_spacing_key_disables_it`, `stored_smart_spacing_is_kept`) | ✅ |
| AC-014 | `src/components/settings/text-formatting/text-formatting-render.test.tsx` (order and smart spacing row), `src-tauri/tests/settings_validation/*` (74 keys, 56 scalars, 008 schema), `bun run check:translations` | ✅ |

## Files Created/Modified

- Created: `src-tauri/src/smart_spacing/{mod.rs,junction.rs,junction_tests.rs,spacing_tests.rs}`, `src-tauri/src/smart_capitalization/{context.rs,preparation_tests.rs}`, this feature's `spec.md`, `plan.md`, `implementation.md`, `changelog.md`, `progress.md`, `pipeline.md`, `checks/2026-10-29-review.md`.
- Modified (015 extended): `smart_capitalization/{mod.rs,app.rs,reader.rs,reader_tests.rs,macos.rs}` (tests moved from `mod.rs` to `preparation_tests.rs`), `destination/{field.rs,macos_field.rs,field_context.rs}` (`with_remembered_input` replaces `read_remembered_before_caret`; `FieldNode::character_count`), `tests/smart_capitalization_context.rs`, `tests/destination_field/fake.rs`.
- Modified (insertion): `destination/delivery.rs` (`method_settings`, `deliver` takes the decided flag, `paste_as_before`), `destination/delivery_tests.rs`.
- Modified (setting): `settings.rs`, `settings_service/{descriptor_data.rs,public.rs,domains/output.rs}`, `shortcut/mod.rs`, `lib.rs` (module and command), `src/bindings.ts` (regenerated), `src/stores/settingsStore.ts`, settings validation tests (74 keys, 56 scalars), 008 contract (`settings-cli.md`, `settings-cli.schema.json`).
- Modified (UI, docs): `text-formatting-options.ts` (first row), its render test, 26 locales, `README.md` (section "Smart spacing", CLI line), 015 spec back-references, 015 implementation and changelog, registry, roadmap, global changelog.

## Pipeline position (AC-009)

Recognition → script conversion → replacements (002) → custom-word correction (003) → fillers → normalization → AI post-processing (005) → history (unchanged) → smart paragraphs (016) → **final preparation (`insert_final_text` → `smart_capitalization::app::prepare`): one context read, then smart capitalization (015), then smart spacing (017)** → `method_settings` (trailing space off when 017 decided it) → 006 session / smart paste / legacy paste; the recovery slot retains exactly what the method would insert.

## Articulation with `append_trailing_space`

| Smart spacing | Context | Trailing space decided by | Result at the end of the text |
| --- | --- | --- | --- |
| Off | any | method (`append_trailing_space`) | as before 017 |
| On | Not a text field | method (`append_trailing_space`) | as before 017 |
| On | Field, side after read, text follows | smart spacing (junction rule) | one space or none; method adds nothing |
| On | Field, side after read, end of the field | smart spacing (`append_trailing_space` as the preference) | one space if the option is on and the script is spaced; method adds nothing |
| On | Unavailable, or side after unknown | smart spacing (fallback) | one space unless already present or unspaced; method adds nothing |

## Gaps

- Penflow: `penflow/ui.pen` and the `output-settings` mockup are not regenerated for the new first row; recorded, not claimed.
- Windows and Linux readers are not built (inherited decision `macos_only_reader`): there, the option only adds the fallback final space; a new installation therefore gets one trailing space per dictation (flagged to Julien as a product choice).
- The goal-runner ceremony of `/spec-feature` (goal render/prove/archive, nested phase agents, LiveSpec spec-review receipts, Penflow forward chain) was not run in this autonomous pass, as for 015 and 016; `pipeline.md` records Specify only. Evidence: the command output in `progress.md` and the independent native review in `checks/2026-10-29-review.md`.

## Native proof protocol (macOS, pending Julien)

Build with both gates (`bun run tauri dev --features destination-return,smart-paste-dispatch` or the release build), Accessibility granted, Advanced > Text formatting > "␣ Smart spacing" on (and "Aa Smart capitalization" on unless stated), Debug log level. For each line, place the caret (or the selection) as described, dictate the phrase, and compare the field. `|` is the caret, `[…]` a selection. Record the result and attach the log excerpt: it must show only `Smart spacing: <context kind>, text changed: <bool>, trailing space decided: <bool>` (and 015's line), never field text or the transcript.

| # | App and field | Field before (caret) | Dictate | Append trailing space | Expected field after |
| --- | --- | --- | --- | --- | --- |
| 1 | TextEdit, new document | `|` (empty) | "Bonjour" | off | `Bonjour` (no leading space; log `Field`) |
| 2 | TextEdit, new document | `|` (empty) | "Bonjour" | on | `Bonjour ` (one space, never two) |
| 3 | TextEdit | `Bonjour|demain` | "à tous" | off | `Bonjour à tous demain` |
| 4 | TextEdit | `Je souhaite|` | "Vérifier ce point." | off, then on | `Je souhaite vérifier ce point.` (then the same plus one space) |
| 5 | TextEdit | `Bonjour [tout le monde] demain` | "à tous" | off | `Bonjour à tous demain` |
| 6 | TextEdit | `Bonjour |, ça va` | "à tous" | off | `Bonjour à tous, ça va` |
| 7 | TextEdit | `(|)` | "voir plus haut" | off | `(voir plus haut)` (no space inside; first letter's case as recognized, 015 keeps it after "(") |
| 8 | TextEdit | `Est-ce que| ?` | "tu viens" | off | `Est-ce que tu viens ?` (the typed space before "?" kept) |
| 9 | TextEdit | `今日は|です` (Japanese input) | "晴れ" | on | `今日は晴れです` (no space) |
| 10 | Chrome, a page `<textarea>` | `Hello|world` | "big" | off | `Hello big world` (log `Field`); see the note below |
| 11 | Chrome, a password field | anything | any phrase | on | as with the option off (log `NotText`; nothing read) |
| 12 | Codex (desktop app composer) | `We will|` | "meet tomorrow" | off | ` meet tomorrow` appended after "will" if Codex answers the read (log `Field`); otherwise one final space only (log `Unavailable`); record which |
| 13 | cmux, in front, at a shell prompt | `$ |` | "ls" | off, then on | **unchanged** compared with the option off (log `NotText`): `ls`, then `ls ` with the trailing option on |
| 14 | Any app, smart spacing off | `Bonjour|demain` | "à tous" | on | `Bonjourà tous demain` (byte-for-byte as before 017; no read after the selection in the log) |
| 15 | Accessibility permission removed | any field | "Hello" | off | `Hello ` (log `Unavailable`: one final space, no leading space) |
| 16 | Destination return: start in TextEdit after `Bonjour|demain`, switch to Safari before stopping | — | "à tous" | off | inserted in TextEdit as `Bonjour à tous demain` (011's remembered selection read on both sides) |
| 16b | 013 terminal route: a cmux workspace remembered as the default input, dictation ends while TextEdit is in front | — | "ls" | off | typed in cmux as `ls ` (log `Unavailable`: the fallback space; accepted consequence in the spec) |
| 17 | Fresh install vs upgrade | — | — | — | a new settings store has "␣ Smart spacing" on, first row of Text formatting; an existing store without the key shows it off |

Note on row 10: when Chrome answers both reads the log shows `Field` with an `After`; when it does not answer `AXNumberOfCharacters` the log shows `Field` with `after: Absent` and the fallback final space still gives `Hello big world`. Only a fully `Unavailable` context (Chrome refusing `AXStringForRange`) gives `Hellobig world` (no leading space is deduced). Record the context kind.
