# Implementation Progress — Word Replacements

Core-first run authorized 2026-10-13. Read [the reviewed plan](plan.md) for the complete implementation steps. Feature 008 remains unavailable; no alternate settings service, revision engine or CLI framework is introduced.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a — UI behavioral TDD | Blocked | None | None | Waiting for UI design certification and 008 transaction ports | 2026-10-13 |
| 1a — Core model, Unicode and validation | Done | Native replacement domain, additive settings field, Specta bindings | Full Rust suite 306 PASS/17 ignored; Clippy/fmt/lint/format/repo conventions PASS ([proofs](proofs/claude/)) | Audit findings 10–14 corrected; independent delta review PASS; re-verified and committed by the implementation agent | 2026-10-13 |
| 1b — Adapter and startup normalization | Blocked | None | None | Waiting for 008 actual service/domain interfaces; no substitute framework | 2026-10-13 |
| 2 — Exchange and deterministic previews | Done | `word_replacements/{exchange,preview,materialize}.rs` + unit test files, `src-tauri/tests/replacement_exchange.rs` | Unit 25 new PASS; file integration 5 PASS (cap 999999/1000000/1000001 bytes, invalid UTF-8 offset, BOM/CRLF, 1/100/1000 round-trips); full suite 331 PASS/17 ignored, Clippy/fmt/lint/format and repo conventions `20261013T144051Z` PASS in [proofs](proofs/claude/) `step2-*` | Pure parse/serialize/preview/materialize; revision binding and atomic commit Waiting for 008 | 2026-10-13 |
| 3 — Transaction domain and live publication | Blocked | None | None | Waiting for 008: domain trait, snapshot, commit, revisions and live effects | 2026-10-13 |
| 4 — Matcher and final transcription pipeline | Done | `word_replacements/{matcher,dictation,unicode}.rs` + `matcher_tests.rs`, `audio_toolkit/text.rs`, `managers/transcription.rs`, `actions.rs`, `commands/history.rs`, `lib.rs`, `tests/replacement_pipeline.rs` | 50 exact matching fixtures (incl. Devanagari/NFD combining marks, several-space rule, no match inside `ß`) + segment/offset invariants; span, pipeline-order and capture tests; stop-path harness 7 PASS; full library 348 PASS/16 ignored; strict Clippy on lib/bins/replacement tests, fmt/lint/format and repo conventions PASS in [audit-fix proofs](proofs/claude-fix1/) | Rules captured at recording start, selected at stop (capture, else current settings), compiled once for stream finalize and batch fallback, applied after language/script conversion and before approximate correction (protected spans), fillers, normalization and AI. Independent audit findings 1–5 dispositioned below | 2026-10-13 |
| 5 — Editor and native file adapters | Blocked | None | None | Waiting for 008 transaction/revision integration; UI design certification remains incomplete | 2026-10-13 |
| 6 — CLI and bounded live transfer | Blocked | None | None | Waiting for 008: domain dispatcher, transport, envelopes, clock/deadline and store ownership | 2026-10-13 |
| 7 — Performance and native acceptance | Partial | `src-tauri/tests/replacement_performance.rs` | Release benchmark (1000 adversarial rules, 2000 scalars, compile + apply, 30 samples): max 1.78 ms on Apple M4 Pro ([proof](proofs/claude/step4-release-performance.txt)) | Budget is stated for M1: same command on an M1 Mac is pending release evidence. Native dictation insertion/history observation and all UI/CLI acceptance Waiting for 008 | 2026-10-13 |

## Independent audit fixes (2026-10-13)

Audit: [independent audit](proofs/claude/independent-audit.md) (FAIL: 1 major, 2 minor, 2 nit). Raw outputs: [proofs/claude-fix1](proofs/claude-fix1/).

| # | Severity | Disposition |
|---|---|---|
| 1 | major | Fixed. AC-006 clarified by orchestrator decision (Clarify ledger, IDs preserved): letters, decimal digits and combining marks Mn/Mc/Me are word characters on both sides of a match. The pinned Unicode 17 generator emits a new `M` table (no new crate: `unicode-general-category` is not in `Cargo.lock`, and one pinned Unicode version must serve letters, marks, whitespace and folding). Fixtures: `भारत→India` on `भारतीय संविधान` and `रत→Z` on `भारत` unchanged, virama conjunct unchanged, `cafe→X` on NFD and precomposed `café` unchanged, decomposed accent before a match, whole Devanagari words and a whole NFD word replaced. A red run without the fix is recorded. |
| 2 | minor | Fixed. Fixtures for a rule with several internal spaces and for a match that would start inside `ß` (`se` on `ße`). |
| 3 | minor | Fixed at the function seam. `DictationRuleCapture::rules_for_stop` (capture, else current settings, read lazily) and `finalize_or_transcribe` (one compiled matcher for stream finalize and batch fallback, cancel never starts a batch) are extracted from `actions.rs` and driven by [the harness](../../../src-tauri/tests/replacement_pipeline.rs) with injected engine and settings ports. Remaining: the plan's full harness with LLM/history/output ports needs `actions.rs` restructuring and the 008 published snapshot; output/history stage proof stays with native acceptance. |
| 4 | nit | Fixed. `decode_exchange_bytes` returns the decoded text unchanged; `parse_exchange` removes exactly one initial BOM for every source. Test: two leading BOMs keep one as content. |
| 5 | nit | Reverted to `dev`: Specta builder macro, ignored export test and `trim_bindings_whitespace` formatter in `lib.rs`; Clippy edits in `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs`, `portable.rs`; whitespace churn in `src/bindings.ts` (prettier-ignored). Kept: the regenerated replacement types in `bindings.ts`, in Specta's raw output form (`git diff -w` against the previous file is empty for those types). Not necessary for Clippy: `--all-targets -D warnings` fails on the `dev` baseline with 4 lint errors in unrelated test code (`items_after_test_module` at `lib.rs:413`, plus the 3 reverted files, byte-identical to `dev`); the previous edits did not fix the `lib.rs` one either. All feature targets pass strict Clippy, and the library unit tests pass it with only those 4 baseline lints allowed. |

### Spec review and gates

- The AC-006 edit invalidates the semantic spec review. The review cache was already stale before it: `livespec validate .specs/features/002-word-replacements --progression implement` returned `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review` (with `--model gpt-6.1-sol`, plus `spec_review_model_mismatch` with `claude-opus-5-5`). No current gate consumes a fresh review during this core-first run, so the independent spec review must be refreshed before any progression gate; no prior PASS is reused.
- Release performance was not re-measured: the boundary test adds one range lookup per candidate side and a release build would add a build profile on a nearly full disk. Re-run `cargo test --release --manifest-path src-tauri/Cargo.toml --test replacement_performance -- --nocapture` with the M1 release evidence.
- Pre-existing `livespec validate` structure errors in `implementation.md` (missing front matter, `## AC mapping` heading) are unchanged by this run.

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
