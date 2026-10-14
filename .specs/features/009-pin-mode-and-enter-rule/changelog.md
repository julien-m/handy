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

## 2026-10-14 — [Plan Update]: 008 sources as code paths, explicit AC-006, claim_enter, two pinned failure reasons

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; structural coverage back to 13/13)
- **Author:** Claude Code (spec-refine 009 plan, claude-opus-5-5)
- Applies the round 2 independent review.
- 008's spec and implementation map are cited as code paths, and 006's Step 0 is named in text (006's plan stays the inspected context), so the plan review no longer loads 008's document graph or an untracked file. Plan review preparation at 180,000 characters: 20 batches, no error (round 2 simulation: 30).
- AC lists written one by one ("AC-004, AC-006" instead of "AC-004/006"), so AC-006 has a structural reference again.
- The atomic read is 006's `DestinationManager::claim_enter`: the `Pending → Consumed` transition grants once on `Insert` and refuses `Pending`/`Recover`/`Cancelled`, a stale generation and a second claim, under the session lock of `set_disposition` (Enter veto, Step 4, state diagram, risks).
- Orchestrator decision 2026-10-14: AC-003 "pinned app unavailable" is carried by `pinned_app_not_running` (pin not running; new in 006's reason enum) and `activation_failed` (pin running, not brought forward within 1 s; 006's existing reason), the reasons 010 presents; Gherkin, sequence diagram, Step 2 and risks updated; no untyped subtype.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:f275f226 -->

## 2026-10-14 — [Plan Update]: plain not_found for auto_submit, cross source FrontmostAtEnd, resolved_target aligned with 006

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-004 and AC-005 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 009 plan, claude-opus-5-5)
- Applies the round 3 independent review (`proofs/claude-review/independent-review-r3.md`).
- Orchestrator decision 2026-10-14: the retired-key hint is removed (no `descriptor::RETIRED_KEYS`, `ErrorReason::RetiredKey`, `SettingsError::retired_key` or `details.field = "automatic_enter"`), since no 009 requirement asks for it. `auto_submit` is refused with 008's plain `not_found`/4, like any unknown key; the legacy value is still migrated by `settings::apply_settings_migrations` (contract table, Gherkin, Step 3 files, CLI test row, risks).
- The Remove destination cross gives source `FrontmostAtEnd`, as in 006's route table (`.specs/features/006-destination-application/plan.md`), not `badge`; pin_only still sends no Enter for that dictation.
- `resolved_target` is exactly 006's `DestinationContext.target`, recorded at the pre-dispatch guard: for `FrontmostAtEnd` the frontmost application read at the guard (`None` only if unreadable or Handy). Only captured, badge and pinned routes require it still frontmost before Enter, as AC-005 states.
- The plan reads `RecoveryDisposition` through `claim_enter` and no longer lists it among extended types.
- Full acceptance: Windows native evidence is pending release evidence (D1), never claimed.
- Plan review preparation at 180,000 characters: 18 batches, no error (round 3: 20).
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:86e43d44 -->

## 2026-10-14 — [Plan Update]: history_status from the actual save_entry result, text-only row when the WAV fails

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-003 and AC-005 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 009 plan, claude-opus-5-5)
- Aligns with Julien's decision 2026-10-14, carried by the 006 spec and plan (commit `306c368`, `.specs/features/006-destination-application/plan.md`): a dictation whose WAV cannot be saved still gets a text-only history entry; `history_status` comes only from the actual `save_entry` result (`saved` text-only included, `failed` on save_entry failure, `not_saved` only when no write was attempted).
- The current-state note says the `wav_saved` gate is removed by the 006 plan; the retained-text paragraph, Step 5 action, P0 conservation test row, Conservation P0 gate and the refreshed WAV-gated conservation risk now test a WAV failure as a saved text-only row and a save_entry failure as `failed`, instead of "WAV disabled" or "WAV/save absence" treated as not saved.
- Plan review preparation at 180,000 characters: 18 batches, no error (previous: 18).
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:48c67c3f -->

## 2026-10-14 — [Plan Update]: round 4 review ingested, 009 behind 006's destination-return gate, Enter keyed by dictation and generation

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-004, AC-005 and AC-006 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 009 plan, claude-opus-5-5)
- Applies the round 4 independent plan review (synthesis of 18 batches, never ingested before) and the orchestrator decisions of 2026-10-14, checked against the delivered 006 and 007 code in `dev`.
- **Rollout gate:** all of 009 (pin, three-value policy and migration, Enter authority, `destination pin|unpin|status`) ships behind 006's `destination-return` Cargo feature. Closed (default build): `dev` unchanged, with the legacy `auto_submit` emitters, `automatic_enter` answering `not_found`/4, pin commands refused `validation_failed`/2 `unsupported_platform`, and the checked 008 schema equal to `dev`'s. Open: the specified behavior. The full suite runs in both states.
- **Return to a closed build:** the migration never deletes or rewrites `auto_submit` or `auto_submit_key`; `automatic_enter` is optional and never written by a closed build; a P0 test migrates in the open state and rereads in the closed state with an identical Enter decision.
- **Enter key:** `EnterTransaction` is keyed by `(dictation_id, destination_generation)`, the `claim_enter` pair; `outcome_id`, allocated by `delivery::publish` after the method, is no longer used (text, sequence diagram, ER).
- **Delivered-code warnings:** a claim after the next dictation's start is refused `Stale` with zero Enter (intended, tested); Windows chord failure becomes a method error in the open build instead of the outer `Ok`; a live CLI timeout is `unavailable`/5 with `details.outcome = "unknown"`; `pin`/`unpin` refuse `--expected-revision` with `usage_error`/2 and `status` ignores it.
- **Scope:** `destination status` returns only the pinned application or null (AC-006); route, source, phase and support removed.
- **Names and files:** delivered names (`delivery::HistoryRecord`, `history::save_dictation`, `outcome::DestinationFailure`, `clipboard::send_auto_submit`); Step 2 and Step 5 file lists completed with the 006 and 007 files they touch.
- Plan review preparation at 180,000 characters: 18 batches, no error (round 4: 18).
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:19f333f0 -->
