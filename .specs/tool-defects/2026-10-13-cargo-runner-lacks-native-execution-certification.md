# Tool defect: Cargo runner lacks native execution certification

- **Status:** resolved (2026-10-13)
- **Date:** 2026-10-13
- **Feature:** 008-settings-cli
- **Phase:** implement
- **Runtime mode:** dev
- **Runtime source:** /Users/julienm/projects/livespec

## Command

```text
HANDY_FORCE_AI_STUB=1 livespec test --feature 008-settings-cli --acceptance-mapping .specs/features/008-settings-cli/proofs/current-step1/partial-mapping.json --execution-command "cargo test --manifest-path src-tauri/Cargo.toml --test settings_platform" --report-adapter junit
```

## Error

```text
Unsupported runner: structured reports alone cannot certify execution; Missing or unstable fresh report: runner-report; Missing actual assertion execution trace: runner-assertions
```

## Resume

1. update the dev checkout /Users/julienm/projects/livespec to a commit containing the fix
2. resume phase `implement` of `008-settings-cli` (`livespec pipeline update --feature 008-settings-cli --phase implement --status in_progress`)

## Program decision (2026-10-13)

The user decided to continue the program: features are implemented and verified with local cargo/frontend/conventions evidence, committed on their branches and merged into `dev`; every AC stays "LiveSpec certification pending (tool defect)" until the LiveSpec repository ships a Cargo runner, then certification is replayed per feature.

## Resolution (2026-10-13)

Fixed in the LiveSpec checkout used by this project (dev mode, branch `fix/cargo-runner-and-capture-scope`, merge commit `e272452`): trusted Cargo execution runner (LiveSpec feature 085) and execution capture limited to the project (policy 078.2). Captures made under policy 078.1 must be redone.
