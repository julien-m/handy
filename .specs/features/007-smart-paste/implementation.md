# Smart Paste — implementation map

Inspect [the specification](spec.md) and [the progress checkpoints](progress.md) before resuming. This run implements the native/core scope independent of 008; no complete feature acceptance or native application compatibility is claimed.

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-005: Multi-type receipt transaction](spec.md#fr-005) | [snapshot](../../../src-tauri/src/smart_paste/snapshot.rs), [snapshot_helper](../../../src-tauri/src/smart_paste/snapshot_helper.rs), [macos_pasteboard](../../../src-tauri/src/smart_paste/macos_pasteboard.rs), [lifecycle](../../../src-tauri/src/smart_paste/lifecycle.rs), [pasteboard receipt](evidence/native-pasteboard-adapter.json) | FR-005 | Partial — bounded snapshot, helper and on-demand publication implemented, certification pending; general pasteboard and real applications pending release evidence | 2026-10-13 |
| [FR-006: Guarded restoration](spec.md#fr-006) | [lifecycle](../../../src-tauri/src/smart_paste/lifecycle.rs), [settlement](../../../src-tauri/src/smart_paste/settlement.rs) | FR-006 | Partial — pure timing and guarded decisions implemented, certification pending; native adapter pending | 2026-10-13 |
| [FR-007: App and keyboard strategies](spec.md#fr-007) | [strategy](../../../src-tauri/src/smart_paste/strategy.rs), [keyboard](../../../src-tauri/src/smart_paste/keyboard.rs), [key_events](../../../src-tauri/src/smart_paste/key_events.rs), [macos_keyboard](../../../src-tauri/src/smart_paste/macos_keyboard.rs), [keyboard receipt](evidence/native-keyboard-adapter.json) | FR-007 | Partial — strategies and layout handling implemented, certification pending; selection/posting natively pending release evidence | 2026-10-13 |
| [FR-008: Truthful outcomes](spec.md#fr-008) | [outcome](../../../src-tauri/src/smart_paste/outcome.rs) | FR-008 | Partial — outcome contract implemented, certification pending; 009/010 consumers pending | 2026-10-13 |
| [FR-009: Settings, migration and CLI](spec.md#fr-009) | None | Pending | Waiting for 008 | — |
| [FR-010: Native evidence](spec.md#fr-010) | [Native spike](../../../src-tauri/tests/smart_paste_native.rs), [harness](../../../scripts/smart-paste-native-proof.ts), [AX observations](evidence/native-ax-spike.json) | FR-010 | Partial — synthetic native observations, release matrix pending | 2026-10-13 |
| [FR-011: Verified direct insertion](spec.md#fr-011) | [direct](../../../src-tauri/src/smart_paste/direct.rs), [ax_port](../../../src-tauri/src/smart_paste/ax_port.rs), [macos_ax](../../../src-tauri/src/smart_paste/macos_ax.rs), [AX adapter receipt](evidence/native-ax-adapter.json) | FR-011 | Partial — bounded verified insertion and fallback guard implemented, certification pending; chain wiring waits for 006 | 2026-10-13 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| AC-008 | [snapshot tests](../../../src-tauri/tests/smart_paste/snapshot.rs), [native scenario](../../../src-tauri/tests/smart_paste_native_pasteboard/scenario.rs) | Implemented, certification pending — production adapter on a named pasteboard; key posting and layout resolution in steps 5–6; AppKit repeat-read cache limitation remains |
| AC-009 | [lifecycle tests](../../../src-tauri/tests/smart_paste_lifecycle.rs), [settlement](../../../src-tauri/src/smart_paste/settlement.rs) | Implemented (pure timing/guard), certification pending — real repeat reads not observable after AppKit cache fulfillment |
| AC-010 | [strategy](../../../src-tauri/src/smart_paste/strategy.rs) | Partial — bundle table implemented, certification pending; composition in the coordinator (step 5); remote apps pending release evidence |
| AC-011 | [keyboard tests](../../../src-tauri/tests/smart_paste/keyboard.rs), [native read-only check](../../../src-tauri/tests/smart_paste_native_keyboard.rs) | Implemented, certification pending — real ABC/US selection and restoration pending release evidence (layouts disabled on this Mac) |
| AC-012 | [outcome](../../../src-tauri/src/smart_paste/outcome.rs) | Partial — eight kinds and eligibility implemented, certification pending |
| AC-013 | None | Missing |
| AC-014 | None | Waiting for 008 — shared startup migration and UI |
| AC-015 | None | Waiting for 008 — settings service, revisions, CLI parity |
| AC-016 | [Native spike](../../../src-tauri/tests/smart_paste_native.rs) | Partial — synthetic checks only; real application/layout matrix pending release evidence |
| AC-017 | [scripted tests](../../../src-tauri/tests/smart_paste/direct.rs), [native adapter test](../../../src-tauri/tests/smart_paste_native_ax.rs) | Implemented, certification pending — production adapter verified on synthetic fields; real applications pending release evidence |
| AC-018 | [scripted tests](../../../src-tauri/tests/smart_paste/direct.rs) | Partial — 250 ms direct budget implemented; chain composition in the coordinator (step 5); 010 consumer pending |
| has_dropdown / is_keyboard_navigable | None | Waiting for 008 — UI contract and settings integration |

## Native observations and boundaries

- Targeted native Cargo fixture: one ignored opt-in integration test executed and passed. The Cargo application build compiled the real Apple Intelligence bridge; the fixture exercises Swift/AppKit and does not test Apple Intelligence inference. No general clipboard or user focus was touched.
- Read [the provider evidence](evidence/native-spike.json): AppKit cached fulfilled text. A pre-key read suppressed a subsequent provider callback, and two post-key readers produced one callback. Callback-controlled mechanics work; quiet timing after every actual consumer read is not certified. A callback is not an authenticated destination acknowledgement.
- Read [the AX evidence](evidence/native-ax-spike.json): synthetic AXSelectedText replacement and UTF-16 length guard were observed. Requested 20 ms messaging timeouts had measured scheduling overhead; no universal wall-clock timeout or total 250 ms native guarantee is claimed.
- Read [the TIS evidence](evidence/native-keyboard-spike.json): French source resolves Command-V to keycode 9. ABC, US and Dvorak are installed but disabled; this run does not enable layouts or change system configuration.

## Files Created/Modified

- Read [the native integration test](../../../src-tauri/tests/smart_paste_native.rs) for the opt-in harness assertions.
- Run [the public native fixture harness](../../../scripts/smart-paste-native-proof.ts) for named pasteboard/AppKit observations.
- Read [the progress file](progress.md) for the exact resume state and deferred dependencies.

## Reproducible feasibility probes

- Run [the retained AX spike](evidence/probes/native_ax_spike.py) with `--out .specs/features/007-smart-paste/evidence/native-ax-spike.json`; it uses [the bounded synthetic AX adapter](evidence/probes/native_ax_probe.py), owns and reaps every fixture, and records source/binary/toolchain identity.
- Compile and run [the retained Carbon spike](evidence/probes/native-keyboard-spike.swift) with the exact commands preserved in [its receipt](evidence/native-keyboard-spike.json). It inventories sources and resolves Command-V without selecting layouts or sending keys.
- Run [the AppKit harness](../../../scripts/smart-paste-native-proof.ts), which validates [the native report schema](../../../scripts/smart-paste-native-report.ts) and uses [bounded subprocess cleanup](../../../scripts/smart-paste-native-process.ts). The current macOS cache counterexamples are characterization witnesses; changed observations require a new feasibility review.

## Quality gate scope

Raw command outputs for every checkpoint are kept under [proofs/claude](proofs/claude/) as `<step>-<gate>.{out,err,exit}`.

- Step 6: `--test smart_paste` 50/0 and the read-only native keyboard check ([stdout](proofs/claude/step-6-native-keyboard.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 4: `--test smart_paste` 43/0 and the harness-free native pasteboard scenario ([stdout](proofs/claude/step-4-native-pasteboard.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 3: `--test smart_paste` 30/0 and the native AX adapter test 1/0 ([stdout](proofs/claude/step-3-native-ax.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 2: `cargo test --lib smart_paste` 16/0 and `--test smart_paste_lifecycle` 14/0 within the full suite ([stdout](proofs/claude/step-2-cargo-test.out)); clippy unchanged (5 existing, 0 new); fmt, lint, format and conventions PASS.
- Step 1: the opt-in native fixture passed 1/0 ([stdout](proofs/claude/step-1-native-cargo.out), [exit](proofs/claude/step-1-native-cargo.exit)); full `cargo test` passed ([stdout](proofs/claude/step-1-cargo-test.out)); `cargo clippy --all-targets` reports the same 5 pre-existing warnings in untouched code (`lib.rs`, `managers/transcription.rs`, `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs`, `portable.rs`) and no new one ([stderr](proofs/claude/step-1-clippy.err)); fmt, lint, format and repo conventions PASS.
