# Changelog - 002-word-replacements

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
