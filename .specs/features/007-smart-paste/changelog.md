# Changelog - 007-clipboard-recovery-window

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:aa9ea82f -->

### 2026-10-12 — ## Plan: Clipboard recovery technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle metadata only: Planned; functional content unchanged)
- **Code modified:** None (plan.md and planning evidence only)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex (gpt-6, spec-plan --auto)
- **Artifacts:** Inspect [the plan](plan.md) for 7 implementation steps, 3 diagrams, shared 008 output-domain integration and explicit native/UI evidence gates.
- **Design:** Approved lightweight UI decision retained; no mockup generated, no Penflow certification claimed.

<!-- finalize:spec-plan:2026-10-12:1f19f822 -->

## 2026-10-12 — [Spec Update]: Scope replaced by smart paste

- **Type:** Spec Update
- **Spec modified:** Yes (scope replaced — all sections; previous spec and plan moved to `superseded/`)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-007 retired (not reused); AC-008 through AC-016 created
- **Decision:** Julien, 2026-10-12, relayed by Codex: implement Spokenly-inspired smart paste instead of the 10–15 s clipboard recovery window; keep number 7
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-refine)

<!-- finalize:spec-refine:2026-10-12:3471b770 -->

### 2026-10-12 — ### Plan: Smart paste technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status only: Draft to Planned via finalize; normative requirements unchanged)
- **Code modified:** None (plan-only run)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol
- **Plan:** 9 implementation steps; 2 sequence diagrams, 1 state diagram, 1 logical ER diagram.
- **Decisions:** asynchronous AppKit/key worker; bounded read-only snapshot helper; guarded restoration and copy mode; 008 output-domain reuse and one-shot migration; six truthful outcomes.
- **Evidence boundary:** UI/Penflow deferred by user until shared pre-spec-ship pass; native/runtime proof required later.

<!-- finalize:spec-plan:2026-10-12:307d8e77 -->

### 2026-10-13 — Plan: Smart paste technical plan fully rewritten from 2026-10-13 spec

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** None (plan-only)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol, spec-plan --auto
- **Plan:** Read [the rewritten plan](plan.md) for 9 implementation steps, 2 sequence diagrams, 1 state diagram and 1 transient logical ER diagram.
- **Decisions:** AX-first verified insertion; restoration only after post-key read plus quiet/cap; not-read snapshot discarded; Jump 3 s decision; bounded native calls/snapshots; shared 008 transaction/output domain and one-time migration.
- **Evidence boundary:** Independent semantic review and progression recorded separately; native/AppKit/AX late-write/compatibility and shared Penflow UI gates remain mandatory before release. No mockups generated.

<!-- finalize:spec-plan:2026-10-13:17322c07 -->

## 2026-10-13 — [Feature]: Native feasibility checkpoint

Recorded reproducible public macOS AppKit, AX and input-source probes with bounded subprocesses and typed reports. Targeted Cargo integration testing builds the real Apple Intelligence bridge; the probe itself tests Swift/AppKit rather than AI inference.

AppKit caching and delayed AX mutation counterexamples remain explicit: callback-only quiet timing is unproven and affected production clipboard wiring is held. This checkpoint does not certify complete acceptance or application/layout compatibility. Settings service, revisions, CLI, migration and UI integration wait for 008.

<!-- finalize:spec-feature:2026-10-13:72e02927 -->

## 2026-10-13 — Feature: Step 1 native feasibility closed (core-first run)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/tests/smart_paste_native.rs, scripts/smart-paste-native-*.ts, evidence probes
- **AC impacted:** AC-008, AC-011, AC-016, AC-017 (feasibility only, no acceptance)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Evidence:** AppKit receipt regenerated against the current harness; AX and TIS receipts match their probe hashes. Raw gate outputs in [proofs/claude](proofs/claude/).

## 2026-10-13 — Feature: Step 2 pure lifecycle and outcome contracts

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/src/lib.rs, src-tauri/src/smart_paste/{mod,ids,outcome,timing,clock,lifecycle,settlement}.rs, src-tauri/tests/smart_paste_lifecycle.rs
- **AC impacted:** AC-008, AC-009, AC-012, AC-013 (implemented in pure form, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)

