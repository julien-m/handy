// Pure rules of the session pin on the recording indicator (009): which Pin/Unpin entries the
// badge menu lists above the application list, and the badge's accessible name with the pin
// mark. No Tauri access here; the actions live in `use-destination.ts`.
import type { DestinationMenu, DestinationSnapshot } from "@/bindings";
import type { BadgeView } from "./destination-model";

/** Test ids of the pin entries (contract screen `recording-indicator-pinned`). */
export const PIN_TEST_IDS = {
  /** Not in the Penflow contract (its pinned screen shows Unpin only); derived from it. */
  pin: "recording-indicator-pinned-menu-pin",
  unpin: "recording-indicator-pinned-menu-unpin",
  /** Pin mark of the badge; derived from the contract's `recording-indicator-pinned` screen. */
  mark: "recording-indicator-pinned-destination-mark",
} as const;

/** One pin entry of the badge menu. */
export interface PinEntry {
  kind: "pin" | "unpin";
  /** Application the entry names (`{{name}}` of its label). */
  name: string;
  labelKey: string;
  testId: string;
}

const CHOICE_PHASES = new Set(["recording", "processing"]);

// @spec AC-001: Pin <destination> and Unpin in the badge menu — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/**
 * Pin entries listed above the application list: "Pin <destination>" when the current
 * destination is an application that is not the pinned one, "Unpin <pinned app>" when an
 * application is pinned; nothing once insertion began.
 * @param snapshot Session snapshot of the badge.
 * @param menu The open menu (it must belong to the same dictation).
 */
export const pinEntries = (
  snapshot: DestinationSnapshot | null,
  menu: DestinationMenu,
): PinEntry[] => {
  if (!snapshot || snapshot.dictation_id !== menu.dictation_id) return [];
  if (!CHOICE_PHASES.has(snapshot.phase)) return [];
  const entries: PinEntry[] = [];
  const selection = snapshot.selection;
  if (selection.kind === "application" && snapshot.source !== "pinned") {
    entries.push({
      kind: "pin",
      name: selection.display_name,
      labelKey: "overlay.destination.menu.pin",
      testId: PIN_TEST_IDS.pin,
    });
  }
  if (snapshot.pin) {
    entries.push({
      kind: "unpin",
      name: snapshot.pin.display_name,
      labelKey: "overlay.destination.menu.unpin",
      testId: PIN_TEST_IDS.unpin,
    });
  }
  return entries;
};

/** Translation function shape used by `badgeLabel` (i18next's `t`). */
export type Translate = (
  key: string,
  values?: Record<string, string>,
) => string;

// @spec AC-001: Accessible name of a pinned badge — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/**
 * Accessible name of the badge: "Destination: <name>, pinned" for a pinned route; a temporary
 * other application never claims the pin.
 */
export const badgeLabel = (badge: BadgeView, t: Translate): string => {
  if (badge.kind === "hidden") return "";
  const name =
    badge.kind === "application"
      ? badge.name
      : t("overlay.destination.frontmostApp");
  const key =
    badge.kind === "application" && badge.pinned
      ? "overlay.destination.pinnedLabel"
      : "overlay.destination.label";
  return t(key, { name });
};
