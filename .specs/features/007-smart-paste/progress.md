# Smart Paste — implementation checkpoints

This is a partial core-first run, authorized on 2026-10-13. Inspect [the plan](plan.md) and [the specification](spec.md) before resuming. Feature 008 is being implemented in a different worktree; no substitute settings service, revision, CLI framework or migration is introduced here.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a — settings behavioral TDD | Waiting for 008 | None | Not run | UI and shared settings foundation deferred | 2026-10-13 |
| 1 — native feasibility | Done — implemented, certification pending | [Native fixture test](../../../src-tauri/tests/smart_paste_native.rs), [harness](../../../scripts/smart-paste-native-proof.ts), [AppKit](evidence/native-spike.json), [AX](evidence/native-ax-spike.json) and [TIS](evidence/native-keyboard-spike.json) observations | `cargo test --manifest-path src-tauri/Cargo.toml --test smart_paste_native -- --ignored --test-threads=1 --nocapture` 1/0; full `cargo test` 286 passed/0 failed/17 ignored; clippy (5 existing warnings, 0 new); fmt; `bun run lint`; `bun run format:check`; conventions PASS — see [step-1 proofs](proofs/claude/) | AppKit evidence regenerated against current harness hashes; AX and TIS receipts match their probe hashes. AppKit cache limitation (pre-key and repeated reads served without callback) recorded: conservative `not_read`, quiet measured from observed reads only | 2026-10-13 |
| 2 — pure state and outcomes | Todo | None | Not run | Awaiting Step 1 native observations | 2026-10-13 |
| 3 — bounded AX insertion | Todo | None | Not run | Awaiting pure contracts and AX feasibility | 2026-10-13 |
| 4 — bounded snapshot and provider | Todo | None | Not run | Awaiting native feasibility | 2026-10-13 |
| 5 — serialized coordinator | Todo | None | Not run | Awaiting native adapters and pure state | 2026-10-13 |
| 6 — per-app keyboard strategy | Todo | None | Not run | Awaiting native input-source observations | 2026-10-13 |
| 7 — insertion pipeline and consumers | Todo | None | Not run | Destination 006 context is absent; implement independent contract portions | 2026-10-13 |
| 8 — settings/revision/CLI/UI integration | Waiting for 008 | None | Not run | AC-014, AC-015 and settings behavioral AC remain incomplete | 2026-10-13 |
| 9 — native/release evidence | Todo | None | Not run | Headless feasible observations required; interactive and unavailable targets remain pending release evidence | 2026-10-13 |

## Scope and remaining dependencies

- No UI code, final acceptance certification or Implemented lifecycle transition is authorized by this partial run.
- Destination 006 activation/PID/generation, automatic Enter 009 and recovery bubble 010 must consume the typed insertion contracts when those features are delivered.
- An unavailable app, layout, accessibility prompt or interactive field check is pending release evidence with an exact procedure, never a native PASS.
- After each completed independent step, the supervisor audits and commits its verified files before the next step starts.
