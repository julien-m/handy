---
feature: destination-favorites
title: Destination Favorites Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-29
---

# Implementation — 012-destination-favorites

Status: In Progress (automated evidence complete; native macOS proof pending with Julien). Branch `feat/destination-favorites` from `dev` `be40524b`, behind 006's `destination-return` rollout gate for every visible part (badge, CLI domain); the setting and its descriptor exist in every build.

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Validated favorites, settings service only](spec.md#fr-001) | `src-tauri/src/destination/favorites.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,protocol.rs,validation.rs,errors.rs,errors_pin_enter.rs}` | `@spec FR-001: One shared validation, AC-002: Cap and unique identities` | ✅ Implemented | 2026-10-29 |
| [FR-002: Pure bottom-up menu order](spec.md#fr-002) | `src-tauri/src/destination/{menu_order.rs,menu.rs,manager.rs}` | `@spec FR-002: Pure bottom-up menu order`, `@spec FR-002: Favorites last, nearest the badge` | ✅ Implemented | 2026-10-29 |
| [FR-003: Star a token, session unchanged, same tokens](spec.md#fr-003) | `src-tauri/src/destination/favorite_toggle.rs`, `src-tauri/src/commands/destination.rs`, `src-tauri/src/destination/snapshot.rs`, `src-tauri/src/lib.rs`, `src/bindings.ts` | `@spec FR-003: Star a menu token, session unchanged`, `@spec FR-003: Star toggle saved, menu re-ordered` | ✅ Implemented | 2026-10-29 |
| [FR-004: Star, favorites section, fixed footer](spec.md#fr-004) | `src/overlay/{destination-menu.tsx,destination-menu-rows.tsx,destination-model.ts,use-destination.ts,indicator-stack.tsx,indicator.css,pin-state.ts}`, `src/i18n/locales/*/translation.json` (26 locales) | `@spec FR-004: Star, favorites and fixed footer rendered`, `@spec AC-001: Compact row, full app name readable` | ✅ Implemented | 2026-10-29 |
| [FR-005: CLI get, favorite-add, favorite-remove](spec.md#fr-005) | `src-tauri/src/settings_service/domains/{destination.rs,destination_favorites.rs,mod.rs}`, `src-tauri/src/settings_service/cli/destination.rs`, `src-tauri/src/cli_destination.rs`, `src-tauri/src/destination/pin.rs` | `@spec FR-005: Favorites from the settings CLI`, `@spec AC-006: favorite-add and favorite-remove` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| AC-001 | `src-tauri/src/destination/favorite_toggle_tests.rs`, `src-tauri/src/destination/favorites_tests.rs`, `tests/destination-favorites.spec.ts` (star toggle; compact row with full names, N29) | ✅ (automated; native pending) |
| AC-002 | `src-tauri/src/destination/favorites_tests.rs`, `src-tauri/src/settings_service/domains/destination_favorites_tests.rs`, `src/overlay/favorites-model.test.ts` | ✅ |
| AC-003 | `src-tauri/src/destination/menu_order_tests.rs`, `src/overlay/favorites-model.test.ts`, `src/overlay/favorites-render.test.tsx`, `tests/destination-favorites.spec.ts` | ✅ (automated; native pending) |
| AC-004 | `src-tauri/src/destination/favorite_toggle_tests.rs`, `src/overlay/favorites-render.test.tsx`, `tests/destination-favorites.spec.ts` | ✅ (automated; native pending) |
| AC-005 | `src/overlay/favorites-render.test.tsx`, `src/overlay/favorites-model.test.ts`, `tests/destination-favorites.spec.ts` | ✅ (headless Chromium; VoiceOver pending) |
| AC-006 | `src-tauri/src/settings_service/domains/destination_favorites_tests.rs` (also covered by the settings CLI and validation integration suites) | ✅ |

## Files Created/Modified

- Created: `src-tauri/src/destination/{favorites.rs,favorites_tests.rs,menu_order.rs,menu_order_tests.rs,favorite_toggle.rs,favorite_toggle_tests.rs}`, `src-tauri/src/settings_service/domains/{destination_favorites.rs,destination_favorites_tests.rs}`, `src/overlay/{destination-menu-rows.tsx,favorites-model.test.ts,favorites-render.test.tsx}`, `tests/harness/destination-menu.{html,tsx}`, `tests/destination-favorites.spec.ts`.
- Modified: menu storage keeps `MenuTarget` rows (optional process) in `destination/{menu.rs,manager.rs}`; `ApplicationIdentity` deserializes; `MenuEntry` gains `favorite` and `running`; `FavoriteAnswer`/`FavoriteNotSaved`; `destination_menu` passes the saved favorites; new command `toggle_destination_favorite`; `AppSettings.destination_favorites` (lenient read); descriptor, public projection, schema item, refusal naming the domain; `ValidationReason::{DestinationDomain, FavoritesFull, FavoriteInvalid}`; destination domain actions; overlay menu restructured (scrolling running apps, fixed favorites, fixed footer with 009's Pin/Unpin); 26 locales; `README.md`; 008 contract (`settings-cli.md`, `settings-cli.schema.json`); 009 spec, changelog and implementation (Pin moved to the footer).
- Integration tests: `src-tauri/tests/settings_validation/{inventory.rs,contracts.rs,request_contracts.rs,scalars.rs}` and `src-tauri/tests/settings_cli/contract.rs` count the new key. They also count `plain_dictation_prompt_id`, which `bd08db0f` (005 N23) added on `dev` without updating them or the checked schema: their counts and the checked schema were off by that key on `dev` (inferred from the failures before this run's fix, not rerun on a `dev` checkout); the checked schema now lists both keys.

## Decisions taken in this run

See `spec.md` Clarifications: `favorite_not_running` (shown, never launched, not choosable), `star_not_pin`, `pin_entries_in_fixed_footer` (009 amended), `favorites_fixed`, `favorites_cap` (10), `star_accessible_name`, `top_placement`, `toggle_stays_open`, `compact_row` (N29 fix, 009 amended with `no_dictation_only_hint`).

## Gaps

- **Penflow mockup and contract not updated (requirement 7).** `penflow/ui.pen` (Pencil, editable only through a Pencil session) and `penflow/flow-ui-contract/contract.json` still show the 009 layout (Pin above the list) and have no favorites. New test ids not in the contract: `<screen>-menu-apps`, `-favorites`, `-favorites-label`, `-favorite`, `-favorite-toggle`, `-not-open`, `-footer`. A Penflow pass should add these nodes, the star action (`state_change`, no transition of the destination) and the footer order, then regenerate `ui.pen` and the docs.
- **Overlay placed at the top of the screen:** same internal order (decision `top_placement`).
- ~~**Names truncated next to "This dictation only":**~~ closed on 2026-10-29 (N29, decision `compact_row`; 009 decision `no_dictation_only_hint`): the hint is removed, the star is 24 px (glyph 14 px), the row gap 2 px; names up to about 20 characters are shown in full at the 280 px menu width (`tests/destination-favorites.spec.ts`, « realistic app names are shown in full beside the star, with no hint »; harness captures `proofs/n29/menu-before.png` and `proofs/n29/menu-after.png`).
- **Penflow contract still lists the removed hint:** `penflow/flow-ui-contract/contract.json` and `screens/recording-indicator-pinned.md` keep the node `recording-indicator-pinned.menu.hint` ("This dictation only"); the next Penflow pass removes it with the 012 nodes above.
- **ARIA:** the star is a `button` with `aria-pressed` inside `role="menu"` (requested toggle semantics), not a `menuitemcheckbox`.

## Native proofs pending with Julien

1. Live overlay on macOS: star a running app during a dictation; menu stays open, badge unchanged; the app is listed just above "Frontmost app at the end"; at the next dictation it is still there.
2. With about twenty open apps: "Pin <destination>", "Unpin <app>" and the favorites visible without scrolling, nearest the badge.
3. Quit a favorite: listed "Not open", dimmed, not choosable; nothing launches; its star removes it.
4. VoiceOver: the star announces "Add <app> to favorites", toggle button, not pressed / pressed.
5. `handy settings destination favorite-add <bundle id>` against the running app, then the menu shows it.
