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

## 2026-10-14 — [Spec Update]: focus in another process, conditional history, "some text" detail, destination-return gate, new dictation before 4 s

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Story 4 Gherkin and Mermaid, has_tooltip Gherkin, AC-001, AC-005, Key Entities, Clarifications, Edge Cases)
- **Code modified:** None
- **AC impacted:** AC-001, AC-003 (decision only), AC-005 (IDs unchanged)
- **Author:** Claude Code (spec-refine 010, claude-opus-5-5)
- **Key Entities:** focus owned by another process is no longer a recovery reason; it is an undetermined classification with the verdict `Insert`, as AC-002 states. Pinned reasons named `pinned_app_not_running` and `activation_failed`.
- **AC-003 (persisted Clarify decision):** a running pinned app not brought forward shows "Could not bring <App> forward"; "is pinned but not open" is reserved for `pinned_app_not_running`.
- **Story 4 and AC-005:** "kept in history" only when the history save succeeded (006 AC-005, `history_status`); otherwise the explanation says the transcript is not in history.
- **AC-001:** "Some text may have reached the application" only for method failures and the smart paste outcomes "not read", "uncertain" and "insertion failed"; never for "permission missing" (no keystroke sent) or pre-insertion reasons.
- **AC-005:** a new dictation closes the bubble immediately, even before 4 seconds; the 006 minimum applies to passive display. Checked against the 006 spec and delivered code (no timed reason display in `dev`, only `paste-error`).
- **Delivery precision (Clarification 2026-10-14):** all of 010 ships behind the `destination-return` gate of 006; gate closed, Handy behaves exactly as `dev`.
- Review preparation at 180,000 characters: spec 1 batch, plan 14 batches, no error.
- Review: the spec and plan reviews are stale until an independent reviewer refreshes them. The plan still discloses possible partial insertion for `permission_missing` (reason table and decision table); a plan alignment is needed before the plan review.

<!-- finalize:spec-refine:2026-10-14:8ecc2640 -->

## 2026-10-14 — [Plan Update]: permission_missing aligned on the spec, no possible-partial-insertion detail

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-001 planning only, ID unchanged)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- Orchestrator decision 2026-10-14: smart paste `permission_missing` sends no keystroke and leaves the clipboard unchanged, so it cannot cause a partial insertion. The plan now matches spec AC-001 and its Clarification of 2026-10-14. The bubble shows "Insertion failed" and the explanation says nothing was sent to the application.
- Updated: decision table (separate `permission_missing` row), method-outcome paragraph, history explanation, sequence diagram (`permission_missing` branch), canonical reason table (detail by outcome), new Gherkin scenario "Permission missing".
- Plan review preparation at 180,000 characters: 14 batches, no error (unchanged).
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:3d66e47e -->

