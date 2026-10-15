// Pure rules of the destination badge and menu on the recording indicator (feature 006): what
// the badge shows for a session snapshot, which snapshot wins when events and command answers
// cross, how the menu is split, and how a refused choice is explained. No Tauri access here.
import type {
  DestinationMenu,
  DestinationSnapshot,
  MenuEntry,
  SelectionRefusal,
  SessionPhase,
} from "@/bindings";

/** What the destination badge shows; `hidden` renders nothing (dev behaviour). */
export type BadgeView =
  | { kind: "hidden" }
  | { kind: "frontmost"; canChange: boolean }
  | {
      kind: "application";
      name: string;
      iconRef: string | null;
      canChange: boolean;
      /** The route comes from the session pin (009): the badge shows the pin mark. */
      pinned: boolean;
    };

/** Destination menu split as the mockup lists it. */
export interface MenuSections {
  frontmost: MenuEntry | null;
  others: MenuEntry[];
  frontmostAtEndCurrent: boolean;
}

const PHASE_RANK: Record<SessionPhase, number> = {
  recording: 0,
  processing: 1,
  inserting: 2,
  finished: 3,
  cancelled: 3,
};

/** Phases during which a badge choice is still accepted (before insertion begins). */
const CHANGEABLE_PHASES: ReadonlySet<SessionPhase> = new Set([
  "recording",
  "processing",
]);

/** Phases during which the badge is shown at all. */
const VISIBLE_PHASES: ReadonlySet<SessionPhase> = new Set([
  "recording",
  "processing",
  "inserting",
]);

// @spec AC-003: Badge icon, name or Frontmost app — .specs/features/006-destination-application/spec.md#ac-003
// @spec AC-001: Pinned badge, shown even with return off — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/**
 * Badge content for a snapshot (AC-003, AC-004). No session (rollout gate closed), the setting
 * off with nothing pinned (009 keeps the badge for a pin), an unsupported system or a finished
 * session hide the badge; the cross and the menu are offered only before insertion begins.
 */
export const badgeView = (snapshot: DestinationSnapshot | null): BadgeView => {
  if (!snapshot) return { kind: "hidden" };
  if (!snapshot.return_enabled && !snapshot.pin) return { kind: "hidden" };
  if (snapshot.support.state !== "supported") return { kind: "hidden" };
  if (!VISIBLE_PHASES.has(snapshot.phase)) return { kind: "hidden" };
  const canChange = CHANGEABLE_PHASES.has(snapshot.phase);
  const selection = snapshot.selection;
  if (selection.kind === "frontmost_at_end") {
    return { kind: "frontmost", canChange };
  }
  return {
    kind: "application",
    name: selection.display_name,
    iconRef: selection.icon_ref,
    canChange,
    pinned: snapshot.source === "pinned",
  };
};

const snapshotOrder = (snapshot: DestinationSnapshot): number[] => [
  snapshot.dictation_id,
  snapshot.destination_generation,
  snapshot.selection_version,
  PHASE_RANK[snapshot.phase],
];

/**
 * Keep the newest snapshot: an older dictation, generation, selection version or phase never
 * replaces a newer one (stale command answers and late events are ignored).
 */
export const newerSnapshot = (
  current: DestinationSnapshot | null,
  incoming: DestinationSnapshot,
): DestinationSnapshot => {
  if (!current) return incoming;
  const before = snapshotOrder(current);
  const after = snapshotOrder(incoming);
  for (let index = 0; index < before.length; index += 1) {
    if (after[index] !== before[index]) {
      return after[index] > before[index] ? incoming : current;
    }
  }
  return incoming;
};

/** Split the menu: application frontmost now first, then the other windowed applications. */
export const menuSections = (menu: DestinationMenu): MenuSections => ({
  frontmost: menu.entries.find((entry) => entry.frontmost) ?? null,
  others: menu.entries.filter((entry) => !entry.frontmost),
  frontmostAtEndCurrent: menu.frontmost_at_end_current,
});

/** True when `menu` still belongs to the session of `snapshot` at its current version. */
export const menuMatches = (
  menu: DestinationMenu,
  snapshot: DestinationSnapshot | null,
): boolean =>
  snapshot !== null &&
  menu.dictation_id === snapshot.dictation_id &&
  menu.selection_version === snapshot.selection_version;

/** Translation key explaining a refused badge choice (nothing changed). */
export const refusalKey = (refusal: SelectionRefusal): string =>
  `overlay.destination.refused.${refusal}`;

/**
 * The application a destination notice names: the selection of the snapshot of that dictation
 * (`null` when the snapshot belongs to another dictation or targets the frontmost app).
 */
export const selectionName = (
  snapshot: DestinationSnapshot | null,
  dictationId: number,
): string | null => {
  if (!snapshot || snapshot.dictation_id !== dictationId) return null;
  return snapshot.selection.kind === "application"
    ? snapshot.selection.display_name
    : null;
};

/** An icon reference the badge can render safely (an inline image), or `null` for a monogram. */
export const safeIconSource = (iconRef: string | null): string | null =>
  iconRef !== null && iconRef.startsWith("data:image/") ? iconRef : null;

/** First letter of an application name, for the monogram tile. */
export const monogram = (name: string): string =>
  Array.from(name.trim())[0]?.toUpperCase() ?? "?";
