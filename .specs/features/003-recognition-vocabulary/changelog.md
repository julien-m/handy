# Changelog - 003-recognition-vocabulary

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:d5c7ff88 -->

### 2026-10-12 — [Plan]: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle metadata only (Draft to Planned via finalize); normative requirements unchanged
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Plan:** Six implementation steps, two sequence diagrams and one state diagram
- **Review:** Independent grounded plan review and progression; actual verdict retained in current .reviews/plan.json and validation receipts
- **Decisions:** Shared 008 service/domain framework; Unicode and legacy preservation; capability/request snapshot; documented native budget spike and fixed-audio proof
- **UI:** Planned flow/screen IDs retained; shared contract/mockup production deferred before spec-ship by user decision
- **Author:** Codex (spec-plan --auto, model gpt-6)

<!-- finalize:spec-plan:2026-10-12:32171392 -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality Medium, blast radius Shared, primary risk UX, taken from the existing QE Analysis; `livespec delivery lane`: light once the spec review is current
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: align the plan on the merged 008 and the program decisions D1–D4

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **QE:** plan `## QE Analysis` aligned on the spec (Medium, Shared, UX then Regression); delivery lane stays light
- **008 names:** `Domain`/`DomainPlan`/`DomainChange(ChangeKind::Apply)`, `SettingsBackend::commit_change`, `sequence::write_change`, `input::read_source`, `View::Export`; new `tauri_adapter::write_change` for the UI write with its base revision; `commit_mutation` and `LiveSettingsEffects` (never shipped) removed
- **D2:** `vocabulary status` is a live operation of 006 step 0 (`DomainPlan::Live`, `VocabularyStatus`, `managed_state::<Arc<TranscriptionManager>>`); offline it returns the 008 live-only refusal (`unavailable`/5); Step 4 depends on 006 step 0
- **D3:** per-line refusals in `details.items` (`ErrorItem`, ≤100) with new `ErrorReason` variants `term_empty`, `term_too_long`, `term_line_break`, `term_forbidden_character`, `term_duplicate`
- **D4:** `context_budget_unverified` becomes `internal_error`/1 with `details.reason`, never exit 5 (not transient, not fixable by the user)
- **Export:** stdout only (`vocabulary export -`), a file by shell redirection, because the merged 008 framework has no file output
- **Budget:** Whisper 223 tokens sourced from the engine rule `n_text_ctx / 2 − 1` (OpenAI Whisper decoder, whisper.cpp); other mechanisms measured and documented in Step 1 (`contracts/context-budget.md`), no application ceiling
- **D1:** Step 6 requires macOS native proof only; Windows/Linux observations are pending release evidence
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: make the plan review preparable, rename the durable-error rule, justify offline status

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **Review budget:** 008's plan and implementation record are cited as inline paths, not links; the untracked program notes are no longer referenced and the D2 names are copied into the CLI section; `Inspect [Whisper model documentation](…)` so the external link is context, not a normative contract. Plan review preparation at 180,000 characters: 1 batch, no error (was `paired_review_batch_budget_exceeded` and `external_contract_not_loaded`)
- **Durable-error rule:** the plan's "Rule D4" (never `unavailable`/5 for `context_budget_unverified`) is renamed, since program D4 is the unsupported-platform refusal; behavior unchanged
- **Offline status:** `vocabulary status` offline returns 008's delivered live-only refusal (`unavailable`/5, scenario « Live state without running app » of the 008 spec) applied to a D2 operation; justified in the plan, spec unchanged
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Bugfix]: UI hardening of the vocabulary editor (Apply focus, Retry of a failed Apply)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/components/settings/CustomWords.tsx, src/components/settings/vocabulary/{vocabulary-paste.tsx, vocabulary-paste-model.ts, vocabulary-terms.tsx, vocabulary-retry.ts (new)}, src/stores/vocabulary-store.ts, tests (vocabulary-retry.test.ts new, vocabulary-model, vocabulary-notices, vocabulary-store), package.json (`test:vocabulary`)
- **AC impacted:** AC-001 (paste box: no empty merge sent, Retry follows the Apply rule); behavioral AC is_keyboard_navigable (focus kept on Apply, stable focus after Retry)
- **Evidence:** [proofs/claude-hardening/summary.txt](proofs/claude-hardening/summary.txt)
- **Author:** Claude Code (UI hardening pass)


