# Documentation and final checks of the integration

Commands and raw outputs (`*.txt` with their `*.exit`). Rust with `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`.

| Check | Output |
|---|---|
| `cargo test --no-fail-fast` (full suite: lib 464 passed/17 ignored, settings_cli 19, settings_domains 8, settings_live 10, settings_platform 33, settings_transactions 42/1 ignored, settings_validation 21, settings_writers 7, shortcuts_cli 8) | `cargo-test-full.txt` |
| `cargo clippy --lib --bins -- -D warnings`, `cargo fmt -- --check` | `cargo-clippy.txt`, `cargo-fmt-check.txt` |
| `bun run lint`, `format:check`, `tsc --noEmit`, `check:translations`, `test:shortcut-list`, `test:settings-errors`, `test:settings-store`, `test:keyboard` | matching `*.txt` |
| `livespec validate .specs/features/004-multiple-dictation-shortcuts` | `livespec-validate-feature.txt` (exit 0) |
| `livespec validate … --progression implement --model claude-opus-5-5 --review-max-chars 180000` | `livespec-progression-implement.txt`: `BLOCKED: analyze_not_ready:incomplete`, already so at `e2b0ab5`, before this documentation commit (the Analyze receipt predates the merge of `dev`); a fresh Analyze belongs to the reviewers |
| `livespec conventions verify --feature repo --json` | `conventions-repo-summary.json` (PASS, new 0, baseline unchanged); an earlier PASS run of the same tree is `conventions-repo-run1-summary.json` |

`preexisting-conventions-runs.json` inventories the 16 untracked conventions run directories left by the 2026-10-13 runs (file names, sizes and hashes) before they were moved to the Trash, so that the worktree keeps no untracked file except `pipeline.md`.

The repository pre-commit hook runs `livespec doctor`, which requires a current conventions receipt on disk: the final run directory stays until the commit is made, then goes to the Trash like the others.
