# Changelog - 005-custom-ai-prompts

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:e57cf592 -->

### 2026-10-12 — ### Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle metadata only (Planned); no behavioral change
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex (gpt-6)
- **Details:** Eight steps, two sequence diagrams, one state diagram and one logical ER; shared 008 transactions, 004 shortcut adapter, bounded AI processing, text-only history and native evidence gates. Penflow production deferred to common pre-spec-ship pass by user decision.

<!-- finalize:spec-plan:2026-10-12:ea4a26ff -->

### 2026-10-12 — Plan: Technical plan refined after independent review

- **Type:** Feature
- **Spec modified:** Lifecycle metadata only; no behavioral change
- **Code modified:** None
- **AC impacted:** None (pre-implementation)
- **Author:** Codex (gpt-6.1-sol)
- **Details:** Eight work packages; execution order 1-2-3-4-7-5-6-8. Clarified durable-history/cancellation race. Final native reviews use actual runtime model and explicit complete-context budget 180000. Visual authority deferred to common pre-spec-ship pass; no certification claimed.

<!-- finalize:spec-plan:2026-10-12:86a3d72b -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Data, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Spec Update]: AC-005 failure notice no longer claims the insertion succeeded

- **Type:** Spec Update
- **Spec modified:** Yes (sections: AC-005, Story 4 scenario, Clarifications; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** AC-005
- **Clarify decision:** `fallback-notice-insertion-wording`: on AI failure the indicator says for at least 4 seconds that the AI step failed and that the original text was used for insertion; it never states that the insertion succeeded, whose result belongs to 006, 007 and 010
- **Review:** the spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: align the plan on the merged 008, D1–D4 and the clarified AC-005

- **Type:** Spec Update
- **Spec modified:** No (plan entry; see the spec entry above)
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **D1:** Steps 1, 5 and 8 gated by macOS only; Windows/Linux native checks are pending release evidence with their exact command
- **008 names:** `Domain`/`DomainChange(ChangeKind::Apply)`, `SettingsBackend::commit_change`, `tauri_adapter::{ordered, commit_prepared_edit, write_change}`, `native_changes`, `sequence::PreparedChange`, `apply_settings_migrations`; never-shipped names removed
- **Shortcut removal:** through 008's native preparation (`bindings` preparation in `native_changes`, new `sequence::write_change_prepared` for routed CLI changes), released before the commit and re-registered on refusal
- **Actions:** the domain declares canonical `remove` and `update`; `delete`/`edit` keep working through `canonical_action`
- **D2:** `prompts list` is the `PromptsList` live operation (readiness and Apple Intelligence availability in the instance); offline refusal, library readable with `get post_process_prompts`
- **D3/D4:** 005 `ErrorReason` variants; Apple Intelligence override off macOS refused with `unsupported_platform` (`validation_failed`/2)
- **Dependencies:** 004 is not merged into `dev` (Step 3 and prompt-shortcut parts wait); 006 step 0 for D2/D3
- **QE:** plan QE aligned on the spec (High, Shared, Data then Security)
- **History/WAV:** text-only AI-failure row kept (spec AC-005); divergence with the 006 WAV-gated rule signalled in `.specs/ship.md`
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: make the plan review preparable, order Step 2, justify offline prompt list

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **Review budget:** 008's plan and implementation record are cited as inline paths, not links; the untracked program notes are no longer referenced, the D2/D3/D4 names are copied into the Summary and the history/WAV question with 006 is stated in the plan only. Plan review preparation at 180,000 characters: 1 batch, no error (was `paired_review_batch_budget_exceeded`)
- **Step 2 order:** prerequisite names `tauri_adapter::write_change` (added by Step 2 with the same signature if neither 001 nor 003 has delivered it); deletion commits without preparation because no prompt shortcut exists before Step 3; Step 3 adds the `bindings` preparation and `sequence::write_change_prepared` and switches UI deletion to `ordered` + `commit_prepared_edit`
- **Offline list:** `prompts list` offline returns 008's delivered live-only refusal (`unavailable`/5, scenario « Live state without running app » of the 008 spec) applied to a D2 operation; justified in the plan, spec unchanged
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)
