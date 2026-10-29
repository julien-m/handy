# Changelog - 012-destination-favorites

## 2026-10-29 — [Feature]: Destination favorites with a star in the badge menu; Pin and Unpin in a fixed footer

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md; 009 spec amended: Behavioral AC and decision `pin_entries_in_fixed_footer`)
- **Code modified:** `src-tauri/src/destination/` (favorites, menu order, toggle, menu, manager, snapshot, pin, context), `src-tauri/src/commands/destination.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/` (descriptor, public, protocol, validation, errors, destination domain and CLI), `src-tauri/src/cli_destination.rs`, `src/overlay/` (menu, rows, model, hook, stack, CSS, pin-state), `src/bindings.ts`, 26 locales, tests and harness, `README.md`, 008 contract
- **AC impacted:** AC-001 to AC-006 (automated evidence; native proofs pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's request of 2026-10-29: a star on each app row of the badge menu, favorites kept across dictations, listed bottom-up nearest the badge; the coordinator's precisions the same day: a star, never a pin, and Pin/Unpin always visible (Julien could not see Pin above a long list).
- Decisions in `spec.md` Clarifications; Penflow mockup and contract gap in `implementation.md`.
