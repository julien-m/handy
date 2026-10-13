# Changelog - 002-word-replacements

## 2026-10-13 — [Spec Update]: AC-006 joiners and CJK-side marks, AC-004 tab header, AC-005 and AC-007 precision

- **Type:** Spec Update (Clarify decisions recorded with `persist_clarification_decision`, `spec-refine` traceability rules)
- **Spec modified:** Yes (sections: AC-004, AC-005, AC-006 and AC-007 criteria; Story 4 Gherkin scenario "A zero-width non-joiner belongs to its word"; Edge Cases; Clarifications ledger)
- **Code modified:** None in this entry (see the following Bugfix entry)
- **AC impacted:** AC-004, AC-005, AC-006, AC-007 (meaning clarified; IDs preserved, no AC/FR/SC added or renumbered)
- **Decisions (orchestrator, 2026-10-13, independent re-audit `proofs/claude-fix1/independent-reaudit.md`):**
  - `002-word-replacements:AC-006` / `decision:ac-006:join-controls-are-word-characters` (N1): ZWNJ U+200C and ZWJ U+200D are word characters.
  - `002-word-replacements:AC-006` / `decision:ac-006:cjk-exemption-applies-to-base-characters` (N3): a CJK side ignores neighbouring base characters only; an adjacent combining mark still blocks.
  - `002-word-replacements:AC-006` / `decision:ac-006:emoji-variation-selector-accepted-false-negative` (N4): behavior unchanged, documented in Edge Cases.
  - `002-word-replacements:AC-005` / `decision:ac-005:one-megabyte-is-decimal-bytes`: 1 MB is 1,000,000 bytes.
  - `002-word-replacements:AC-004` / `decision:ac-004:header-recognized-with-either-separator`: the header is recognized with tab or `;`.
  - `002-word-replacements:AC-007` / `decision:ac-007:inserted-text-guarantee-before-ai-post-processing`: the guarantee covers the text handed to AI post-processing, not the AI rewrite.
- **Plan:** `plan.md` aligned (re-audit N2): boundary rule (Letter, Decimal_Number, Mn/Mc/Me, Join_Control; CJK side rejects adjacent marks), header with either separator, AC-007 guarantee scope, testing rows and risk dispositions. Plan refine ran although `implementation.md` exists, by orchestrator instruction, to remove a plan ↔ spec contradiction; no new scope.
- **Review:** these normative edits invalidate the spec and plan review receipts. Both independent reviews must be redone by another agent before any progression gate; no prior PASS is retained.
- **Author:** Claude Code (implementation agent, orchestrator decisions)

## 2026-10-13 — [Spec Update]: AC-006 word characters include combining marks

- **Type:** Spec Update (Clarify decision, `spec-refine` traceability rules)
- **Spec modified:** Yes (sections: AC-006 criterion, Story 4 Gherkin scenario "A combining mark belongs to its word", Edge Cases, Clarifications ledger, front matter `updated`)
- **Code modified:** None in this entry (see the following Bugfix entry)
- **AC impacted:** AC-006 (meaning clarified; IDs preserved, no AC/FR/SC added or renumbered); FR-005 unchanged (it already requires whole-word semantics defined by AC-006)
- **Decision:** Orchestrator decision 2026-10-13 on independent audit finding 1: for boundaries a word character is a Unicode letter, a decimal digit or a combining mark (Mn, Mc, Me), on both sides of a match, so a rule never replaces a word fragment in Indic scripts (dependent vowels, virama) or next to a decomposed (NFD) accent. The side-by-side CJK exemption is unchanged. Recorded with `persist_clarification_decision` (`002-word-replacements:AC-006` / `decision:ac-006:word-character-includes-combining-marks`).
- **Review:** The semantic spec review cache was already stale before this edit (`livespec validate --progression implement` returned `clarify_not_ready:stale_or_incomplete_spec_review`); this normative edit also invalidates it. Refresh the independent spec review before any progression gate; no prior review PASS is retained.
- **Author:** Claude Code (implementation agent, orchestrator decision)

## 2026-10-13 — [Bugfix]: Independent audit fixes (combining-mark boundaries, BOM, stop-path harness, scope)

