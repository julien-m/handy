# Global Changelog — handy

> Project-level changelog. Add a summary entry here when features are shipped.
> Per-feature changelogs are in `.specs/features/NNN-feature-name/changelog.md`.

---

## 2026-10-29 — [Feature 017] Smart spacing: an option (on for new installs, off on upgrade, `smart_spacing`, first row of "Text formatting", CLI with the same validation, 26 locales) fits only the spaces at the two edges of a dictation to the text before and after the replaced selection: one space between words, existing spaces and line breaks kept and never doubled, none before `, . ; : ! ? ) ]` nor after `( [` (no French space invented), none in CJK, Thai, Lao, Khmer, Myanmar or Tibetan; unavailable context: one final space only; really empty field: no leading space; a focus proven not to be a text field (terminal, password) unchanged; while on it decides the trailing space and turns the method's `append_trailing_space` off on every path (never two final spaces); runs after 015's capitalization in the same preparation step, 015's reader extended (field length and 16 units after the selection, same 50 ms budget) (AC-001 to AC-014, automated evidence; native proof pending)
## 2026-10-29 — [Feature 015] Reader extended by 017 (side after the selection, `NotText` class); an answer shorter than the window before the caret is no longer trusted (Absent) (AC-006, AC-009)
## 2026-10-29 — [Feature 014] Keys during a dictation: Option left pressed alone sends the dictation in progress to the app in front at the end without unpinning (press again to undo), Control left pressed alone inserts it without the automatic Enter (`Suppressed(UserSkipped)`, indicator mark); one modifier key each, configurable in the shortcut settings and the CLI, observed only during a dictation (automated evidence; native proof pending)
## 2026-10-29 — [Feature 016] Smart paragraphs: an option (off by default, `smart_paragraphs_enabled`, UI row and CLI with the same validation) splits a long dictation into paragraphs at sentence boundaries when 45 words or 4 substantial sentences are reached, with a blank line and no space added in Chinese or Japanese; runs locally after AI post-processing and before the insertion, never rearranges lists, code, quotations or existing line breaks; NaturalLanguage on macOS, UAX #29 elsewhere (CJK unchanged there) (AC-001 to AC-010, automated evidence; native proof pending)
## 2026-10-29 — [Feature 015] Smart capitalization: the first letter of a dictation follows the text before the cursor (capital at a field, line, sentence or list start; lower case in continuation; unchanged when the context is absent or undecidable), protecting acronyms, inner capitals, "I", replacements, custom words and AI-decided text; macOS context read bounded (64 UTF-16 units, 50 ms, never a password field, never logged); setting on for new installs, off on upgrade; new "Text formatting" section (AC-001 to AC-011, automated evidence; native proof pending)
## 2026-10-29 — [Feature 007] Bugfix N28: with `paste_verification` on, a terminal (cmux) no longer receives the transcript twice nor the recovery bubble: only web text (an element below an `AXWebArea`) is verified, any other element is `Unreadable` (`NotWebText`) with the behavior without verification, and a last read right before the repeated Cmd+V cancels it when the page pasted late (AC-019, FR-012)
## 2026-10-29 — [Feature 012] Fix N29: compact badge menu rows (smaller star, no "This dictation only" hint, 009 amended); names up to about 20 characters shown in full (1/1 gap closed)

## 2026-10-29 — [Feature 007] Fix N11 (experimental): `paste_verification` (off by default) checks after the smart-paste Cmd+V that the focused field changed, sends Cmd+V once more under the clipboard ownership guard, and reports an unchanged field as `field_unchanged` (recovery bubble, no automatic Enter); unreadable fields keep the current behavior (AC-019, FR-012)
## 2026-10-29 — [Feature 013] Default input per app: the badge menu remembers the destination's field (role, subrole, accessible name, placeholder, stable DOM id; never content) or, for an app whose bundle ships an addressing tool (cmux), its workspace (`destination_default_inputs`, at most 20, one per app); when the active element is not a text field, Handy focuses the unique matching field found by a bounded search of the focused window, or types through `cmux send` without Enter; zero/several matches or a gone workspace show 010's bubble with a dedicated reason; settings list with Forget, CLI `destination default-input-remove|default-input-clear` (AC-001 to AC-010, automated evidence; native proof pending)
## 2026-10-29 — [Feature 012] Destination favorites: a star toggle on every app row of the badge menu saves favorites through the settings service (`destination_favorites`, at most 10, CLI `destination favorite-add|favorite-remove`), listed bottom-up just above "Frontmost app at the end"; a favorite that is not running stays listed "Not open" and is never launched; 009's Pin and Unpin moved to a fixed footer, always visible (AC-001 to AC-006, automated evidence; native proof pending)
## 2026-10-29 — [Feature 010] Fix N26: after Escape during a drag of the recovery bubble the drag image no longer slides back and the bubble no longer flashes before closing (panel ordered out before it is made opaque again); a release without a target keeps the slide back (AC-004, FR-003)
## 2026-10-29 — [Feature 010] Fix N25: Escape during a drag of the recovery bubble cancels it (nothing inserted) and dismisses the bubble through Close's path, logged `drag cancelled by Escape, dismissed`; a release without a target still keeps the bubble; Escape watched for the session only (AC-004, FR-003)
## 2026-10-29 — [Feature 010] Fix N24: the whole recovery bubble (text, background, border) is the drag source, with the same native drag, payload and `drop accepted` log as the handle; Copy, Why? and Close stay clicks, a click without movement is not a drag, grab cursor over the surface (AC-004, FR-003)
## 2026-10-29 — [Feature 003] Bugfix N22: approximate correction no longer turns « je ne le vois pas » into « je LiveSpec »; a sound-alike (Soundex) only weighs a spelling distance under twice the threshold (AC-008, FR-006)
## 2026-10-29 — [Feature 005] Bugfix N23: a prompt chosen on the indicator during a plain dictation is remembered (settings service, `plain_dictation_prompt_id`) and starts the next plain dictations until None is chosen (AC-009); the AI step logs provider, duration and outcome at debug level; the prompt editor's provider and model controls share one box
## 2026-10-29 — [Feature 011] Bugfix N18: an application that answers late after its activation (Codex, Chrome pages) receives the paste in its remembered input instead of the recovery bubble; the guard verifies the state at dispatch
## 2026-10-29 — [Feature 006] Bugfix N19: with a captured or pinned destination, the recording indicator (and the 010 bubble at its place) stays on the monitor of its first placement until the next dictation (AC-009)

