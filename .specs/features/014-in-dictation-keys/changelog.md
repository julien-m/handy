# Changelog - 014-in-dictation-keys

## 2026-10-29 — [Feature]: Keys during a dictation: send to the app in front, insert without Enter

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md, implementation.md, progress.md)
- **Code modified:** `src-tauri/src/dictation_keys/` (new: actions, lone-press detector, observer), `src-tauri/src/destination/{dictation_toggles.rs,session.rs,snapshot.rs,manager.rs,app.rs,mod.rs}`, `src-tauri/src/automatic_enter/{policy.rs,transaction.rs}`, `src-tauri/src/shortcut/{mod.rs,edits.rs,handler.rs,action_shortcuts.rs,conflicts.rs}`, `src-tauri/src/{settings.rs,actions.rs,lib.rs}`, `src/components/settings/{shortcut-list*.ts*,general/GeneralSettings.tsx}`, `src/overlay/{enter-skipped-mark.tsx,enter-skipped-mark.css,indicator-stack.tsx}`, `src/bindings.ts`, 26 locales, `package.json`, `README.md`, tests
- **AC impacted:** AC-001 to AC-006 (automated evidence; native proof pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's requests of 2026-10-29 relayed by the coordinator: a start shortcut « Dictate to the active app » was built first, then replaced (option 2) by a key pressed during the dictation; the « Insert without Enter » key was added to the same feature; final defaults Option left and Control left, pressed alone (left hand, right hand on the mouse). Decisions in `spec.md` Clarifications.
- An independent review of the first observer found the macOS listener reports a modifier held at its start the wrong way round, key repeats cancelled lone presses, a second press could restore an unpinned route, a dead listener could spin, and the mark could sit outside the pill; all fixed before commit (see `implementation.md`).

## 2026-10-30 — [Bugfix]: 1 gap closed (1 functional, 0 visual)

- **Type:** Bug Fix
- **Spec modified:** Yes (AC-007 added, FR-003 references AC-007, AC-001 and Story 1 scenario « Without a pin » amended, decision `no_pin_notice`, edge case after an unpin)
- **Code modified:** `src-tauri/src/destination/{dictation_toggles.rs,dictation_toggles_tests.rs}`, `src-tauri/src/dictation_keys/watcher.rs`, `src/overlay/{notice-model.ts,use-overlay-notices.ts,overlay-notices.tsx,indicator-screen.ts,indicator.css}` and their tests (new `src/overlay/no-pin-notice.test.ts`, in `test:indicator`), `package.json`, 26 locales (`overlay.dictationKeys.noPin`)
- **Gaps closed:** AC-007 (decision `no_pin_notice`, Julien, 2026-10-30)
- **Remaining:** native proof of the notice (coordinator, signed build); Penflow mockup of the notice
- **Author:** spec-fix (Claude Code, claude-opus-5-5)
- Pressing « Send to the App in Front » with nothing pinned did nothing visible, so the key looked broken. `toggle_active_app` now returns `RetargetOutcome` (`Changed`, `NoPin`, `Unchanged`) instead of `Option`, so the observer tells « no pin » apart from a frontmost route the key did not set with a pin; only `NoPin` logs at debug level and emits `dictation-key-no-pin`, which the indicator shows as the « No pinned app » notice (4 s, replaced by a newer one).
