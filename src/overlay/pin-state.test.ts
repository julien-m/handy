/**
 * Pin entries of the destination badge menu and pinned badge of the recording indicator (009).
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: has_dropdown, is_keyboard_navigable (the badge menu stays pointer-operated per 006;
 * the keyboard trait is covered by the settings selector tests).
 * Run with `bun run test:indicator`.
 */
import { describe, expect, test } from "bun:test";
import type {
  DestinationMenu,
  DestinationSnapshot,
  DestinationSource,
  PinView,
} from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import { badgeView } from "./destination-model";
import { badgeLabel, pinEntries, pinTestIds } from "./pin-state";

const chatgptPin: PinView = { display_name: "ChatGPT", icon_ref: null };

const snapshot = (
  overrides: Partial<DestinationSnapshot> = {},
): DestinationSnapshot => ({
  dictation_id: 3,
  destination_generation: 7,
  selection_version: 0,
  phase: "recording",
  return_enabled: true,
  support: { state: "supported" },
  selection: { kind: "application", display_name: "Comet", icon_ref: null },
  source: "captured" as DestinationSource,
  pin: null,
  disposition: "pending",
  ...overrides,
});

const menu: DestinationMenu = {
  dictation_id: 3,
  selection_version: 0,
  entries: [
    {
      token: 1,
      display_name: "Comet",
      icon_ref: null,
      current: true,
      frontmost: true,
      pinned: false,
      favorite: false,
      running: true,
    },
  ],
  frontmost_at_end_current: false,
};

const translate = (key: string, values?: Record<string, string>): string => {
  const path = key.split(".");
  let node: unknown = en;
  for (const part of path) {
    node = (node as Record<string, unknown> | undefined)?.[part];
  }
  if (typeof node !== "string") throw new Error(`missing translation ${key}`);
  return node.replace(
    /\{\{(\w+)\}\}/g,
    (_, name: string) => values?.[name] ?? "",
  );
};

describe("pin entries in the badge menu (has_dropdown)", () => {
  test("nothing pinned: 'Pin <destination>' is offered (listed in the fixed footer)", () => {
    const entries = pinEntries(snapshot(), menu);
    expect(entries).toEqual([
      {
        kind: "pin",
        name: "Comet",
        labelKey: "overlay.destination.menu.pin",
      },
    ]);
    expect(translate(entries[0].labelKey, { name: "Comet" })).toBe("Pin Comet");
  });

  test("the current destination pinned: only Unpin is offered", () => {
    const pinned = snapshot({
      selection: {
        kind: "application",
        display_name: "ChatGPT",
        icon_ref: null,
      },
      source: "pinned",
      pin: chatgptPin,
    });
    expect(pinEntries(pinned, menu)).toEqual([
      {
        kind: "unpin",
        name: "ChatGPT",
        labelKey: "overlay.destination.menu.unpin",
      },
    ]);
    expect(pinTestIds("recording-indicator-pinned").unpin).toBe(
      "recording-indicator-pinned-menu-unpin",
    );
    // 009-01: Pin is offered while nothing is pinned, on the destination menu screen.
    expect(pinTestIds("recording-indicator-destination-menu").pin).toBe(
      "recording-indicator-destination-menu-menu-pin",
    );
  });

  test("another app pinned: Pin the current destination and Unpin the pinned app", () => {
    const entries = pinEntries(
      snapshot({ source: "badge", pin: chatgptPin }),
      menu,
    );
    expect(entries.map((entry) => [entry.kind, entry.name])).toEqual([
      ["pin", "Comet"],
      ["unpin", "ChatGPT"],
    ]);
  });

  test("no application destination: nothing to pin", () => {
    const frontmost = snapshot({
      selection: { kind: "frontmost_at_end" },
      source: "frontmost_at_end",
    });
    expect(pinEntries(frontmost, menu)).toEqual([]);
  });

  test("insertion begun: no pin entry at all", () => {
    expect(pinEntries(snapshot({ phase: "inserting" }), menu)).toEqual([]);
  });
});

describe("pinned badge", () => {
  test("pinned route: pin mark and accessible name 'Destination: <name>, pinned'", () => {
    const view = badgeView(
      snapshot({
        selection: {
          kind: "application",
          display_name: "ChatGPT",
          icon_ref: null,
        },
        source: "pinned",
        pin: chatgptPin,
      }),
    );
    expect(view).toMatchObject({ kind: "application", pinned: true });
    const label = badgeLabel(view, translate);
    expect(label).toBe("Destination: ChatGPT, pinned");
  });

  test("temporary other app while pinned: no pin claim on the badge", () => {
    const view = badgeView(snapshot({ source: "badge", pin: chatgptPin }));
    expect(view).toMatchObject({ kind: "application", pinned: false });
    expect(badgeLabel(view, translate)).toBe("Destination: Comet");
  });

  test("return off and an app pinned: the badge is shown (009 exception to 006 AC-003)", () => {
    const view = badgeView(
      snapshot({ return_enabled: false, source: "pinned", pin: chatgptPin }),
    );
    expect(view.kind).toBe("application");
  });

  test("return off and nothing pinned: no badge, as in 006", () => {
    expect(badgeView(snapshot({ return_enabled: false })).kind).toBe("hidden");
  });

  test("foreground unreadable: no badge even when pinned", () => {
    const unreadable = snapshot({
      support: { state: "unsupported", reason: "frontmost_unreadable" },
      pin: chatgptPin,
    });
    expect(badgeView(unreadable).kind).toBe("hidden");
  });
});
