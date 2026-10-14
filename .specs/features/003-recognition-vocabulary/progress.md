# Progress — 003-recognition-vocabulary

`/spec-implement`, light lane (`proofs/claude-review/delivery-lane-r3.json`). Branch
`feature/003-recognition-vocabulary`, worktree from `dev`. Implementer: Claude (`claude-opus-5-5`).
Scope of this run: every non-UI step of the plan. The settings screen (Step 5) waits for the
Penflow pass and is not started. Per-step evidence lives in `proofs/claude-impl/<step>/`
(`*.stdout`, `*.stderr`, `*.exit`).

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a Behavioral TDD | Todo | — (UI tests of the Behavioral AC, written with Step 5) | — | Deferred with the UI | 2026-10-14 |
| 1 Capability and budget | Done | `recognition_vocabulary/{mod,capability,budget,context}.rs`, `engine_supervisor/{protocol,worker,supervisor,supervisor_context}.rs`, `contracts/context-budget.md` | `cargo test --lib -- recognition_vocabulary engine_supervisor::protocol engine_supervisor::supervisor::context` | Pass (18) | 2026-10-14 |
