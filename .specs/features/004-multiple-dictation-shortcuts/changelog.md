# Changelog - 004-multiple-dictation-shortcuts

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-006 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:bf4ea80d -->

### 2026-10-12 — ### Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status/updated only through finalize; requirements unchanged)
- **Code modified:** None (plan.md and local CLI contract created)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol
- **Artifacts:** plan.md; contracts/shortcuts-cli.md
- **Scope:** Seven implementation steps, two sequence diagrams, one state diagram and one logical ER diagram; UI production deferred by approved program decision.

<!-- finalize:spec-plan:2026-10-12:218e2fcd -->

### 2026-10-12 — Plan: Technical plan refreshed after approved legacy-conflict decision

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** None (plan.md revised)
- **AC impacted:** AC-002, SC-003 planning treatment aligned with approved specification; no acceptance behavior changed
- **Author:** Codex gpt-6
- **Artifacts:** plan.md; existing contracts/shortcuts-cli.md retained
- **Scope:** Seven implementation steps, two sequence diagrams, one state diagram and one logical ER diagram. Preserve pre-upgrade legacy registration/event behavior and visible conflicts; reject new conflicting candidates. Independent reviews and progression verified separately; no visual/runtime certification.

<!-- finalize:spec-plan:2026-10-12:12d2f055 -->

## 2026-10-13 — [Feature]: Core multiple shortcuts without the 008 settings service

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** settings.rs, shortcut/{mod,handler,tauri_impl,handy_keys,identity,conflicts,action_shortcuts,edits,migration_tests}.rs, secure_input.rs, transcription_coordinator.rs, lib.rs, bindings.ts, settings shortcut list UI, settingsStore.ts, useSettings.ts, 26 locales
- **AC impacted:** AC-001, AC-002, AC-003, AC-004, AC-005 (implemented, certification pending); AC-006 Waiting for 008
- **Author:** Claude Code (core-first implementation agent)
- **Scope:** Ordered 1–4 shortcuts per transcription action with additive storage; action and shortcut identity through both backends, Carbon shadows and the coordinator; shared normalized conflict checker with legacy conflicts kept and flagged; settings list UI on existing commands. CLI, revision commit, quiescence lease and inactive-action OS probe Waiting for 008. Checks before the code commits: Rust 338 then 340 PASS / 16 ignored, Clippy warnings identical to `dev` ([dev baseline](proofs/claude/dev-baseline/)), fmt, ESLint, Prettier, tsc, translations and repository conventions PASS ([core](proofs/claude/commit-1-core/), [busy ownership fix](proofs/claude/commit-2-busy-ownership/)).
- **Fix (same day):** a press remembered while busy keeps the ownership it was remembered with (toggle = action), and external triggers act on the action, preserving the pre-004 mixed external/keyboard parity.

