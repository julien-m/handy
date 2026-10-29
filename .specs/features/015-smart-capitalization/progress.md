# Progress — 015-smart-capitalization

Run of 2026-10-29 (Claude Code, `/spec-feature --auto`, model claude-opus-5-5), branch `feat/smart-capitalization` from `dev` `00f2ec6c`. Feature number 015: 013 and 014 are reserved by the branches `feat/dictate-to-active-app` and `feat/default-input-per-app`. Conventions loaded from `.conventions/index.md` (sub-domain `code`: general, rust, react, javascript, testing). Native agents only; no Codex, no `audit` skill. Clean room: `.investigations/` never opened (see the spec header). Open decisions are recorded in `spec.md` (Clarifications). Two coordinator messages during the run were applied: Julien's clean-room contract (default on for new installs, off on upgrade; provenance table; reference example) and his UI instruction (a "Text formatting" section with one independent row per option).

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | `smart_capitalization/decision.rs`, `decision_tests.rs` | `cargo test --lib smart_capitalization` | Pass | 2026-10-29 |
| 2 | Done | `smart_capitalization/transform.rs`, `transform_tests.rs` | same | Pass | 2026-10-29 |
| 3 | Done | `smart_capitalization/mod.rs` (preparation order, smart spacing slot) | same | Pass | 2026-10-29 |
| 4 | Done | `smart_capitalization/reader.rs`, `reader_tests.rs` | same | Pass | 2026-10-29 |
| 5 | Done | `smart_capitalization/macos.rs`, `destination/{field.rs,macos_field.rs}` | `cargo clippy --all-targets --features destination-return,smart-paste-dispatch`, and without features | exit 0, no warnings | 2026-10-29 |
| 6 | Done | `destination/field_context.rs`, `tests/smart_capitalization_context.rs`, `tests/destination_field/fake.rs` | `--test smart_capitalization_context --test destination_field_lifetime --test destination_field_return` | 5, 7, 10 passed | 2026-10-29 |
| 7 | Done | `smart_capitalization/app.rs`, `destination/delivery.rs`, `dictation_output.rs`, `actions.rs` | `cargo test --lib` (gates open) | 1195 passed | 2026-10-29 |
| 8 | Done | settings (`settings.rs`, descriptor, public, output domain, UI command, bindings), 008 contract, settings tests | `--test settings_validation --test settings_domains --test settings_cli --test settings_live`; `HANDY_WRITE_BINDINGS=1` bindings test | 25, 8, 19, 10 passed | 2026-10-29 |
| 9 | Done | `src/components/settings/text-formatting/*`, `SettingContainer`, `ToggleSwitch`, `AdvancedSettings`, store, 26 locales | `bun run lint`, `tsc --noEmit`, `test:text-formatting`, `test:output-settings`, `test:settings-store`, `check:translations`, `format:check` | Pass | 2026-10-29 |
| 10 | Done | independent review (native agent, effort high) and corrections | `checks/2026-10-29-review.md` | Round 1 BLOCKED (1 blocking, 1 warning, 6 info) → corrected; round 2 PASS with 1 warning → corrected (`put_back` checks `serves`) | 2026-10-29 |
| 11 | Done | spec, plan, implementation, changelogs, README, registry | `livespec conventions verify --feature repo --json` | PASS | 2026-10-29 |
| 12 | Pending (Julien) | — | Native macOS protocol in `implementation.md` | Not run | 2026-10-29 |
