# Changelog - 010-recovery-bubble

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:f1da6064 -->

### 2026-10-12 — Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle metadata only (Planned/updated through finalize); normative body unchanged
- **Code modified:** None (plan.md and contracts/recovery-contract.md created)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex (gpt-6)
- **Summary:** 7 implementation steps, 2 sequence diagrams, 1 state diagram, 1 logical ER diagram; shared 008 framework, early native spike, explicit QE gates and deferred shared UI preparation.

<!-- finalize:spec-plan:2026-10-12:70754b4a -->

### 2026-10-12 — Plan Update: Align recovery with smart paste

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/recovery-contract.md refined)
- **AC impacted:** AC-001/AC-005 integration clarified (pre-implementation; requirement IDs unchanged)
- **Author:** Codex (gpt-6; spec-refine plan --auto via spec-plan protocol)
- **Summary:** Replace retired clipboard-window dependency/cancellation with the six smart paste outcomes, direct Copy/changeCount guard, 008 output smart_paste integration and preserved Windows reliable paste. Existing native/UI/history decisions unchanged; independent spec review refreshed for current source before plan refinement.

<!-- finalize:spec-plan:2026-10-12:1cac4499 -->

### 2026-10-13 — Plan Update: recovery outcome and read-confirmed restoration handoff

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan and local recovery contract updated)
- **AC impacted:** AC-001, AC-005 integration; numbering unchanged
- **Author:** Codex (gpt-6)
- Eight terminal smart-paste outcomes: uncertain always recovers without retry/Enter; direct AX verified can reach 009 policy.
- 007 owns last-read quiet restoration with 8 s cap; no read never restores; deliberate Copy changes changeCount and defeats eligible restoration.
- Preserve 008 service/output domain, 006 reason handoff, saved-only history explanation, early native spike, Rust/native evidence and deferred shared UI pass.

<!-- finalize:spec-plan:2026-10-13:dd714906 -->

### 2026-10-13 — Plan Update: current upstream contract and Analyze references

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan and local contract only)
- **AC impacted:** AC-001, AC-005, AC-006; IDs unchanged
- **Author:** Codex (gpt-6)
- Align readback window (150 ms) and total direct-AX budget (250 ms) with the current 007 source, consume its destination-PID guard, and omit the removed direct-first option.
- Write AC-006 explicitly in Step 7 platform acceptance; native Windows/Linux reduced actions remain mandatory.
- Reprepare independent review after concurrent dependency changes; do not reuse stale readiness.

<!-- finalize:spec-plan:2026-10-13:09d4dd98 -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Data, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: one Enter veto with 009, method outcome from 006, late-read update, D1

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/recovery-contract.md updated)
- **AC impacted:** None (pre-implementation)
- **D1:** Steps 1 and 7 gated by macOS only; Windows/Linux native spikes and matrices are pending release evidence with their exact command; AC-006 platform policy proven by automated tests. Supersedes the 2026-10-13 entry that made Windows/Linux native proof mandatory
- **Method outcome:** no separate 007 adapter; 010 reads `InsertionOutcome.method_outcome` (007 `PasteOutcome.kind` wire values) with 006's current field names (`dictation_id`, `destination_generation`, `transaction_id`)
- **Late read:** 007 `PasteDiagnostic` updates the existing correlated bubble ("may have been pasted after all"), no second bubble, no outcome or Enter change
- **Enter veto:** single contract shared with 009: 006's `RecoveryDisposition { Pending, Insert, Recover, Cancelled }`; 010 answers `RecoveryPort::before_insertion` with `PreInsertionVerdict`, never calls `set_disposition`, publishes no other veto
- **008 names:** descriptor row, `OUTPUT_KEYS`, `validate_cli`/`validate_ui`, `tauri_adapter::write`; never-shipped names removed
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)
