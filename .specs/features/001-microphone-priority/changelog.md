# Changelog - 001-microphone-priority

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:efcdfaa5 -->

### 2026-10-12 — [Plan]: Technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status only: Draft → Planned via finalize; functional requirements unchanged)
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Summary:** Seven implementation steps, two sequence diagrams, one state diagram and one logical ER diagram. Reuse 008 transactional service and microphones Domain; generation-owned audio resolution, prefix retention, migration, UI/CLI parity and native/QE gates.
- **UI authority:** Common forward-contract/mockup pass deferred before spec-ship per user decision; no Penflow certification claimed.
- **Author:** Codex (spec-plan --auto; gpt-6)

<!-- finalize:spec-plan:2026-10-12:7f8d42a3 -->

## 2026-10-13 — [Spec Update]: Quality Engineering classification for the delivery lane

- **Type:** Spec Update
- **Spec modified:** Yes (sections: Quality Engineering added after QE Analysis; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** None (no AC, FR or SC changed)
- **Classification:** criticality High, blast radius Shared, primary risk Regression, taken from the existing QE Analysis; `livespec delivery lane`: full
- **Review:** the previous spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Spec Update]: AC-006 migration of the literal "default" and two CLI clarifications

- **Type:** Spec Update
- **Spec modified:** Yes (sections: AC-006, Clarifications, Edge Cases; frontmatter `updated`)
- **Code modified:** None
- **AC impacted:** AC-006 (reworded), AC-008 (clarified, wording unchanged)
- **Clarify decisions:** `literal-default-migration` (null → empty list; any non-null value, including `"default"`, → one exact entry), `offline-list-content` (`microphones list` is a live operation; offline 008 live-only refusal; order readable with `get microphone_priority`), `legacy-selected-microphone-key` (read-only; generic `set` → `validation_failed` with `details.reason = "microphones_domain"`)
- **Review:** the spec review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: align the plan on the merged 008 and the program decisions D1, D2 and D3

- **Type:** Spec Update
- **Spec modified:** No (plan entry; see the spec entry above)
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **D1:** macOS native proof gates Implemented; Windows/Linux spikes and acceptance are pending release evidence with their exact command
- **D2:** `microphones list` is the `MicrophonesList` live operation of 006 step 0; Step 5 depends on 006 step 0; offline it returns the 008 live-only refusal
- **D3:** per-name refusals in `details.items` (`microphone_name_empty`, `microphone_name_duplicate`); `details.reason = "microphones_domain"` for `selected_microphone` and `microphone_priority` under generic `set`
- **008 names:** `DomainChange(ChangeKind::Apply)`, `SettingsBackend::commit_change`, `tauri_adapter::write_change` (shared with 003), `effects.rs`, `settings::apply_settings_migrations`, `SettingsRepository::initialize`; never-shipped names removed
- **Legacy key:** `selected_microphone` read-only descriptor, `ValidationReason`/`ErrorReason::MicrophonesDomain`
- **Budget:** each open attempt keeps its full 2 s; the 6 s total limits how many attempts start and reserves the default slot (start by t = 4 s)
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Plan Update]: make the plan review preparable and declare 006 step 0 for Step 2

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation)
- **Review budget:** 008's plan and implementation record are cited as inline paths, not links, so the review no longer loads 008's whole document graph; the untracked program notes are no longer referenced and the D2/D3 names the plan uses are copied into the Summary (from the « Step 0 » section of 006's plan). Plan review preparation at 180,000 characters: 1 batch, no error (was `paired_review_batch_budget_exceeded`)
- **Step 2:** prerequisite now names 006 step 0 merged (D3 `ErrorReason`, `ErrorItem`, `with_items`, `with_reason`); Summary and Next Action aligned
- **Review:** the plan review is stale; an independent reviewer refreshes it
- **Author:** Claude Code (spec-refine --auto, claude-opus-5-5)

## 2026-10-14 — [Feature]: Non-UI implementation (Steps 1–5), status In Progress

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** bounded native opens and loss handling (`audio_toolkit/audio/{attempt, recorder}.rs`), `microphone_priority` setting and migration (`settings.rs`, `settings_service/*`), ordered resolver and catalog (`microphones/*`), audio manager boundaries (`managers/{audio, audio_priority}.rs`, `transcription_coordinator.rs`), `handy settings microphones` domain and typed commands (`settings_service/domains/microphones.rs`, `cli.rs`, `commands/audio.rs`), interim selector bridge and cut-short toast (frontend), tests `microphone_{native, settings, resolution, sessions, cli}.rs`, `scripts/microphone-native-proof.ts`, README and contracts
- **AC impacted:** AC-003, AC-004, AC-005, AC-006, AC-008 covered by automated tests; AC-001, AC-002, AC-007 backend only (UI editor not built). No native, UI or release evidence yet
- **Commits:** `4c47acb`, `cc86127`, `046a4d0`, `22a592a`, `0b829ec`
- **Remaining:** Step 6 settings editor (after the common Penflow pass), Step 7 macOS native proof and UI acceptance; Windows/Linux as pending release evidence; end-of-feature delivery review (full lane) by other agents
- **Author:** Claude Code (spec-implement, claude-opus-5-5)

## 2026-10-14 — [Sync]: Merge dev (smart paste 007, word replacements 002)

- **Type:** Integration
- **Spec modified:** No
- **Code modified:** conflict resolution only (`settings_service/{error_reason, validation, public, cli}.rs`, `domains/mod.rs`, settings tests counters 65/66 → 67, README, 008 schema and contract doc); `tests/microphone_cli.rs` gains the older-Handy refusal case
- **AC impacted:** None (behaviour of 001, 002 and 007 kept)
- **Commits:** `6c114bc7` (merge), `4c664550`
- **Proofs:** `proofs/claude-sync/summary.txt`; full cargo test 1133 passed, conventions PASS, progression implement READY
- **Author:** Claude Code (claude-opus-5-5)

## 2026-10-14 — [Fix]: UI delivery review, correction round 1 (Step 6)

- **Type:** Fix
- **Spec modified:** No (the missing Reset control, the "None"/"Default" labels and the position of the Unsaved changes bar are declared deviations; Reset needs a `spec-refine`)
- **Code modified:** status polling paused while the window is hidden (`status-polling.ts`, `use-microphone-status.ts`); focus kept or moved after Save, Retry, Remove, Discard and Reload (`editor-focus.ts`, views); Add and Save inactive while the settings load and no write without a revision; no phantom draft; interrupted drags cancelled; Unknown after a failed device listing (`settingsStore.ts`); `success` / `error` / `valid` state anchors; three functions brought under 30 lines; 3 orphaned locale keys removed
- **AC impacted:** AC-001, AC-002, AC-007, SC-003 (automated: 75 bun tests, 15 headless Chromium tests in `tests/microphone-priority.spec.ts`); real-window and native evidence still pending with Julien
- **Author:** Claude Code (claude-opus-5-5)
