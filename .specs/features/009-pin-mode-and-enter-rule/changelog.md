# Changelog - 009-pin-mode-and-enter-rule

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-006 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:04b5e2f9 -->

### 2026-10-12 — Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status Draft to Planned only via finalize; requirements unchanged)
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Scope:** 7 implementation steps; 2 sequence diagrams, 1 state diagram and 1 logical ER diagram
- **UI:** User-approved one-pass contract/mockups before spec-ship; current Penflow incomplete, no certification claimed
- **Author:** Codex (spec-plan --auto, gpt-6.1-sol)

<!-- finalize:spec-plan:2026-10-12:5e13c1dd -->

### 2026-10-12 — Plan: Independent review correction

- **Type:** Feature
- **Spec modified:** No (requirements unchanged)
- **Code modified:** None
- **AC impacted:** AC-005 and SC-001 planning evidence only
- **Correction:** Latch a qualifying read during chord injection; wait for actual successful chord completion; define reliable readiness t0=max(completion, receipt) and test both orders, failure, cancellation and duplicates
- **Review:** Initial independent receipt complete=true, ready=false; renewed independent review required
- **Author:** Codex (spec-plan --auto, gpt-6.1-sol)

<!-- finalize:spec-plan:2026-10-12:5e51aa59 -->

### 2026-10-12 — ### Plan Update: Align Enter integration with smart paste

- **Type:** Spec Update
- **Spec modified:** No (existing requirements and lifecycle preserved)
- **Code modified:** None (plan.md updated only)
- **AC impacted:** AC-005/AC-006 and SC-001 planning evidence only; IDs unchanged
- **Change:** Replace retired clipboard-window cancellation/recovery dependencies with all six current 007 outcomes; Enter only on terminal read observed/typed; preserve Windows reliable paste and its classic fallback; reuse 008 output smart_paste descriptor and commit_mutation; Copy abandons pending restoration through changeCount.
- **Scope:** Existing 7 steps, 2 sequence diagrams, 1 state diagram and 1 logical ER diagram preserved; no mockups or application tests produced.
- **Author:** Codex (spec-refine --auto / spec-plan, gpt-6.1-sol)

<!-- finalize:spec-plan:2026-10-12:6e7fa9b3 -->

### 2026-10-13 — ## Plan Update: Align Enter with the 2026-10-13 insertion decisions

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** AC-002, AC-005 (pre-implementation plan alignment)
- **Author:** Codex (GPT-6)
- Accept inserted, verified alongside read observed/typed; suppress uncertain without fallback or Enter and hand it to 010.
- Consume 007 AX-first outcomes without a clipboard/chord prerequisite for verified insertion; keep restoration read-required, 300 ms after the last read, capped at 8 s, and never restore on the 800 ms no-read outcome.
- Preserve 006 Remove destination as a current-only override while retaining the session pin; document tests at insertion freeze.
- Update outcome matrix, timing anchors, native/QE risks and source-backed review/progression; preserve 008 transaction/domain names and the deferred one-pass UI gate.

<!-- finalize:spec-plan:2026-10-13:183e40c6 -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Functional, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Spec Update]: Pinned badge rule, D2 live pin operations, retired auto_submit and single Enter veto

- **Type:** Spec Update
- **Spec modified:** Yes (sections: AC-001 badge visibility sentence added; Clarifications decision 2026-10-14; frontmatter `updated`)
- **Code modified:** None (plan.md updated)
- **AC impacted:** AC-001 (badge shown while an app is pinned even with 006 destination return off; IDs unchanged); AC-004, AC-005, AC-006 plan alignment
- **Author:** Claude Code (spec-refine 009 and 009 plan, claude-opus-5-5)
- Applies the 2026-10-14 independent review and orchestrator decisions D1–D4.
- Spec: the contradiction with 006 AC-003 is resolved in this feature: an active pin shows the badge whatever "Return to the dictation's application" is; without a pin 006 AC-003 applies unchanged; 006's spec is not modified.
- Plan: `destination pin|unpin|status` are D2 live operations (`DestinationDomain`, `PinOperation`, `UnpinOperation`, `StatusOperation` returning `DomainPlan::Live`, `managed_state`), never writing settings; precedence usage → `unsupported_platform` (`validation_failed`/2, D4) → no instance (`unavailable`/5, delivered live-only refusal) → `not_found`/4.
- Plan: retired `auto_submit` is `not_found`/4 (008 resolves keys before validation) with D3 `details.reason = "retired_key"` and `details.field = "automatic_enter"`; migration in `settings::apply_settings_migrations`, persisted once by `SettingsRepository::initialize`.
- Plan: the Enter veto is 006's single `RecoveryDisposition {Pending, Insert, Recover, Cancelled}` read at the atomic claim; `RecoverInstead` retired; identifiers `dictation_id`/`destination_generation` from 006/007; delivered 008 names only (`LiveSettingsEffects` removed).
- Plan: D1 Windows/Linux/older-binary native checks as pending release evidence; QE aligned with the spec (High, Shared, Functional); Next Action no longer pins a model.
- Review: the spec and plan reviews are stale until an independent reviewer refreshes them.

<!-- finalize:spec-refine:2026-10-14:0feee4a1 -->
