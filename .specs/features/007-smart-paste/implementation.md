# Smart Paste — implementation map

Inspect [the specification](spec.md) and [the progress checkpoints](progress.md) before resuming. This run implements the native/core scope independent of 008; no complete feature acceptance or native application compatibility is claimed.

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-005: Multi-type receipt transaction](spec.md#fr-005) | [snapshot](../../../src-tauri/src/smart_paste/snapshot.rs), [snapshot_helper](../../../src-tauri/src/smart_paste/snapshot_helper.rs), [macos_pasteboard](../../../src-tauri/src/smart_paste/macos_pasteboard.rs), [lifecycle](../../../src-tauri/src/smart_paste/lifecycle.rs), [pasteboard receipt](evidence/native-pasteboard-adapter.json) | FR-005 | Partial — bounded snapshot, helper and on-demand publication implemented, certification pending; general pasteboard and real applications pending release evidence | 2026-10-13 |
| [FR-006: Guarded restoration](spec.md#fr-006) | [lifecycle](../../../src-tauri/src/smart_paste/lifecycle.rs), [settlement](../../../src-tauri/src/smart_paste/settlement.rs), [pending](../../../src-tauri/src/smart_paste/pending.rs), [coordinator](../../../src-tauri/src/smart_paste/coordinator.rs), [macos_pasteboard](../../../src-tauri/src/smart_paste/macos_pasteboard.rs) | FR-006 | Partial — restoration, copy mode, succession and exit implemented, certification pending; app exit hook waits for step 7 | 2026-10-13 |
| [FR-007: App and keyboard strategies](spec.md#fr-007) | [strategy](../../../src-tauri/src/smart_paste/strategy.rs), [keyboard](../../../src-tauri/src/smart_paste/keyboard.rs), [key_events](../../../src-tauri/src/smart_paste/key_events.rs), [macos_keyboard](../../../src-tauri/src/smart_paste/macos_keyboard.rs), [keyboard receipt](evidence/native-keyboard-adapter.json) | FR-007 | Partial — strategies and layout handling implemented, certification pending; selection/posting natively pending release evidence | 2026-10-13 |
| [FR-008: Truthful outcomes](spec.md#fr-008) | [outcome](../../../src-tauri/src/smart_paste/outcome.rs), [coordinator](../../../src-tauri/src/smart_paste/coordinator.rs), [actor](../../../src-tauri/src/smart_paste/actor.rs) | FR-008 | Partial — terminal outcomes and late-read diagnostics produced, certification pending; 009/010 consumers and pipeline wiring wait for 006/009/010 | 2026-10-13 |
| [FR-009: Settings, migration and CLI](spec.md#fr-009) | [settings.rs](../../../src-tauri/src/settings.rs) (`PasteMethod::SmartPaste` persisted value only) | FR-009 | Waiting for 008 — migration, default, CLI domain, validation and UI | 2026-10-13 |
| [FR-010: Native evidence](spec.md#fr-010) | [Native spike](../../../src-tauri/tests/smart_paste_native.rs), [harness](../../../scripts/smart-paste-native-proof.ts), [AX observations](evidence/native-ax-spike.json), [AX adapter](evidence/native-ax-adapter.json), [pasteboard adapter](evidence/native-pasteboard-adapter.json), [keyboard read-only](evidence/native-keyboard-adapter.json) | FR-010 | Partial — synthetic native observations, release matrix pending | 2026-10-13 |
| [FR-011: Verified direct insertion](spec.md#fr-011) | [direct](../../../src-tauri/src/smart_paste/direct.rs), [ax_port](../../../src-tauri/src/smart_paste/ax_port.rs), [macos_ax](../../../src-tauri/src/smart_paste/macos_ax.rs), [AX adapter receipt](evidence/native-ax-adapter.json) | FR-011 | Partial — bounded verified insertion and fallback guard implemented, certification pending; chain wiring waits for 006 | 2026-10-13 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| AC-008 | [snapshot tests](../../../src-tauri/tests/smart_paste/snapshot.rs), [native scenario](../../../src-tauri/tests/smart_paste_native_pasteboard/scenario.rs) | Implemented, certification pending — production adapter on a named pasteboard; key posting and layout resolution in steps 5–6; AppKit repeat-read cache limitation remains |
| AC-009 | [lifecycle tests](../../../src-tauri/tests/smart_paste/lifecycle.rs), [coordinator tests](../../../src-tauri/tests/smart_paste/coordinator.rs), [native scenario](../../../src-tauri/tests/smart_paste_native_pasteboard/scenario.rs) | Implemented, certification pending — real repeat reads are not observable after AppKit cache fulfillment (quiet measured from observed reads) |
| AC-010 | [strategy](../../../src-tauri/src/smart_paste/strategy.rs), [chain tests](../../../src-tauri/tests/smart_paste/chain.rs) | Implemented in the coordinator, certification pending — remote apps pending release evidence |
| AC-011 | [keyboard tests](../../../src-tauri/tests/smart_paste/keyboard.rs), [native read-only check](../../../src-tauri/tests/smart_paste_native_keyboard.rs) | Implemented, certification pending — real ABC/US selection and restoration pending release evidence (layouts disabled on this Mac) |
| AC-012 | [outcome](../../../src-tauri/src/smart_paste/outcome.rs), [coordinator tests](../../../src-tauri/tests/smart_paste/coordinator.rs), [chain tests](../../../src-tauri/tests/smart_paste/chain.rs) | Implemented in the coordinator, certification pending — indicator message (006) and Enter/recovery consumers (009/010) pending |
| AC-013 | [coordinator tests](../../../src-tauri/tests/smart_paste/coordinator.rs), [chain tests](../../../src-tauri/tests/smart_paste/chain.rs) | Implemented, certification pending — history is not touched by smart paste |
| AC-014 | None | Waiting for 008 — shared startup migration and UI |
| AC-015 | None | Waiting for 008 — settings service, revisions, CLI parity |
| AC-016 | [native targets](../../../src-tauri/tests/) | Pending release evidence — synthetic and adapter-level native checks only; the application × layout matrix is listed below |
| AC-017 | [scripted tests](../../../src-tauri/tests/smart_paste/direct.rs), [native adapter test](../../../src-tauri/tests/smart_paste_native_ax.rs) | Implemented, certification pending — production adapter verified on synthetic fields; real applications pending release evidence |
| AC-018 | [direct tests](../../../src-tauri/tests/smart_paste/direct.rs), [chain tests](../../../src-tauri/tests/smart_paste/chain.rs) | Implemented in the coordinator, certification pending — recovery bubble consumer (010) pending |
| has_dropdown / is_keyboard_navigable | None | Waiting for 008 — UI contract and settings integration |

## Native observations and boundaries

- Read [the AX adapter receipt](evidence/native-ax-adapter.json): the production `MacAxTextPort` verified an astral-character replacement, skipped a 1,000,001-unit field and a password field, and detected a write applied 151 ms after the window (guard `AppliedLate`). A cold first AX contact with a freshly launched app may exceed the 100 ms preparation budget and then only skips.
- Read [the pasteboard adapter receipt](evidence/native-pasteboard-adapter.json): on a named pasteboard, rich multi-item content round-tripped byte for byte through the real helper binary, a newer copy was never overwritten, materialization left plain text without markers, an empty pasteboard came back empty, a 1 s blocking promise was cut at about 200 ms keeping completed types, and the byte budget skipped by name. AppKit served the second external read from its cache without a callback.
- Read [the keyboard receipt](evidence/native-keyboard-adapter.json): French layout → decision `TryAbcThenUs`, Cmd+V key code 9; nothing selected or posted.

- Targeted native Cargo fixture: one ignored opt-in integration test executed and passed. The Cargo application build compiled the real Apple Intelligence bridge; the fixture exercises Swift/AppKit and does not test Apple Intelligence inference. No general clipboard or user focus was touched.
- Read [the provider evidence](evidence/native-spike.json): AppKit cached fulfilled text. A pre-key read suppressed a subsequent provider callback, and two post-key readers produced one callback. Callback-controlled mechanics work; quiet timing after every actual consumer read is not certified. A callback is not an authenticated destination acknowledgement.
- Read [the AX evidence](evidence/native-ax-spike.json): synthetic AXSelectedText replacement and UTF-16 length guard were observed. Requested 20 ms messaging timeouts had measured scheduling overhead; no universal wall-clock timeout or total 250 ms native guarantee is claimed.
- Read [the TIS evidence](evidence/native-keyboard-spike.json): French source resolves Command-V to keycode 9. ABC, US and Dvorak are installed but disabled; this run does not enable layouts or change system configuration.

## Files Created/Modified

- `src-tauri/src/smart_paste/` (new module, `pub mod smart_paste` in [lib.rs](../../../src-tauri/src/lib.rs)): pure contracts (`ids`, `outcome`, `timing`, `lifecycle`, `settlement`, `snapshot`, `snapshot_protocol`, `strategy`, `keyboard`, `key_events`, `direct`, `clock`), ports (`ax_port`, `pasteboard_port`, `request`), composition (`coordinator`, `coordinator_chord`, `pending`, `actor`) and macOS adapters (`macos_ax`, `macos_pasteboard`, `macos_snapshot_stream`, `snapshot_helper`, `macos_keyboard`, `macos_cf`, `main_thread`). Read the module header of [mod.rs](../../../src-tauri/src/smart_paste/mod.rs) for the layout.
- [main.rs](../../../src-tauri/src/main.rs): hidden `--smart-paste-snapshot` helper dispatch before CLI parsing (read-only, no Tauri).
- [input.rs](../../../src-tauri/src/input.rs): exports the existing Cmd+V layout resolver for the keyboard adapter.
- [settings.rs](../../../src-tauri/src/settings.rs), [bindings.ts](../../../src/bindings.ts), [clipboard.rs](../../../src-tauri/src/clipboard.rs): additive `smart_paste` value, explicitly refused by the pipeline until 006.
- [Cargo.toml](../../../src-tauri/Cargo.toml): two harness-free opt-in native test targets (main-thread run loop needed); no new dependency.
- Tests: [smart_paste.rs](../../../src-tauri/tests/smart_paste.rs) with [its modules](../../../src-tauri/tests/smart_paste/), and the native targets `smart_paste_native_ax`, `smart_paste_native_pasteboard`, `smart_paste_native_keyboard`.

## Pending release evidence

Never claimed passed. Each check runs on a release build of Handy.app with accessibility permission granted to Handy itself, records the fields of plan Step 9 and keeps public synthetic fixtures only.

| Check | Exact procedure | Why not run here |
|---|---|---|
| AC-016 matrix | For TextEdit, Notes, ChatGPT, Comet and one Chromium app × AZERTY, US, Dvorak: 10 dictations each with a rich clipboard (text+RTF+image, multi-item); record path, outcome, inserted text multiplicity and restored clipboard bytes; thresholds per AC-016 | Needs interactive dictation into real apps, ChatGPT/Comet and enabled US/Dvorak layouts (enabling changes system settings) |
| AC-011 selection | With French active and ABC (then only US) enabled: verify ABC (US) is selected before Cmd+V, French restored 100 ms after key-up, also when the chord fails; with Japanese Kana verify no change | ABC/US are disabled on this Mac; selecting/enabling changes system input settings |
| Key posting | Verify Cmd+V delivery in the resolved layout and Unicode typing chunks into a focused field | Posting keys would type into the user's focused application |
| AC-010 remote apps | iPhone Mirroring and Windows App receive Unicode typing; Screen Sharing gets the hand-over text on the remote side and no keystroke; Jump Desktop 1.5 s/3 s timings | Apps or remote peers not available |
| Clipboard managers | With a manager honoring `org.nspasteboard` markers, verify the transcript is not recorded; record pre-key reads (cached by AppKit) leading to conservative `not_read` | Needs a third-party manager installed and interactive pastes |
| Helper start | First snapshot after launching an installed, signed Handy.app stays within 200 ms (debug binary: ~10 ms warm, seconds on the very first execution of a newly written binary) | Needs a signed release build |
| Derived types | Real owners whose derived types (e.g. UTF-16 text) need a conversion round trip: capture stays within 200 ms or skips by name | Needs real source applications |
| Late reads / overlap | Chromium probe-then-read, reads after quiet, overlapping dictations: no previous content or transcript delivered (SC-008) | Interactive, real targets |
| Exit | Quit Handy before quiet with a read observed: transcript materialized, never restored early | Needs the app wiring (step 7) |
| Other platforms | Windows reliable paste and Linux methods unchanged; Windows/Linux builds compile the platform-neutral smart_paste modules | Not runnable on this Mac |

## Reproducible feasibility probes

- Run [the retained AX spike](evidence/probes/native_ax_spike.py) with `--out .specs/features/007-smart-paste/evidence/native-ax-spike.json`; it uses [the bounded synthetic AX adapter](evidence/probes/native_ax_probe.py), owns and reaps every fixture, and records source/binary/toolchain identity.
- Compile and run [the retained Carbon spike](evidence/probes/native-keyboard-spike.swift) with the exact commands preserved in [its receipt](evidence/native-keyboard-spike.json). It inventories sources and resolves Command-V without selecting layouts or sending keys.
- Run [the AppKit harness](../../../scripts/smart-paste-native-proof.ts), which validates [the native report schema](../../../scripts/smart-paste-native-report.ts) and uses [bounded subprocess cleanup](../../../scripts/smart-paste-native-process.ts). The current macOS cache counterexamples are characterization witnesses; changed observations require a new feasibility review.

## Quality gate scope

Raw command outputs for every checkpoint are kept under [proofs/claude](proofs/claude/) as `<step>-<gate>.{out,err,exit}`.

- Final (HEAD after all steps): full `cargo test` 319 lib + 86 smart paste integration tests, 0 failed ([stdout](proofs/claude/final-cargo-test.out)); the four opt-in native targets PASS ([AppKit](proofs/claude/final-smart_paste_native.out), [AX](proofs/claude/final-smart_paste_native_ax.out), [pasteboard](proofs/claude/final-smart_paste_native_pasteboard.out), [keyboard](proofs/claude/final-smart_paste_native_keyboard.out)); clippy `--all-targets` 5 pre-existing warnings, 0 new; fmt, lint, format and repo conventions PASS. The step-1 AppKit test first timed out ([failed rerun](proofs/claude/final-smart_paste_native-rerun.err)): it never drained Bun's pipes and Bun spins on a full pipe; it now drains both while waiting, and [the AppKit receipt](evidence/native-spike.json) was regenerated.

- Settings value: `--lib settings` 27/0 (2 new) within the full suite ([stdout](proofs/claude/settings-cargo-test.out)), `tsc --noEmit` exit 0; clippy unchanged; fmt, lint, format and conventions PASS.
- Step 5: `--test smart_paste` 86/0 including real-time actor and log-privacy tests ([stdout](proofs/claude/step-5-cargo-test.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 6: `--test smart_paste` 50/0 and the read-only native keyboard check ([stdout](proofs/claude/step-6-native-keyboard.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 4: `--test smart_paste` 43/0 and the harness-free native pasteboard scenario ([stdout](proofs/claude/step-4-native-pasteboard.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 3: `--test smart_paste` 30/0 and the native AX adapter test 1/0 ([stdout](proofs/claude/step-3-native-ax.out)); clippy unchanged; fmt, lint, format and conventions PASS.
- Step 2: `cargo test --lib smart_paste` 16/0 and `--test smart_paste_lifecycle` 14/0 within the full suite ([stdout](proofs/claude/step-2-cargo-test.out)); clippy unchanged (5 existing, 0 new); fmt, lint, format and conventions PASS.
- Step 1: the opt-in native fixture passed 1/0 ([stdout](proofs/claude/step-1-native-cargo.out), [exit](proofs/claude/step-1-native-cargo.exit)); full `cargo test` passed ([stdout](proofs/claude/step-1-cargo-test.out)); `cargo clippy --all-targets` reports the same 5 pre-existing warnings in untouched code (`lib.rs`, `managers/transcription.rs`, `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs`, `portable.rs`) and no new one ([stderr](proofs/claude/step-1-clippy.err)); fmt, lint, format and repo conventions PASS.
