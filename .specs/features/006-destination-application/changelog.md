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

## 2026-10-14 — [Plan Update]: Settings framework Step 0 (D2–D4), merged 008 names and 007 interface

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/destination-contract.md updated)
- **AC impacted:** None (pre-implementation plan alignment; AC-005, AC-006, AC-007, AC-008 coverage clarified)
- **Author:** Claude Code (spec-refine 006 plan, claude-opus-5-5)
- Applies the 2026-10-14 independent plan review (3 blocking findings) and orchestrator decisions D1–D4.
- D1: only macOS native evidence gates a step; Windows, Linux, an older binary and a second account are pending release evidence with exact checks (Steps 1, 7, Testing Strategy, QE).
- New Step 0 extends the 008 framework: `DomainPlan::Live` with `domains/live_operation.rs` (`LiveOperation`, `LiveCall`, `LiveAnswer`, `LiveContext`, `managed_state`), `SettingsBackend::live_context` (only `RoutedBackend`), offline refusal identical to the delivered live-only refusal (`unavailable`, exit 5, empty details); `platform_capability.rs` (`PlatformCapability::Destination`, `PlatformCapabilities`, `required_capability`, `check`); `ErrorDetails.reason` (`ErrorReason`) and `ErrorDetails.items` (`ErrorItem`, max 100); `SettingsError::unsupported_platform` (D4: `validation_failed`/2, `details.reason = "unsupported_platform"`). Names also recorded in `.specs/ship.md` (Contrat D2/D3).
- Uses delivered 008 names only (`tauri_adapter::write`, `shortcut/mod.rs` output commands, `OUTPUT_KEYS: [&str; 11]`, `effects.rs`); `LiveSettingsEffects` removed. Static platform capability decides the CLI/UI refusal offline and routed; runtime foreground readability stays in the running instance and is never a CLI validation; `output get` returns the plain value.
- Defines `DestinationContext` (PID, stable identity incl. bundle ID, `dictation_id`, `destination_generation`, frontmost-at-end and `target: None` cases) and the single `RecoveryDisposition {Pending, Insert, Recover, Cancelled}` veto shared with 009/010; aligns identifiers and outcome wire values with 007; 007 alone displays its remote-screen clipboard message; 006 tests map injected 007 results instead of 007 timers.
- QE Analysis aligned with the spec (High, Shared, Functional); Next Action no longer pins a model.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:b0be88bc -->

## 2026-10-14 — [Plan Update]: 008 sources as code paths, four ValidationContext constructions, atomic Enter claim for 009

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/destination-contract.md updated)
- **AC impacted:** None (pre-implementation; structural coverage 17/17)
- **Author:** Claude Code (spec-refine 006 plan, claude-opus-5-5)
- Applies the round 2 independent review: its plan review could not be prepared (`paired_review_batch_budget_exceeded`, more than 64 batches) because the second 008 link was normative and pulled 008's whole document graph, including an untracked file.
- 008's spec and implementation map are now cited as code paths (`.specs/features/008-settings-cli/...`), so they import no normative closure; the contract's links to this feature's plan and spec each carry their own "Inspect". Plan review preparation at 180,000 characters: 1 batch, no error.
- Step 0: `ValidationContext.platform` is set in the four production constructions (`operations::set`, `LiveSettings::validate_ui_value`, `domains::output::set` and the under-lock revalidation `edit::revalidate_mutation`, `edit.rs:98`) and in the 29 test constructions of `src-tauri/tests/`.
- New `DestinationManager::claim_enter(dictation_id, destination_generation) -> Result<EnterClaim, EnterRefusal>` (plan, contract, Step 2 tests, Gherkin): reads the disposition and records the single Enter claim under the session lock of `set_disposition`; refuses `Stale`, `Disposition(current)` and `AlreadyClaimed`; a later `Recover` does not revoke a granted claim. 006 never calls it; 009 does.
- Reason enum: 009 adds `pinned_app_not_running`; a running pinned app not brought forward keeps `activation_failed` (orchestrator decision 2026-10-14).
- Shared D2/D3 names are recorded in this plan and its contract only; no reference to the untracked program notes.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:8325a713 -->
