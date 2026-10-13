# Implementation — Word Replacements

This partial core-first run follows [the approved scope](spec.md) and [the technical plan](plan.md). Complete persistence, revisions, CLI and UI acceptance remains Waiting for 008. Inspect [progress](progress.md) and [the execution log](logs/2026-10-13.md) for observed checks; no complete feature certification is asserted.

## FR mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
|---|---|---|---|---|
| [FR-001](spec.md#fr-001) | [model](../../../src-tauri/src/word_replacements/model.rs), [settings](../../../src-tauri/src/settings.rs) | `FR-001: Ordered stable replacement rules` | ⚠️ Partial — native ordered model and serde-empty field; transactions/UI Waiting for 008 | 2026-10-13 |
| [FR-002](spec.md#fr-002) | [validation](../../../src-tauri/src/word_replacements/validation.rs), [Unicode](../../../src-tauri/src/word_replacements/unicode.rs) | `FR-002: Shared typed field validation`, `AC-002: Full folded whitespace key` | ⚠️ Partial — shared pure validator; UI/import/CLI integration pending | 2026-10-13 |
| [FR-003](spec.md#fr-003) | None | None | ❌ Missing — next core step | — |
| [FR-004](spec.md#fr-004) | None | None | ❌ Missing — pure preview next; atomic revisions Waiting for 008 | — |
| [FR-005](spec.md#fr-005) | [Unicode](../../../src-tauri/src/word_replacements/unicode.rs) | `AC-006: Folding with source offsets` | ⚠️ Partial — primitives only; matcher/pipeline pending | 2026-10-13 |
| [FR-006](spec.md#fr-006) | None | None | ❌ Missing — Waiting for 008 domain/CLI framework | — |

## AC mapping

| AC | Test File | Status |
|---|---|---|
| [AC-001](spec.md#ac-001) | [legacy/default model tests](../../../src-tauri/src/word_replacements/mod.rs), [identity tests](../../../src-tauri/src/word_replacements/model.rs) | ⚠️ Partial model/empty legacy defaults; UI/restart persistence Waiting for 008 |
| [AC-002](spec.md#ac-002) | [validation tests](../../../src-tauri/src/word_replacements/mod.rs), [Unicode tests](../../../src-tauri/src/word_replacements/unicode.rs) | ⚠️ Partial pure-domain evidence; complete no-partial-write/UI/CLI evidence Waiting for 008 |
| [AC-003](spec.md#ac-003) | None | ❌ Missing — preview/editor pending |
| [AC-004](spec.md#ac-004) | None | ❌ Missing — exchange core next |
| [AC-005](spec.md#ac-005) | None | ❌ Missing — exchange/preview next; persistence faults Waiting for 008 |
| [AC-006](spec.md#ac-006) | [Unicode primitive tests](../../../src-tauri/src/word_replacements/unicode.rs) | ⚠️ Partial primitives; matching fixtures and certified assertions pending |
| [AC-007](spec.md#ac-007) | None | ❌ Missing — matcher and recording-start pipeline pending; committed publication Waiting for 008 |
| [AC-008](spec.md#ac-008) | None | ❌ Missing — Waiting for 008 |

## Files Created/Modified

- Read [the domain modules](../../../src-tauri/src/word_replacements/mod.rs) for opaque identities, one validator and pinned Unicode 17 transformations.
- Inspect [Unicode provenance](../../../src-tauri/src/word_replacements/unicode-sources.json), [the compact data](../../../src-tauri/src/word_replacements/unicode-data.txt) and [the license](../../../src-tauri/src/word_replacements/unicode-license.txt) for exact source identities.
- Inspect [settings](../../../src-tauri/src/settings.rs) for the additive `word_replacements` field and empty serde/default behavior; no parallel settings writer is introduced.
- Inspect [the native entry point](../../../src-tauri/src/lib.rs) for the shared Specta command registry and explicit ignored binding export tool. The exporter constructs no Tauri application and reads no user store.
- Read [generated bindings](../../../src/bindings.ts) for the actual native-exported replacement types.
- Inspect [Cargo dependencies](../../../src-tauri/Cargo.toml) and [the lock](../../../src-tauri/Cargo.lock) for the UUID `=1.21.0` pin.

## Visual and release evidence

UI design certification remains BLOCKED. No visual baseline, native rule editor/dialog, real user dictation or complete AC is claimed by the core unit tests. Windows/Linux and interactive native checks stay pending release evidence when unavailable, with exact protocols to be recorded at the relevant implementation step.
