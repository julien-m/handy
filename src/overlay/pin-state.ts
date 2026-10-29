// Pure rules of the session pin on the recording indicator (009): which Pin/Unpin entries the
// badge menu lists in its fixed footer (nearest the badge, decision of 2026-10-29), and the
// badge's accessible name with the pin mark. No Tauri access here; the actions live in
// `use-destination.ts`.
import type { DestinationMenu, DestinationSnapshot } from "@/bindings";
import type { BadgeView } from "./destination-model";

/**
 * Test ids of the pin entries under the contract screen shown: Pin is
 * `recording-indicator-destination-menu-menu-pin` (nothing pinned yet), Unpin, the badge's pin
 * mark and the "This dictation only" hint are `recording-indicator-pinned-…` (an application is
 * pinned).
 * @param screen test-id prefix of the screen shown
 * @returns the test id of each pin element
 */
export const pinTestIds = (screen: string) => ({
  pin: `${screen}-menu-pin`,
  unpin: `${screen}-menu-unpin`,
  /** Pin mark of the badge; derived from the contract's `recording-indicator-pinned` screen. */
  mark: `${screen}-destination-mark`,
  /** "This dictation only" hint of the other applications while pinned; derived as well. */
  hint: `${screen}-menu-hint`,
});

/** One pin entry of the badge menu. */
export interface PinEntry {
  kind: "pin" | "unpin";
  /** Application the entry names (`{{name}}` of its label). */
  name: string;
  labelKey: string;
}

const CHOICE_PHASES = new Set(["recording", "processing"]);

// @spec AC-001: Pin <destination> and Unpin in the badge menu — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/**
 * Pin entries of the badge menu, both listed in its fixed footer, nearest the badge and never
 * scrolled out of view (009 AC-001 as amended on 2026-10-29): "Pin <destination>" when the
 * current destination is an application that is not the pinned one, "Unpin <pinned app>" when
 * an application is pinned; nothing once insertion began.
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
    });
  }
  if (snapshot.pin) {
    entries.push({
      kind: "unpin",
      name: snapshot.pin.display_name,
      labelKey: "overlay.destination.menu.unpin",
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
