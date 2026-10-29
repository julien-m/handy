# Changelog - 012-destination-favorites

## 2026-10-29 — [Bugfix]: N29 compact badge menu rows, no "This dictation only" hint (1 gap closed, visual)

- **Type:** Bug Fix
- **Spec modified:** Yes (AC-001 precision and decision `compact_row`; 009 decision `no_dictation_only_hint`)
- **Code modified:** `src/overlay/{destination-menu-rows.tsx,destination-menu.tsx,indicator-stack.tsx,pin-state.ts,indicator.css}`, 26 locales (key `overlay.destination.menu.thisDictationOnly` removed), `tests/harness/destination-menu.tsx` (`?names=`), `tests/destination-favorites.spec.ts`, `src/overlay/{indicator-render.test.tsx,pin-state.test.ts}`
- **Gaps closed:** N29 (application names cut to 2 or 3 letters beside the star and the hint)
- **Remaining:** Penflow contract still lists the hint node (recorded in `implementation.md`); native check by Julien
- **Author:** spec-fix (Claude Code, claude-opus-5-5)
- Julien's native finding N29: « Avec l'étoile, ça prend trop de place, je ne vois plus les textes. Et il y a marqué "cette dictée seulement" : ça, tu me l'enlèves. »

## 2026-10-29 — [Feature]: Destination favorites with a star in the badge menu; Pin and Unpin in a fixed footer

- **Type:** Feature
- **Spec modified:** Yes (created: spec.md, plan.md; 009 spec amended: Behavioral AC and decision `pin_entries_in_fixed_footer`)
- **Code modified:** `src-tauri/src/destination/` (favorites, menu order, toggle, menu, manager, snapshot, pin, context), `src-tauri/src/commands/destination.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/` (descriptor, public, protocol, validation, errors, destination domain and CLI), `src-tauri/src/cli_destination.rs`, `src/overlay/` (menu, rows, model, hook, stack, CSS, pin-state), `src/bindings.ts`, 26 locales, tests and harness, `README.md`, 008 contract
- **AC impacted:** AC-001 to AC-006 (automated evidence; native proofs pending)
- **Author:** Claude Code (spec-feature, autonomous mode, claude-opus-5-5)
- Julien's request of 2026-10-29: a star on each app row of the badge menu, favorites kept across dictations, listed bottom-up nearest the badge; the coordinator's precisions the same day: a star, never a pin, and Pin/Unpin always visible (Julien could not see Pin above a long list).
- Decisions in `spec.md` Clarifications; Penflow mockup and contract gap in `implementation.md`.
