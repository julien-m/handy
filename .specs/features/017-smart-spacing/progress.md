# Progress — 017-smart-spacing

Run of 2026-10-29 (Claude Code, `/spec-feature` in autonomous mode, model claude-opus-5-5), branch `feat/smart-spacing` from `dev` `07b298ad`. Feature number 017 (015 and 016 are on `dev`). Conventions loaded from `.conventions/index.md` (sub-domain `code`: general, rust, react, javascript, testing). Native agents only; no Codex, no `audit` skill. Clean room: `.investigations/` never opened, listed or searched (excluded from every search; the conventions baseline that names a path there was not read; convention reports were filtered before display); the only Spokenly source read is the public page S1.

Cargo ran with `nice -n 19`, `CARGO_BUILD_JOBS=3` and `CARGO_TARGET_DIR=src-tauri/target` of this worktree; `node_modules` is a symlink to the main checkout's.

Pipeline state: `pipeline.md` records Specify only. `livespec pipeline update --phase plan` answers `BLOCKED: clarify_not_ready:missing_current_spec_review` (no LiveSpec spec-review receipt was produced in this single-agent pass, as for 015 and 016); the later phases stay Pending rather than being forced.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 1 | Done | `smart_capitalization/context.rs` | `cargo test --lib -- smart_capitalization` | Pass | 2026-10-29 |
| 2 | Done | `smart_spacing/junction.rs`, `junction_tests.rs` | `cargo test --lib -- smart_spacing` | Pass | 2026-10-29 |
| 3 | Done | `smart_spacing/mod.rs`, `spacing_tests.rs` | same (40 smart spacing tests) | Pass | 2026-10-29 |
| 4 | Done | `smart_capitalization/mod.rs`, `preparation_tests.rs` | `cargo test --lib -- smart_` (256) | Pass | 2026-10-29 |
| 5 | Done | `smart_capitalization/{reader.rs,reader_tests.rs,macos.rs}`, `destination/{field.rs,macos_field.rs}` | same; `cargo clippy --all-targets --features destination-return,smart-paste-dispatch -- -D warnings` | exit 0 | 2026-10-29 |
| 6 | Done | `destination/field_context.rs`, `tests/smart_capitalization_context.rs`, `tests/destination_field/fake.rs` | `--test smart_capitalization_context` and the seven `destination_field_*` suites | 6, 11, 11, 7, 2 (+3 ignored), 1, 7, 10 passed | 2026-10-29 |
| 7 | Done | `smart_capitalization/app.rs`, `destination/{delivery.rs,delivery_tests.rs}` | `cargo test --lib --features destination-return,smart-paste-dispatch` | 1375 passed, 0 failed, 19 ignored | 2026-10-29 |
| 8 | Done | settings (`settings.rs`, descriptor, public, output domain, UI command, `lib.rs`), `bindings.ts` (`HANDY_WRITE_BINDINGS=1` test), store, 008 contract, settings validation counters (74, 56) | `--test settings_validation` (25, also with the gates closed), `settings_domains` 8, `settings_cli` 19, `settings_live` 10, `settings_platform` 33, `settings_transactions` 42, `settings_writers` 7, `recovery_output` 8, `recovery_payload` 1 | Pass | 2026-10-29 |
| 9 | Done | `text-formatting-options.ts` (first row), render test, 26 locales | `bun run lint` 0, `tsc --noEmit` 0, `check:translations` 0, `test:text-formatting` 0 (8 pass), `test:output-settings` 0, `test:smart-paragraphs` 0, `format:check` 0 | Pass | 2026-10-29 |
| 10 | Done | independent native review, effort `high` | `checks/2026-10-29-review.md` | Round 1 PASS (0 blocking, 4 warnings, 8 info) → corrected; round 2 PASS (0 blocking, 2 warnings, 7 info; file identities unchanged) → documentation only | 2026-10-29 |
| 11 | Done | `livespec conventions verify --feature repo --json` | first run FAIL (5 new errors in 017 tests: `panic!` in tests, a 75-line test) → corrected; second run | PASS, 0 new, 0 blockers (`checks/conventions-verify-repo.json`) | 2026-10-29 |
| 12 | Done | spec, plan, implementation, changelogs, registry, roadmap, README, 015 back-references | — | — | 2026-10-29 |
| 13 | Pending (Julien) | — | Native macOS protocol in `implementation.md` | Not run | 2026-10-29 |
