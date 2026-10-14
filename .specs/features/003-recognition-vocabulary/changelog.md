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
