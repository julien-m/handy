# Constitution — Handy

## Project Identity

Retain observed packages handy-app, handy.
Independent families:

| Layer | Choice | Version | Evidence |
|---|---|---|---|
| JavaScript | JavaScript | Unknown | package.json |
| React | React | ^18.3.1 | package.json |
| Vite | Vite | ^6.4.1 | package.json |
| TypeScript | TypeScript | ~5.6.3 | package.json |
| Rust | Rust | Unknown | src-tauri/Cargo.toml |
| Tauri | Tauri | 2.11.5 | src-tauri/Cargo.toml |


## Architecture Principles

- Preserve the existing source and native/frontend boundaries.
- Derive stack and manager choices from manifests; unsupported business decisions remain Unknown.
- Tool version availability certifies tooling only;
  application build, UI and runtime require separate execution evidence.
- Specify changed behavior before implementing it;
  require meaningful tests and current source verification.

## Testing Standards

Read [testing strategy](testing/strategy.md).

### Native platform evidence (user decision 2026-10-13)

- Code targets every supported platform (macOS, Windows, Linux) and must build and pass its automated tests.
- macOS native evidence is required before a feature is marked Implemented.
- Native evidence that needs Windows, Linux, an older Handy binary or a second user account is recorded as **pending release evidence** with the exact check to run. It never blocks implementation steps, feature completion on the integration branch, or the merge into `dev`; it is never claimed as passed.
- Implementations follow the reviewed spec at its minimal scope: a plan mechanism that the spec does not require (for example replaying commands for an older running Handy) is not built.
