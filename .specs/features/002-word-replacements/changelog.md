# Changelog - 002-word-replacements

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
