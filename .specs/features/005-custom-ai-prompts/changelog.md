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

## 2026-10-14 — [Plan Update]: Text-only history entry for every dictation whose WAV fails

- **Type:** Spec Update
- **Spec modified:** No (checked: no 005 requirement ties history to the WAV)
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-005 unchanged)
- **Author:** Claude Code (spec-refine 005 plan, claude-opus-5-5)
- Julien's decision 2026-10-14 settles the open question "history without WAV": every noncancelled dictation that produced a transcript, not only an AI failure, gets a text-only row when its WAV fails, whatever the cause. The open question is removed from the history decision and the risk table.
- Step 7 removes the `wav_saved` gate for every such dictation; its evidence and success criteria add an ordinary dictation saved text-only on WAV failure. The history change shared with 006 (nullable `file_name` rebuild, gate removal, guards for rows without audio) is built once by whichever of 005 Step 7 and 006 Step 6 runs first; 005's prompt metadata columns stay in Step 7.
- Constitution check: the text-only row beyond AI failures is required by 006 AC-005, so 005 shares it rather than adding scope.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:d7cad1e7 -->

## 2026-10-14 — [Feature]: Indicator prompt pill, menu and failure notice (UI pass)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src/overlay/* (prompt pill and menu, notices), src/hooks/use-overlay-none-notices.ts, App.tsx, 26 locales
- **AC impacted:** AC-003, AC-005 (indicator part); browser-tested, native proof pending
- **Author:** Claude Code (UI pass, feature/ui-overlay 69a6ee73)

## 2026-10-14 — [Bugfix]: Prompt settings UI, delivery-review fix round 1

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** `src/components/settings/post-processing/*` (new `prompt-editor-sync.ts`, `prompt-provider-fix.tsx`, `prompt-library-notice.tsx`, `use-focus-on-mount.ts`), `shortcut-list.tsx`, `shortcut-row.tsx`, `PostProcessingToggle.tsx`, `PostProcessingSettingsApi/ModelSelect.tsx`, `ui/Select.tsx`, 26 locales, `package.json`
- **AC impacted:** AC-001, AC-002, AC-004, AC-006, AC-007
- **Author:** Claude Code (claude-opus-5-5)
- Own save no longer rolled back by a lagging library; failed reload shown with Retry; readiness fix inline in the editor for the resolved provider (contract navigation replaced, see implementation.md); typing during a save kept; Cancel during a creation honoured; `committed`/`unknown` outcomes reread; default-prompt failures shown; outdated test results flagged; invalid prompts highlighted at once; shortcut hint; focus and accessible names; function sizes reduced.

## 2026-10-14 — [Bugfix]: Recording indicator hardening

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/overlay/menu-timer.ts (new, + test), prompt-pill.tsx, use-dictation-prompt.ts, indicator-stack.tsx, indicator-render.test.tsx, package.json
- **AC impacted:** AC-003
- **Why:** indicator audit warnings: the hover close timer was never cleared on unmount or at a new dictation; functions over 30 lines; the gate-closed test froze the new markup instead of comparing with dev.
- **Tests:** menu-timer.test.ts (6), indicator-render.test.tsx; [proofs](proofs/claude-hardening/summary.txt)
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-14 — [Bugfix]: UI hardening, review round 1

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/overlay/indicator-render.test.tsx split with indicator-gates-render.test.tsx and indicator-test-fixtures.ts (tests only)
- **AC impacted:** none (tests only)
- **Why:** hardening review: the indicator test file exceeded 400 lines.
- **Tests:** test:indicator; [proofs](proofs/claude-hardening-fix1/summary.txt)
- **Author:** Claude Code (claude-opus-5-5)
