// Browser harness of the destination menu (006, 009, 012): the real `DestinationMenuPanel` with
// the overlay styles, at the bottom of the page like the indicator at the bottom of the screen,
// for `tests/destination-favorites.spec.ts`. The star toggle is answered here as the backend
// would (the row moves into or out of the favorites, tokens unchanged); choices are recorded.
// `?apps=<n>` sets the number of running applications. A headless check of the real React
// code and CSS, not a native window proof.
import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/overlay/RecordingOverlay.css";
import type {
  DestinationMenu,
  DestinationSnapshot,
  MenuChoice,
  MenuEntry,
} from "@/bindings";
import { DestinationMenuPanel } from "@/overlay/destination-menu";
import { IndicatorScreenContext } from "@/overlay/indicator-screen";
import { pinEntries } from "@/overlay/pin-state";
import "@/overlay/indicator.css";

const params = new URLSearchParams(window.location.search);
const runningCount = Number(params.get("apps") ?? "20");

/** Running applications (the first is frontmost), then a favorite that is not running. */
const rows: MenuEntry[] = [
  ...Array.from({ length: runningCount }, (_, index) => ({
    token: index + 1,
    display_name: index === 0 ? "Comet" : `Sample app ${index + 1}`,
    icon_ref: null,
    current: index === 1,
    frontmost: index === 0,
    pinned: index === 0,
    favorite: false,
    running: true,
  })),
  {
    token: runningCount + 1,
    display_name: "Notes",
    icon_ref: null,
    current: false,
    frontmost: false,
    pinned: false,
    favorite: true,
    running: false,
  },
];

/** Stored favorites (oldest first): two running apps and Notes, which is not running. */
const initialFavorites = [3, runningCount + 1, 5];

const snapshot: DestinationSnapshot = {
  dictation_id: 7,
  destination_generation: 1,
  selection_version: 1,
  phase: "recording",
  return_enabled: true,
  support: { state: "supported" },
  selection: {
    kind: "application",
    display_name: "Sample app 2",
    icon_ref: null,
  },
  source: "badge",
  pin: { display_name: "Comet", icon_ref: null },
  disposition: "pending",
};

/** The menu the backend answers for `favorites`: running non-favorites, then the favorites. */
const menuOf = (favorites: number[]): DestinationMenu => {
  const byToken = new Map(rows.map((row) => [row.token, row]));
  const others = rows.filter(
    (row) => row.running && !favorites.includes(row.token),
  );
  const starred = favorites.flatMap((token) => {
    const row = byToken.get(token);
    return row ? [{ ...row, favorite: true }] : [];
  });
  return {
    dictation_id: snapshot.dictation_id,
    selection_version: snapshot.selection_version,
    frontmost_at_end_current: false,
    entries: [
      ...others.map((row) => ({ ...row, favorite: false })),
      ...starred,
    ],
  };
};

const record = { toggles: [] as number[], choices: [] as MenuChoice[] };

const Harness = () => {
  const [favorites, setFavorites] = useState(initialFavorites);
  const menu = menuOf(favorites);
  const toggle = (token: number) => {
    record.toggles.push(token);
    setFavorites((current) =>
      current.includes(token)
        ? current.filter((known) => known !== token)
        : [...current, token],
    );
  };
  return (
    <div className="ov-stage bottom" data-testid="harness-stage">
      <div className="ov-stack">
        <button type="button" data-testid="harness-start">
          start
        </button>
        <IndicatorScreenContext.Provider value="recording-indicator-pinned">
          <DestinationMenuPanel
            menu={{ status: "open", menu }}
            feedbackKey={null}
            onChoose={(choice) => record.choices.push(choice)}
            pinEntries={pinEntries(snapshot, menu)}
            pinActive
            onToggleFavorite={toggle}
          />
        </IndicatorScreenContext.Provider>
        <div className="sindicator" data-testid="harness-badge">
          badge
        </div>
      </div>
    </div>
  );
};

const element = document.getElementById("root");
if (element) {
  createRoot(element).render(
    <StrictMode>
      <Harness />
    </StrictMode>,
  );
}

(window as unknown as Record<string, unknown>).harness = record;
