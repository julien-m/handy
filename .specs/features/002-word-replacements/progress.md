# Implementation Progress — Word Replacements

Core-first run authorized 2026-10-13. Read [the reviewed plan](plan.md) for the complete implementation steps. Feature 008 remains unavailable; no alternate settings service, revision engine or CLI framework is introduced.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a — UI behavioral TDD | Blocked | None | None | Waiting for UI design certification and 008 transaction ports | 2026-10-13 |
| 1a — Core model, Unicode and validation | Done | Native replacement domain, additive settings field, Specta bindings | Full Rust suite 306 PASS/17 ignored; Clippy/fmt/lint/format/repo conventions PASS ([proofs](proofs/claude/)) | Audit findings 10–14 corrected; independent delta review PASS; re-verified and committed by the implementation agent | 2026-10-13 |
| 1b — Adapter and startup normalization | Blocked | None | None | Waiting for 008 actual service/domain interfaces; no substitute framework | 2026-10-13 |
| 2 — Exchange and deterministic previews | Done | `word_replacements/{exchange,preview,materialize}.rs` + unit test files, `src-tauri/tests/replacement_exchange.rs` | Unit 25 new PASS; file integration 5 PASS (cap 999999/1000000/1000001 bytes, invalid UTF-8 offset, BOM/CRLF, 1/100/1000 round-trips); full suite 331 PASS/17 ignored, Clippy/fmt/lint/format and repo conventions `20261013T144051Z` PASS in [proofs](proofs/claude/) `step2-*` | Pure parse/serialize/preview/materialize; revision binding and atomic commit Waiting for 008 | 2026-10-13 |
| 3 — Transaction domain and live publication | Blocked | None | None | Waiting for 008: domain trait, snapshot, commit, revisions and live effects | 2026-10-13 |
| 4 — Matcher and final transcription pipeline | Done | `word_replacements/{matcher,dictation}.rs` + `matcher_tests.rs`, `audio_toolkit/text.rs`, `managers/transcription.rs`, `actions.rs`, `commands/history.rs`, `lib.rs` | 40 exact matching fixtures + segment/offset invariants; 4 span-aware correction tests; 4 pipeline-order tests; 3 capture tests; full suite 346 PASS/17 ignored, Clippy/fmt/lint/format and repo conventions `20261013T145603Z` PASS in [proofs](proofs/claude/) `step4-*` | Rules captured at recording start, compiled once per dictation, applied after language/script conversion and before approximate correction (protected spans), fillers, normalization and AI; port-injected `replacement_pipeline.rs` harness not built (see changelog) | 2026-10-13 |
| 5 — Editor and native file adapters | Blocked | None | None | Waiting for 008 transaction/revision integration; UI design certification remains incomplete | 2026-10-13 |
| 6 — CLI and bounded live transfer | Blocked | None | None | Waiting for 008: domain dispatcher, transport, envelopes, clock/deadline and store ownership | 2026-10-13 |
| 7 — Performance and native acceptance | Partial | `src-tauri/tests/replacement_performance.rs` | Release benchmark (1000 adversarial rules, 2000 scalars, compile + apply, 30 samples): max 1.78 ms on Apple M4 Pro ([proof](proofs/claude/step4-release-performance.txt)) | Budget is stated for M1: same command on an M1 Mac is pending release evidence. Native dictation insertion/history observation and all UI/CLI acceptance Waiting for 008 | 2026-10-13 |

## Waiting for 008

- Step 1 adapter compatibility and startup normalization through the actual settings service.
- Step 3 shared settings-service registration, revision CAS, atomic persistence, immutable live matcher publication and transaction fault evidence.
- Step 5 UI/native commands using the shared transaction and revision events, restart and external conflict evidence.
- Step 6 all replacement CLI operations, bounded transfer and one public JSON response.
- Step 7 complete UI/CLI round-trip, persistence/conflict and live-next-dictation acceptance.
- Pipeline seam: today the recording-start capture reads the rules from the loaded settings; with 008 it must take the compiled snapshot published by the committed revision (no second matcher owner).

## Pending release evidence (never claimed as passed)

- M1 budget: `cargo test --release --manifest-path src-tauri/Cargo.toml --test replacement_performance -- --nocapture` on an Apple M1 Mac; max sample must stay under 20 ms.
- Windows and Linux: `cargo test --manifest-path src-tauri/Cargo.toml --lib word_replacements` and `cargo test --manifest-path src-tauri/Cargo.toml --test replacement_exchange` on each OS.
- macOS native dictation: with a rule in the store, dictate "open cloud code now" in batch and streaming models and read back the inserted text and the WAV-gated history entry (needs the 008/UI commit path to set rules without manual store edits).

No complete AC or lifecycle status is certified by this partial run. Pending release evidence stays distinct from missing implementation.

## Scoped pipeline and UI state

The semantic specification/plan reviews and source-backed Analyze/progression check were ready before the user-authorized core implementation. The Plan child returned BLOCKED for its unavailable design gate, so the pipeline keeps Plan Blocked and Plan Review Pending. Core-first user authorization permits the independent native work recorded here; it does not certify C51 or make those phases Done.

The invalid UI preparation draft was preserved in the [historical archive](../../.runs/002-ui-draft-20261013T134300Z.zip), then removed from current source; the original microphone contract was restored. Historical C46/structural outputs do not establish current design PASS. Read [the Penflow defect report](../../tool-defects/penflow-flow-projection-20261013.md) before repeating projection generation. The canonical 002 UI contract/design remains absent and BLOCKED; no UI renderer or service framework was implemented.