## 2026-10-28 — [Feature 006] Bugfix N16: a declined return to the dictation's app falls back to the accessibility activation request within the same 1 s

## 2026-10-28 — [Feature 010] Bugfix N17: the recovery bubble is shown again (the panel stayed unregistered after `to_window`, `PanelNotFound`); a failed insertion without bubble is a notice of the recording indicator (toast only with the overlay style none)

## 2026-10-28 — [Feature 007] Bugfix N12: smart paste no longer aborts with CFRelease(NULL) when ABC/US cannot be selected; truthful "no actor reply" log

## 2026-10-28 — [Bugfix, onboarding] Permission checks run one at a time (native session N10, review)

The polling, Try again, the page `focus` event and the Tauri window focus event share one permission check at a time (`checkInFlightRef`), so a failure is counted once; the onboarding completes once and its completion timer is the only one, cleared on unmount. Code: `src/components/onboarding/AccessibilityOnboarding.tsx`; test: `tests/accessibility-onboarding.spec.ts` (overlapping checks: one check, one completion), which fails on the previous code.

## 2026-10-28 — [Bugfix, 001/002/003/004/005] Settings window findings of the native session (N2, N3, N4, N5, N6, N8)

Branch `fix/native-session-ui`: the window scrolls only its content (N4, transverse); explicit Save and Discard for an unsaved shortcut draft, Reset disabled meanwhile (N8, 004); compact sticky save bar with Reload during a conflict (N2, N3, 001); refused-import alert under the Replacements title (N6, 002); keyboard focus given back after Done, Save, Discard and dialogs (N5, 002, also 003 and 005). Verifications: [native-session-fixes-ui.txt](native-session-fixes-ui.txt).

## 2026-10-28 — [Bugfix, onboarding] « Permissions Required » no longer waits without a way out (native session N10)

After five seconds of « Waiting… » for the accessibility permission, the screen offers « Open System Settings » (Privacy & Security › Accessibility, allowed by a scoped `opener:allow-open-url` in `src-tauri/capabilities/default.json`), « Try again » (asks macOS again) and the hint « If Handy is already checked, remove it, then add it again. » (26 locales); the permissions are checked again when the window takes the focus back. Code: `src/components/onboarding/{AccessibilityOnboarding.tsx, accessibility-waiting-help.tsx, use-window-focus-check.ts}`; tests: `accessibility-waiting-help.test.tsx` (`bun run test:onboarding`), `tests/accessibility-onboarding.spec.ts`. No feature spec covers onboarding; the real System Settings pane is not observed by these tests (native proof pending with Julien).

## 2026-10-28 — [Feature 004] Bugfix: hold-or-toggle measures the hold between the key edges received by the backend, so a tap behind a slow start stays locked — native session finding N13

## 2026-10-28 — [Feature 003] Bugfix: vocabulary limit raised to 5,000 terms (Julien's decision; spec-refine 003 pending) with an indexed approximate correction, 21 ms per dictation — native session finding N7

## 2026-10-28 — [Feature 001] Bugfix: the recording cut short by a lost microphone is a notice of the recording indicator (toast only with the overlay style none) — native session finding N1

## 2026-10-15 — [Feature 011] Implementation in progress: input focused at the dictation start remembered, given focus back, range restored and verified before the paste, behind destination-return (automated evidence in four feature combinations; macOS native proofs pending with Julien)
<!-- finalize:spec-implement:2026-10-15:28770870 -->

## 2026-10-15 — [Feature 010] Implementation in progress: recovery bubble, bounded field classification, deliberate Copy and the no-text-field setting behind destination-return (automated evidence in four feature combinations; macOS native proofs pending with Julien)
<!-- finalize:spec-implement:2026-10-15:0ab0ed79 -->

## 2026-10-15 — [Feature 009] Implementation in progress: session pin, single Automatic Enter authority and destination CLI behind destination-return (automated evidence in four feature combinations; macOS native proofs pending with Julien)
<!-- finalize:spec-implement:2026-10-15:851e4c61 -->

## 2026-10-15 — [Feature 011] Plan created: Dictation Field Return — 6 implementation steps, 4 diagrams (input read after 006's session commit, focus restored after activation, verified in 006's guard before 010's verdict)
<!-- finalize:spec-plan:2026-10-15:67c12fe0 -->

## 2026-10-15 — [Feature 011] Spec created: Dictation Field Return — 4 stories, 8 AC, 5 FR (the dictation's input remembered at start, focused, raised and verified before the paste, fallback to 006/010)
<!-- finalize:spec-specify:2026-10-15:5625200d -->

## 2026-10-14 — [Features 004, 005, 006, 007, 008] UI hardening: shortcut draft exit and one-shot record requests (004), return-to-app row hidden while its gate is closed (006), Reload failure kept in the conflict alert and history limit saves tested (008), indicator window bounded by its monitor, menu timers cleaned, closed-gate test compared with dev (005/006/007)

## 2026-10-14 — [Feature 010] Plan refined: permission_missing aligned on the spec (no keystroke sent, so no possible-partial-insertion detail)
<!-- finalize:spec-refine:2026-10-14:3d66e47e -->

## 2026-10-14 — [Feature 010] Spec refined: focus in another process inserts (Key Entities), history claim conditional on a successful save, no 'some text' detail for permission_missing, destination-return gate recorded, a new dictation closes the bubble even before 4 s
<!-- finalize:spec-refine:2026-10-14:8ecc2640 -->

## 2026-10-14 — [Feature 009] Spec refined: AC-003 reasons split into pinned_app_not_running and activation_failed, destination-return gate recorded; plan keeps dev's 50 ms pause before Enter
<!-- finalize:spec-refine:2026-10-14:b5422949 -->

