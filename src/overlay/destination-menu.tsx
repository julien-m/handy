// Destination menu of the recording indicator (features 006, 009, 012). From top to bottom: a
// scrolling area with the application frontmost now and the other running applications; the
// favorites (012, star toggle on every row), newest last; then a fixed footer, nearest the badge,
// with "Frontmost app at the end" and 009's "Pin <destination>" and "Unpin <app>". The favorites
// and the footer never scroll, so they stay visible however many applications are open (decision
// of 2026-10-29). Pointer-operated: no press takes keyboard focus from the user's application.
import { AppWindow, Check, Pin, PinOff } from "lucide-react";
import type React from "react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { MenuChoice } from "@/bindings";
import { keepFocus } from "./destination-badge";
import { menuSections } from "./destination-model";
import {
  AppRow,
  MenuLabel,
  STAR_TOKEN_ATTRIBUTE,
  useMenuId,
} from "./destination-menu-rows";
import { pinTestIds, type PinEntry } from "./pin-state";
import type { DestinationMenuState } from "./use-destination";
import { useIndicatorScreen } from "./indicator-screen";

type OpenMenu = Extract<DestinationMenuState, { status: "open" }>["menu"];

interface DestinationMenuProps {
  menu: Exclude<DestinationMenuState, { status: "closed" }>;
  feedbackKey: string | null;
  onChoose: (choice: MenuChoice) => void;
  /** Pin entries of 009, listed in the fixed footer. */
  pinEntries?: PinEntry[];
  /** An application is pinned: the other applications are "This dictation only". */
  pinActive?: boolean;
  onPin?: () => void;
  onUnpin?: () => void;
  /** Star or unstar a menu entry (012). */
  onToggleFavorite?: (token: number) => void;
}

/** One pin entry of 009 (Pin or Unpin). */
const PinRow: React.FC<{ entry: PinEntry; onClick: () => void }> = ({
  entry,
  onClick,
}) => {
  const { t } = useTranslation();
  const screen = useIndicatorScreen();
  const Glyph = entry.kind === "pin" ? Pin : PinOff;
  return (
    <button
      type="button"
      role="menuitem"
      className="smenu-item"
      data-testid={pinTestIds(screen)[entry.kind]}
      onMouseDown={keepFocus}
      onClick={onClick}
    >
      <Glyph className="smenu-glyph" aria-hidden="true" />
      <span className="smenu-name">
        {t(entry.labelKey, { name: entry.name })}
      </span>
    </button>
  );
};

interface RowHandlers {
  pinActive: boolean;
  onChoose: (choice: MenuChoice) => void;
  onToggleFavorite: (token: number, keyboard: boolean) => void;
}

/** The scrolling area: application frontmost now, then the other running applications. */
const RunningApps: React.FC<{ menu: OpenMenu } & RowHandlers> = (props) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const { frontmost, others } = menuSections(props.menu);
  if (!frontmost && others.length === 0) return null;
  return (
    <div className="smenu-scroll" data-testid={`${menuId}-apps`}>
      {frontmost && (
        <>
          <MenuLabel
            testId={`${menuId}-now-label`}
            text={t("overlay.destination.menu.frontmostNow")}
          />
          <AppRow {...props} entry={frontmost} testId={`${menuId}-frontmost`} />
        </>
      )}
      {others.length > 0 && (
        <MenuLabel
          testId={`${menuId}-others-label`}
          text={t("overlay.destination.menu.otherApps")}
        />
      )}
      {others.map((entry) => (
        <AppRow
          {...props}
          key={entry.token}
          entry={entry}
          testId={`${menuId}-app`}
        />
      ))}
    </div>
  );
};

// @spec AC-003: Favorites fixed, newest nearest the badge — .specs/features/012-destination-favorites/spec.md#ac-003
/** The favorites (012), fixed above the footer, oldest first so the newest is nearest the badge. */
const Favorites: React.FC<{ menu: OpenMenu } & RowHandlers> = (props) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const { favorites } = menuSections(props.menu);
  if (favorites.length === 0) return null;
  return (
    <div className="smenu-favorites" data-testid={`${menuId}-favorites`}>
      <hr className="smenu-divider" />
      <MenuLabel
        testId={`${menuId}-favorites-label`}
        text={t("overlay.destination.menu.favorites")}
      />
      {favorites.map((entry) => (
        <AppRow
          {...props}
          key={entry.token}
          entry={entry}
          testId={`${menuId}-favorite`}
        />
      ))}
    </div>
  );
};

