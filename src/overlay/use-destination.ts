// Destination session of the recording indicator (feature 006): follows `destination-changed`,
// opens the pointer-operated menu and applies badge choices through the delivered commands
// (`destination_menu`, `choose_destination`, `remove_destination`). Without a session (rollout
// gate `destination-return` closed) the snapshot stays `null` and nothing is shown.
import { listen } from "@tauri-apps/api/event";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type MutableRefObject,
} from "react";
import {
  commands,
  type ChoiceAnswer,
  type DestinationMenu,
  type DestinationSnapshot,
  type MenuChoice,
} from "@/bindings";
import {
  badgeView,
  menuMatches,
  newerSnapshot,
  refusalKey,
  type BadgeView,
} from "./destination-model";

/** Backend event carrying a session snapshot (`destination::app::DESTINATION_CHANGED`). */
export const DESTINATION_CHANGED_EVENT = "destination-changed";

/** State of the destination menu. */
export type DestinationMenuState =
  | { status: "closed" }
  | { status: "loading" }
  | { status: "open"; menu: DestinationMenu }
  | { status: "error" };

/** What the indicator needs from the destination session. */
export interface DestinationControls {
  snapshot: DestinationSnapshot | null;
  /** Newest accepted snapshot, updated before React renders (read by event listeners). */
  latest: MutableRefObject<DestinationSnapshot | null>;
  badge: BadgeView;
  menu: DestinationMenuState;
  feedbackKey: string | null;
  toggleMenu: () => void;
  closeMenu: () => void;
  choose: (choice: MenuChoice) => void;
  remove: () => void;
}

/** State setters shared by the menu loader and the choice settlement. */
interface MenuPorts {
  accept: (incoming: DestinationSnapshot | null) => void;
  setMenu: (menu: DestinationMenuState) => void;
  setKey: (key: string | null) => void;
}

const CLOSED: DestinationMenuState = { status: "closed" };