## 2026-10-14 — [Feature 010] Plan refined: round 4 review ingested, 010 behind 006's destination-return gate (closed build keeps dev's paste-error), before_insertion synchronous in the first guard check with a ~50 ms AX budget, names aligned on delivered 006/007
<!-- finalize:spec-refine:2026-10-14:fb255547 -->

## 2026-10-14 — [Feature 009] Plan refined: round 4 review ingested, 009 behind 006's destination-return gate (closed build keeps dev's auto_submit, migration never rewrites it), Enter keyed by dictation and generation, status limited to the pinned app
<!-- finalize:spec-refine:2026-10-14:19f333f0 -->

## 2026-10-14 — [Feature 007] Smart paste wired into dictation through 006's insertion contract (006 guard context, method_outcome, disposition, Enter and late-read hooks), behind the smart-paste-dispatch gate closed by default; dev (004, 006) merged; native AC-016 matrix pending with Julien

## 2026-10-14 — [Feature 007] Plan refined: history follows the 006 rule (no WAV gate, text-only row when the WAV fails, Julien's decision), delivered 008 settings names replace LiveSettingsEffects and commit_mutation as live commit
<!-- finalize:spec-refine:2026-10-14:d424af86 -->

## 2026-10-14 — [Feature 007] Smart paste settings on the 008 service: macOS migration, default, shared refusal, output CLI and selector implemented behind a rollout gate, inactive until step 7 wires the dictation dispatch; certification pending

## 2026-10-13 — [Feature 007] Smart paste fix round 1: current-host-only restoration, concealed/transient content never restored (Julien's decision on AC-009), audit A/B minors fixed or documented; release blockers R-1/R-2 still open

## 2026-10-13 — [Feature 007] Smart paste core implemented (steps 1–6): verified AX insertion, bounded multi-type snapshot helper, on-demand provider, serialized coordinator, per-app/keyboard strategies; settings (008) and destination wiring (006) pending; certification pending

## 2026-10-13 — Smart Paste: record native feasibility and explicit cache/late-write limits; settings integration awaits 008
<!-- finalize:spec-feature:2026-10-13:72e02927 -->

## 2026-10-14 — [Feature 006] Synced with `dev` (004 merged, one closed settings error reason enum shared by 004 and 006); settings UI command `change_return_to_dictation_app_setting` with a typed `unsupported_platform` refusal handled by the store; history rule delivered (text-only row when the WAV fails, nullable `file_name` migration shared with 005, `history_status` from the actual `save_entry` result). Full `cargo test` 681 passed / 20 ignored, frontend checks and repository conventions PASS; no visual UI, no AC certified.

## 2026-10-14 — [Feature 002] Integrated with the merged settings CLI (008): `handy settings replacements` (list, add, update/edit, remove/delete, import with dry run, export) on the shared rule-set edits, one compiled matcher per committed revision applied from the next dictation, settings window transaction commands (no UI yet), output-stage harness, and paged/staged transfer beyond one 1 MiB message. Full `cargo test` 642 passed / 0 failed / 19 ignored; conventions PASS, new 0; macOS real-app CLI check (debug). UI, real dictation, M1, Windows/Linux pending.

## 2026-10-13 — [Feature 002] Spec refined: AC-006 Korean rationale made factual (particles attach to the preceding word); halfwidth U+FF9E/U+FF9F and NFD hangul jamo documented as accepted CJK-side false positives with fixtures (1 Clarify decision, IDs preserved); spec/plan reviews to redo.

## 2026-10-13 — [Feature 002] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Data; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 002] Spec and plan refined, re-audit fixes: AC-006 word characters include ZWNJ/ZWJ and a CJK side still rejects an adjacent combining mark; AC-004 header recognized with tab or `;`; AC-005 1 MB = 1,000,000 bytes; AC-007 guarantee scoped before AI post-processing (6 Clarify decisions, IDs preserved); plan aligned; spec/plan reviews to redo.

## 2026-10-13 — [Feature 002] Spec refined and audit fixes: AC-006 word characters now include combining marks (Clarify decision, IDs preserved); matcher, BOM, stop-path harness and scope fixes; lib 348 PASS, pipeline harness 7 PASS, conventions PASS.

## 2026-10-13 — [Feature 002] Step 4 core: compiled single-pass replacement matcher (40 fixtures) applied after script conversion with protected spans in both final transcription paths, rules captured at recording start; release benchmark 1.78 ms max on M4 Pro (M1 pending).

## 2026-10-13 — [Feature 002] Step 2 core: two-column exchange parser/serializer with bounded UTF-8 input and deterministic merge/replace previews; commit and revisions remain Waiting for 008.

## 2026-10-13 — [Feature 002] Step 1 core: ordered replacement model, pinned Unicode 17 offsets/properties and shared scalar/ID validation; additive empty settings and native-generated bindings. Rust target 20 PASS, regression 306 PASS / 17 ignored, generator boundary tests 6 PASS, Clippy/frontend checks and repository conventions PASS. Complete UI/transactions/CLI remain Waiting for 008; no acceptance certification claimed.

## 2026-10-14 — [Feature 009] Plan refined: history_status from 006's actual save_entry result, a WAV failure tested as a saved text-only row (Julien's decision), save_entry failure as failed
<!-- finalize:spec-refine:2026-10-14:48c67c3f -->

## 2026-10-14 — [Feature 010] Plan refined: history_status carried from 006's actual save_entry result, text-only row saved when the WAV fails (Julien's decision), not_saved no longer tied to a disabled or unavailable WAV
<!-- finalize:spec-refine:2026-10-14:e2c204ed -->

## 2026-10-14 — [Feature 005] Plan refined: text-only history row for every dictation whose WAV fails (Julien's decision), open WAV question removed, history change shared with 006
<!-- finalize:spec-refine:2026-10-14:d7cad1e7 -->

## 2026-10-14 — [Feature 006] Spec and plan refined: text-only history entry when the WAV fails (Julien's decision), AC-005 history clause no longer tied to the audio-gated save, history_status from the actual save_entry result
<!-- finalize:spec-refine:2026-10-14:2190dbf9 -->

