# Progress — 013-default-input-per-app

Run of 2026-10-29 (Claude Code, `/spec-feature` in autonomous mode, model claude-opus-5-5), branch `feat/default-input-per-app` from `dev` `ab08ca1b`. Conventions loaded from `.conventions/index.md` (sub-domain `code`: general, javascript; plus rust, react, accessibility and naming from `$AIRESOURCES`). No pause for questions: open decisions are recorded in `spec.md` (Clarifications). Native agents only (the front was written by one native sub-agent against the typed contract of the plan); no Codex, no `audit` skill. The Penflow forward contract (Phase 0.5) and the independent review gates were not run in this autonomous pass (recorded as gaps; the mockup gap is in `implementation.md`). Native calibration before coding: an accessibility probe of Codex's focused window (1572 elements, composer at depth 31, 108 ms) and read-only `cmux identify`/`tree --json` calls (no `cmux send` was run).

Final verification: `proofs/final/summary.txt` (11 commands with their exit codes) and `proofs/conventions-verify-repo.json`.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | `default_input/model.rs`, `settings.rs`, `settings_service/{descriptor_data,public,protocol,errors,errors_pin_enter}.rs`, 008 contract | `cargo test --lib -- default_input::model` | Pass (cap, duplicates, distinctive field, replace, remove/clear, wire shape) | 2026-10-29 |
| 2 | Done | `default_input/{fingerprint,terminal,route}.rs` | `cargo test --lib -- default_input` | Pass (generated ids, match rule, chunks, parsing, plan table) | 2026-10-29 |
| 3 | Done | `default_input/{tree,search,runner,services,fake_tree}.rs` | same | Pass (BFS bounds, password, real process bound and kill, bundle capability) | 2026-10-29 |
| 4 | Done | `destination/{guard,insertion,delivery,manager,app,context,outcome}.rs`, `default_input/dispatch.rs`, `automatic_enter/{policy,transaction}.rs`, `recovery/{types,decision}.rs` | `cargo test --lib -- default_input::orchestrator_tests transaction_flow decision_tests` | Pass (editable unchanged, focused, not found, terminal typed, gone, no Enter) | 2026-10-29 |
| 5 | Done | `default_input/{capture,instance,macos_tree,macos_bundle}.rs`, `commands/default_input.rs`, `lib.rs`, `src/bindings.ts` | `HANDY_WRITE_BINDINGS=1 cargo test --lib bindings_tests` | Bindings regenerated and checked | 2026-10-29 |
| 6 | Done | `cli_destination.rs`, `settings_service/{cli,domains}/destination*.rs`, integration counts | `cargo test --lib -- destination_default_inputs`, `--test settings_validation --test settings_cli --test pin_enter_cli --test settings_domains` | Pass | 2026-10-29 |
| 7 | Done | `src/overlay/*default-input*`, `src/components/settings/default-inputs*`, `recovery-preview.ts`, 26 locales | `bun run test:indicator`, `test:output-settings`, `test:recovery-bubble`, `lint`, `tsc`, `check:translations`, prettier | Pass | 2026-10-29 |
| 8 | Done | 013 spec/plan/implementation/changelog/progress, README, roadmap, registry, changelogs | `livespec conventions verify --feature repo --json` | See `proofs/conventions-verify-repo.json` | 2026-10-29 |
| 9 | Pending (Julien) | — | Native macOS protocol in `implementation.md` | Not run | 2026-10-29 |
