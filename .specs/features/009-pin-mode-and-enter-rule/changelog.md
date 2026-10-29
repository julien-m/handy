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

## 2026-10-14 — [Spec Update]: pinned reasons split, destination-return gate recorded, 50 ms pause before Enter kept in the plan

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Story 2 Gherkin and Mermaid, AC-003, Clarifications)
- **Code modified:** None (plan.md updated: Enter authorization, Step 5, risk table)
- **AC impacted:** AC-003 (reason wording; ID unchanged), AC-005 (plan timing only)
- **Author:** Claude Code (spec-refine 009, claude-opus-5-5)
- **AC-003:** the reason "pinned app unavailable" becomes `pinned_app_not_running` (the pinned application is closed) or `activation_failed` (running, not frontmost within the 006 limit), the reasons `.specs/features/010-recovery-bubble/spec.md` expects. Story 2 gains the activation-failure scenario and branch.
- **Delivery precision (Clarification 2026-10-14):** all of 009 ships behind the `destination-return` gate of 006; gate closed, Handy behaves exactly as `dev`. No AC, FR or SC changes.
- **Plan (orchestrator decision 2026-10-14):** the plan had removed `dev`'s pause before Enter. It now keeps a 50 ms stabilization pause after t0 (`PRE_ENTER_SETTLE`, the value of `clipboard.rs` and `AUTO_SUBMIT_SETTLE` in `smart_paste/app.rs`), waited through the Clock off the main thread, with the frontmost check and claim after it, inside the 100 ms budget.
- Clarification inventory was empty: both decisions are written in `## Clarifications`.
- Review preparation at 180,000 characters: spec 1 batch, plan 16 batches, no error.
- Review: the spec and plan reviews are stale until an independent reviewer refreshes them.

<!-- finalize:spec-refine:2026-10-14:b5422949 -->

## 2026-10-14 — [Spec Update]: round 5 review applied, decisions under Julien's mandate traced

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Story 4 Gherkin and Mermaid, AC-002, AC-004, AC-005, AC-006, Clarifications, QE Analysis, Edge Cases, SC-002)
- **Code modified:** None (plan.md updated)
- **AC impacted:** AC-001 (lifetime decision), AC-002, AC-004, AC-005, AC-006 (wording; IDs unchanged)
- **Author:** Claude Code (spec-refine 009 and 009 plan, claude-opus-5-5)
- Applies the round 5 independent reviews (spec and plan) and the orchestrator decisions of 2026-10-14, decided under Julien's mandate (instruction of 2026-10-14) and to be confirmed at the final review. Each one is written in `## Clarifications`.
- **Pin lifetime (decision):** the pin lasts until unpinned or until Handy quits and is not restored after a restart. The `[ASSUMED]` marker becomes a decision. Interpretation recorded: the Input's "transcription stops/restarts" are successive dictations, not quitting and relaunching Handy.
- **Closed build (decision):** identical to `dev`, nothing added. No `destination` CLI domain (`usage_error`/2, as in `dev`; `DestinationDomain` registered under `cfg(feature = "destination-return")` only), `automatic_enter` is `not_found`/4, the migration never modifies `auto_submit` or `auto_submit_key`. The plan's former `unsupported_platform` answer of the closed build is removed, and so is the gate-state UI command: the UI reads the presence of `automatic_enter` in the settings snapshot.
- **`Stale` refusal (decision):** a dictation B that starts before dictation A's Enter is decided cancels A's Enter. Written in AC-005, a Story 4 scenario and flowchart, Edge Cases and Clarifications, and referenced from the plan.
- **Persisted Clarify decisions** (`persist_clarification_decision`, same mandate): `unpin_with_return_setting_off` (with return off, unpin gives the `FrontmostAtEnd` route, no destination); `legacy_auto_submit_after_migration` (legacy keys kept, `automatic_enter` authoritative once stored); `cli_legacy_auto_submit_key` (keys `automatic_enter` and `auto_submit_key`, `auto_submit` `not_found`); `cli_pin_during_dictation` (same effect as the badge). Normative wording updated in AC-002, AC-004, AC-006 and SC-002.
- **Earlier decisions, kept and checked:** frontmost check after `claim_enter`, right before the key (pause → claim → frontmost check → key, sequence and state diagrams, Step 4, tests); 50 ms pause kept; Linux "Only in PIN mode" hint removed from the plan (Design Reference, CLI section, risk row), consistent with 006 AC-007, which already shows destination return unavailable with its reason; tests, clippy and checks in four feature combinations, including `destination-return,smart-paste-dispatch`, and the macOS native proof binary built with both features; Linux native evidence pending release evidence, Linux behavior gated by automated tests only; both reasons `pinned_app_not_running` and `activation_failed` asserted per case.
- **Plan warnings treated:** stale Technical Context (`wav_saved.then_some`, `delivery.rs`); gate-conditional descriptors, `OUTPUT_KEYS`, `PublicSettings` and one `bindings.ts`; 007's `session_method` named as an emitter replaced in the open build, with a `claim_enter` caller scan; open → closed → open precedence defined (`automatic_enter` wins once stored) and tested; invalid stored `automatic_enter` gives Never without write; several instances resolved through `pid_hint` and 006's `resolve_instance` (spec Edge Case aligned with delivered 006); runtime support `Unsupported` refuses pin/unpin; route-keyed frontmost check; Windows reliable rule stated per dictation; 006 `paste-error` toast before 010 (no ≥4 s indicator in delivered 006); lookup `not_found` message; unpin after a badge override and with return off; pin_only suppression without pinned source; activation failure per dictation in the pin state model; `--dry-run`, reply shape and identity `{kind, value}` evidence; Step 6 prerequisite and files aligned on delivered 006 overlay files; `enter_insertion.rs` and Step 1 limited to Enter decisions over 007 outcomes; seams aligned on delivered ports (`PasteboardPort`, `AxTextPort`, 006 `Clock`/`Pause`), `ClipboardPort`, `HistoryPort`, `AudioDeviceCatalog` and the LLM seam removed; SC-003 native protocol named; policy frozen at insertion entry and tested; single 010 hand-off by 006.
- **Spec warnings and infos treated:** AC-002 return-off fallback, AC-004 legacy keys, AC-006 CLI keys, Story 4 flowchart aligned with AC-005, 100 ms marked as a QE target measured natively, return-off pinning from the CLI in Edge Cases.
- **Not changed, with reason:** the Windows chord-failure change and the gate authority are recorded in this plan only, since this pass may not modify 006's plan or contract (006's mapping code is unchanged; the 006 owner records it when 009 lands). `--expected-revision` offline answering `unavailable`/5 before `usage_error`/2 is accepted as documented (the expected flag is only known inside the instance). `testing/strategy.md` stays untouched (shared file outside this feature). No new CLI error reason is added for lookups (not required by AC-006).

