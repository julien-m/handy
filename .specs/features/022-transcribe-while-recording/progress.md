# Progress — 022-transcribe-while-recording

Run of 2026-11-03 to 2026-11-05 (Claude Code, light `/spec-feature`, model claude-opus-5-5), branch `dev`. Conventions loaded from `.conventions/index.md` (sub-domain `code`). Native agents only; no Codex, no `audit` skill.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | Prefetch worker, reuse at the stop, `--live` | `cargo test --lib` (1420 passed) | Pass | 2026-11-03 |
| 2 | Done | Diagnosis: live replay 21–25 s on a loaded machine; per-stage timing; App Nap confirmed with `NSAppSleepDisabled` (2.08 s → 0.67 s) | headless `--live` from a normal-priority terminal | Cause found | 2026-11-05 |
| 3 | Done | Batch of the missing pieces (`RunBatch`), `Info.plist` | `cargo test --lib` (1422 passed, 0 failed, 19 ignored); clippy `-D warnings` 0 | Pass | 2026-11-05 |
| 4 | In progress | Signed build, measurement with the bundled app | — | Pending | 2026-11-05 |
| 5 | Pending (Julien) | Native long dictation | — | Not run | 2026-11-05 |
