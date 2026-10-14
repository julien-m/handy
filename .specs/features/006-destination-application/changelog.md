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

## 2026-10-14 — [Feature]: Non-UI implementation of destination application (partial)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** `src-tauri/src/settings_service/` (Step 0: live operations, platform capability, error details; Step 3: `return_to_dictation_app` key), `src-tauri/src/destination/` (new: session manager, contracts, macOS/Windows adapters, orchestrator, guard, outcome, recovery), `src-tauri/src/commands/destination.rs`, `src-tauri/src/{actions,clipboard,utils,settings,lib,cli}.rs`, `src/bindings.ts`, `src/stores/settings-errors.ts`, 26 locales, tests (`destination_cli.rs`, `destination_platform.rs`, settings test updates), README, 008 contract and schema
- **AC impacted:** AC-001, AC-002, AC-004 (backend), AC-005, AC-006, AC-007, AC-008 — all partial, none certified; AC-003 not started (UI)
- **Author:** Claude Code (spec-implement 006, claude-opus-5-5)
- Commits: `0867599` (Step 0), `7a95f0c` (Step 2, Step 1 read-only part), `b98ee80` (Step 3), `4823d8f` (Step 4 and the non-UI part of Step 6), `3645b28` (implementation map).
- Not done: UI (badge, menu, reason display, settings toggle and its write command), 007/009/010 adapters, native activation/focus/timing evidence; Windows compile-only; Linux and Windows native checks pending release evidence. See [implementation](implementation.md) and [progress](progress.md).

## 2026-10-14 — [Spec Update]: Text-only history entry when the audio recording fails

- **Type:** Spec Update
- **Spec modified:** Yes (sections: AC-005, Clarifications, frontmatter `updated`)
- **Code modified:** None (plan.md and contracts/destination-contract.md updated)
- **AC impacted:** AC-005 (history clause; ID unchanged)
- **Author:** Claude Code (spec-refine 006 + plan, claude-opus-5-5)
- Julien's decision 2026-10-14: a dictation whose WAV cannot be saved, whatever the cause, still gets a text-only history entry. Recorded as a dated decision under `## Clarifications` (Clarify inventory empty).
- AC-005 now reads "the transcript stays in history whenever the history save succeeds, as a text-only entry when the audio recording could not be saved" instead of "whenever history saving succeeds as today". Story 4, SC-002 and the cancellation behavior are unchanged.
- Plan: the history save no longer depends on the WAV; `InsertionOutcome.history_status` comes only from the actual `save_entry` result (`saved` includes a text-only row; `failed` on save_entry failure; `not_saved` when no write was attempted); Step 6 removes the `wav_saved` gate and its history matrix covers WAV failure saved text-only. The shared history change (nullable `file_name` migration, gate removal, guards for rows without audio) is built once by whichever of 006 Step 6 and 005 Step 7 runs first. Contract history line aligned.
- Review: the spec and plan reviews are stale until an independent reviewer refreshes them.

<!-- finalize:spec-refine:2026-10-14:2190dbf9 -->

## 2026-10-14 — [Feature]: Sync with dev, settings UI command and history rule (partial)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** merge `3b3a27a` (`settings_service/{errors,protocol,render,domains/mod,domains/shortcuts}.rs`, `shortcut/live_change.rs`, `src/stores/settings-errors.ts`, `src/bindings.ts`, README, 008 schema and contract); `3fd745d` (`shortcut/mod.rs`, `lib.rs`, `settings_service/sequence_destination_tests.rs`, `src/stores/settingsStore.ts`, `settings-store.test.ts`, `bindings.ts`); `8f374d4` (`managers/{history,history_store,mod}.rs`, `destination/{history,delivery,outcome,mod}.rs`, `actions.rs`, `commands/history.rs`, `lib.rs`, `tray.rs`, `tests/destination_recovery.rs`, `HistorySettings.tsx`, `bindings.ts`, 26 locales)
- **AC impacted:** AC-005 (history clause), AC-007, AC-008 (UI command) — still partial, none certified
- **Author:** Claude Code (spec-implement 006, claude-opus-5-5)
- The history change shared with 005 Step 7 is built once here (nullable `file_name`, WAV gate removed for the transcript, guards for rows without audio). Not done: visual UI (badge, cross, menu, toggle, reason display), 007/009/010 adapters, native evidence. See [progress](progress.md#sync-run--notes-2026-10-14) and [implementation](implementation.md).

## 2026-10-14 — [Fix]: Delivery review fix round 1 — rollout gate and review warnings

- **Type:** Fix
- **Spec modified:** No
- **Code modified:** `422063a` (`Cargo.toml` feature `destination-return`; `destination/{rollout,macos_focus,delivery_tests}.rs` new; `destination/{app,delivery,guard,insertion,macos,windows,workspace,manager,context}.rs` and their tests; `settings_service/{error_reason.rs (new), errors.rs, platform_capability.rs, mod.rs}`; `settings.rs`, `clipboard.rs`, `actions.rs`; `tests/{destination_platform,destination_recovery}.rs`; `App.tsx`, 26 locales); `c2953cd` (`tests/destination_cli.rs`); contract updated (insertion method shape, live guard read, `NoTextField`)
- **AC impacted:** AC-002, AC-005, AC-007, AC-008 — still partial, none certified
- **Author:** Claude Code (delivery review corrections, claude-opus-5-5)
- Destination return is now behind the `destination-return` Cargo feature, off by default: the default build dictates exactly as `dev` and refuses `return_to_dictation_app` as `unsupported_platform`; the text-only history rule stays active. Opening conditions: native macOS activation and guard proof with Julien, Penflow-certified UI ([implementation, Rollout gate](implementation.md#rollout-gate)).
- Behind the gate: live Accessibility foreground read for the guard and the activation wait (macOS), one settings snapshot per insertion, truthful worker-failure outcome with the transcript retained, a "not saved" toast on a double failure, Windows hosted UWP apps resolved by their core window process. Also `RecoveryReason::NoTextField`, `errors.rs` split under 400 lines, the two new clippy warnings removed. Deviation recorded: `InsertionPort::dispatch(guard)` kept, contract updated. Details: [progress](progress.md#delivery-review-fix-round-1--notes-2026-10-14).