## 2026-10-28 — [Bugfix]: Lost keyboard focus goes to the section title (native session N5)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/components/settings/CustomWords.tsx
- **AC impacted:** none (keyboard focus)
- **Why:** native session of 2026-10-28, finding N5 (seen in 002, same pattern checked here).

## 2026-10-28 — [Bugfix]: vocabulary limit raised to 5,000 terms, indexed approximate correction (native session N7)

- **Type:** Bugfix
- **Spec modified:** No — spec-refine 003 à faire : limite 5 000 termes (décision Julien 2026-10-28)
- **Code modified:** src-tauri/src/recognition_vocabulary/{validation.rs, edits.rs, edits_tests.rs, preview.rs, occurrences.rs}, src-tauri/src/audio_toolkit/{text.rs, custom_word_index.rs (new), custom_word_index_tests.rs (new), custom_words_perf_tests.rs (new)}, src-tauri/src/cli.rs, src-tauri/Cargo.toml (`strsim`, `natural` moved to dev-dependencies), src-tauri/tests/vocabulary_transactions.rs, src/components/settings/vocabulary/{vocabulary-model.ts, vocabulary-terms.tsx, vocabulary-paste-model.ts, vocabulary-limit.test.ts (new)}, package.json (`test:vocabulary`), src/i18n/locales/*/translation.json (26 locales), README.md
- **AC impacted:** AC-001 (limit 5,000 instead of 500, pending spec-refine), FR-004 (approximate correction about 21 ms per 96-word dictation with 5,000 terms, optimized test profile; 468 ms before)
- **Summary:** finding N7 of Julien's native proof session of 2026-10-28: Julien's decision to hold 5,000 terms, applied to the service validation, UI count, CLI, messages and README; Whisper still sends only what fits its prompt budget
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-28 — [Bugfix]: review of the native session fixes (N7)

- **Type:** Bugfix
- **Spec modified:** No (spec-refine 003 à faire : limite 5 000 termes, décision Julien 2026-10-28)
- **Code modified:** src/components/settings/vocabulary/{vocabulary-status-model.ts, vocabulary-status.tsx, vocabulary-model.ts, vocabulary-terms.tsx, vocabulary-status-render.test.tsx (new)}, src/i18n/locales/*/translation.json (`status.notSent.more`), src-tauri/src/audio_toolkit/{text.rs, custom_word_index.rs, custom_word_index_tests.rs, custom_words_perf_tests.rs}, package.json (`test:vocabulary`)
- **AC impacted:** AC-001 (status of a full vocabulary: 20 terms named per group, the rest counted; Latin digits in every locale), FR-004 (threshold overflow fixed; one index around 002 replacements, under 50 ms)
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-29 — [Bugfix]: a sound-alike no longer turns an ordinary phrase into a vocabulary term (native finding N22)

- **Type:** Bugfix
- **Spec modified:** Yes (sections: Story 3 Gherkin and flowchart, AC-008 and FR-006 added, Clarifications decision of 2026-10-29, Edge Cases, frontmatter `updated`)
- **Code modified:** src-tauri/src/audio_toolkit/{custom_word_index.rs, custom_word_index_tests.rs, text.rs}
- **AC impacted:** AC-008 (new), FR-004 (approximate correction fallback, unchanged routing)
- **Gaps closed:** N22 — « je ne le vois pas » became « je LiveSpec » (Soundex weight 0.3 on a raw distance of 5/9)
- **Remaining:** native dictation with Parakeet, with Julien
- **Author:** Claude Code (spec-fix, claude-opus-5-5)