## 2026-10-13 — [Bugfix]: Independent audit fixes (fix1)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** shortcut/{registration_delta,implementation_switch,edits,tauri_impl,mod,action_shortcuts}.rs (+ tests), secure_input.rs, secure_input_shadow.rs, lib.rs, bindings.ts, settingsStore.ts, lib/shortcut-edit-failure.ts, hooks/useShortcutFailureText.ts, shortcut-list.tsx, shortcut-list-model.ts (+ test), GlobalShortcutInput.tsx, HandyKeysShortcutInput.tsx, debug/KeyboardImplementationSelector.tsx, 26 locales
- **AC impacted:** AC-002, AC-004, AC-005 (implemented, certification pending; no acceptance claimed)
- **Author:** Claude Code (implementation agent, fix pass under supervision)
- **Scope:** Fixes the [independent audit](proofs/claude/independent-audit.md) minors m1, m2, m3, m5, m6, m7 and nits n2, n4, n7; m4 (recorder states) stays with the Penflow/UI pass; n1, n3, n5, n6 documented in [progress](progress.md#audit-fix-1-2026-10-13). Tauri probe of disabled post-processing shortcuts before saving; inherited-conflict slots handed to the surviving action instead of unregistered; keyboard switch repair by destination parser, keeping and reporting the conflicts it introduces (new `ImplementationChangeResult.conflicts`, translated toast); typed `BindingResponse.edit_error` so cancel refusals are translated and a refused reset is shown; conflict warnings refreshed on any action change, fetch failures shown; delta/rollback and Carbon shadow tests; committed conventions outputs reduced to a summary and hash. Behavior note: the HandyKeys first-start switch now also emits `settings-changed`. Checks: Rust 363 PASS / 16 ignored, `shortcut::` 61, coordinator 50, `secure_input` 5, fmt, Clippy warnings identical to `dev`, ESLint, Prettier, tsc, translations, keyboard and list tests, repository conventions PASS (`new_count` 0) ([fix1 proofs](proofs/claude-fix1/)).

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Regression, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-13 — [Bugfix]: Independent re-audit fixes (fix2)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** shortcut/{registration_delta,edits,mod,tauri_impl}.rs (+ tests), secure_input_shadow.rs (test), bindings.ts, settingsStore.ts, useSettings.ts, PostProcessingToggle.tsx, 26 locales; `implementation.md` structure
- **AC impacted:** AC-002, AC-004, AC-005 (implemented, certification pending; no acceptance claimed)
- **Author:** Claude Code (implementation agent, fix pass under supervision)
- **Scope:** Fixes the [independent re-audit](proofs/claude-fix1/independent-reaudit.md) minor M1 and nits N1–N6 (N6 history residue documented: squash merge). The post-processing toggle registers or releases through the registration delta before saving: disabling hands a shared Tauri slot back to Transcribe; enabling leaves a slot held by another action to it, rolls back on refusal and keeps post-processing off with a translated message (`change_post_process_enabled_setting` now returns `ShortcutEditError`). Cancel shortcuts are probed with the OS outside recording (Tauri, not Linux). Probe release retried once; a failed handover release no longer registers the survivor; unknown action ids never receive a slot; the first switch to HandyKeys repairs the lists first; invalid syntax on the legacy recorders is a typed, translated refusal. `implementation.md` gains front matter and the AC mapping heading the validator expects. Checks: Rust 372 PASS / 16 ignored, `shortcut::` 69, coordinator 50, `secure_input` 6, fmt, ESLint, tsc, Prettier, translations, keyboard and list tests, repository conventions PASS (`new_count` 0), `livespec validate` exit 0 ([fix2 proofs](proofs/claude-fix2/)).

## 2026-10-13 — [Bugfix]: Post-processing activation as before, refused shortcuts reported (fix3)

- **Type:** Bugfix
- **Spec modified:** No (`spec.md` and `plan.md` unchanged; the code is aligned on the existing spec)
- **Code modified:** shortcut/{registration_delta,edits,mod}.rs (+ tests, edits tests moved to `edits/tests.rs`), bindings.ts, settingsStore.ts, useSettings.ts, PostProcessingToggle.tsx, shortcut-list.tsx, shortcut-list-model.ts (+ test), KeyboardImplementationSelector.tsx, 26 locales
- **AC impacted:** AC-002 (implemented, certification pending; no acceptance claimed)
- **Author:** Claude Code (fix pass under supervision, claude-opus-5-5)
- **Scope:** Fixes the toggle finding of the [independent spec review](proofs/claude-review/independent-review.md): enabling post-processing always succeeds and is saved, as before this feature (Header, Edge Cases, AC-002), instead of being refused when the OS refuses a shortcut. The slot handover of fix2 is kept. Each refused shortcut is reported by a translated toast and a warning on its row, which stays editable in the Post Processing settings. `plan.md:338` contradicts `plan.md:76` on this point; recorded in [progress](progress.md#independent-spec-review-fix-2026-10-13) for the next plan revision. Checks: Rust 376 PASS / 16 ignored, `shortcut::` 73, coordinator 50, fmt, ESLint, tsc, Prettier, translations, list tests, repository conventions PASS (`new_count` 0), progression implement READY ([fix3 proofs](proofs/claude-fix3/)).

## 2026-10-14 — [Feature]: Integration with 008, shortcuts CLI and prepared shortcut changes

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** merge of `dev` (`22e0f33`); shortcut/{edits,mod,handy_keys,bindings_change,live_change,keyboard_switch,fake_backend}.rs, transcription_coordinator.rs + quiescence.rs, secure_input.rs, settings_service/{sequence,native_changes,tauri_adapter,domains,errors,operations,render,cli,protocol,public,effects,command_error}.rs, cli.rs, tests/shortcuts_cli.rs, settings_cli/settings_domains tests, bindings.ts, settingsStore.ts, settings-errors.ts, shortcut-list-model.ts, PostProcessingToggle.tsx, 26 locales, README, AGENTS.md, 008 checked schema (`e2b0ab5`)
- **AC impacted:** AC-001, AC-004, AC-006 (implemented, certification pending; no acceptance claimed)
- **Author:** Claude Code (implementation agent, claude-opus-5-5)
- **Scope:** Every former "Waiting for 008" item: list edits, the post-processing toggle and the keyboard switch are prepared changes of the 008 service (registration before one-revision commits, compensation on refusal, refused with `shortcut_session_busy` during a dictation through the coordinator quiescence lease); `handy settings shortcuts list/add/remove/reset` with `--json`, `--dry-run`, live-only changes (`instance_required` offline) and stable codes; startup HandyKeys fallback repairs the lists for Tauri. Decisions to confirm and remaining items in [progress](progress.md#integration-with-008-2026-10-14). Checks in [integration proofs](proofs/claude-integration/).

## 2026-10-14 — [Bugfix]: Hardening of the shortcut lists

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/components/settings/shortcut-chip.tsx, record-request-model.ts (new), shortcut-draft-model.ts, shortcut-list-feedback.ts, shortcut-list.tsx, shortcut-list-rows.tsx, use-shortcut-list.ts, GlobalShortcutInput.tsx, HandyKeysShortcutInput.tsx, ShortcutInput.tsx, shortcut-list-model.ts, src/stores/settingsStore.ts, src/hooks/useSettings.ts; removed src/hooks/useShortcutFailureText.ts, src/lib/shortcut-edit-failure.ts
- **AC impacted:** AC-001, AC-002, AC-004
- **Why:** delivery review warnings: a recorder refused at start (Secure Input) left the draft stuck recording; Try again vanished with the next edit; a "Change" request could replay on a remounted row; dead non-capture recorders and store actions.
- **Tests:** shortcut-draft-model, shortcut-list-feedback, record-request-model (new), shortcut-list-render; [proofs](proofs/claude-hardening/summary.txt)
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-14 — [Bugfix]: UI hardening, review round 1

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/components/settings/recorder-lifecycle.ts (new, + test), GlobalShortcutInput.tsx, HandyKeysShortcutInput.tsx, shortcut-chip.tsx, shortcut-draft-model.ts, shortcut-list-render.test.tsx, package.json, 26 locales (two orphan keys removed)
- **AC impacted:** AC-001, AC-004
- **Why:** delivery audit and review of the hardening: a start answered after the chip unmounted could leave shortcuts suspended; the one-shot request wiring had no test; orphan translation keys.
- **Tests:** recorder-lifecycle.test.ts (6), shortcut-list-render.test.tsx (4 new); [proofs](proofs/claude-hardening-fix1/summary.txt)
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-28 — [Bugfix]: Side-aware modifier-only overlap (native session N9)

- **Type:** Bugfix
- **Spec modified:** No (AC-004 refinement on modifier sides pending `spec-refine` 004)
- **Code modified:** src-tauri/src/shortcut/conflicts.rs
- **AC impacted:** AC-004
- **Why:** native session of 2026-10-28, finding N9, Julien's request: the modifier-only overlap now takes the registered side into account; a right-only modifier shortcut (`command_right`) no longer refuses `command_left+j`. A modifier without side still overlaps both sides; equality ignores sides as before.
- **Tests:** conflicts.rs `side_specific_modifier_only_overlaps_only_on_its_side` (both directions), `tauri_modifiers_register_both_sides`; existing tests unchanged
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-28 — [Bugfix]: Explicit Save and Discard for the "Add shortcut" draft (native session N8)

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src/components/settings/{shortcut-draft-model.ts, shortcut-chip.tsx, shortcut-list.tsx, shortcut-list-rows.tsx, shortcut-list-feedback.ts}; keys `settings.general.shortcut.list.{save,discardDraft,unsaved}` in the 26 locales
- **AC impacted:** AC-001 (adding records with the existing recorder)
- **Why:** native session of 2026-10-28, finding N8: the captured draft looked like a saved shortcut and was added by clicking "Add shortcut" again, so Julien thought it was added and Reset cleared it. The draft chip is now dashed with a "Not saved" label; during a draft, Save (enabled only for a valid combination) and Discard replace "Add shortcut"; Reset is disabled while a draft is open; the focus returns to "Add shortcut" after Save or Discard. The prompt shortcuts of 005 (`ShortcutListEditor`) do not use the draft: a capture is added at once, so they are unchanged.
- **Tests:** shortcut-draft-model.test.ts (`listControls`, 6), shortcut-list-render.test.tsx (draft look, Save/Discard/Reset, 4 new or updated)
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-28 — [Bugfix]: Focus on Discard when the draft recorder is refused (native session N8, review)

- **Type:** Bugfix (review of `fix/native-session-ui`)
- **Spec modified:** No
- **Code modified:** src/components/settings/{shortcut-draft-model.ts, shortcut-list-rows.tsx}; tests/shortcut-list-draft.spec.ts and its harness (new)
- **AC impacted:** AC-001
- **Why:** with Secure Input on, the draft opened blocked, "Add shortcut" disappeared and the keyboard focus had no way out.
- **Tests:** shortcut-draft-model.test.ts (blocked draft), tests/shortcut-list-draft.spec.ts

## 2026-10-28 — [Bugfix]: hold-or-toggle measures the hold at the key edges (native session N13)

- **Type:** Bugfix
- **Spec modified:** No (AC-003: the hold threshold now holds whatever the start duration)
- **Code modified:** src-tauri/src/transcription_coordinator.rs, src-tauri/src/transcription_coordinator/{slow_start_tests.rs (new), input_lost_tests.rs, quiescence_tests.rs}, src-tauri/src/shortcut/{handler.rs, handy_keys.rs, tauri_impl.rs}
- **AC impacted:** AC-003
- **Why:** finding N13 of Julien's native session of 2026-10-28: a short tap stopped the dictation at once when the start took longer than the 300 ms threshold, because the key-up was measured when processed after the start
- **Tests:** `slow_start_tests.rs` (injected clock: tap processed after a 400 ms start stays locked; late hold stops; queued auto-repeat cancels the release; busy tap locks)
- **Author:** Claude Code (claude-opus-5-5)