## 2026-10-13 — Feature: Step 3 bounded verified AX insertion

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/src/smart_paste/{ax_port,direct,macos_ax,mod}.rs, src-tauri/tests/smart_paste.rs, src-tauri/tests/smart_paste/{direct,fakes,lifecycle}.rs, src-tauri/tests/smart_paste_native_ax.rs
- **AC impacted:** AC-017, AC-018 (implemented, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Evidence:** [native AX adapter receipt](evidence/native-ax-adapter.json) on synthetic offscreen fields; real applications pending release evidence.

## 2026-10-13 — Feature: Step 4 bounded snapshot and on-demand provider

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/src/main.rs, src-tauri/Cargo.toml, src-tauri/src/smart_paste/{snapshot,snapshot_protocol,snapshot_helper,macos_snapshot_stream,pasteboard_port,macos_pasteboard,main_thread,mod}.rs, src-tauri/tests/smart_paste/snapshot.rs, src-tauri/tests/smart_paste_native_pasteboard{.rs,/scenario.rs,/support.rs}
- **AC impacted:** AC-008, AC-009, AC-013 (adapter implemented, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Evidence:** [native pasteboard receipt](evidence/native-pasteboard-adapter.json) on an isolated named pasteboard. Findings: AppKit lists derived types whose conversion needs the owner process (saved as read, they consume part of the 200 ms budget); the first execution of a newly written binary costs seconds, so the first helper spawn of an installed Handy.app is pending release evidence.

## 2026-10-13 — Feature: Step 6 per-application strategy and keyboard handling

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/Cargo.toml, src-tauri/src/input.rs, src-tauri/src/smart_paste/{strategy,keyboard,key_events,macos_keyboard,macos_cf,macos_ax,mod}.rs, src-tauri/tests/smart_paste/{keyboard,fakes}.rs, src-tauri/tests/smart_paste_native_keyboard.rs
- **AC impacted:** AC-010, AC-011 (implemented, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Evidence:** [read-only keyboard receipt](evidence/native-keyboard-adapter.json); selection, key posting and remote applications pending release evidence. Step 6 is committed before step 5 so the coordinator composes finished ports.

## 2026-10-13 — Feature: Step 5 serialized coordinator

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/src/smart_paste/{coordinator,coordinator_chord,pending,request,actor,settlement,snapshot,ax_port,mod}.rs, src-tauri/tests/smart_paste/{coordinator,chain,actor,privacy,rig,fake_pasteboard}.rs
- **AC impacted:** AC-009, AC-010, AC-012, AC-013, AC-017, AC-018 (composed, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Notes:** The actor is not yet owned by the app nor called by the paste pipeline: that wiring consumes the 006 destination context (step 7). The pre-publication AX guard runs after the snapshot so a write applied during the capture is still verified with the clipboard untouched.

## 2026-10-13 — Feature: smart_paste persisted paste-method value (minimal, additive)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src-tauri/src/settings.rs, src-tauri/src/clipboard.rs, src/bindings.ts
- **AC impacted:** AC-014, AC-015 (still waiting for 008: migration, default, CLI domain, validation, UI)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Notes:** The value is not a default and cannot be selected from the UI yet; the pipeline refuses it explicitly instead of falling back to the classic fixed-delay paste. Progress and implementation maps list what waits for 006 and 008 and the exact pending release checks.

## 2026-10-13 — Bugfix: step-1 native test drains the Bun pipes

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src-tauri/tests/smart_paste_native.rs
- **AC impacted:** None (evidence harness only)
- **Author:** Claude Opus 5.5 (implementation agent, orchestrated run)
- **Notes:** The opt-in AppKit test polled Bun without reading its stdout/stderr; Bun busy-loops when a pipe fills, so the test hit its 120 s deadline. Both pipes are now drained concurrently; the AppKit receipt was regenerated.

## 2026-10-13 — [Spec Update]: Concealed or transient clipboard content is never restored

- **Type:** Spec Update
- **Spec modified:** Yes (AC-009 wording, Story 1 Gherkin scenario and flowchart, Edge Cases, Clarifications; requirement IDs preserved)
- **Code modified:** None in this entry (see the next entry)
- **AC impacted:** AC-009 (decision recorded with `persist_clarification_decision`, key `decision:ac-009:concealed-or-transient-content-not-restored`); AC-012/AC-013 known copy-mode limitation listed in Clarifications, to confirm with Julien
- **Decision:** Julien, 2026-10-13: content marked `org.nspasteboard.ConcealedType` or `TransientType` is not restored after a dictation; other content is restored, always current-host-only. Reason: a dictation changes `changeCount`, so a password manager's automatic clearing no longer fires
- **Review:** the spec changed after review; the dependent spec review is stale and the final delivery review covers it. `spec-refine` refuses specs In Progress; this edit was made on explicit instruction with the same discipline (decision ledger, preserved IDs, changelogs)
- **Author:** Claude Opus 5.5 (implementation agent, fix round 1)

## 2026-10-13 — Bugfix: Fix round 1 after the two independent audits

- **Type:** Bugfix
- **Spec modified:** No (see the previous entry)
- **Code modified:** src-tauri/src/smart_paste/{actor,actor_tests,coordinator,coordinator_chord,direct,macos_pasteboard,macos_snapshot_stream,mod,pasteboard_port,pasteboard_types,pending,snapshot,snapshot_helper,snapshot_helper_tests,snapshot_protocol,warm_helper}.rs, src-tauri/src/{clipboard,settings}.rs, src-tauri/tests/smart_paste{.rs,/*}, src-tauri/tests/smart_paste_native_pasteboard{.rs,/*}
- **AC impacted:** AC-008, AC-009, AC-012, AC-013, AC-017 (implemented, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, fix round 1)
- **Evidence:** [fix-round proofs](proofs/claude-fix1/), regenerated [pasteboard](evidence/native-pasteboard-adapter.json) and [AX](evidence/native-ax-adapter.json) receipts with run-time binary hashes. Per-finding dispositions are in [the implementation map](implementation.md#independent-audit-fix-round-1-2026-10-13). Release blockers R-1/R-2, unauthenticated receipts and late AX writes stay open.

## 2026-10-14 — Feature: settings integration on 008, behind a rollout gate

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** merge of `dev` (008) as `5b4e3e2`; `3b66ff1`: src-tauri/src/smart_paste/{setting_policy,setting_policy_tests,mod}.rs, src-tauri/src/{settings,lib}.rs, src-tauri/src/settings_service/{descriptor_data,validation,errors,public}.rs and domains/output.rs, src-tauri/src/shortcut/mod.rs, src-tauri/Cargo.toml (`smart-paste-dispatch` feature), src-tauri/tests/smart_paste_settings{.rs,/*}, 008 tests and checked schema, src/components/settings/{PasteMethod.tsx,paste-method-options.ts,debug/ReliablePaste.tsx}, src/components/ui/{Dropdown.tsx,dropdown-navigation.ts}, src/hooks/use-smart-paste-availability.ts, frontend tests, 26 locales, src/bindings.ts
- **AC impacted:** AC-014, AC-015, has_dropdown, is_keyboard_navigable (implemented, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, integration run)
- **Notes:** The pipeline still refuses smart paste until step 7 (006), so the migration, the macOS default and the selector option are ready, tested in both states, and inactive until step 7 enables `smart-paste-dispatch` with the wiring. In the shipped build `smart_paste` is refused with `validation_failed` on every surface. See [the rollout gate](implementation.md#rollout-gates) and [the proofs](proofs/claude-integration/).

## 2026-10-14 — [Plan Update]: history without the WAV gate (006 rule) and delivered 008 settings names

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (AC-013 and AC-014/AC-015 planning text only, IDs unchanged)
- **Author:** Claude Code (spec-refine 007 plan, claude-opus-5-5)
- Eligibility: `spec-refine` refuses a plan once `implementation.md` exists; this alignment was made on explicit instruction with the same discipline (diff limited to the plan, preserved IDs, changelogs). Step 7 (pipeline wiring), which the history text governs, is not implemented yet.
- History: Julien's decision 2026-10-14, carried by the 006 spec and plan on `dev` (commit `306c368`, `.specs/features/006-destination-application/plan.md`): a dictation whose WAV cannot be saved still gets a text-only history entry; `history_status` comes only from the actual `save_entry` result. The plan no longer asks Step 7 to "preserve WAV-gated history": 007 does not reintroduce the `wav_saved` gate and never touches history (AC-013). Current-state note, history paragraph, Step 7 action, regression gate and risk row aligned.
- 008 names: `LiveSettingsEffects` (never shipped) removed; the plan now names the delivered `src-tauri/src/settings_service/` API: `SettingDescriptor` rows in `descriptor_data.rs`, `OutputDomain`/`OUTPUT_KEYS`, `SettingsMutation` from `validate_cli`/`validate_ui`, the live commit `LiveSettings::commit_scalar` (UI through `tauri_adapter::write`, CLI routed to the running instance), offline `SettingsRepository::commit_mutation`, live effects `effects::apply_changed` and the `settings-changed` event, startup migration through `settings::apply_settings_migrations`. The Gherkin and sequence diagram no longer present `commit_mutation` as the live UI/CLI commit; the Step 8 file list matches the files actually changed.
- 008's plan is cited as a code path instead of a Markdown link.
- Plan review preparation at 180,000 characters: 1 batch, no error.
- Review: the plan review is stale until an independent reviewer refreshes it.

<!-- finalize:spec-refine:2026-10-14:d424af86 -->

## 2026-10-14 — Feature: step 7, smart paste wired into dictation through 006 (behind its gate)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** merge of `dev` as `e5c2b0f` (004, 006, bindings exclusion, clipboard test fix; inventory counts 65, one `[features]` table, regenerated `bindings.ts` and 008 checked schema); `e7fa570`: src-tauri/src/smart_paste/{app,destination_bridge,destination_bridge_tests,dispatch,dispatch_tests,exit_gate,macos_ports,mod,outcome,request,coordinator,direct,setting_policy,actor_tests}.rs, src-tauri/src/destination/delivery.rs, src-tauri/src/{clipboard,lib,settings}.rs, src-tauri/Cargo.toml (comment), src-tauri/tests/smart_paste{.rs,/pipeline.rs,/chain.rs,/rig.rs}, src-tauri/tests/settings_validation/contracts.rs, src/App.tsx, src/hooks/use-smart-paste-hand-over.ts, 26 locales
- **AC impacted:** AC-010, AC-012, AC-013, AC-018 through dictation; AC-016 procedure written (implemented behind the gate, certification pending)
- **Author:** Claude Opus 5.5 (implementation agent, step 7 run)
- **Notes:** Smart paste is 006's `InsertionPort` (`dispatch(guard)`): the first guard check builds the context, 006's guard is registered for every later destination check of the actor, and the terminal `PasteOutcome` becomes `method_outcome`, status and disposition. Without a 006 session it inserts into the application in front through the same live foreground source. `smart-paste-dispatch` stays closed by default: closed, dictation is identical to `dev`. The four gate combinations, the exit settlement, the Enter (009) and late-read (010) hooks and the Screen Sharing toast are described in [the rollout gates](implementation.md#rollout-gates); the AC-016 matrix is [a procedure for Julien](implementation.md#native-acceptance-procedure-ac-016-with-julien), not run. Proofs: [claude-step7](proofs/claude-step7/).

## 2026-10-14 — [Feature]: Hand-over message on the recording indicator (UI pass)

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** src/overlay/* (notices), src/hooks/use-overlay-none-notices.ts (replaces use-smart-paste-hand-over.ts), App.tsx, smart_paste/app.rs (comment)
- **AC impacted:** AC-012 (presentation); browser-tested, native proof pending
- **Author:** Claude Code (UI pass, feature/ui-overlay 69a6ee73)