## 2026-10-14 — [Spec Update]: round 5 review applied, decisions under Julien's mandate traced

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Story 1 Gherkin and Mermaid, Behavioral AC `dismissible_layer` and `has_tooltip`, AC-001, AC-002, AC-003, Key Entities, Clarifications, QE Analysis, Edge Cases)
- **Code modified:** None (plan.md updated)
- **AC impacted:** AC-001, AC-002, AC-003, AC-007 (wording and decisions; IDs unchanged)
- **Author:** Claude Code (spec-refine 010 and 010 plan, claude-opus-5-5)
- Applies the round 5 independent reviews (spec and plan) and the orchestrator decisions of 2026-10-14, decided under Julien's mandate (instruction of 2026-10-14) and to be confirmed at the final review. Each one is written in `## Clarifications`.
- **Behavioral AC (decision):** the hover hint "Drag into a text field" and the animated "closing" state are removed (not requested, not approved); visual states are "open" and "closed". The plan's tooltip and closing assertions are removed too.
- **AX timeout (decision, plan):** `AXUIElementSetMessagingTimeout` is set per element, on the destination's application element (`AXUIElementCreateApplication`) and its descendants, each time to the remaining budget of one 50 ms deadline; never on the system-wide element, never 0. 006's foreground reads (`macos_focus.rs`, `guard.rs`) keep their behavior, tested. Doubt or timeout gives `Insert`. Native threshold: no sample above 60 ms over at least 30 runs.
- **Focus in another process (decision):** always undetermined, `Insert`, before any role check; the plan rule that let a proven noneditable role win is removed.
- **"Closing loses this recovery" removed** from the plan's explanation; the explanation only says whether the transcript is in history.
- **007 per-application strategies (decision):** Screen Sharing, iPhone Mirroring, Windows App and Jump Desktop show a remote field that cannot be read locally, so the classification is undetermined (AC-002, Edge Cases, Clarifications), as a consequence of the undetermined rule, not an exemption.
- **Persisted Clarify decisions** (`persist_clarification_decision`, same mandate): `paste-method-none-recovery` (paste method none: no bubble), `no-focused-element-classification` (no focused element, AX error or timeout: undetermined). Normative wording updated in AC-001 and AC-002.
- **Plan warnings treated:** stale Technical Context (`delivery.rs`, `wav_saved.then_some`) and Summary (006 and 007 delivered); Constitution Check rows for macOS native evidence (Windows and Linux pending release evidence) and minimal scope; seams limited to delivered ones (`PasteboardPort`, 006 `Clock`/`Pause`, `InsertionPort`, `RecoveryPort`, `OutcomeConsumer`, `DiagnosticSink`) plus 010's own ports; `ClipboardPort`, `HistorySaveOutcomePort`, `AudioDeviceCatalog` and the LLM port removed; overlay style none test; closed-gate default-build test (`paste-error`, no classification, no bubble); reason table with the Windows reliable paste failure ("Insertion failed", "Some text may have reached the application", article kept) and the spec AC-001 list completed accordingly (decision); Step 7 no longer claims AC-006 on macOS; `NoTextField` reaches 010 as the typed reason `no_text_field` (006 extended at `GuardRefusal::RecoverInstead`, `DestinationFailure`, `refusal_report`), replacing the correlation record, so a pre-insertion reason never shows the partial-insertion detail.
- **Earlier decisions, kept and checked:** failures without `method_outcome` classified by reason; transcript preview never in a log or app-wide event (sentinel test); setting refused outside macOS with the gate open (`KEY_CAPABILITIES` row, new `PlatformCapability::FocusedFieldDetection`, no toggle) and absent with it closed, replacing the round 4 "saved everywhere" text; the ≥4 s rule kept for passive display; a new dictation closes the bubble; a late read received before display is buffered and applied; `ReasonPresented` only once the bubble is visible; Copy through 007's actor (`ActorMessage::DeliberateCopy`, queued behind an in-flight insertion); smart paste open with destination closed keeps the `paste-error` toast (documented); never toast and bubble together.
- **Other findings treated:** gate-conditional stored field, descriptor, `OUTPUT_KEYS` and `PublicSettings`, one `bindings.ts`, generators run in the closed state; four feature combinations for tests, clippy and checks, macOS native proof binary built with `destination-return,smart-paste-dispatch`; all 26 locales receive the keys; `scripts/recovery-native-proof.ts` created in Step 1; decision-table input includes an absent `method_outcome`; diagnostic fan-out named; slot released after copy; permission missing answers `Insert` without AX query; ER fields and cardinality; sequence diagram acknowledgements; cross-dictation precondition of "Copy during repeated reads"; Windows/Linux surface proofs worded as pending release evidence; spec Story 1 Gherkin ("Could not bring Notes forward") and flowchart (setting off, insertion failure); transcript defined as the text that would have been inserted; drag handle macOS only; late drop after a new dictation; 300 ms marked as a QE target.
- **Not changed, with reason:** `contracts/recovery-contract.md` still states the round 4 presenter, Copy, late-read, no-text-field and setting rows; this pass may only modify spec, plan, changelog and decisions, so the plan says it prevails and the contract needs a later alignment. The 006 plan's new-recording rule and 007's missing late-read diagnostic under "copy to clipboard" belong to 006 and 007; the 010 spec decision and plan risk rows record them. Smart paste `insertion_failed` keeps the partial-insertion detail even when the transaction could not start, as spec AC-001 lists it. The closed-build `not_found`/4 for the 010 key, unlike 006's `unsupported_platform`, follows from `dev` never having the 010 key.