- **Type:** Bugfix
- **Spec modified:** No (the AC-006 clarification is the previous entry)
- **Code modified:** `word_replacements/{unicode_generate.py,unicode-data.txt,unicode-sources.json,unicode.rs,matcher.rs,matcher_tests.rs}` (pinned Unicode 17 `M` table for Mn/Mc/Me, `is_word_character` boundary on both sides, 10 new fixtures); `word_replacements/{exchange.rs,exchange_tests.rs}` (BOM removed once, by the parser only); `word_replacements/dictation.rs`, `actions.rs`, new `tests/replacement_pipeline.rs` (stop-time rule selection and stream/batch fallback extracted and tested with injected ports); `lib.rs`, `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs`, `portable.rs`, `src/bindings.ts` (out-of-scope changes reverted to `dev`; bindings keep only the generated replacement types).
- **AC impacted:** AC-006 (combining marks, several-space rule, no match inside a folding expansion), AC-004/AC-005 (single BOM removal), AC-007 (capture/stop/fallback wiring now tested).
- **Dependencies:** No new crate. `unicode-general-category` is absent from `Cargo.lock`; the existing checksum-pinned Unicode 17 generator gained a targeted mark table instead, so letters, marks, whitespace and folding stay on one Unicode version.
- **Evidence:** raw outputs in `proofs/claude-fix1/`: library 348 PASS / 16 ignored, exchange 5 PASS, pipeline harness 7 PASS, red check of the new fixtures without the fix, strict Clippy on lib/bins/replacement tests PASS, fmt/lint/format/Ruff PASS, repository conventions PASS.
- **Author:** Claude Code (implementation agent)

## 2026-10-13 — [Feature]: Replacement matcher and transcription pipeline integration

- **Type:** Feature
- **Spec modified:** No.
- **Code modified:** `src-tauri/src/word_replacements/{matcher,dictation,matcher_tests}.rs`, `audio_toolkit/{text,mod}.rs` (approximate correction outside protected spans), `managers/transcription.rs` (replacement step after script conversion, `transcribe`/`finalize_stream` take the dictation snapshot), `actions.rs` (capture at recording start), `commands/history.rs` and `lib.rs` (retry/headless use current rules), `src-tauri/tests/replacement_performance.rs`.
- **AC impacted:** AC-006 (implemented, certification pending), AC-007 (partial: commit-to-next-dictation through UI/CLI Waiting for 008).
- **Evidence:** 40 table-driven matching fixtures (SC-001), segment/offset invariants, span-protection and pipeline-order tests; release benchmark max 1.78 ms on Apple M4 Pro (M1 pending release evidence); raw outputs in `proofs/claude/step4-*`.
- **Deviation:** The plan's port-injected `replacement_pipeline.rs` harness was not built: it would require restructuring `actions.rs` into injectable LLM/history/output ports. Pipeline order is tested at the `post_process_transcription_text` seam; history/output stage proof stays with native acceptance.
- **Author:** Claude Code (implementation agent, core-first run).

## 2026-10-13 — [Feature]: Exchange format and deterministic previews

- **Type:** Feature
- **Spec modified:** No.
- **Code modified:** `src-tauri/src/word_replacements/{exchange,preview,materialize}.rs` with `exchange_tests.rs`/`preview_tests.rs`, `validation.rs` (two issue kinds, one field, shorter field check), `src-tauri/tests/replacement_exchange.rs`.
- **AC impacted:** AC-003/004/005 (pure parsing, export, merge/replace preview and Apply gating); persistence, revision binding, editor and CLI remain Waiting for 008.
- **Evidence:** Library 331 PASS / 17 ignored, exchange file integration 5 PASS, Clippy/fmt/lint/format PASS, repo conventions receipt `20261013T144051Z` PASS; raw outputs in `proofs/claude/step2-*`.
- **Author:** Claude Code (implementation agent, core-first run).

## 2026-10-13 — [Feature]: Core domain model and pinned Unicode validation

- **Type:** Feature
- **Spec modified:** No semantic change; implementation remains partial.
- **Code modified:** Native replacement domain, additive settings field, pinned UUID dependency and generated Specta bindings; four mechanical Clippy corrections preserve existing behavior.
- **AC impacted:** AC-001/002/006 primitives only; complete persistence, UI, revisions and CLI remain Waiting for 008.
- **Evidence:** Target Rust 20 PASS; library regression 306 PASS / 0 failed / 17 ignored; explicit headless binding export 1 PASS; generator retry/staging/rollback boundary tests 6 PASS; Clippy, typecheck, lint, format, Ruff and diff checks PASS. Real Apple Intelligence Swift bridge used.
- **Conventions:** Repo PASS receipt `20261013T135522Z`, unchanged versioned baseline and zero new violations. Feature-scope inspection FAIL remains distinct for existing source/generated-binding violations.
- **Review:** Audit findings 10–14 repaired, including empty/duplicate opaque ID validation and pinned generator reliability. Independent core/final-delta reviews PASS; re-verified (full Rust suite 306 PASS / 17 ignored, Clippy, fmt, lint, format and repo conventions receipt `20261013T141410Z` PASS) and committed; raw outputs in `proofs/claude/step1a-*`.
- **Author:** Codex (`spec-implement`, core-first user authorization).

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:eba866f3 -->

### 2026-10-12 — Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle only (Planned via finalize); no semantic changes
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Scope:** 7 implementation steps; 2 sequence, 1 state, 1 conceptual ER diagram
- **Design:** UI contracts/mockups deferred before spec-ship by user decision
- **Author:** Codex (spec-plan --auto, gpt-6.1-sol)

<!-- finalize:spec-plan:2026-10-12:cbfc6dff -->
