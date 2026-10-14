# Progress — 001-microphone-priority

Read [the specification](spec.md) and [the plan](plan.md) before continuing. Delivery lane: **full** ([delivery-lane.stdout](proofs/claude-impl/preflight/delivery-lane.stdout)); per step only targeted tests, lint, format and the conventions gate run; the end-of-feature delivery review is run by other agents, not by this implementer. Scope of this run: the non-UI steps 1–5; Step 6 (settings UI editor) waits for the common Penflow pass and Step 7 for the native and UI evidence.

Preflight: [preflight](proofs/claude-impl/preflight/preflight.stdout) OK (11/11), [progression implement](proofs/claude-impl/preflight/progression-implement.stdout) READY.

| Step | Status | Files | Tests run | Result | Updated at |
| --- | --- | --- | --- | --- | --- |
| 1 — Bounded native ownership and capture loss | Done (automated); macOS native run pending | `audio_toolkit/audio/{attempt.rs, attempt_tests.rs, recorder/loss_tests.rs}` (new), `audio_toolkit/audio/{recorder.rs, mod.rs}`, `src-tauri/tests/microphone_native.rs` (new, ignored native harness), `scripts/microphone-native-proof.ts` (new) | [lib audio_toolkit::audio](proofs/claude-impl/step1/test-lib-audio.stdout), [native harness build](proofs/claude-impl/step1/native-harness-build.exit), [clippy lib/bins](proofs/claude-impl/step1/clippy.exit), [cargo fmt](proofs/claude-impl/step1/cargo-fmt-check.exit), [lint](proofs/claude-impl/step1/bun-lint.exit), [format:check](proofs/claude-impl/step1/bun-format-check.exit), [tsc](proofs/claude-impl/step1/tsc.exit), [conventions](proofs/claude-impl/step1/conventions-repo-summary.json), [enumeration probe in the agent sandbox](proofs/claude-impl/step1/native-enumeration-agent-sandbox.exit) | 38 passed (7 attempt, 3 loss); all checks exit 0; conventions PASS, baseline unchanged. Native: in the agent sandbox CoreAudio enumeration never returned (900 s run failed, 20 s alarm exit 142), so no macOS device evidence exists yet | 2026-10-14 11:45 |
| 2 — Schema, migration and transaction validation | Todo | | | | |
| 3 — Pure ordered resolver and catalog projection | Todo | | | | |
| 4 — Stream ownership and recording boundaries | Todo | | | | |
| 5 — `microphones` domain and typed commands | Todo | | | | |
| 6 — Accessible draft editor and status refresh (UI) | Todo — out of this run's scope | | | | |
| 7 — Acceptance, regression and native evidence | Todo | | | | |

## Notes

### Step 1

- Every open waits at most `OPEN_ATTEMPT_TIMEOUT` (2 s, AC-003). A timed-out worker is abandoned, never joined; if its native open completes later, `AttemptWorker::publish` refuses and the worker drops the stream itself. `OpenLedger` counts abandoned workers still running; at `MAX_STALLED_OPENS` (4) a new open is refused with an error, never a successful capture.
- Loss: the consumer reports a stream error once per stream through `with_loss_callback`; Stop no longer waits the 2 s pause acknowledgement of a dead stream; the processed recording buffer is shared so `stop()` returns the processed prefix even if the consumer thread died.
- Native evidence not produced: the macOS proof (`bun scripts/microphone-native-proof.ts --platform macos --out <dir> [--binary <abs>] [--interactive]`) must run outside the agent sandbox, by an operator, with microphone permission. It blocks `Implemented` (constitution), not the next implementation steps of this run.
