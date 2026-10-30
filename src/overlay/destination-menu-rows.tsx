// Rows of the destination menu (features 006, 009, 012): an application row (icon, name, the
// "Not open" hint of a favorite that is not running, the current check) followed by its compact
// favorite star, a sibling toggle button, so a star click never chooses the row. The name takes
// the room left and is truncated only when it cannot fit (decision of 2026-10-29: no "This
// dictation only" hint). Pointer presses never take keyboard focus from the user's application.
import { Check, Star } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import type { MenuChoice, MenuEntry } from "@/bindings";
import { AppIcon, keepFocus } from "./destination-badge";
import { favoriteLabelKey } from "./destination-model";
import { useIndicatorScreen } from "./indicator-screen";

/**
 * Test id of the destination menu, under the screen shown (`<screen>-menu`).
 * @returns the menu's test id
 */
export const useMenuId = (): string => `${useIndicatorScreen()}-menu`;

/** Data attribute naming the token of a star, used to give it focus back after a toggle. */
export const STAR_TOKEN_ATTRIBUTE = "data-favorite-token";

/** "Not open" (012): a favorite with no running instance, never launched. */
const NotOpenHint: React.FC = () => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  return (
    <span className="smenu-hint" data-testid={`${menuId}-not-open`}>
      {t("overlay.destination.menu.notOpen")}
    </span>
  );
};

// @spec AC-005: Star is a named toggle button — .specs/features/012-destination-favorites/spec.md#ac-005
/** The star of an application row: adds it to the favorites or removes it (012). */
const FavoriteStar: React.FC<{
  entry: MenuEntry;
  onToggle: (token: number, keyboard: boolean) => void;
}> = ({ entry, onToggle }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  return (
    <button
      type="button"
      className={`smenu-star ${entry.favorite ? "on" : ""}`}
      data-testid={`${menuId}-favorite-toggle`}
      data-instance-id={`star-${entry.token}`}
      data-favorite-token={entry.token}
      aria-pressed={entry.favorite}
      aria-label={t(favoriteLabelKey(entry), { name: entry.display_name })}
      onMouseDown={keepFocus}
      onClick={(event) => {
        // A star never chooses its row nor closes the menu (012 AC-001).
        event.stopPropagation();
        onToggle(entry.token, document.activeElement === event.currentTarget);
      }}
    >
      <Star className="smenu-star-glyph" aria-hidden="true" />
    </button>
  );
};

export interface AppRowProps {
  entry: MenuEntry;
  testId: string;
  onChoose: (choice: MenuChoice) => void;
  onToggleFavorite: (token: number, keyboard: boolean) => void;
}

// @spec AC-004: Favorite not open is shown dimmed, not choosable — .specs/features/012-destination-favorites/spec.md#ac-004
// @spec AC-001: Compact row, full app name readable — .specs/features/012-destination-favorites/spec.md#ac-005
/** One application row and its star; the name is never pushed aside by a hint (2026-10-29). */
export const AppRow: React.FC<AppRowProps> = (props) => {
  const { entry, testId, onChoose } = props;
  return (
    <div className="smenu-row" role="none">
      <button
        type="button"
        role="menuitemradio"
        aria-checked={entry.current}
        className={`smenu-item ${entry.current ? "current" : ""}`}
        data-testid={testId}
        data-instance-id={`app-${entry.token}`}
        disabled={!entry.running}
        onMouseDown={keepFocus}
        onClick={() => onChoose({ kind: "application", token: entry.token })}
      >
        <AppIcon name={entry.display_name} iconRef={entry.icon_ref} />
        <span className="smenu-name">{entry.display_name}</span>
        {!entry.running && <NotOpenHint />}
        {entry.current && <Check className="smenu-check" aria-hidden="true" />}
      </button>
      <FavoriteStar entry={entry} onToggle={props.onToggleFavorite} />
    </div>
  );
};

/** A section label of the menu. */
export const MenuLabel: React.FC<{ testId: string; text: string }> = ({
  testId,
  text,
}) => (
  <p className="smenu-label" data-testid={testId}>
    {text}
  </p>
);