## 2026-10-14 — [Plan Update]: local recovery contract aligned with the round 5 plan

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (contracts/recovery-contract.md and plan.md updated)
- **AC impacted:** None (pre-implementation; AC-001, AC-002, AC-005 and AC-007 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- The contract now states what the plan states: `ConsumerAck::ReasonPresented` only once the bubble is visible (300 ms budget, otherwise 006's `paste-error` and no bubble); Copy through 007's actor (`ActorMessage::DeliberateCopy`); the late-read diagnostic buffered before admission, through `FanOutDiagnostics`; the typed reason `no_text_field` (no correlation record); the setting refused outside macOS with the gate open (`KEY_CAPABILITIES`, `FocusedFieldDetection`, no toggle) and absent with it closed. The Enter veto row follows the per-element AX timeouts and the foreign-focus rule.
- The plan sentence that let the plan prevail over the contract is removed: both say the same thing.

## 2026-10-14 — [Plan Update]: round 6 plan review applied (presentation, Copy, setting)

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md and contracts/recovery-contract.md updated)
- **AC impacted:** None (pre-implementation; AC-001, AC-002, AC-005, AC-006 and AC-007 planning only, IDs unchanged)
- **Author:** Claude Code (spec-refine 010 plan, claude-opus-5-5)
- Applies the round 6 independent plan review and the orchestrator decisions of 2026-10-14 taken under Julien's mandate. The spec stays unchanged.
- **300 ms (blocking, business scope):** a QE target only. The bubble is never cancelled for slowness and no "toast only" fallback exists; `paste-error` appears only when the bubble cannot be presented at all (recovery window absent, or a confirmed creation or show error). New section "Presentation hand-off".
- **No blocking wait on an async worker:** `consume` copies the transcript, admits the instance and returns at once a oneshot receipt (`ConsumerAck::PresentationPending`); `insert_final_text` awaits it with a 2 s guard (`tokio::time::timeout`) that never cancels the bubble. On expiry the bubble stays scheduled; 006's slot is released only after `Presented`, `Superseded` or a confirmed `Unpresentable`, so no text is lost.
- **Copy (blocking):** through 007's actor (`ActorMessage::DeliberateCopy`, added by 010, unmarked `write_plain_text`) only when it runs; otherwise `clipboard::write_text_to_clipboard`, the "copy to clipboard" write of `dev`. Both paths are tested in the feature combinations. A failed Copy shows no dedicated message: the bubble stays open with its other actions and the failure is logged without text.
- **Setting (blocking):** real `AppSettings` field (`Option<bool>`, exposed through Specta), value per state set by normalization without a write (macOS `true` by default, Windows and Linux `false`, closed build absent), gated descriptor row and `OUTPUT_KEYS` entry, `PublicSettings` option always filled in the open build, `FocusedFieldDetection` capability refusing writes outside macOS, toggle only on macOS.
- **Warnings treated:** `recheck()` rerun after an `Insert` verdict; explicit per-application bundle list including Jump Desktop; Linux decision row no longer shadows macOS `FrontmostAtEnd`; surface arbiter defined in `overlay.rs`, `end_indicator` never hides the recovery window; the indicator's delivered 006 destination notice replaced by the bubble; non-activating foreign panel recorded as a known limit; one AX threshold explained; failed AX spike blocks Implemented pending the user; Close during Copy; Windows and Linux automated runs and the design comparison named; history fault through `HistorySink`; 007 timer boundaries left to 007's suite; Next Action gates; 008 `commit_mutation` statement corrected; owner deltas listed for 006 and 007; decisions under Julien's mandate relied on, with final confirmation.
- **Fact found in code:** delivered 006 shows a ≥4 s destination notice on the indicator for a failed `insertion-outcome` (`src/overlay/notice-model.ts`), which the spec Clarification of 2026-10-14 does not mention; the plan replaces it with the bubble, as spec AC-001 requires.
- Other features' documents are now cited as paths, not links.
- Review: the plan review is stale until an independent reviewer refreshes it.

## 2026-10-15 — [Spec Update]: Julien's decisions and round 7 plan review applied

- **Type:** Spec Update
- **Spec modified:** Yes (Clarifications, Edge Cases; normative AC text unchanged)
- **Code modified:** None (spec.md, plan.md and contracts/recovery-contract.md updated)
- **AC impacted:** AC-001, AC-002, AC-007 (decisions; IDs unchanged)
- **Author:** Claude Code (spec-refine 010 and 010 plan, claude-opus-5-5)
- Applies the round 7 plan review with the decisions validated by Julien (maintainer) on 2026-10-15, who also approved an exceptional review round beyond the plan's round cap. Each decision is written in `## Clarifications` with its `livespec-clarification` marker.
- **AC-002, foreign non-activating panel (blocking):** accepted limit. When a non-activating panel of another process (Spotlight, a launcher) has keyboard focus while the destination stays frontmost, the classifier, which queries only the destination application and never the system-wide element, cannot see it and may show the bubble instead of inserting. No text is lost; no system-wide query, so Handy cannot freeze. Also an Edge Case; the plan's Step 1 calibration matrix refers to it.
- **AC-001, late read (unapproved business scope):** new clarification adopting `.specs/features/007-smart-paste/spec.md` AC-012: after "not read", a late clipboard read makes the bubble state that the text may have been pasted after all; the outcome stays "not read" for automatic Enter. The plan's bubble update is now required by 010's spec.
- **AC-007 (ambiguities):** with the gate open on Windows and Linux, reading the key returns `false`; writes are refused `validation_failed` with the platform reason (`unsupported_platform`).
- **Earlier decisions** marked "to be confirmed at the final review" (hover hint and closing state, per-application strategies, Windows reliable paste detail, `paste-error` replacement and macOS-only setting, paste method none, no focused element) are now "validated by Julien on 2026-10-15".
- **Plan warnings and infos treated:** `PRESENTATION_HANG_LIMIT` (10 s) gives a surface that never answers a terminal `Unpresentable` (window destroyed, one `paste-error`, late render ignored), with a test; Step 7 modifies `.github/workflows/test.yml` (four feature combinations on Linux, gate-open `cargo check` on Windows); the 008 names row says only `LiveSettingsEffects` was never shipped; the 300 ms latency is measured from the recovery decision in plan and contract; `OUTPUT_KEYS` has its own fixed length per gate state.
- Review: the spec and plan reviews are stale until the round 8 independent review refreshes them.

## 2026-10-15 — [Feature]: Recovery bubble implemented behind the destination-return gate (automated evidence; native proofs pending with Julien)

- **Type:** Feature
- **Spec modified:** No (status Planned → In Progress through finalize; requirements unchanged)
- **Code modified:** recovery service, manager, decision table, bounded AX classifier, Tauri surface and macOS/Windows/Linux window adapters with the macOS drag source in `src-tauri/src/recovery/`; 006 integration (`destination/{recovery,presentation,delivery,guard,insertion,outcome,manager,app}.rs`: presentation receipt, typed `no_text_field`, recheck after `Insert`, one report per failure); 007 deliberate Copy through the actor and late-read listener (`smart_paste/{actor,coordinator,dispatch,app}.rs`); 008 setting `show_recovery_bubble_without_text_field` with `FocusedFieldDetection` (`settings.rs`, `settings_service/`); indicator arbiter in `overlay.rs`; recovery commands; recovery window UI in `src/overlay/recovery-*`, setting row in the output settings, 26 locales; README; CI four combinations and Windows check
- **AC impacted:** AC-001 to AC-007 partial (automated evidence only), SC-001 automated, SC-002 pending (native), SC-003 automated form only
- **Author:** Claude Code (spec-implement, claude-opus-5-5)
- Commits on `feature/010-recovery-bubble`: `6d8288e9`, `52dbf1eb`, `1510e59d`, `c2a95ed2` and the documentation commit of this entry.
- Gate closed (default build): identical to `dev` (`paste-error`, no bubble, no classification, key `not_found`, no toggle); tested in the four feature combinations.
- Differences from the plan: commands registered in both gate states (one `bindings.ts`); drag started from the webview handle on the panel's content view; setting frozen at the dictation start and bound at commit; unsupported edit attributes count as not settable for a known non-text role; the bubble ends notice holds and hides the indicator (precedence to confirm); presentation hand-off in `destination/presentation.rs`; Penflow design profile BLOCKED (`code-ir.json` absent), UI built from the mockups and contract test IDs.
- Native macOS proofs (required before Implemented) and the Windows and Linux release evidence are listed in `progress.md` and `.specs/native-proofs-checklist.md` (section 010); none is claimed.

<!-- finalize:spec-implement:2026-10-15:0ab0ed79 -->

## 2026-10-17 — [Bugfix]: the bubble follows a held notice that grows, and a refused drag is placed again

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** `src-tauri/src/{overlay_indicator.rs,overlay_surface.rs,overlay_surface_tests.rs}`, `src-tauri/src/recovery/surface.rs`, `src-tauri/src/destination/{delivery.rs,delivery_tests.rs,recovery.rs,recovery_tests.rs}`, `src-tauri/tests/{recovery_lifecycle.rs,recovery_payload.rs,recovery_native.rs,recovery/support.rs}`
- **AC impacted:** FR-002 (placement beside a held notice), AC-003 (inserted text kept with the trailing space); requirements unchanged
- **Author:** Claude Code (claude-opus-5-5), branch `fix/010-bubble-placement` from `dev` `96eb3e1e`
- Answers the round 3 delivery review and audit. A 005/007 notice is measured after its hold started, so a refusal decided in milliseconds (AI failure, then the app quit) could present the bubble beside the base pill, then the notice's extent grew the indicator over it. `set_overlay_extent` now places the indicator then the bubble while a notice holds it; moving the bubble never changes the indicator's extent, so there is no placement loop. `show_again` places the bubble with the same rule after a refused drag.
- `deliver` reads its settings snapshot into a named `FinalText { text, append_trailing_space }` (no anonymous tuple) and is back to 30 lines; `publish` hands it to `RecoverySlot::retain_failed`. The setting is tested from the settings snapshot to the slot, on and off.
- The AC-003 payload test moved from `tests/recovery_lifecycle.rs` (now 375 lines) to `tests/recovery_payload.rs`.
- Native checklist (section 010): the notices item now also quits the app right after the AI failure and expects the bubble to follow a growing notice; the drag item expects a refused drag to come back beside a held notice. Native proofs stay pending with Julien. Verification: `proofs/fix-placement/summary.txt`.

## 2026-10-28 — [Bugfix]: the bubble is shown again (`PanelNotFound`), and a failed insertion without bubble is a notice of the indicator (native session N17)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** `src-tauri/src/recovery/platform/macos.rs`, `src-tauri/tests/recovery_native.rs`, `src/App.tsx`, `src/hooks/use-overlay-none-notices.ts`, `src/overlay/{notice-model.ts,use-overlay-notices.ts,overlay-notices.tsx,indicator-screen.ts}`, tests `src/overlay/{notice-listeners.test.ts,indicator-models.test.ts,indicator-render.test.tsx}`
- **AC impacted:** FR-002, AC-001, AC-003 (requirements unchanged)
- **Author:** Claude Code (spec-fix, claude-opus-5-5), branch `fix/n17-recovery-panel` from `dev` `2a9e4866`
- Cause: `platform::macos::create_window` returned `panel.to_window()`. In `tauri-nspanel` v2.1 (`src/panel.rs`, `to_window`) this is a conversion, not a getter: it removes the panel from the plugin store (`remove_webview_panel`), sets the window class back to `NSWindow` and marks it released when closed. `show` then looked the panel up by label (`get_webview_panel`) and always got `PanelNotFound`, so every presentation failed (`Recovery window could not be shown: PanelNotFound`, then `Unpresentable`), in every build with the `destination-return` gate open. The creation now keeps the panel registered and reads its webview window back by label; `tests/recovery_native.rs` (`the_recovery_panel_stays_registered_after_its_creation`) fails on the previous code.
- `paste-error` (no bubble took the failure: gate closed, bubble unpresentable, legacy and smart paste paths) is now a notice of the recording indicator held by `hold_overlay_for_notice`, like the 005/007/N1 notices, and a toast of the settings window only with the overlay style none. When the bubble is presented, `paste-error` is not emitted (`presentation::toast_needed`), so there is no double report. The indicator screen `recording-indicator-paste-failed` is not yet in the Penflow contract (generated from the `.pen` canvas), as for N1.
- Native proof pending with Julien: release build with `--features destination-return,smart-paste-dispatch`, an `ActivationFailed` insertion shows the bubble beside the indicator without activating Handy, Copy works; a forced unpresentable bubble shows the indicator notice with the settings window closed.

## 2026-10-29 — [Spec Update + Bugfix]: the whole bubble is the drag source (native session N24)

- **Type:** Spec Update + Bugfix
- **Spec modified:** Yes (Story 3 independent test and scenarios, AC-004, FR-003, Clarifications decision N24)
- **Code modified:** `src/overlay/{recovery-bubble.tsx,recovery-preview.ts,use-recovery-bubble.ts,recovery-bubble.css}`, tests `src/overlay/{recovery-preview.test.ts,recovery-bubble-render.test.tsx}`, `tests/recovery-bubble.spec.ts`
- **AC impacted:** AC-004, FR-003
- **Review:** the semantic spec review cache was already stale before this edit (`livespec validate --progression implement` on `dev` `b258ed7e` returned `clarify_not_ready:stale_or_incomplete_spec_review`); this normative edit also invalidates it. Refresh the independent spec review before the next readiness gate.
- **Author:** Claude Code (spec-fix, claude-opus-5-5), branch `fix/n24-drag-whole-bubble` from `dev` `b258ed7e`
- Julien: grabbing exactly the "Drag into a field" button is impractical; grabbing the whole bubble is more instinctive. The pointer gesture moved from the handle to the bubble; Copy, Why? and Close are marked as controls and never arm it; the 3 px threshold keeps a click from being a drag; the grab cursor shows over the drag surface. The native path is unchanged, so the payload, receipts and `drop accepted` log are the handle's.
- Native proof pending with Julien: checklist item "Toute la bulle est la source du glisser (N24)" in `.specs/native-proofs-checklist.md`.

## 2026-10-29 — [Spec Update + Bugfix]: Escape during a drag dismisses the bubble (native session N25)

- **Type:** Spec Update + Bugfix
- **Spec modified:** Yes (Story 3 independent test, scenario and state diagram, AC-004, FR-003, Clarifications decision N25; plan state diagram)
- **Code modified:** `src-tauri/src/recovery/{types.rs,manager.rs,service.rs,surface.rs}`, `src-tauri/src/recovery/platform/{mod.rs,macos_drag.rs,macos_escape.rs}`, `src-tauri/Cargo.toml` (`block2`, already in the lock through `objc2-app-kit`), tests `src-tauri/src/recovery/manager_tests.rs`, `src-tauri/tests/{recovery_lifecycle.rs,recovery_payload.rs,recovery_privacy.rs}`
- **AC impacted:** AC-004, FR-003
- **Review:** this normative edit invalidates the semantic spec review cache (already stale since N24); refresh the independent spec review before the next readiness gate.
- **Author:** Claude Code (spec-fix, claude-opus-5-5), branch `fix/n25-escape-cancels-drag` from `dev` `91422f7b`
- Julien: when dragging the bubble, Escape must cancel the action and the bubble must go away. macOS already cancels the drop on Escape (nothing inserted); Handy now tells Escape from a release without a target by watching Escape for the session only (global and local key-down monitors, plus the event ending the session), removed at the end of every session. Escape closes the bubble through Close's retirement (history untouched) with the log `Recovery bubble N: drag cancelled by Escape, dismissed`; a release without Escape keeps the bubble for another attempt; an accepted drop stays `drop accepted`.
- Native proof pending with Julien: checklist item "Échap pendant un glisser ferme la bulle (N25)" in `.specs/native-proofs-checklist.md`.