const useSnapshot = (shown: boolean) => {
  const [snapshot, setSnapshot] = useState<DestinationSnapshot | null>(null);
  const latest = useRef<DestinationSnapshot | null>(null);
  const accept = useCallback((incoming: DestinationSnapshot | null) => {
    if (!incoming) return;
    latest.current = newerSnapshot(latest.current, incoming);
    setSnapshot(latest.current);
  }, []);
  useEffect(() => {
    const unlisten = listen<DestinationSnapshot>(
      DESTINATION_CHANGED_EVENT,
      (event) => accept(event.payload),
    );
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [accept]);
  useEffect(() => {
    if (!shown) return;
    commands.destinationSnapshot().then(accept, (error: unknown) => {
      // The `destination-changed` event still updates the badge.
      console.warn("Could not read the destination snapshot", error);
    });
  }, [shown, accept]);
  return { snapshot, latest, accept };
};

/** Re-read the session after a refusal; the newest known snapshot wins. */
const refreshed = async (
  known: DestinationSnapshot,
  ports: MenuPorts,
): Promise<DestinationSnapshot> => {
  const fresh = (await commands.destinationSnapshot()) ?? known;
  ports.accept(fresh);
  return newerSnapshot(known, fresh);
};

/**
 * List the menu of `current`. A stale or unavailable refusal re-reads the session and lists it
 * once more; a frozen session (insertion begun) closes the menu.
 */
const loadMenuOf = async (
  current: DestinationSnapshot,
  ports: MenuPorts,
  retry = true,
): Promise<void> => {
  ports.setMenu({ status: "loading" });
  try {
    const { dictation_id: id, selection_version: version } = current;
    const answer = await commands.destinationMenu(id, version);
    if (answer.status === "menu") {
      return ports.setMenu({ status: "open", menu: answer.value });
    }
    if (answer.value !== "frozen" && retry) {
      return await loadMenuOf(await refreshed(current, ports), ports, false);
    }
    ports.setKey(refusalKey(answer.value));
    ports.setMenu(answer.value === "frozen" ? CLOSED : { status: "error" });
  } catch (error) {
    console.warn("Destination menu failed", error);
    ports.setMenu({ status: "error" });
  }
};

/**
 * Apply a choice answer: show the acknowledged snapshot, or explain the refusal and refresh
 * (the menu is listed again only for a menu choice, never for the cross).
 */
const settleChoice = async (
  known: DestinationSnapshot,
  pending: Promise<ChoiceAnswer>,
  ports: MenuPorts,
  reopenMenu: boolean,
): Promise<void> => {
  try {
    const answer = await pending;
    if (answer.status === "chosen") {
      ports.accept(answer.value);
      ports.setKey(null);
      return ports.setMenu(CLOSED);
    }
    ports.setKey(refusalKey(answer.value));
    if (answer.value === "frozen") return ports.setMenu(CLOSED);
    const fresh = await refreshed(known, ports);
    if (reopenMenu) await loadMenuOf(fresh, ports, false);
  } catch (error) {
    console.warn("Destination choice failed", error);
    ports.setKey("overlay.destination.choiceFailed");
  }
};

const useChoices = (
  snapshot: DestinationSnapshot | null,
  canChange: boolean,
  ports: MenuPorts,
) => {
  const choose = useCallback(
    (choice: MenuChoice) => {
      if (!snapshot || !canChange) return;
      const { dictation_id: id, selection_version: version } = snapshot;
      const pending = commands.chooseDestination(id, version, choice);
      void settleChoice(snapshot, pending, ports, true);
    },
    [snapshot, canChange, ports],
  );
  const remove = useCallback(() => {
    if (!snapshot || !canChange) return;
    const { dictation_id: id, selection_version: version } = snapshot;
    const pending = commands.removeDestination(id, version);
    void settleChoice(snapshot, pending, ports, false);
  }, [snapshot, canChange, ports]);
  return { choose, remove };
};

/**
 * Controls of the destination menu: closed whenever choices end (insertion begun, overlay
 * hidden) or when its version is stale. The toggle reads the rendered state, so the first click
 * after a stale menu opens a fresh one instead of closing the hidden one.
 */
const useMenuControls = (
  menu: DestinationMenuState,
  snapshot: DestinationSnapshot | null,
  canChange: boolean,
  shown: boolean,
  ports: MenuPorts,
) => {
  const { setMenu, setKey } = ports;
  useEffect(() => {
    if (!canChange || !shown) setMenu(CLOSED);
  }, [canChange, shown, setMenu]);
  const stale = menu.status === "open" && !menuMatches(menu.menu, snapshot);
  const rendered = stale ? CLOSED : menu;
  const toggleMenu = useCallback(() => {
    setKey(null);
    if (rendered.status !== "closed") return setMenu(CLOSED);
    if (snapshot && canChange) void loadMenuOf(snapshot, ports);
  }, [rendered.status, snapshot, canChange, ports, setKey, setMenu]);
  const closeMenu = useCallback(() => setMenu(CLOSED), [setMenu]);
  return { menu: rendered, toggleMenu, closeMenu };
};

// @spec AC-004: Cross, pointer menu, versioned choices — .specs/features/006-destination-application/spec.md#ac-004
/**
 * Destination badge state and actions for the indicator; `shown` is true while a dictation
 * shows the overlay (the snapshot is re-read then).
 */
export const useDestination = (shown: boolean): DestinationControls => {
  const { snapshot, latest, accept } = useSnapshot(shown);
  const [menu, setMenu] = useState<DestinationMenuState>(CLOSED);
  const [feedbackKey, setKey] = useState<string | null>(null);
  const [ports] = useState<MenuPorts>(() => ({ accept, setMenu, setKey }));
  const badge = badgeView(snapshot);
  const canChange = badge.kind !== "hidden" && badge.canChange;
  const { choose, remove } = useChoices(snapshot, canChange, ports);
  const controls = useMenuControls(menu, snapshot, canChange, shown, ports);
  return { snapshot, latest, badge, feedbackKey, ...controls, choose, remove };
};
