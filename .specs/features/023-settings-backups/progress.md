# Progress — 023-settings-backups

Run of 2026-11-07 (Claude Code, light `/spec-feature`, model claude-opus-5-5), branch `dev`. Conventions loaded from `.conventions/index.md` (sub-domain `code`). Native agents only; no Codex, no `audit` skill.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | Diagnosis: an older build in `/Applications` rewrote the store at 07:11 and 07:12; six settings back to defaults, replacement rules lost, no copy anywhere | Log and store inspection | Cause found | 2026-11-07 |
| 2 | Done | Backups, startup restore, newer-schema guard | `cargo test --lib` | Pass | 2026-11-07 |
| 3 | Pending | Signed build installed in `/Applications`, replacing the old one | — | Not run | 2026-11-07 |
| 4 | Pending (Julien) | Native check | — | Not run | 2026-11-07 |