## 2026-10-15 — [Spec Update]: round 6 review applied, Julien's decisions of 2026-10-15 traced

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Story 1 Mermaid, Story 4 Mermaid, AC-006, FR-001, Clarifications, Edge Cases)
- **Code modified:** None (plan.md updated)
- **AC impacted:** AC-002, AC-003, AC-006 (wording and decisions; IDs unchanged), AC-005 (plan only)
- **Author:** Claude Code (spec-refine 009 and 009 plan, claude-opus-5-5)
- Applies the round 6 independent reviews (spec and plan) and the decisions validated by Julien (maintainer) on 2026-10-15, after the plan reached the LiveSpec round ceiling; Julien also approved one exceptional review round (r8).
- **Decisions persisted** (`persist_clarification_decision`, each "validated by Julien (maintainer) on 2026-10-15"): `return_off_and_pin` (AC-002: return off and nothing pinned is `FrontmostAtEnd`; a pinned application is activated and receives the text with return off, with the AC-005 frontmost check before Enter); `claim_enter_scan_step` (FR-004: the `claim_enter` caller scan moves from Step 4 to Step 5, which replaces 007's interim caller; no other step depends on it at Step 4); `cli_pin_target` (AC-006: `pin <identifier>` only, `current` removed, no current/next effect report); `cli_pin_change_options` (AC-006: `pin`, `unpin` and `status` take neither `--dry-run` nor `--expected-revision`; the 008 framework refuses an option an action does not declare at parse, with `usage_error`/2, as for `get` and `microphones list`); `pinned_with_unreadable_foreground` (AC-003: a pin with an unreadable foreground inserts nowhere and fails with 006's existing `activation_failed`, transcript recoverable as in 006).
- **Earlier decisions validated by Julien on 2026-10-15:** pin lifetime (until Unpin or Handy quits), closed build identical to `dev`, `Stale` refusal. The four persisted decisions `unpin_with_return_setting_off`, `legacy_auto_submit_after_migration`, `cli_legacy_auto_submit_key` and `cli_pin_during_dictation` stay "to be confirmed"; the plan's Next Action makes their confirmation a gate before `/spec-implement`.
- **Spec warnings treated:** Story 1 flowchart and FR-001 keep 006's capture at start while a pin wins; Story 4 node D covers the classic paste; the several-instances edge case explains `activation_failed`; the CLI identifier is 006's stable identity (`cli_pin_identifier_format`, a precision derived from 006, not a decision of Julien); edge cases for the unreadable foreground with a pin and for the 006 cross while pinned; the `[ASSUMED]` support clarification keeps run-time readability out of the CLI, as 006 does.
- **Plan warnings treated:** return-off rule corrected (blocking); scan moved (blocking); reliable-paste emitters named, macOS beta reliable paste included on the session path; Windows chord failure keeps its delivered outcome (the open-build method-error change is withdrawn, since 006 keeps Windows reliable paste unchanged and the rendezvous already suppresses Enter); open-build `PublicSettings` without `auto_submit`; fresh open install `Some(Never)`; migration reads raw legacy values; salvage and descriptor inventory keep `auto_submit`; startup normalization write of an invalid `automatic_enter` stated; runtime support no longer refuses `pin`/`unpin`, `status` behavior stated; both UI commands registered in both states for one `bindings.ts`; open-state schema test; sessionless open path reads Automatic Enter; localized `pinned_app_not_running` and `Recover` hand-off in Step 2; Chromium double-read check in Step 1; `test:indicator`, `test:output-settings` and `--include-ignored`; exact legacy-binary check; sequence and state diagrams cover none, pre-insertion recovery and the reliable classic fallback; risk rows qualified.
- **Not changed, with reason:** AC-005 keeps no frontmost check for a dictation without destination (`FrontmostAtEnd`, as today; confirmed by decision `return_off_and_pin`); a later auto-submit change in a closed or older build stays ignored once `automatic_enter` exists (`legacy_auto_submit_after_migration`); the parsed "AC-007" reference is 006's AC-007.

