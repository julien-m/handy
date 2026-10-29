// LiveSpec traceability: @spec(AC-003) @spec(AC-004) @spec(AC-005)
/**
 * Markup of the destination menu with favorites (012) and the fixed footer (009 as amended on
 * 2026-10-29): the running apps in the scrolling area, the favorites below them with a label,
 * every row followed by its star toggle (name, pressed state), a favorite that is not running
 * dimmed with "Not open", and Frontmost app at the end, Pin and Unpin last, nearest the badge.
 * Run with `bun run test:indicator`.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import type { DestinationMenu, MenuEntry } from "@/bindings";
import {
  chatgpt,
  destination,
  initIndicatorI18n,
  render,
  session,
} from "./indicator-test-fixtures";

beforeAll(initIndicatorI18n);

const entry = (
  token: number,
  name: string,
  extra: Partial<MenuEntry> = {},
) => ({
  token,
  display_name: name,
  icon_ref: null,
  current: false,
  frontmost: false,
  pinned: false,
  favorite: false,
  running: true,
  ...extra,
});

const menu: DestinationMenu = {
  dictation_id: 7,
  selection_version: 1,
  frontmost_at_end_current: false,
  entries: [
    entry(1, "Comet", { frontmost: true }),
    entry(2, "ChatGPT", { current: true }),
    entry(3, "Notes", { favorite: true, running: false }),
    entry(4, "Claude", { favorite: true }),
  ],
};

const MENU = "recording-indicator-destination-menu-menu";

const renderMenu = (pin: { pinned: boolean } = { pinned: false }): string => {
  const base = session(chatgpt);
  const snapshot = pin.pinned
    ? { ...base, pin: { display_name: "Comet", icon_ref: null } }
    : base;
  return render({
    destination: destination(snapshot, { status: "open", menu }),
  });
};

const position = (html: string, needle: string): number => {
  const index = html.indexOf(needle);
  expect(index).not.toBe(-1);
  return index;
};

describe("favorites in the destination menu (012)", () => {
  test("running apps scroll, favorites and the footer follow in order", () => {
    const html = renderMenu();
    const order = [
      `data-testid="${MENU}-apps"`,
      `data-testid="${MENU}-frontmost"`,
      `data-testid="${MENU}-favorites-label"`,
      ">Notes<",
      ">Claude<",
      `data-testid="${MENU}-footer"`,
      `data-testid="${MENU}-frontmost-at-end"`,
      `data-testid="${MENU}-pin"`,
    ];
    const indexes = order.map((needle) => position(html, needle));
    expect([...indexes].sort((a, b) => a - b)).toEqual(indexes);
    expect(html).toContain('class="smenu-scroll"');
  });
  test("every row has a named star toggle with its pressed state", () => {
    const html = renderMenu();
    const stars = html.match(
      new RegExp(`data-testid="${MENU}-favorite-toggle"`, "g"),
    );
    expect(stars?.length).toBe(menu.entries.length);
    expect(html).toContain('aria-label="Add ChatGPT to favorites"');
    expect(html).toContain('aria-label="Remove Claude from favorites"');
    expect(html).toMatch(/aria-pressed="true"[^>]*aria-label="Remove Claude/);
    expect(html).toMatch(/aria-pressed="false"[^>]*aria-label="Add Comet/);
  });
  test("a favorite that is not running is disabled and marked Not open", () => {
    const html = renderMenu();
    expect(html).toMatch(/data-instance-id="app-3" disabled=""/);
    expect(html).toContain(`data-testid="${MENU}-not-open"`);
    expect(html).toContain(">Not open<");
  });
  test("pinned: Unpin closes the footer, after Frontmost app at the end", () => {
    const html = renderMenu({ pinned: true });
    const prefix = "recording-indicator-pinned-menu";
    const end = position(html, `data-testid="${prefix}-frontmost-at-end"`);
    const pin = position(html, `data-testid="${prefix}-pin"`);
    const unpin = position(html, `data-testid="${prefix}-unpin"`);
    expect(end).toBeLessThan(pin);
    expect(pin).toBeLessThan(unpin);
    expect(position(html, `data-testid="${prefix}-favorites"`)).toBeLessThan(
      end,
    );
  });
});