const FrontmostAtEnd: React.FC<{
  current: boolean;
  onChoose: (choice: MenuChoice) => void;
}> = ({ current, onChoose }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={current}
      className={`smenu-item ${current ? "current" : ""}`}
      data-testid={`${menuId}-frontmost-at-end`}
      onMouseDown={keepFocus}
      onClick={() => onChoose({ kind: "frontmost_at_end" })}
    >
      <AppWindow className="smenu-glyph" aria-hidden="true" />
      <span className="smenu-name">
        {t("overlay.destination.menu.frontmostAtEnd")}
      </span>
      {current && <Check className="smenu-check" aria-hidden="true" />}
    </button>
  );
};

// @spec AC-001: Pin and Unpin in the fixed footer — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/**
 * The fixed footer, nearest the badge: "Frontmost app at the end", then the 009 entries "Pin
 * <destination>" and "Unpin <app>" (amended on 2026-10-29: never pushed out of view).
 */
const Footer: React.FC<{
  menu: OpenMenu;
  onChoose: (choice: MenuChoice) => void;
  pinEntries: PinEntry[];
  onPin: () => void;
  onUnpin: () => void;
}> = ({ menu, onChoose, pinEntries, onPin, onUnpin }) => {
  const menuId = useMenuId();
  const pin = pinEntries.find((entry) => entry.kind === "pin");
  const unpin = pinEntries.find((entry) => entry.kind === "unpin");
  return (
    <div className="smenu-footer" data-testid={`${menuId}-footer`}>
      <hr className="smenu-divider" />
      <FrontmostAtEnd
        current={menuSections(menu).frontmostAtEndCurrent}
        onChoose={onChoose}
      />
      {(pin || unpin) && <hr className="smenu-divider" />}
      {pin && <PinRow entry={pin} onClick={onPin} />}
      {unpin && <PinRow entry={unpin} onClick={onUnpin} />}
    </div>
  );
};

/**
 * Keyboard focus back on the star of the same application once the re-ordered menu renders
 * (012 AC-005); a pointer toggle never moves focus.
 */
const useStarFocus = (menu: DestinationMenuProps["menu"]) => {
  const panel = useRef<HTMLDivElement>(null);
  const pending = useRef<number | null>(null);
  useEffect(() => {
    const token = pending.current;
    if (token === null || menu.status !== "open") return;
    pending.current = null;
    const selector = `[${STAR_TOKEN_ATTRIBUTE}="${token}"]`;
    panel.current?.querySelector<HTMLButtonElement>(selector)?.focus();
  }, [menu]);
  const remember = (token: number, keyboard: boolean) => {
    pending.current = keyboard ? token : null;
  };
  return { panel, remember };
};

// @spec FR-004: Star, favorites and fixed footer rendered — .specs/features/012-destination-favorites/spec.md#fr-004
/** The destination menu panel in its loading, error or open state. */
export const DestinationMenuPanel: React.FC<DestinationMenuProps> = ({
  menu,
  feedbackKey,
  onChoose,
  pinEntries = [],
  pinActive = false,
  onPin = () => undefined,
  onUnpin = () => undefined,
  onToggleFavorite = () => undefined,
}) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const { panel, remember } = useStarFocus(menu);
  const handlers: RowHandlers = {
    pinActive,
    onChoose,
    onToggleFavorite: (token, keyboard) => {
      remember(token, keyboard);
      onToggleFavorite(token);
    },
  };
  return (
    <div
      ref={panel}
      className="smenu"
      role="menu"
      aria-label={t("overlay.destination.menu.title")}
      data-testid={menuId}
    >
      {menu.status === "loading" && (
        <p className="smenu-status" data-testid={`${menuId}-loading`}>
          {t("overlay.destination.menu.loading")}
        </p>
      )}
      {menu.status === "error" && (
        <p className="smenu-status" data-testid={`${menuId}-failed`}>
          {t("overlay.destination.menu.failed")}
        </p>
      )}
      {menu.status === "open" && (
        <>
          <RunningApps menu={menu.menu} {...handlers} />
          <Favorites menu={menu.menu} {...handlers} />
          <Footer
            menu={menu.menu}
            onChoose={onChoose}
            pinEntries={pinEntries}
            onPin={onPin}
            onUnpin={onUnpin}
          />
        </>
      )}
      {feedbackKey && (
        <p
          className="smenu-feedback"
          role="status"
          data-testid={`${menuId}-feedback`}
        >
          {t(feedbackKey)}
        </p>
      )}
    </div>
  );
};
