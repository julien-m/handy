# Shortcuts CLI domain, quiescence lease and prepared shortcut changes

Commands and raw outputs of this commit (`*.txt` with their `*.exit`). Rust with `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`.

| Check | Output |
|---|---|
| `cargo test --lib -- shortcut settings_service transcription_coordinator secure_input` | `cargo-test-lib-targeted.txt` |
| `cargo test --test shortcuts_cli` (real binary, isolated portable store, simulated instance) | `cargo-test-shortcuts-cli.txt` |
| `cargo test --test settings_cli --test settings_domains --test settings_validation --test settings_writers --test settings_live` | `cargo-test-settings-integration.txt` |
| `cargo clippy --lib --bins -- -D warnings`, `cargo fmt -- --check` | `cargo-clippy.txt`, `cargo-fmt-check.txt` |
| `bun run lint`, `bun run format:check`, `tsc --noEmit`, `check:translations`, `test:shortcut-list`, `test:settings-errors`, `test:settings-store` | matching `*.txt` |
| `livespec conventions verify --feature repo --json` | `conventions-repo-summary.json` (PASS, new 0, baseline unchanged) |

Earlier runs kept for traceability: `conventions-repo-run1-fail-summary.json` (an `allow` directive in the test exceeded the suppression budget, removed), `conventions-repo-run2-summary.json` (PASS before the last fixes), `cargo-test-settings-integration-run1-flake.*` (one 008 process test got an empty stdout from its child, the harness flake of a freshly linked debug binary documented by 008; two reruns passed).
