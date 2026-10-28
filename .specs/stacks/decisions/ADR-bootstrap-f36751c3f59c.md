# ADR — Observed stack retention

- **Date:** 2026-10-09
- **Status:** Accepted (observed existing implementation)

## Context

| Layer | Choice | Version | Evidence |
|---|---|---|---|
| JavaScript | JavaScript | Unknown | package.json |
| React | React | ^18.3.1 | package.json |
| Vite | Vite | ^6.4.1 | package.json |
| TypeScript | TypeScript | ~5.6.3 | package.json |
| Rust | Rust | Unknown | src-tauri/Cargo.toml |
| Tauri | Tauri | 2.11.5 | src-tauri/Cargo.toml |


## Decision

Retain independent observed stack families and package manager bun.
This records existing code rather than selecting a new stack.

## Consequences

Unobserved business/deployment choices remain Unknown; future changes require an explicit decision.
