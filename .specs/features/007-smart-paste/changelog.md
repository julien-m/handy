# Changelog - 007-clipboard-recovery-window

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:aa9ea82f -->

### 2026-10-12 — ## Plan: Clipboard recovery technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle metadata only: Planned; functional content unchanged)
- **Code modified:** None (plan.md and planning evidence only)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex (gpt-6, spec-plan --auto)
- **Artifacts:** Inspect [the plan](plan.md) for 7 implementation steps, 3 diagrams, shared 008 output-domain integration and explicit native/UI evidence gates.
- **Design:** Approved lightweight UI decision retained; no mockup generated, no Penflow certification claimed.

<!-- finalize:spec-plan:2026-10-12:1f19f822 -->

## 2026-10-12 — [Spec Update]: Scope replaced by smart paste

- **Type:** Spec Update
- **Spec modified:** Yes (scope replaced — all sections; previous spec and plan moved to `superseded/`)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 retired (not reused); AC-008 through AC-016 created
- **Decision:** Julien, 2026-10-12, relayed by Codex: implement Spokenly-inspired smart paste instead of the 10–15 s clipboard recovery window; keep number 7
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-refine)

<!-- finalize:spec-refine:2026-10-12:3471b770 -->

### 2026-10-12 — ### Plan: Smart paste technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status only: Draft to Planned via finalize; normative requirements unchanged)
- **Code modified:** None (plan-only run)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol
- **Plan:** 9 implementation steps; 2 sequence diagrams, 1 state diagram, 1 logical ER diagram.
- **Decisions:** asynchronous AppKit/key worker; bounded read-only snapshot helper; guarded restoration and copy mode; 008 output-domain reuse and one-shot migration; six truthful outcomes.
- **Evidence boundary:** UI/Penflow deferred by user until shared pre-spec-ship pass; native/runtime proof required later.

<!-- finalize:spec-plan:2026-10-12:307d8e77 -->

### 2026-10-13 — Plan: Smart paste technical plan fully rewritten from 2026-10-13 spec

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** None (plan-only)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol, spec-plan --auto
- **Plan:** Read [the rewritten plan](plan.md) for 9 implementation steps, 2 sequence diagrams, 1 state diagram and 1 transient logical ER diagram.
- **Decisions:** AX-first verified insertion; restoration only after post-key read plus quiet/cap; not-read snapshot discarded; Jump 3 s decision; bounded native calls/snapshots; shared 008 transaction/output domain and one-time migration.
- **Evidence boundary:** Independent semantic review and progression recorded separately; native/AppKit/AX late-write/compatibility and shared Penflow UI gates remain mandatory before release. No mockups generated.

<!-- finalize:spec-plan:2026-10-13:17322c07 -->
