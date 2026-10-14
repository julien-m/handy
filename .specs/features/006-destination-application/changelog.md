# Changelog - 006-destination-application

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:2ddb4fdd -->

### 2026-10-12 — ### Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle metadata Draft to Planned only; normative requirements unchanged)
- **Code modified:** None (plan.md and local destination contract created)
- **AC impacted:** None (pre-implementation; AC-001 through AC-008 planned)
- **Plan:** 7 implementation steps; 2 sequence, 1 state and 1 logical entity diagram
- **Decisions:** 008 shared output domain/service; async activation and guarded dispatch; conditional history; 010 reason delegation
- **UI:** One-pass contracts/mockups deferred before spec-ship by user instruction; no visual certification
- **Author:** Codex (gpt-6.1-sol, spec-plan --auto)

<!-- finalize:spec-plan:2026-10-12:688a7be7 -->

### 2026-10-12 — Plan Update: Align destination integration with smart paste

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (targeted plan.md and local destination contract refinement)
- **AC impacted:** AC-006 integration; AC-005 recovery ownership retained (pre-implementation)
- **Changes:** Retired 007 clipboard-window handoff replaced by its six terminal smart-paste results; async result waiting; output smart_paste through the 008 service; Windows reliable paste retained.
- **Consumers:** 009 Enter eligible only on read observed/typed; 010 handles destination and method failures; Copy abandons pending restoration via changeCount.
- **Unchanged guarantees:** Conditional WAV/history retention, 010 reason delegation ≥4 s, activation/UI boundaries, planned UI identities.
- **Author:** Codex (gpt-6.1-sol, spec-refine plan with spec-plan protocol)

<!-- finalize:spec-plan:2026-10-12:134acf63 -->

### 2026-10-13 — Plan Update: One-click destination removal and current smart-paste handoff

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and local destination contract updated)
- **AC impacted:** AC-004, AC-006 (pre-implementation plan alignment)
- **Author:** Codex (gpt-6)
- Badge cross reuses the current-session FrontmostAtEnd mutation; translated Remove destination label, isolated pointer click, no persistent setting change and insertion-entry race rejection.
- Preserve all eight method outcomes, including inserted, verified and uncertain; destination still reports insertion attempted, 009 evaluates only eligible outcomes, 010 receives uncertainty/failure.
- Delegate the AX-first chain and last-read quiet/no-read/changeCount rules to 007; add routing, no-duplicate and no-006-clipboard-effect tests.
- Retain asynchronous activation, conditional WAV/history conservation, native proof gates, exact 008 transaction/domain framework and deferred one-pass UI contract.
- Refresh independent spec/plan review and Analyze/progression; no implementation, commit or push.

<!-- finalize:spec-plan:2026-10-13:53c4754c -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Functional, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)
