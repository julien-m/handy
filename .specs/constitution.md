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
