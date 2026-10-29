# Changelog — 017-smart-spacing

## 2026-10-29 — [Spec Update]: Final period inside a sentence (N30)

- **Type:** Spec Update
- **Spec modified:** Yes (new AC-017 and FR-006; AC-001 and AC-016 amended; Story 1 scenarios and flowchart; decision `n30_trailing_period`; provenance row; QE gates; edge cases)
- **Code modified:** `src-tauri/src/smart_spacing/{final_period.rs,final_period_tests.rs}` (new), `src-tauri/src/smart_spacing/{mod.rs,spacing_tests.rs}`, `src-tauri/src/smart_capitalization/preparation_tests.rs`, 26 locales (`smartSpacing.description`), `README.md`, `.specs/native-proofs-checklist.md`
- **AC impacted:** AC-001, AC-005, AC-016, AC-017
- **Author:** Claude Code (claude-opus-5-5), `/spec-fix` from Julien's requirement after native finding N30

Recognition engines such as Canary end every dictation with a period, which cut sentences when the dictation was inserted inside one ("Bonjour à tous. demain"). With smart spacing on and the text after the selection read, a single sentence period is now dropped when what follows starts with a continuation or ending mark or a lower-case letter; abbreviations, ellipses, `?`, `!`, `。`, a capital, a line break, the field end, an unknown side after, a non-text focus and 013's addressed terminal keep it. The existing test `punctuation_on_the_dictation_side` now covers both a capital (period kept) and a lower-case word (period dropped).

## 2026-10-29 — [Feature]: Smart spacing

- **Type:** Feature
- **Spec modified:** Yes (created: stories, AC-001 to AC-014, FR-001 to FR-005, clarifications, provenance, QE analysis)
- **Code modified:** `src-tauri/src/smart_spacing/*`, `src-tauri/src/smart_capitalization/{context.rs,preparation_tests.rs,mod.rs,app.rs,reader.rs,reader_tests.rs,macos.rs}`, `src-tauri/src/destination/{field.rs,field_context.rs,macos_field.rs,delivery.rs,delivery_tests.rs}`, `src-tauri/src/{settings.rs,lib.rs}`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,domains/output.rs}`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/tests/{smart_capitalization_context.rs,destination_field/fake.rs,settings_validation/*}`, `src/bindings.ts`, `src/stores/settingsStore.ts`, `src/components/settings/text-formatting/{text-formatting-options.ts,text-formatting-render.test.tsx}`, 26 locales, `README.md`, 008 contract
- **AC impacted:** AC-001 to AC-014
- **Author:** Claude Code (claude-opus-5-5)

The spaces at the two edges of a dictation follow the text before and after the replaced selection: one space between words, existing spaces and line breaks kept and never doubled, no space before closing punctuation nor after opening marks, no French space invented before `; : ! ?`, none in scripts written without spaces (Korean spaced). Unavailable context: one final space only; really empty field: no leading space; a focus proven not to be a text field: unchanged. While on, smart spacing decides the trailing space and the method's `append_trailing_space` is off for that insertion on every path (never two final spaces); `append_trailing_space` remains the end-of-field preference. Runs after 015's capitalization in the same preparation step and context read, extended to the field length and 16 units after the selection. Setting on for new installs, off on upgrade, first row of the "Text formatting" section.

## 2026-10-29 — [Bugfix]: Independent review corrections

- **Type:** Bugfix
- **Spec modified:** Yes (AC-002, AC-005, AC-006 wording; decisions `not_text_steps_aside`, `elision_apostrophe`, `attached_joiners`, `after_window_16`, `macos_only_reader`)
- **Code modified:** `smart_capitalization/reader.rs` (an answer shorter than the window before the caret is not trusted), `smart_spacing/mod.rs` (an empty text that does not reach the field start keeps the leading edge), `reader_tests.rs`, `spacing_tests.rs`, `src/i18n/locales/da/translation.json` (quoted option name), 008 `settings-cli.md` (14 output settings, `paste_verification` listed)
- **AC impacted:** AC-002, AC-005, AC-006, AC-009, AC-014
- **Author:** Claude Code (claude-opus-5-5), from the native independent review in `checks/2026-10-29-review.md`

## 2026-10-29 — [Bugfix]: Addressed terminal unspaced; fallback named outside macOS

- **Type:** Bugfix
- **Spec modified:** Yes (new AC-015 and AC-016; FR-004 and FR-005 references; Story 3 scenario; decisions `addressed_terminal_unspaced` (new), `not_text_steps_aside` and `macos_only_reader` updated; provenance; edge case)
- **Code modified:** `smart_capitalization/mod.rs` (`PreparedText::unspaced`, `for_addressed_terminal`), `destination/delivery.rs` (`deliver` takes the `PreparedText`; 013's `AddressedMethod` types the text before spacing), `smart_capitalization/preparation_tests.rs`, `destination/default_input/orchestrator_tests.rs`, `text-formatting-options.ts` (`smartSpacingDescriptionKey`), its render test, 26 locales (`smartSpacing.descriptionLimited`), `README.md`
- **AC impacted:** AC-005, AC-010, AC-015, AC-016
- **Author:** Claude Code (claude-opus-5-5), from the coordinator's decisions for Julien on the two open points of review round 2

A dictation typed by 013's addressing tool into a terminal (a background cmux workspace included) no longer receives the fallback final space: that route gets no spacing adjustment at all, as with the option off. On Windows and Linux the fallback is kept and the option's description now says it.
