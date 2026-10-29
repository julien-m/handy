# Tool defect: Native Cargo execution capture crosses generated rule symlink

- **Status:** resolved (2026-10-13)
- **Date:** 2026-10-13
- **Feature:** 008-settings-cli
- **Phase:** implement
- **Runtime mode:** dev
- **Runtime source:** /Users/julienm/projects/livespec

## Command

```text
livespec test --feature 008-settings-cli --acceptance-mapping .specs/features/008-settings-cli/proofs/resume-step1/partial-mapping.json --execution-command "cargo test --manifest-path src-tauri/Cargo.toml --test settings_platform" --report-adapter junit
```

## Error

```text
Invalid execution evidence: '/Users/julienm/projects/livespec/.agent-sync/rules/livespec/commands.md' is not in the subpath of '/Users/julienm/projects/handy'
```

## Resume

1. update the dev checkout /Users/julienm/projects/livespec to a commit containing the fix
2. resume phase `implement` of `008-settings-cli` (`livespec pipeline update --feature 008-settings-cli --phase implement --status in_progress`)

## Workaround applied (2026-10-13, orchestrator)

`livespec runtime dev` had retargeted the git-ignored `.agent-sync.local/{skills,rules}` entries to symlinks into the LiveSpec checkout, so execution capture followed them out of the project. They were replaced by copies of the same content (same LiveSpec commit). Re-running the failing command no longer reports the out-of-project path; its remaining gap is the stale `partial-mapping.json` that still cites the removed `src-tauri/tests/settings_platform/handoff.rs`. The underlying LiveSpec defect (capture should not follow ignored symlinks outside the project) still needs a fix in the LiveSpec repository.

## Resolution (2026-10-13)

Fixed in the LiveSpec checkout used by this project (dev mode, branch `fix/cargo-runner-and-capture-scope`, merge commit `e272452`): trusted Cargo execution runner (LiveSpec feature 085) and execution capture limited to the project (policy 078.2). Captures made under policy 078.1 must be redone.
