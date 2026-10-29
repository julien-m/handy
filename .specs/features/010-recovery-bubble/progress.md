# Progress — 010-recovery-bubble

Run `impl-20261015` (Claude Code, `/spec-implement`, model claude-opus-5-5), branch `feature/010-recovery-bubble` from `dev` `5bd1a7cd` (009 merged).
Gates before code: `livespec validate --pre-impl` exit 0 (semantic readiness covered, 16/16, no finding) and `livespec validate --progression implement` exit 0 (`Progression: READY`), both with `--model claude-opus-5-5 --review-max-chars 180000`; proofs under `proofs/impl-20261015/`. The review receipts (`.reviews/spec.json`, `.reviews/plan.json`) carry no `carry_over` field; the r8 synthesis findings are answered in `implementation.md` ("Differences from the plan"). Delivery lane: full (High criticality); the end-of-feature delivery review is run by the orchestrator, not by this run. Penflow design profile: BLOCKED (`penflow/code-ir.json` absent), as for 009; the UI follows the mockups and the contract test IDs.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a | Done | `src/overlay/recovery-preview.test.ts`, `src/overlay/recovery-bubble-render.test.tsx`, `src/components/settings/recovery-bubble-toggle-state.test.ts` | `bun test` of the three files before any UI code | RED as required (modules absent, exit 1, `proofs/impl-20261015/step0a-red.*`), GREEN after Step 5 | 2026-10-15 |
| 1 | In Progress (native part pending with Julien) | `src-tauri/src/recovery/platform/{mod.rs,macos.rs,macos_ax.rs,macos_drag.rs,windows.rs,linux.rs}`, `src-tauri/tests/recovery_native.rs`, `scripts/recovery-native-proof.ts`, `proofs/impl-20261015/windows-check/` | `cargo test --test recovery_native` (source check pass, 2 native tests ignored), Windows adapter `cargo clippy --target x86_64-pc-windows-msvc -D warnings` (exit 0) | Adapters written and compiled (macOS; Windows by the harness; Linux only in CI); the spike (first click, AX matrix and 60 ms criterion, drop matrix, VoiceOver, latency) is not run: no app activation, key, clipboard or bubble on Julien's Mac | 2026-10-15 |
| 2 | Done (automated) | `src-tauri/src/recovery/{types.rs,decision.rs,classifier.rs,verdict.rs,instance.rs,manager.rs,ports.rs,service.rs}` and their tests, `src-tauri/src/destination/recovery.rs` (receipt), `src-tauri/tests/{recovery_lifecycle.rs,recovery/support.rs}` | `cargo test --lib recovery`, `cargo test --test recovery_lifecycle` | Pass (810-case decision table, classifier deadline at 49/50/51 ms, 15 lifecycle cases) | 2026-10-15 |
| 3 | Done (automated) | `src-tauri/src/settings.rs`, `src-tauri/src/recovery/setting.rs`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,platform_capability.rs,domains/output.rs}`, gate-aware 008 tests, `src/bindings.ts`, `README.md` | `cargo test --test recovery_settings --test settings_validation --test settings_cli --test settings_transactions` (both gate states) | Pass | 2026-10-15 |
| 4 | Done (automated) | `src-tauri/src/destination/{delivery.rs,presentation.rs,guard.rs,insertion.rs,outcome.rs,manager.rs,app.rs}`, `src-tauri/src/smart_paste/{actor.rs,coordinator.rs,dispatch.rs,app.rs,macos_cf.rs}`, `src-tauri/src/{clipboard.rs,overlay.rs,actions.rs,lib.rs}`, `src-tauri/src/recovery/{app.rs,surface.rs}`, `src-tauri/src/commands/recovery.rs`, `src/overlay/use-overlay-notices.ts`, `src-tauri/tests/{recovery_output.rs,recovery_clipboard.rs}` | `cargo test --test recovery_output --test recovery_clipboard --lib destination::` | Pass (zero Enter with Always, typed `no_text_field`, recheck after `Insert`, Copy through 007's real actor and coordinator) | 2026-10-15 |
| 5 | Done (automated; native window and Penflow certification pending) | `src/overlay/{recovery.html,recovery-main.tsx,recovery-window.tsx,recovery-bubble.tsx,recovery-bubble.css,recovery-preview.ts,use-recovery-bubble.ts}`, `src/components/settings/{recovery-bubble-toggle.tsx,recovery-bubble-toggle-state.ts,advanced/AdvancedSettings.tsx}`, `src/stores/settingsStore.ts`, 26 locales, `vite.config.ts`, `tests/recovery-bubble.spec.ts`, `tests/harness/recovery-*`, baselines | `bun run test:recovery-bubble`, `bun run test:output-settings`, `bun run test:recovery-bubble-ui`, `bun run check:translations` | Pass (10 Chromium cases, 5 baselines compared by eye with the mockups) | 2026-10-15 |
| 6 | In Progress (native drag pending with Julien) | `src-tauri/src/recovery/platform/macos_drag.rs`, `src-tauri/src/recovery/{manager.rs,service.rs,surface.rs}`, `src/overlay/use-recovery-bubble.ts` | lifecycle and gesture tests above | Identity-guarded drag lifecycle and gesture pass; the AppKit session (full payload, copy only, hidden while dragging, TextEdit/Notes/ChatGPT/Comet) is not run | 2026-10-15 |
| 7 | In Progress (native acceptance pending with Julien) | `.github/workflows/test.yml`, `proofs/impl-20261015/` (`summary.txt`, `final/`, `acceptance-mapping.json`) | Final verification (`proofs/impl-20261015/summary.txt`: tsc, lint, format, translations, build, 12 bun suites, every Playwright spec, cargo fmt, clippy lib/bins and all-targets and the full cargo test in the four feature combinations, conventions repo, progression, feature validation, acceptance mapping; addendum: privacy source check in the four combinations) | 39/39 exit 0, addendum 8/8 exit 0; macOS native acceptance not run | 2026-10-15 |

## Delivery review round 1 — corrections

Review PASS and audit PASS (`.reviews/delivery.json`), with warnings corrected before the merge on the coordinator's instruction. Verification in `proofs/impl-20261015-fix1/summary.txt`.

| Finding | Correction | Tests |
|---|---|---|
| False bubbles in Chromium and Electron (`classifier.rs`) | "Not editable" only on positive proof: a known non-text role (list, button, image…) whose three edit attributes are read and not settable; `AXWebArea`, `AXWindow`, `AXGroup`, `AXApplication`, `AXScrollArea` and any unreadable attribute are undetermined (`Insert`); Comet, Chrome, ChatGPT, Slack and VS Code added to the native matrix | `classifier_tests::{chromium_and_electron_containers_are_undetermined, an_unreadable_edit_attribute_is_a_doubt_never_a_proof}`, `recovery_output::an_editable_or_undetermined_focus_inserts_as_usual` |
| Text lost when a new recording fails to start (`actions.rs`) | The setting is frozen at the start; the bubble closes only in `dictation_started`, after `try_start_recording` accepted the recording | `recovery_lifecycle::a_dictation_whose_recording_fails_to_start_keeps_the_bubble_and_its_text`, `recovery_native::the_bubble_closes_only_after_the_recording_started` |
| 005 AI notice cut (`overlay.rs`) | `yield_to_recovery` no longer ends notice holds; the 005 and 007 notices keep their duration | `recovery_native::presenting_the_bubble_never_cuts_an_indicator_notice` |
| Late read only behind "Why?" | Shown in the bubble body (`role="status"`, same key, no i18n change) | `recovery-preview.test.ts`, `recovery-bubble-render.test.tsx`, `tests/recovery-bubble.spec.ts` |
| Answer lost at the 2 s guard (`recovery.rs`) | At expiry the receipt is closed then drained (`after_guard`): one side reports, never none | `recovery_tests::an_answer_at_the_guard_boundary_is_never_lost` |
| Sentinel test was a source grep | Runtime test with a capturing logger and the recording surface over every bubble path | `tests/recovery_privacy.rs` |
| Emit isolation by convention | Documented; a test checks no other frontend source listens to the bubble events | `recovery_native::only_the_recovery_window_listens_to_the_bubble_events` |
| Bubble window in the `default` capability | Own capability `recovery-bubble` (`core:event:default`, `os:allow-locale`) | build (capability schema), Playwright |
| Dead 006 notice code | `destinationNotice`, `parseOutcome`, `OutcomeFacts`, `selectionName`, their tests and `overlay.destination.notice.*` in 26 locales removed | `bun run test:indicator`, `check:translations` |
| Functions over 30 lines | `RecoveryBubble`, `Actions`, `Header` split into small components; `useActions` extracted | conventions |
| Info: CF type and drag source lifetime | Focused element checked as `AXUIElement` before any AX call; drag source autoreleased at the end of the run-loop turn | build, clippy |

## Delivery review round 2 — corrections

Review BLOCKED on one point and audit PASS (`.reviews/delivery.json`, round 2); corrected on the coordinator's instruction. Verification in `proofs/impl-20261015-fix2/summary.txt`. 38/39 exit 0; the implement progression gate is BLOCKED (`analyze_not_ready:incomplete`) by the LiveSpec update `4a5ac29` merged during the run, which rebuilds the round-8 plan review receipt as incomplete with unchanged context and sources (the unchanged parent `929277a6` is blocked the same way): the plan review is to be refreshed by the orchestrator.

| Finding | Correction | Tests |
|---|---|---|
| Blocking: Copy and drag without the paste's trailing space (AC-003) | 006's slot keeps the exact inserted text beside the transcript (`retain_final` with the `append_trailing_space` of the insertion's settings snapshot, `read_inserted`); Copy and drag carry it, the preview stays the transcript | `recovery_tests::the_slot_keeps_the_exact_inserted_text_with_and_without_the_trailing_space`, `recovery_lifecycle::copy_and_drag_carry_the_paste_trailing_space_and_the_preview_never_shows_it` (both setting states) |
| Bubble over a held 005/007 notice (`overlay.rs`) | While a notice holds the indicator, the bubble is placed beside its window (above at the bottom edge, below at the top edge, 8 pt, the other side without room, inside the indicator's monitor); a hold that starts while the bubble is shown moves it the same way | `overlay::surface::tests::{the_bubble_stacks_above_a_held_bottom_notice_and_below_a_held_top_one, without_room_on_its_side_the_bubble_takes_the_other_and_never_leaves_the_screen}`; visual proof native (checklist) |
| 009 traceability still citing `notice-model.ts` | `.specs/features/009-pin-mode-and-enter-rule/implementation.md` FR-002 and AC-003 rows point to the Rust anchors; 009 row added to the 010 deltas | documentation |
| Checklist line displaced | "Couvre : AC-001 … method-failures/" back under its item; the notices item asks that both stay readable | documentation |
| Comments citing the removed 006 notice | `overlay.rs`, `RecordingOverlay.tsx`, `notice-model.ts` (`NOTICE_MIN_MS`: 005 AC-005, 007 AC-012) and the indicator test headers | lint |
| Sentinel test comment | Successful Copy and accepted drop added (one dictation each); helpers split; comment aligned | `recovery_privacy` |

## Delivery review round 3 — placement fix

Review PASS and audit PASS with warnings (`dr010-review-r3.json`, `dr010-audit-r3.json`); corrected on `fix/010-bubble-placement` (from `dev` `96eb3e1e`) on the coordinator's instruction. Verification in `proofs/fix-placement/summary.txt`.

| Finding | Correction | Tests |
|---|---|---|
| Bubble covered by a notice measured after its hold (`overlay_indicator.rs:108`) | `set_overlay_extent` places the indicator, then calls `recovery::surface::follow_indicator` while a notice holds it (`SurfaceState::bubble_follows_extent`); moving the bubble never changes the indicator's extent, so no placement loop; without a recovery window (gate closed) nothing happens | `overlay::surface::tests::{only_a_held_indicator_moves_the_bubble_when_its_extent_changes, a_notice_measured_after_the_hold_would_cover_the_bubble_until_it_is_stacked_again}`, `recovery_native::the_bubble_is_placed_again_after_a_held_extent_change_and_a_refused_drag` |
| `show_again` after a refused drag not placed (`surface.rs:166`) | `place` before `restore_after_drag`, same rule as above | `recovery_native::the_bubble_is_placed_again_after_a_held_extent_change_and_a_refused_drag` |
| `deliver` 31 lines and anonymous `(String, bool)` (`delivery.rs:90`) | `FinalText { text, append_trailing_space }` read once from the settings snapshot; `publish` hands it to `RecoverySlot::retain_failed`; `deliver` back to 30 lines | `delivery::tests::the_insertion_settings_carry_the_trailing_space_to_the_recovery_slot`, `recovery::tests::the_slot_keeps_the_exact_inserted_text_with_and_without_the_trailing_space` |
| `tests/recovery_lifecycle.rs` 406 lines | AC-003 payload test moved to `tests/recovery_payload.rs` (375 lines left) | `recovery_payload::copy_and_drag_carry_the_paste_trailing_space_and_the_preview_never_shows_it` |
| `append_trailing_space` to the slot not tested end to end | settings snapshot with the setting on and off → `FinalText::of` → `retain_failed` of a failed report → outcome reference → `read_inserted` (simulated workspace and clock, real slot); a non-failed status keeps nothing | `delivery::tests::the_insertion_settings_carry_the_trailing_space_to_the_recovery_slot` |

The native proof of both windows readable (item "Avis de l'indicateur conservés", now with an app quit right after the AI failure) stays pending with Julien; nothing was shown, activated, typed or copied on his Mac.

## Step 1 — feasibility notes

- The native read-only tests are written and ignored by default; they need a logged-in session and Accessibility for the runner, never activate an application, type or touch the clipboard.
- The plan's macOS gate of Step 1 (first click without activation, full TextEdit/Notes drop, AX within 50 ms with no sample above 60 ms on the main thread, 006's foreground read unchanged, presentation delays recorded) is not evidenced: Steps 2 to 7 were implemented on the automated evidence, as for 009, by instruction (native proofs with Julien). If the AX criterion fails natively, field detection is not delivered (verdict `Insert`) and the feature stays out of Implemented until Julien decides (plan).

## Native proofs pending with Julien

None is declared passed. Exact procedure: `.specs/native-proofs-checklist.md`, section "010 — Bulle de récupération", with `bun scripts/recovery-native-proof.ts --binary <absolute release binary> --platform macos --out .specs/features/010-recovery-bubble/proofs/native-macos/ --interactive` on `bun run tauri build --bundles app --features destination-return,smart-paste-dispatch`.

1. Closed gate identical to `dev` (`handy-gates-closed.app`): `paste-error`, no bubble, key `not_found`, no toggle.
2. Surface spike: first click on Copy, Close and Why? without activating Handy; bubble in place of the indicator, also with overlay style none.
3. Classification matrix (Finder list and desktop, Safari page without field, TextEdit, Notes, search field, unknown control; Comet, Chrome, ChatGPT, Slack and VS Code with a focused field, never a bubble; Spotlight over Finder and over TextEdit) and `recovery_native` (30 samples, none above 60 ms; 006's foreground read unchanged).
4. Reasons before insertion with Automatic Enter Always (closed, pinned not running, not brought forward, no text field): texts, no Enter, no `paste-error`.
5. Method failures: smart paste not read then late read, permission missing, classic failure.
6. Native drag (SC-002): TextEdit and Notes full transcript, ChatGPT and Comet recorded; refusal and Escape; modifiers; new dictation during the drag. Since N24 (2026-10-29), also from the text, background and border of the bubble, with Copy, Why? and Close kept as clicks and the grab cursor. Since N25 (2026-10-29), Escape during a drag inserts nothing and closes the bubble (log `drag cancelled by Escape, dismissed`), while a release on the desktop without Escape keeps it for another attempt. Since N26 (2026-10-29), after Escape the drag image does not slide back and the bubble never flashes before closing.
7. Copy against a pending smart paste restoration.
8. Lifetime: 60 s, screen lock, Close, new dictation after 1 s, a start that fails keeps the bubble.
8b. The 005 and 007 notices keep their duration when the bubble appears, and both stay readable (bubble beside the indicator, top and bottom).
9. VoiceOver names and activation; settings toggle with Tab and Space.
10. Decision-to-visible samples against the 300 ms QE target.
11. CLI with Handy running.

Pending release evidence (never a gate): Windows reduced bubble and Narrator, Linux method-failure bubble on X11, Wayland and with layer shell disabled, Orca, the Windows `cargo test --features destination-return`, and the CI jobs of `test.yml` (checklist §5).
