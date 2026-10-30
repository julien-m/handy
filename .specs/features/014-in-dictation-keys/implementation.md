---
feature: in-dictation-keys
title: Keys During a Dictation Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-30
---

# Implementation — 014-in-dictation-keys

Status: In Progress (automated evidence complete; native macOS proof pending with Julien). Branch `feat/dictate-to-active-app` from `dev` `b0de2fe1`. The observer runs only for a dictation with a destination session (`destination-return` rollout gate open); the bindings exist in every build.

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Two in-dictation key actions](spec.md#fr-001) | `src-tauri/src/dictation_keys/mod.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/shortcut/{action_shortcuts.rs,mod.rs,edits.rs}`, `src/components/settings/{shortcut-list-model.ts,shortcut-list-feedback.ts,shortcut-list.tsx,shortcut-list-rows.tsx,general/GeneralSettings.tsx}` | `@spec FR-001: Two in-dictation key actions`, `@spec FR-001: One key or none, so the CLI can change it` | ✅ Implemented | 2026-10-29 |
| [FR-002: Lone press only](spec.md#fr-002) | `src-tauri/src/dictation_keys/tap.rs` | `@spec FR-002: Tap only: press then release, nothing between` | ✅ Implemented | 2026-10-29 |
| [FR-003: Re-route, pin intact](spec.md#fr-003) | `src-tauri/src/destination/dictation_toggles.rs`, `src-tauri/src/dictation_keys/watcher.rs`, `src/overlay/{notice-model.ts,use-overlay-notices.ts,overlay-notices.tsx,indicator-screen.ts}` | `@spec FR-003: Re-route this dictation, pin intact`, `@spec AC-007: No-pin notice on the indicator` | ✅ Implemented | 2026-10-30 |
| [FR-004: Skip Enter for this dictation](spec.md#fr-004) | `src-tauri/src/destination/{dictation_toggles.rs,session.rs,snapshot.rs}`, `src-tauri/src/automatic_enter/{policy.rs,transaction.rs}`, `src/overlay/{enter-skipped-mark.tsx,enter-skipped-mark.css,indicator-stack.tsx}`, `src/bindings.ts` | `@spec FR-004: Skip Automatic Enter for this dictation`, `@spec FR-004: Suppressed(UserSkipped)`, `@spec AC-003`, `@spec AC-004` | ✅ Implemented | 2026-10-29 |
| [FR-005: Observer from start to insertion](spec.md#fr-005) | `src-tauri/src/dictation_keys/watcher.rs`, `src-tauri/src/actions.rs`, `src-tauri/src/destination/app.rs`, `src-tauri/src/shortcut/handler.rs` | `@spec FR-005: Keys read from start to insertion` | ✅ Implemented | 2026-10-29 |
| [FR-006: Lone-press conflict rule](spec.md#fr-006) | `src-tauri/src/shortcut/{conflicts.rs,conflicts_lone_press.rs,action_shortcuts.rs}` | `@spec FR-006: Lone-press keys conflict only when one press fires both` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| AC-001 | `src-tauri/src/destination/dictation_toggles_tests.rs`, `src-tauri/tests/in_dictation_keys.rs` | ✅ (automated; native pending) |
| AC-002 | `src-tauri/tests/in_dictation_keys.rs` | ✅ |
| AC-003 | `src-tauri/src/destination/dictation_toggles_tests.rs`, `src-tauri/tests/in_dictation_keys.rs` | ✅ |
| AC-004 | `src/overlay/enter-skipped-render.test.tsx`, `bun run check:translations` | ✅ (markup; native look pending) |
| AC-005 | `src-tauri/src/dictation_keys/tests.rs`, `src-tauri/src/dictation_keys/tap_tests.rs`, `src-tauri/tests/shortcuts_cli.rs`, `src/components/settings/shortcut-list-model.test.ts`, `src/components/settings/shortcut-list-feedback.test.ts`, `src/components/settings/shortcut-list-dictation-keys-render.test.tsx` | ✅ (automated; native pending) |
| AC-006 | `src-tauri/src/dictation_keys/tests.rs`, `src-tauri/src/dictation_keys/tap_tests.rs` | ✅ (pure; native pending) |
| AC-007 | `src-tauri/src/destination/dictation_toggles_tests.rs` (`NoPin` against `Unchanged`), `src/overlay/no-pin-notice.test.ts`, `src/overlay/indicator-render.test.tsx`, `src/overlay/notice-listeners.test.ts`, `bun run check:translations` | ✅ (automated; native pending) |

## Files Created/Modified

- Created: `src-tauri/src/dictation_keys/{mod.rs,tests.rs,tap.rs,tap_tests.rs,watcher.rs}`, `src-tauri/src/destination/{dictation_toggles.rs,dictation_toggles_tests.rs}`, `src-tauri/src/shortcut/conflicts_lone_press.rs`, `src-tauri/tests/in_dictation_keys.rs`, `src/overlay/{enter-skipped-mark.tsx,enter-skipped-mark.css,enter-skipped-render.test.tsx}`, `src/components/settings/shortcut-list-dictation-keys-render.test.tsx`.
- Moved without change: the inline tests of `src-tauri/src/shortcut/conflicts.rs` to `conflicts_tests.rs` (the file passed the 500-line limit of the conventions check).
- Modified: `src-tauri/src/{lib.rs,settings.rs,actions.rs}`, `src-tauri/src/destination/{mod.rs,session.rs,session_tests.rs,snapshot.rs,manager.rs,app.rs}`, `src-tauri/src/automatic_enter/{policy.rs,transaction.rs}`, `src-tauri/src/shortcut/{mod.rs,edits.rs,handler.rs,action_shortcuts.rs,conflicts.rs}`, `src-tauri/tests/shortcuts_cli.rs`, `src/bindings.ts` (`DestinationSnapshot.enter_skipped?`), `src/components/settings/{shortcut-list-model.ts,shortcut-list-feedback.ts,shortcut-list.tsx,shortcut-list-rows.tsx,general/GeneralSettings.tsx}` and their tests, `src/overlay/indicator-stack.tsx`, 26 locales, `package.json` (`test:indicator`), `README.md`.

## Review findings fixed before commit

An independent read-only review of the first version found, and this commit fixes:

1. macOS: the HandyKeys listener toggles its own modifier state, which starts empty, so a modifier held when the observer starts (the dictation shortcut itself) was reported the wrong way round and blocked every later lone press. The observer now reads the held modifiers from the system (`CGEventSourceFlagsState`, side bits) and derives each direction (`ModifierState`).
2. Push-to-talk Option+Space: the auto-repeat of Space cancelled any lone press. Repeats of a key already down are ignored.
3. A second press after an unpin or a badge choice could restore an unpinned route: the restore now needs the same selection version and a pin.
4. A dead listener made the loop spin: it now stops on any error but a timeout.
5. The shortcut count was read late after a 30 ms sleep: the press is now recorded at once and the lone press confirmed 30 ms after its release, without blocking the loop.
6. The mark could sit outside the pill without a badge: it now lives in the destination segment, which the overlay sizes for.

## Gaps

- **Penflow:** `penflow/ui.pen` and the flow-ui contract do not show the two settings rows, the indicator mark nor the no-pin notice (new test ids `shortcut-settings-active-app-key-*`, `shortcut-settings-no-enter-key-*`, `recording-indicator-enter-skipped`, `recording-indicator-no-pin-notice`, screen `recording-indicator-no-pin`).
- **LiveSpec pipeline gates:** the spec and plan pass `livespec validate`; the reviewer-bound gates of the pipeline (spec review receipt, clarify, plan review, analyze, goal archive) were not run in this autonomous pass; one independent native review of the code was run instead (findings above).
- **Tauri keyboard implementation:** the keys work (the observer is HandyKeys' listener) but the settings recorder of the Tauri implementation cannot record a lone modifier; change them from the CLI.
- **Limits:** a modifier used with a scroll or a gesture is seen as a lone press; a lone press released less than 30 ms before the insertion freeze is refused.

## Native proof protocol (macOS, Julien)

Build with `bun run tauri dev -- --features destination-return,smart-paste-dispatch` (HandyKeys implementation, accessibility granted). Bindings as today (`transcribe` = option+space, command_left+j, command_right; post-process option+shift+space; cancel escape). In Settings > General check the rows « Send to the App in Front » (Option left) and « Insert without Enter » (Control left).

1. Pin Claude from the badge. Bring Codex to the front, start a dictation with Command right, speak, tap Option left alone: the badge shows « Frontmost app » without the pin mark; stop: the text goes to Codex. Start a new dictation from Codex: badge « Claude » with the pin mark; the text goes to Claude.
2. During a dictation tap Option left twice: the badge returns to « Claude » with the pin mark.
3. Unpin, start a dictation, tap Option left: the route does not change and the indicator shows « No pinned app » (« Aucune application épinglée » in French) for about 4 s; the debug log reads `In-dictation key of dictation N: no pinned app`. With Claude pinned and the route set to « Frontmost app » by the badge cross, Option left shows nothing.
4. Automatic Enter « Only in PIN mode », Claude pinned: re-route with Option left: no Enter in Codex. Without the key: Enter in Claude.
5. Automatic Enter « Always »: during a dictation tap Control left: the struck-through Enter mark appears next to the badge (VoiceOver reads « Automatic Enter off for this dictation »); the text is inserted, no Enter; tap twice: no mark, Enter sent. The setting stays « Always ».
6. During a dictation type Shift+A, Ctrl+C, and press Option+Space (push-to-talk on Option+Space: hold it, tap Control left while holding, release): no re-routing from Option, Control left still skips Enter, Option+Space ends the push-to-talk as before.
7. Outside a dictation, tap Option left and Control left in another app: nothing happens in Handy; Option and Control work normally in that app.
8. `handy settings shortcuts list skip_automatic_enter`, then `remove skip_automatic_enter control_left`, `add skip_automatic_enter shift_left`, `reset skip_automatic_enter` against the running app.
9. In the logs (debug): `In-dictation key applied to dictation N`, `Automatic Enter of dictation N: Suppressed(UserSkipped)`.