## 2026-10-14 — [Feature 009] Plan refined: auto_submit refused by a plain not_found (retired-key hint removed), cross source FrontmostAtEnd and resolved_target aligned with 006, Windows full acceptance pending release evidence
<!-- finalize:spec-refine:2026-10-14:86e43d44 -->

## 2026-10-14 — [Feature 004] Integrated with 008: `dev` merged; shortcut lists, the post-processing toggle and the keyboard switch commit through the settings service with registration prepared before one revision and undone on refusal, refused during a dictation (coordinator quiescence lease); `handy settings shortcuts list/add/remove/reset` (live-only changes, dry run, stable codes). Full `cargo test`, frontend checks and repository conventions PASS; native CLI evidence pending; no AC certified.

## 2026-10-13 — [Feature 004] Spec review fix: enabling post-processing always succeeds again, as the spec requires; a shortcut the OS refuses is reported by a translated toast and a warning on its row and stays editable; the shared-slot handover is kept. Rust 376 PASS / 16 ignored, frontend checks and repository conventions PASS, progression implement READY.

## 2026-10-13 — [Feature 004] Re-audit fixes: disabling post-processing hands a shared Tauri slot back to Transcribe, a refused late activation keeps post-processing off with a translated message, OS probe of cancel outside recording, nits N1–N5 fixed and N6 documented, `implementation.md` structure valid. Rust 372 PASS / 16 ignored, frontend checks and repository conventions PASS.

## 2026-10-13 — [Feature 004] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Regression; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 004] Audit fixes: inherited-conflict slots kept on Tauri edits, OS probe of disabled post-processing shortcuts, switch repair by destination parser with kept-and-reported conflicts, translated cancel refusals, live conflict warnings, rollback and Carbon shadow tests. Rust 363 PASS / 16 ignored, frontend checks and repository conventions PASS; recorder states stay with the UI pass.

## 2026-10-13 — [Feature 004] Core: 1–4 shortcuts per dictation action with additive storage, action+shortcut identity in both backends and the coordinator (toggle/locked any shortcut, held starting shortcut only), shared conflict checker with legacy conflicts flagged, settings list UI. Rust 340 PASS / 16 ignored, Clippy as `dev`, frontend checks and repository conventions PASS. CLI and revision commit Waiting for 008; no acceptance certification claimed.
## 2026-10-14 — [Feature 010] Plan refined: 008 sources cited as code paths (plan review preparable in 18 batches), shared contracts cited from the 006 plan, pinned reasons aligned with 009
<!-- finalize:spec-refine:2026-10-14:61308064 -->

## 2026-10-14 — [Feature 009] Plan refined: 008 sources cited as code paths (plan review preparable in 20 batches), explicit AC-006 (13/13), DestinationManager::claim_enter, pinned_app_not_running and activation_failed aligned with 010
<!-- finalize:spec-refine:2026-10-14:f275f226 -->

## 2026-10-14 — [Feature 006] Plan refined: 008 sources cited as code paths (plan review preparable in 1 batch), four ValidationContext constructions, DestinationManager::claim_enter for 009, pinned_app_not_running reason
<!-- finalize:spec-refine:2026-10-14:8325a713 -->

## 2026-10-14 — [Feature 005] Plan refined: plan review preparable again, Step 2 ordered, offline prompt list justified

The plan cites other features' documents as inline paths instead of links, so its review no longer loads 008's document graph (preparation at 180,000 characters: 1 batch, no error); it no longer references the untracked program notes and copies the D2/D3/D4 names it uses. Step 2 names `tauri_adapter::write_change` as prerequisite (added by Step 2 if 001/003 have not) and commits deletions without preparation until Step 3 adds the `bindings` preparation. Offline `prompts list` is justified as 008's delivered live-only refusal applied to a D2 operation. Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-14 — [Feature 003] Plan refined: plan review preparable again, durable-error rule renamed, offline status justified

The plan cites other features' documents as inline paths instead of links and writes `Inspect [Whisper model documentation](…)`, so its review no longer loads 008's document graph nor an external contract (preparation at 180,000 characters: 1 batch, no error); it no longer references the untracked program notes and copies the D2 names it uses. "Rule D4" becomes the durable-error rule (program D4 is the unsupported-platform refusal). Offline `vocabulary status` is justified as 008's delivered live-only refusal applied to a D2 operation. Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-14 — [Feature 001] Plan refined: plan review preparable again, 006 step 0 before Step 2

The plan cites 008's plan and implementation record and 006's plan as inline paths instead of links, so its review no longer loads 008's document graph (preparation at 180,000 characters: 1 batch, no error); it no longer references the untracked program notes and copies the D2/D3 names it uses. Step 2's prerequisite now names 006 step 0 (D3 variants). Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-14 — [Feature 010] Plan refined: one Enter veto with 009, method outcome from 006, late-read update

The plan consumes `InsertionOutcome.method_outcome` from 006 instead of a separate 007 adapter, updates the existing bubble on 007's late-read diagnostic, shares 006's `RecoveryDisposition` as the single Enter veto (010 only answers `RecoveryPort::before_insertion`), uses the merged 008 names and records Windows/Linux native proof as pending release evidence. Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-14 — [Feature 005] Spec and plan refined: truthful failure notice, merged 008 names, live prompt list

Spec: AC-005 notice says the AI step failed and the original text was used for insertion, never that insertion succeeded (Clarify decision). Plan: merged 008 names, canonical `remove`/`update` actions, shortcut removal through 008's native preparation, `prompts list` as a D2 live operation with the Apple Intelligence check, `unsupported_platform` refusal (D4), 004 merge as explicit dependency, QE aligned, Windows/Linux native proof as pending release evidence; history/WAV divergence with 006 signalled in ship.md. Spec and plan reviews are stale until an independent reviewer refreshes them.

## 2026-10-14 — [Feature 009] Spec and plan refined: pinned badge rule, D2 live pin operations, retired auto_submit, single Enter veto
<!-- finalize:spec-refine:2026-10-14:0feee4a1 -->

