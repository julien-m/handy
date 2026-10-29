// LiveSpec traceability: @spec(AC-002) @spec(AC-003) @spec(AC-005)
/**
 * Menu sections and star labels of the destination favorites (012): favorites in their own
 * section in the backend's order, listed once even when frontmost, star names and refusal keys
 * present in every locale. Run with `bun run test:indicator`.
 */
import { describe, expect, test } from "bun:test";
import type { DestinationMenu, MenuEntry } from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import {
  favoriteLabelKey,
  favoriteNotSavedKey,
  menuSections,
} from "./destination-model";

const entry = (token: number, overrides: Partial<MenuEntry>): MenuEntry => ({
  token,
  display_name: `App ${token}`,
  icon_ref: null,
  current: false,
  frontmost: false,
  pinned: false,
  favorite: false,
  running: true,
  ...overrides,
});

const menuOf = (entries: MenuEntry[]): DestinationMenu => ({
  dictation_id: 7,
  selection_version: 1,
  frontmost_at_end_current: false,
  entries,
});

const tokens = (entries: MenuEntry[]): number[] =>
  entries.map((item) => item.token);

const lookup = (key: string): unknown =>
  key
    .split(".")
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown> | undefined)?.[part],
      en,
    );

// @spec AC-003: Favorites in their own section, listed once — .specs/features/012-destination-favorites/spec.md#ac-003
describe("favorites section (012 AC-003)", () => {
  test("favorites keep the backend order after the running apps", () => {
    const entries = [
      entry(1, { frontmost: true }),
      entry(2, {}),
      entry(3, { favorite: true }),
      entry(4, { favorite: true, running: false }),
    ];
    const sections = menuSections(menuOf(entries));
    expect(sections.frontmost?.token).toBe(entries[0].token);
    expect(tokens(sections.others)).toEqual([entries[1].token]);
    expect(tokens(sections.favorites)).toEqual(tokens(entries.slice(2)));
  });
  test("a frontmost favorite is listed only among the favorites", () => {
    const entries = [
      entry(1, { frontmost: true, favorite: true }),
      entry(2, {}),
    ];
    const sections = menuSections(menuOf(entries));
    expect(sections.frontmost).toBeNull();
    expect(tokens(sections.others)).toEqual([entries[1].token]);
    expect(tokens(sections.favorites)).toEqual([entries[0].token]);
  });
});

// @spec AC-005: Star is a named toggle button — .specs/features/012-destination-favorites/spec.md#ac-005
describe("star labels and refusals (012 AC-002, AC-005)", () => {
  test("the star names the add or remove action with the app", () => {
    const add = favoriteLabelKey(entry(1, {}));
    const remove = favoriteLabelKey(entry(1, { favorite: true }));
    expect(add).not.toBe(remove);
    for (const key of [add, remove]) {
      expect(String(lookup(key))).toContain("{{name}}");
    }
  });
  test("every not-saved reason has a translation", () => {
    for (const reason of ["full", "failed"] as const) {
      expect(typeof lookup(favoriteNotSavedKey(reason))).toBe("string");
    }
  });
  test("favorite strings never use the word pin (reserved to 009)", () => {
    const menu = en.overlay.destination.menu;
    const strings = [
      menu.favorites,
      menu.addFavorite,
      menu.removeFavorite,
      menu.notOpen,
      ...Object.values(en.overlay.destination.favoriteNotSaved),
    ];
    for (const text of strings) {
      expect(text.toLowerCase()).not.toContain("pin");
    }
  });
});
