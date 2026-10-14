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

## 2026-10-14 — [Plan Update]: 008 sources as code paths, pinned reasons aligned with 009

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; structural coverage 15/15)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- Applies the round 2 independent review: 008's plan was a normative link ("[008's plan] is historical context") and pulled 008's whole document graph, so the plan review could not be prepared.
- 008's implementation record and plan are cited as code paths; the shared contracts are cited from the 006 plan and its destination contract only, without the untracked program notes. Plan review preparation at 180,000 characters: 18 batches, no error (round 2 simulation: 33).
- Pinned reasons aligned with the 009 plan (orchestrator decision 2026-10-14): `pinned_app_not_running` ("<App> is pinned but not open") and `activation_failed` with `destination_source = pinned` ("Could not bring <App> forward"), in the decision table, outcome mapping, sequence diagram and canonical reason table; the Enter claim is 006's `DestinationManager::claim_enter`.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:61308064 -->

## 2026-10-14 — [Plan Update]: history_status from the actual save_entry result, text-only row when the WAV fails

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/recovery-contract.md updated)
- **AC impacted:** None (pre-implementation; AC-005 planning only, ID unchanged)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- Aligns with Julien's decision 2026-10-14, carried by the 006 spec and plan (commit `306c368`, `.specs/features/006-destination-application/plan.md` and `contracts/destination-contract.md`): a dictation whose WAV cannot be saved still gets a text-only history entry.
- The recovery instance carries 006's `history_status` unchanged: `saved` when save_entry committed the row, text-only included (WAV disabled, unavailable or failed); `failed` when save_entry was attempted and failed; `not_saved` only when no write was attempted. The plan no longer maps a disabled or unavailable WAV to `not_saved`. A recovery always follows a non-cancelled transcript, so `not_saved` is not expected there and gets the same no-history explanation as `failed`.
- Current-state note (actions.rs `wav_saved` gate removed by the 006 plan), Step 4 inputs, execution evidence, P0 history integration row, risk row and the contract history line aligned.
- Plan review preparation at 180,000 characters: 22 batches, no error (previous: 18; the growth comes from the cited 006 plan, which grew with the same decision).
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:e2c204ed -->

## 2026-10-14 — [Plan Update]: round 4 review ingested, 010 behind 006's destination-return gate, synchronous verdict in the guard check

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/recovery-contract.md updated)
- **AC impacted:** None (pre-implementation; AC-001, AC-002, AC-006 and AC-007 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- Applies the round 4 independent plan review (synthesis never ingested before) and the orchestrator decisions of 2026-10-14, checked against the delivered 006 and 007 code in `dev`.
- **Rollout gate:** all of 010 ships behind 006's `destination-return` Cargo feature. Closed (default build): a failed paste shows the existing `paste-error` message as in `dev`, Linux and 007's sessionless path included, with no bubble, classification or setting (`not_found`/4, checked schema unchanged). Open: a Linux method failure gets the reduced bubble through a `FrontmostAtEnd` session.
- **Order:** the sequence diagram and steps follow delivered 006. `before_insertion` runs in the first accepted `DispatchGuard::check()`, after `paste_delay_ms`, where the disposition becomes `Insert`.
- **Thread:** the verdict is synchronous, on the main thread for main-thread methods. The AX classification is bounded to about 50 ms with `AXUIElementSetMessagingTimeout`, and doubt or timeout answers `Insert`. The worker classification and the impossible post-classification recheck are removed. Step 4 lists `destination/delivery.rs`, `destination/app.rs` and `smart_paste/app.rs`.
- **Scope:** the "field detection macOS-only" settings note is removed. No AX call happens when the setting is off, when no insertion is due or for 007 per-application strategy bundles.
- **Names:** `Recover(NoTextField)` arrives as `insertion_failed` without `method_outcome` and is correlated by `(dictation_id, destination_generation)`. 010's presentation type becomes `BubbleReason`, to avoid 006's `context::RecoveryReason`. The display name comes from the session snapshot, then `WorkspacePort::instances`, then the localized noun. `transaction_id` is compared as a decimal string.
- Plan review preparation at 180,000 characters: 14 batches, no error.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:fb255547 -->