Spec AC-001 now shows the badge while an app is pinned even with 006 destination return off (decision in Clarifications; 006 spec unchanged). Plan: `destination pin|unpin|status` as 006 Step 0 live operations with precedence `unsupported_platform` (2) → no instance (5) → `not_found` (4); retired `auto_submit` is `not_found`/4 with `details.reason = "retired_key"`; the Enter veto is 006's single `RecoveryDisposition`; Windows/Linux native proof as pending release evidence; QE aligned on the spec. Spec and plan reviews are stale until an independent reviewer refreshes them.

## 2026-10-14 — [Feature 001] Spec and plan refined: literal "default" migration, live microphone list, full 2-second attempts

Spec: AC-006 maps only a null legacy selection to an empty list (the device name "default" becomes an exact entry); Clarifications record the live-only `microphones list` (configured order readable with `get microphone_priority`) and the read-only legacy `selected_microphone` (`validation_failed`, `details.reason = "microphones_domain"`). Plan: merged 008 names, D2 live list (depends on 006 step 0), D3 per-name items, full 2-second attempts within the 6-second total, Windows/Linux native proof as pending release evidence. Spec and plan reviews are stale until an independent reviewer refreshes them.

## 2026-10-14 — [Feature 006] Plan refined: settings framework Step 0 (D2–D4) and merged 008 names
<!-- finalize:spec-refine:2026-10-14:b0be88bc -->

New Step 0 extends the 008 framework with live domain operations (`DomainPlan::Live`, `LiveOperation`, `LiveContext`), a static `PlatformCapability`, and optional `details.reason`/`details.items`; unsupported platform is `validation_failed`/2 with `unsupported_platform` (D4). `DestinationContext` and a single `RecoveryDisposition` veto defined for 007/009/010; 007 identifiers and wire values adopted; QE aligned on the spec (High, Shared, Functional); Windows/Linux native proof as pending release evidence (D1). Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-14 — [Feature 003] Plan refined: merged 008 names and program decisions D1–D4

Plan QE aligned on the spec (Medium, light lane); `vocabulary status` as a D2 live operation (depends on 006 step 0); per-line errors in D3 `details.items`; `context_budget_unverified` as `internal_error`/1 (D4); stdout-only export; Whisper budget of 223 tokens sourced from the engine, other budgets measured in Step 1; Windows/Linux native proof as pending release evidence. Spec unchanged; the plan review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 001] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Regression; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 003] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality Medium, blast radius Shared, primary risk UX; `livespec delivery lane` reads it (light once the spec review is current). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 005] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Data; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 006] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Functional; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 009] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Functional; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — [Feature 010] Spec refined: Quality Engineering classification for the delivery lane

`## Quality Engineering` added from the existing QE Analysis: criticality High, blast radius Shared, primary risk Data; `livespec delivery lane` reads it (full). No AC, FR or SC changed; the spec review is stale until an independent reviewer refreshes it.

## 2026-10-13 — 008-settings-cli: final delivery review correction round 1 Done

Both blocking findings fixed (Windows offline write under decision 008; endpoint thread without busy loop) with the audit and review warnings of the same area: bounded endpoint close and republication, Windows pipe instances and deadlines, GUI subsystem with terminal attachment, requests refused before sending when too large, routed VAD/channel writes prepared like the UI, stdin tests, docs. Full `cargo test` 490 passed/0 failed/18 ignored; conventions PASS/new 0. Next: a fresh independent delivery review. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-delivery-review-checkpoint--correction-round-1).

## 2026-10-13 — 008-settings-cli: Step 7 macOS native proof (debug build) Done; release evidence pending

Native harness (macOS 26.5.2, M4 Pro, debug proof binary, isolated HOME): 18/18 checks — older Handy refusal, offline/live reads and writes, start-hidden without Dock icon, routed live effect, no app registration by the CLI, endpoint reclamation, read-only startup. Full `cargo test` 474 passed/0 failed/17 ignored. Pending: M1 release latency, settings window refresh and shortcut re-registration (human), Windows/Linux/second account. Feature not Implemented. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-step-7-checkpoint--macos-native-proof-debug-build).

## 2026-10-13 — 008-settings-cli: Step 6 domain framework and output domain Done

`handy settings output get|set` (with `--dry-run`) over the existing output settings, on a domain framework shared by later features (aliases, dry run through the commit validation, atomic commit, live-only refusal, bounded UTF-8 file/stdin input). README documents the CLI and its recovery rules. Targeted tests and conventions PASS/new 0. Commit `462a5a5`; Step 7 in progress; no AC certified. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-step-6-checkpoint--domain-framework-and-output-domain).

## 2026-10-13 — 008-settings-cli: Step 4 independent audit corrections Done

Both Step 4 audits corrected: every UI and routed CLI write is one ordered write, the service is installed before the first startup reader and falls back to read-only, P0 tests added, minors and simple nits fixed; Linux decision 008 applied (free session bus plus free lease allows offline writes). Targeted tests, lint, format and conventions PASS/new 0. Commit `bc76443`; no AC certified. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-step-4-correction-checkpoint--independent-audits-a-and-b).

## 2026-10-13 — 008-settings-cli: Step 5 early settings CLI and private channel Done

`handy settings get|keys|set` runs before any GUI setup, offline under the shared lock or routed to the running instance over an authenticated private channel (5 s deadline, unknown outcome, no fallback). The system default device is stored as null on every surface. Full suite 429 passed/0 failed/17 ignored; conventions PASS/new 0. Commits `49b2736`…`be94db8`; Steps 6–7 Todo; no AC certified. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-step-5-checkpoint--early-cli-and-private-channel).

## 2026-10-13 — 008-settings-cli: Step 3 audit corrections applied, pending independent audit