## 2026-10-15 — [Feature]: Pin mode and Enter rule implemented behind the destination-return gate (automated evidence; native proofs pending with Julien)

- **Type:** Feature
- **Spec modified:** No (status Planned → In Progress through finalize; requirements unchanged)
- **Code modified:** session pin in `src-tauri/src/destination/` (`pin.rs`, manager, session, snapshot, menu, insertion, outcome), Enter authority `src-tauri/src/automatic_enter/` (policy, transaction, rendezvous, migration, Tauri glue), Automatic Enter setting and `destination` CLI domain in `src-tauri/src/settings.rs` and `src-tauri/src/settings_service/`, method integration in `clipboard.rs`, `paste_tx/`, `destination/delivery.rs`, `smart_paste/app.rs`; UI in `src/overlay/` (pin entries, pin mark, pinned reason) and `src/components/settings/automatic-enter*`; 26 locales; README and the 008 CLI contract
- **AC impacted:** AC-001 to AC-006 partial (automated evidence only), SC-001 and SC-002 automated, SC-003 automated form only
- **Author:** Claude Code (spec-implement, claude-opus-5-5)
- Commits on `feature/009-pin-mode`: `dad3819d`, `a92d931d`, `edd02eb6`, `40143f4b`, `d8ad2a53` and the documentation commit of this entry.
- Gate closed (default build): identical to `dev` (auto-submit switch and emitters, no `destination` domain, `automatic_enter` unknown, no migration); tested in the four feature combinations.
- Differences from the plan: `claim_enter` also stale after a cancellation that follows publication; lenient `automatic_enter` reading and in-memory drop in a closed build; no policy+key batch command; the reliable-paste signals belong to the Enter authority; Pin/Unpin above the application list (spec) where the mockup shows Unpin at the bottom; bindings generated from one registry with a drift test; three existing clippy test lints of `dev` fixed.
- Native macOS proofs (required before Implemented) and the Windows, Linux and older-binary release evidence are listed in `progress.md` and `.specs/native-proofs-checklist.md` (section 009); none is claimed.

<!-- finalize:spec-implement:2026-10-15:851e4c61 -->

## 2026-10-29 — [Spec Update]: Pin and Unpin moved to the fixed footer of the badge menu (with 012)

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Behavioral AC `has_dropdown`, Clarifications: decision `pin_entries_in_fixed_footer`)
- **Code modified:** `src/overlay/destination-menu.tsx` (fixed footer), `src/overlay/pin-state.ts` (comments), `src/overlay/indicator-render.test.tsx`, `src/overlay/pin-state.test.ts`, `tests/destination-favorites.spec.ts`
- **AC impacted:** AC-001 (placement only; which entries are offered and what they do is unchanged)
- **Author:** Claude Code (spec-feature 012, claude-opus-5-5)
- Julien's native finding (2026-10-29): no "Pin" entry was visible, because it sat above a long application list in a menu that opens upwards from the badge. "Pin <destination>" and "Unpin <app>" now close the menu in a fixed footer after "Frontmost app at the end", nearest the badge and never scrolled out of view; a Playwright test with 20 applications in a 600-pixel-high page checks it.
