# Changelog - 014-in-dictation-keys

## 2026-10-29 — [Feature]: Keys during a dictation: send to the app in front, insert without Enter

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md, implementation.md, progress.md)
- **Code modified:** `src-tauri/src/dictation_keys/` (new: actions, lone-press detector, observer), `src-tauri/src/destination/{dictation_toggles.rs,session.rs,snapshot.rs,manager.rs,app.rs,mod.rs}`, `src-tauri/src/automatic_enter/{policy.rs,transaction.rs}`, `src-tauri/src/shortcut/{mod.rs,edits.rs,handler.rs,action_shortcuts.rs,conflicts.rs}`, `src-tauri/src/{settings.rs,actions.rs,lib.rs}`, `src/components/settings/{shortcut-list*.ts*,general/GeneralSettings.tsx}`, `src/overlay/{enter-skipped-mark.tsx,enter-skipped-mark.css,indicator-stack.tsx}`, `src/bindings.ts`, 26 locales, `package.json`, `README.md`, tests
- **AC impacted:** AC-001 to AC-006 (automated evidence; native proof pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's requests of 2026-10-29 relayed by the coordinator: a start shortcut « Dictate to the active app » was built first, then replaced (option 2) by a key pressed during the dictation; the « Insert without Enter » key was added to the same feature; final defaults Option left and Control left, pressed alone (left hand, right hand on the mouse). Decisions in `spec.md` Clarifications.
- An independent review of the first observer found the macOS listener reports a modifier held at its start the wrong way round, key repeats cancelled lone presses, a second press could restore an unpinned route, a dead listener could spin, and the mark could sit outside the pill; all fixed before commit (see `implementation.md`).