Blocker fixed by detection (parent and read-store identities rechecked before replacement), three majors and 8 minors/nit fixed, 3 Windows function-size minors dispositioned. Full suite 356 passed/0 failed/17 ignored; conventions PASS. Code commit `1ae5d84`; Steps 4–7 Todo; no AC certified. Inspect [the checkpoint](features/008-settings-cli/progress.md#current-step-3-checkpoint--audit-corrections-applied-pending-independent-audit).

## 2026-10-13 — 008-settings-cli: Step 3 Blocked by platform refusal; feature delivery incomplete

Supervisor received `This content was flagged for possible cybersecurity risk...` during required Step 3 audit corrections and interrupted workers/reviewer. Historical local21 passed/0 failed/1 ignored and scoped PASS do not close final audit1 blocker/3 major/11 minor/1 nit; no corrective version is certified. Earlier `input_too_large` was mitigated and is not this blocker. Steps 4–7 remain Todo. Prior verified commits:Step1 `50fdd044`, Step2 `0a7a433`, documentary refinement `e3bc314`; no feature delivery. Inspect [the authoritative stop](features/008-settings-cli/progress.md#current-step-3-stop--platform-refusal-blocks-required-corrections). No corrective retry/source action, new commit/archive or alternate method by documenter.

## 2026-10-13 — 008-settings-cli: Step3 pure repository resumes; app adoption moved into Step4 all-writer cutover

Root authorized the bounded sequencing refinement after an independent source-backed snapshot-intent finding. Step3 keeps pure Repository/Locator/normalization; Step4 atomically migrates all writers and app/settings-file plugin ownership, with startup/cache evidence then. Seven steps/FR/AC/business/native/output scope unchanged; direct already-locked dirs6 matches Tauri resolver, fs2 retained. Initial isolated reads RED0/3→GREEN3/0 only; Step3 In Progress, no app activation/full AC/runtime-loss claim. Inspect [finding](features/008-settings-cli/proofs/current-run-step3/ordering-finding.json) and [refined plan](features/008-settings-cli/plan.md#step-3--build-atomic-repository-revision-and-locking).

## 2026-10-13 — 008-settings-cli: Step 2 pure foundation Done;21 tests pass, whole acceptance remains partial

Root accepted current corrective scoped review: descriptors63/ports51/enums17, explicit public redaction, typed errors/version1/closed DTOs and checked generated schema. Actual-bridge GREEN21/0/0, clippy/full format0; unchanged platform22/frontend gates retained. Raw audit0 blocker/7 major/5 minor dispositioned; repository PASS/new0 after3 expect corrections with unchanged baseline. Step3 Todo; feature In Progress, no whole-AC or actual UI-adapter parity/native PASS. Inspect [summary](features/008-settings-cli/proofs/current-run-step2/corrective-summary.json), [review](features/008-settings-cli/proofs/current-run-step2/corrective-review.json) and [checkpoint](features/008-settings-cli/progress.md).

## 2026-10-13 — 008-settings-cli: Step 1 primitive gate Done; 22 tests pass, complete acceptance remains partial

Root accepted scoped final review and audit dispositions: assertion RED101/GREEN22/0/0, actual Swift bridge, strict clippy/Cargo fmt0 and prior unchanged-frontend types/lint/format0. Audit exit0 reported0 blocker/2 major/7 minor/2 nit; pipe major corrected, FreeConsole false positive and remaining observations explicitly dispositioned. Step2 Todo; feature In Progress, no complete AC/SC certified. Inspect [final summary](features/008-settings-cli/proofs/current-run-step1/minimal-final2-summary.json), [scoped review](features/008-settings-cli/proofs/current-run-step1/minimal-final2-review.json) and [checkpoint](features/008-settings-cli/progress.md). Client authentication/reclamation remainStep5, full macOS settings/native proofStep7; foreign checks pending release evidence.

## 2026-10-13 — 008-settings-cli: Bounded Unix mechanism refined; current-source proof pending

- **Prerequisite:** Root-approved correction of actual process-mask finding and independent mechanism decision.
- **Required inputs:** Current plan, kernel peer-UID/directory contract, informative OS references and exact restoration evidence.
- **Action:** CurrentUID directory0700 and pinned guards plus both-side kernel UID before bytes replace socket-mode prescription; no umask/pathchmod/bootstrap permission framework. Step5 retains lease-proven reclamation.
- **Execution evidence:** Native fd-fchmod EINVAL and pinned-open EOPNOTSUPP; own incomplete bootstrap hunks surgically restored, no successful compile/RED/PASS claim. Earlier tests remain historical snapshots.
- **Success criteria:** FR/AC IDs, seven steps, business/release scope unchanged; Step1 In Progress, fresh implementation/tests/reviews/audit and official plan refresh pending.
- **Failure handling:** Read [mechanism review](features/008-settings-cli/proofs/current-run-step1/minimal-channel-review.json) and [restoration](features/008-settings-cli/proofs/current-run-step1/bootstrap-surgical-restore.json). No Step2 or current-source certification before accepted gates.

## 2026-10-13 — 008-settings-cli: Registry activity refreshed; socket failure-path retention recorded

- **Prerequisite:** Renewed audit found stale Recent Activity; Step 1 source corrections remain In Progress.
- **Required inputs:** Global changelog order, registry markers, socket ownership/failure and retention tests.
- **Action:** Refresh latest ten entries newest first; preserve feature statuses/metadata. Retain socket failure paths fail-closed, avoiding same-UID check/unlink races; lease-proven reclamation belongs to Step 5.
- **Execution evidence:** Documentary/source inspection only; prior 20/20 and quality PASS bind the earlier source snapshot. Fresh worker-source checks/review/audit are pending.
- **Success criteria:** Current activity summary; no Step 1 Done, Step 2 start or certified AC.
- **Failure handling:** Read [the ongoing correction disposition](features/008-settings-cli/implementation.md#ongoing-step-1-corrections-and-socket-failure-path-disposition) and [checkpoint](features/008-settings-cli/progress.md). Preserve existing history and foreign pending evidence; no endpoint-deletion shortcut or false fresh PASS.

## 2026-10-13 — 008-settings-cli: Root corrects whole-AC step-block interpretation; Step 1 audit pending

- **Prerequisite:** Root verified the actual Step Gate.
- **Required inputs:** Scoped/transverse checks, independent reviews, checkpoint and renewed audit including untracked files.
- **Action:** Step 1 In Progress, Steps 2–7 Todo; earlier report superseded, history preserved; root owns pipeline/spec metadata.
- **Execution evidence:** Existing actual-bridge 20/20, clippy/types/lint/format exit 0, partial-scope source review PASS; whole-AC mapping complete but not ready, ingest exit 1, no capture attempted.
- **Success criteria:** Renewed audit pending; no Step 1 Done or certified AC. Generic goal execution proof is aggregate after Step 7; final whole-AC certification remains mandatory.
- **Failure handling:** Read [the authoritative checkpoint](features/008-settings-cli/progress.md) and [corrected historical report](tool-defects/2026-10-13-step-execution-proof-requires-whole-acceptance-criterion-bef.md). No runtime bug established, no criterion weakening, fabricated proof or Git history.

## 2026-10-13 — 008-settings-cli: Superseded whole-AC step-block interpretation

Historical only; read the later root correction and authoritative checkpoint. No runtime defect was established.

- **Prerequisite:** Approved minimal Step 1 scope and native evidence policy.
- **Required inputs:** Current source/tests, actual Swift bridge, independently reviewed mapping and trusted runtime.
- **Action:** Preserve Step 1 changes and partial mappings; Steps 2–7 stay Todo, no Git history.
- **Execution evidence:** 20 passed/0 failed/0 ignored; clippy/types/lint/format exit 0; partial-scope source review PASS; mapping ingest exit 1, complete=true/ready=false/errors=[] with AC-005 missing.
- **Success criteria:** Local primitive checks pass; no AC/SC certified, no Step 1 Done. After-step audit NOT RUN.
- **Failure handling:** Read [the tool granularity defect](tool-defects/2026-10-13-step-execution-proof-requires-whole-acceptance-criterion-bef.md) and [authoritative checkpoint](features/008-settings-cli/progress.md). Whole-AC gates remain valid; no fake coverage or criterion rewrite. Old audit failure is historical; foreign native checks remain pending release evidence.

## 2026-10-13 — 008-settings-cli: Step 1 local parser/console tests 17/17; mandatory audit Blocked without verdict; Steps 2–7 Todo, all ACs uncertified

Inspect [the current checkpoint](features/008-settings-cli/progress.md) for scope and pending release evidence.

## 2026-10-13 — 008-settings-cli: private socket correction; implementation blocked by native execution runner

Native primitive tests passed 13/13; complete CLI acceptance remains unimplemented. Read [the recorded LiveSpec defect](tool-defects/2026-10-13-cargo-runner-lacks-native-execution-certification.md).

## 2026-10-13 — 008-settings-cli: Native primitives narrowed

Local 10/10 pass; official typed capture blocked by tool. Read [the current checkpoint](features/008-settings-cli/progress.md).

## 2026-10-13 — Plan validation: normalize partial implementation map frontmatter and AC heading; no mapping or code changes
<!-- finalize:spec-plan:2026-10-13:1953f656 -->

## 2026-10-13 — Plan refined: partial spike reuse, minimal passive legacy refusal without replay, simple Windows console compilation checks, macOS required and exact pending release evidence
<!-- finalize:spec-plan:2026-10-13:ff08720c -->

## 2026-10-13 — [Feature 008] - **Type:** Refactor
<!-- finalize:spec-implement:2026-10-13:9b1995bc -->

## 2026-10-13 — 008-settings-cli: partial isolated startup spike; native and conventions gates blocked
<!-- finalize:spec-implement:2026-10-13:61bf0cb4 -->

## 2026-10-13 — Settings CLI plan resumed; independent semantic review refreshed without rewriting canonical requirements
<!-- finalize:spec-plan:2026-10-13:e428ba3d -->

## 2026-10-13 — [Feature 007] Smart paste plan rewritten from 2026-10-13 spec — 9 steps, 4 diagrams; AX-first and receipt-only restoration; native/design release gates
<!-- finalize:spec-plan:2026-10-13:17322c07 -->

## 2026-10-13 — [Feature 010] Plan refined: current 007 AX/readback contract and explicit AC-006 native acceptance
<!-- finalize:spec-plan:2026-10-13:09d4dd98 -->

## 2026-10-13 — [Feature 009] Plan refined: verified AX success for Enter, uncertain suppression, LAST-read restoration and current-only destination cross — plan only
<!-- finalize:spec-plan:2026-10-13:183e40c6 -->

## 2026-10-13 — [Feature 006] Plan refined: one-click destination removal, eight smart-paste outcomes and owner-bound restoration tests
<!-- finalize:spec-plan:2026-10-13:53c4754c -->

## 2026-10-13 — [Feature 010] Plan refined: uncertain recovery, direct AX verified handoff, read-confirmed last-read restoration and Copy guard
<!-- finalize:spec-plan:2026-10-13:dd714906 -->

## 2026-10-12 — Plan refined: Recovery Bubble — smart paste outcomes and deliberate Copy/changeCount guard replace retired clipboard-window integration; Windows reliable paste retained
<!-- finalize:spec-plan:2026-10-12:1cac4499 -->

## 2026-10-12 — [Feature 009] Plan refined: align Enter with smart paste outcomes, shared output smart_paste configuration and changeCount restoration guard; preserve Windows reliable paste
<!-- finalize:spec-plan:2026-10-12:6e7fa9b3 -->

## 2026-10-12 — Smart Paste — 9 implementation steps, 4 diagrams, 008 output domain, bounded snapshots and truthful outcomes
<!-- finalize:spec-plan:2026-10-12:307d8e77 -->

## 2026-10-12 — [Feature 006] Plan refined: smart-paste terminal results replace retired clipboard-window integration; Windows reliable paste preserved
<!-- finalize:spec-plan:2026-10-12:134acf63 -->

## 2026-10-12 — [Feature 008] Plan refined: replace retired clipboard-window dependencies with smart paste integration; preserve transactional service and domain framework.
<!-- finalize:spec-plan:2026-10-12:7b940570 -->

## 2026-10-12 — [Feature 007]: Spec refined: scope replaced by Smart Paste (clipboard recovery window retired) — 5 stories, 9 AC, 6 FR
<!-- finalize:spec-refine:2026-10-12:3471b770 -->

## 2026-10-12 — [Feature 003] Plan created: Recognition Vocabulary — 6 implementation steps, 3 diagrams
<!-- finalize:spec-plan:2026-10-12:32171392 -->

## 2026-10-12 — [Feature 004] Plan refreshed: Multiple Dictation Shortcuts — 7 implementation steps, 4 diagrams; approved legacy behavior preserved
<!-- finalize:spec-plan:2026-10-12:12d2f055 -->

## 2026-10-12 — [Feature 010] Plan created: Recovery Bubble — 7 implementation steps, 4 diagrams; native recovery/drag, 008 output descriptor, 007 cancellation and QE gates
<!-- finalize:spec-plan:2026-10-12:70754b4a -->

## 2026-10-12 — [Feature 009] Plan corrected after independent review: reliable receipt/success rendezvous and measured Enter readiness
<!-- finalize:spec-plan:2026-10-12:5e51aa59 -->

## 2026-10-12 — [Feature 009] Plan created: PIN Mode and Enter Rule — 7 implementation steps, 4 diagrams; deferred program UI contract
<!-- finalize:spec-plan:2026-10-12:5e13c1dd -->

## 2026-10-12 — [Feature 007] Plan created: Clipboard Recovery Window — 7 implementation steps, 3 diagrams
<!-- finalize:spec-plan:2026-10-12:1f19f822 -->

## 2026-10-12 — Plan created: Destination Application — 7 implementation steps, 4 diagrams
<!-- finalize:spec-plan:2026-10-12:688a7be7 -->

## 2026-10-12 — [Feature 005] Plan refined: Custom AI Prompts — history ordering and cancellation clarified; actual-model independent review
<!-- finalize:spec-plan:2026-10-12:86a3d72b -->

## 2026-10-12 — [Feature 005] Plan created: Custom AI Prompts — 8 implementation steps, 4 diagrams
<!-- finalize:spec-plan:2026-10-12:ea4a26ff -->

## 2026-10-12 — Plan created: Word Replacements — 7 implementation steps, 4 diagrams; UI contracts deferred before spec-ship
<!-- finalize:spec-plan:2026-10-12:cbfc6dff -->

## 2026-10-12 — Plan created: Multiple Dictation Shortcuts — 7 implementation steps, 4 diagrams; shared 008 transactions and deferred program UI preparation
<!-- finalize:spec-plan:2026-10-12:218e2fcd -->

## 2026-10-12 — [Feature 001] Plan created: Microphone Priority — 7 implementation steps, 4 diagrams
<!-- finalize:spec-plan:2026-10-12:7f8d42a3 -->

## 2026-10-12 — Plan created: Settings CLI — 7 implementation steps, 4 diagrams, shared transactions and private IPC
<!-- finalize:spec-plan:2026-10-12:4dabd6e8 -->

## 2026-10-12 — [Feature 008]: Spec created: Settings CLI — 5 stories, 8 AC, 6 FR
<!-- finalize:spec-specify:2026-10-12:d83a0c90 -->

## 2026-10-12 — [Feature 010]: Spec created: Recovery Bubble — 5 stories, 7 AC, 5 FR
<!-- finalize:spec-specify:2026-10-12:f1da6064 -->

## 2026-10-12 — [Feature 009]: Spec created: PIN Mode and Enter Rule — 5 stories, 6 AC, 4 FR
<!-- finalize:spec-specify:2026-10-12:04b5e2f9 -->

## 2026-10-12 — [Feature 006]: Spec created: Destination Application — 5 stories, 8 AC, 6 FR
<!-- finalize:spec-specify:2026-10-12:2ddb4fdd -->

## 2026-10-12 — [Feature 007]: Spec created: Clipboard Recovery Window — 4 stories, 7 AC, 4 FR
<!-- finalize:spec-specify:2026-10-12:aa9ea82f -->

## 2026-10-12 — [Feature 005]: Spec created: Custom AI Prompts — 6 stories, 8 AC, 6 FR
<!-- finalize:spec-specify:2026-10-12:e57cf592 -->

## 2026-10-12 — [Feature 004]: Spec created: Multiple Dictation Shortcuts — 4 stories, 6 AC, 5 FR
<!-- finalize:spec-specify:2026-10-12:bf4ea80d -->

## 2026-10-12 — [Feature 003]: Spec created: Recognition Vocabulary — 4 stories, 7 AC, 5 FR
<!-- finalize:spec-specify:2026-10-12:d5c7ff88 -->

## 2026-10-12 — [Feature 002]: Spec created: Word Replacements — 5 stories, 8 AC, 6 FR
<!-- finalize:spec-specify:2026-10-12:eba866f3 -->

## 2026-10-12 — [Feature 001]: Spec created: Microphone Priority — 4 stories, 8 AC, 8 FR
<!-- finalize:spec-specify:2026-10-12:efcdfaa5 -->

## 2026-10-09 — LiveSpec initialized

- **Type:** Setup
- **Description:** LiveSpec spec system installed
- **Author:** init.sh


## 2026-10-09 — Spec Update: Verified autonomous initialization recovery

- **Description:** Handy observed native/frontend stack, registry, tooling, local assets and hooks verified; 49 goal tasks proved and successful run archived.
- **Code modified:** None; main and original HEAD retained. Application runtime not executed.
- **Evidence:** Read [successful init run](.runs/spec-init-2026-10-09T16-50-33.186059-23de292a-policy2.json).
- **Author:** Codex

## 2026-10-09 — Spec Update: Bounded autonomous initialization closure

- **Description:** Observed Handy stack, registry, local assets, tooling and hooks verified; 48 pre-archive criteria proved.
- **Code modified:** None; original main/HEAD retained. Application runtime not executed.
- **Evidence:** Read [locked goal contract](/var/folders/l_/t1s_zytx2fqb0dkzz40d3v7m0000gn/T/handy-init-prepared-controls-armqc25s/livespec-goals/goal-spec-init-23de292a.contract.json); final closure is recorded by its goal archive.
- **Author:** Codex
