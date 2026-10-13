---
feature: word-replacements
title: Word Replacements Implementation
spec_ref: spec.md
updated: 2026-10-13
---

# Implementation — Word Replacements

This partial core-first run follows [the approved scope](spec.md) and [the technical plan](plan.md). Complete persistence, revisions, CLI and UI acceptance remains Waiting for 008. Inspect [progress](progress.md) and [the execution log](logs/2026-10-13.md) for observed checks; no complete feature certification is asserted.

## FR mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-001](spec.md#fr-001) | [model](../../../src-tauri/src/word_replacements/model.rs), [settings](../../../src-tauri/src/settings.rs) | `FR-001: Ordered stable replacement rules` | ⚠️ Partial — native ordered model and serde-empty field; transactions/UI Waiting for 008 | 2026-10-13 |
| [FR-002](spec.md#fr-002) | [validation](../../../src-tauri/src/word_replacements/validation.rs), [Unicode](../../../src-tauri/src/word_replacements/unicode.rs) | `FR-002: Shared typed field validation`, `AC-002: Full folded whitespace key` | ⚠️ Partial — shared pure validator; UI/import/CLI integration pending | 2026-10-13 |
| [FR-003](spec.md#fr-003) | [exchange](../../../src-tauri/src/word_replacements/exchange.rs) | `FR-003: Parse exchange lines with diagnostics`, `AC-004: Header recognized with tab or semicolon`, `AC-004: Canonical lossless export` | ⚠️ Partial — parser/serializer implemented, certification pending; bulk editor and CLI file/stdin surfaces Waiting for 008 | 2026-10-13 |
| [FR-004](spec.md#fr-004) | [preview](../../../src-tauri/src/word_replacements/preview.rs), [materialize](../../../src-tauri/src/word_replacements/materialize.rs) | `FR-004: Merge/replace preview without writes` | ⚠️ Partial — six-status merge/replace preview and ID materialization implemented; revision binding and atomic commit Waiting for 008 | 2026-10-13 |
| [FR-005](spec.md#fr-005) | [matcher](../../../src-tauri/src/word_replacements/matcher.rs), [Unicode](../../../src-tauri/src/word_replacements/unicode.rs), [span-aware correction](../../../src-tauri/src/audio_toolkit/text.rs), [pipeline](../../../src-tauri/src/managers/transcription.rs), [dictation capture](../../../src-tauri/src/word_replacements/dictation.rs) | `FR-005: Compile rules once per snapshot`, `FR-005: Protect replaced spans from approximate correction`, `AC-006: Case-insensitive whole-word matching`, `AC-006: Word characters include combining marks and joiners`, `AC-007: Replacement pipeline position`, `AC-007: Captured rules apply to their dictation` | Implemented, certification pending — matcher with letter/digit/combining-mark/join-control boundaries, CJK sides that still reject an adjacent combining mark, pipeline position, span protection and stop-path wiring; publication from 008 commits Waiting for 008 | 2026-10-13 |
| [FR-006](spec.md#fr-006) | None | None | ❌ Missing — Waiting for 008 domain/CLI framework | — |

## Acceptance Criteria Mapping

| AC | Test File | Status |
|---|---|---|
| [AC-001](spec.md#ac-001) | [legacy/default model tests](../../../src-tauri/src/word_replacements/mod.rs), [identity tests](../../../src-tauri/src/word_replacements/model.rs) | ⚠️ Partial model/empty legacy defaults; UI/restart persistence Waiting for 008 |
| [AC-002](spec.md#ac-002) | [validation tests](../../../src-tauri/src/word_replacements/mod.rs), [Unicode tests](../../../src-tauri/src/word_replacements/unicode.rs) | ⚠️ Partial pure-domain evidence; complete no-partial-write/UI/CLI evidence Waiting for 008 |
| [AC-003](spec.md#ac-003) | [preview tests](../../../src-tauri/src/word_replacements/preview_tests.rs) | ⚠️ Partial — line-by-line preview and Apply gating implemented, certification pending; bulk editor UI Waiting for 008 and UI design |
| [AC-004](spec.md#ac-004) | [exchange tests](../../../src-tauri/src/word_replacements/exchange_tests.rs), [file integration](../../../src-tauri/tests/replacement_exchange.rs) | Implemented, certification pending — format parse/serialize, header with tab or `;` separator and round-trip |
| [AC-005](spec.md#ac-005) | [file integration](../../../src-tauri/tests/replacement_exchange.rs), [preview tests](../../../src-tauri/src/word_replacements/preview_tests.rs) | ⚠️ Partial — 1,000,000-byte cap, UTF-8, merge/replace and duplicate errors implemented, certification pending; atomic persistence and fault evidence Waiting for 008 |
| [AC-006](spec.md#ac-006) | [61 matching fixtures](../../../src-tauri/src/word_replacements/matcher_tests.rs) (script boundaries in [matcher_script_tests.rs](../../../src-tauri/src/word_replacements/matcher_script_tests.rs)), [Unicode primitive tests](../../../src-tauri/src/word_replacements/unicode.rs) | Implemented, certification pending (SC-001: 61 table-driven cases, including Devanagari and NFD combining-mark boundaries, Persian ZWNJ, ZWJ, NFD kana on a CJK side and the accepted halfwidth sound-mark and NFD jamo false positives) |
| [AC-007](spec.md#ac-007) | [matching fixtures](../../../src-tauri/src/word_replacements/matcher_tests.rs), [pipeline tests](../../../src-tauri/src/managers/transcription.rs), [span tests](../../../src-tauri/src/audio_toolkit/text.rs), [capture tests](../../../src-tauri/src/word_replacements/dictation.rs), [stop-path harness](../../../src-tauri/tests/replacement_pipeline.rs) | ⚠️ Partial — single pass, longest-first, no chaining, pipeline order, span protection, recording-start capture and stop-time selection/fallback implemented, certification pending; UI/CLI commit then next-dictation proof Waiting for 008; native insertion/history observation pending |
| [AC-008](spec.md#ac-008) | None | ❌ Missing — Waiting for 008 |

## Files Created/Modified

- Read [the domain modules](../../../src-tauri/src/word_replacements/mod.rs) for opaque identities, one validator and pinned Unicode 17 transformations.
- Inspect [Unicode provenance](../../../src-tauri/src/word_replacements/unicode-sources.json), [the compact data](../../../src-tauri/src/word_replacements/unicode-data.txt) and [the license](../../../src-tauri/src/word_replacements/unicode-license.txt) for exact source identities.
- Read [the exchange module](../../../src-tauri/src/word_replacements/exchange.rs) for the bounded UTF-8 reader, line parser and canonical serializer, [the preview module](../../../src-tauri/src/word_replacements/preview.rs) for merge/replace previews and [materialization](../../../src-tauri/src/word_replacements/materialize.rs) for commit-time ID allocation. Unit tests live in `exchange_tests.rs` and `preview_tests.rs`.
- Read [the matcher](../../../src-tauri/src/word_replacements/matcher.rs) for the compiled trie, CJK-aware boundaries (letters, decimal digits, combining marks and ZWNJ/ZWJ are word characters; a CJK side ignores neighbouring base characters but not an adjacent combining mark) and protected output segments, and [dictation capture](../../../src-tauri/src/word_replacements/dictation.rs) for rules captured at recording start, selected at stop (capture, else current settings) and compiled once for the stream finalize and its batch fallback. [The stop-path harness](../../../src-tauri/tests/replacement_pipeline.rs) drives those functions with injected engine and settings ports.
- Inspect [actions](../../../src-tauri/src/actions.rs), [transcription](../../../src-tauri/src/managers/transcription.rs), [history retry](../../../src-tauri/src/commands/history.rs) and [headless transcription](../../../src-tauri/src/lib.rs) for the snapshot passed to both final paths; [text](../../../src-tauri/src/audio_toolkit/text.rs) adds approximate correction outside protected spans.
- Inspect [settings](../../../src-tauri/src/settings.rs) for the additive `word_replacements` field and empty serde/default behavior; no parallel settings writer is introduced.
- Inspect [the native entry point](../../../src-tauri/src/lib.rs) for the public module and the headless transcription snapshot; the Specta registry is unchanged from `dev`.
- Read [generated bindings](../../../src/bindings.ts) for the native-exported replacement types (`ReplacementRule`, `RuleId`, `AppSettings.word_replacements`), kept in Specta's raw output form so the diff holds only those types.
- Inspect [Cargo dependencies](../../../src-tauri/Cargo.toml) and [the lock](../../../src-tauri/Cargo.lock) for the UUID `=1.21.0` pin.

## Visual and release evidence

UI design certification remains BLOCKED. No visual baseline, native rule editor/dialog, real user dictation or complete AC is claimed by the core unit tests. Windows/Linux and interactive native checks stay pending release evidence when unavailable, with exact protocols to be recorded at the relevant implementation step.
