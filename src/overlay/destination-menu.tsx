// Destination menu of the recording indicator (feature 006): the application frontmost now,
// the other running applications with windows, then "Frontmost app at the end", the current
// choice checked. 009 adds "Pin <destination>" above the list, "Unpin <app>" at the bottom and,
// while an application is pinned, "This dictation only" on the other applications.
// Pointer-operated: no item takes keyboard focus from the user's application.
import { AppWindow, Check, Pin, PinOff } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import type { MenuChoice, MenuEntry } from "@/bindings";
import { AppIcon, keepFocus } from "./destination-badge";
import { menuSections } from "./destination-model";
import { pinTestIds, type PinEntry } from "./pin-state";
import type { DestinationMenuState } from "./use-destination";
import { useIndicatorScreen } from "./indicator-screen";

/**
 * Test id of the destination menu, under the screen shown (`<screen>-menu`).
 * @returns the menu's test id
 */
const useMenuId = (): string => `${useIndicatorScreen()}-menu`;

interface DestinationMenuProps {
  menu: Exclude<DestinationMenuState, { status: "closed" }>;
  feedbackKey: string | null;
  onChoose: (choice: MenuChoice) => void;
  /** Pin entries of 009: Pin above the application list, Unpin at the bottom. */
  pinEntries?: PinEntry[];
  /** An application is pinned: the other applications are "This dictation only". */
  pinActive?: boolean;
  onPin?: () => void;
  onUnpin?: () => void;
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

// @spec AC-001: Pin above the list, Unpin at the bottom — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/** The pin entry of `kind` with its divider: Pin above the list, Unpin below it. */
const PinSection: React.FC<{
  entries: PinEntry[];
  kind: PinEntry["kind"];
  onClick: () => void;
}> = ({ entries, kind, onClick }) => {
  const entry = entries.find((candidate) => candidate.kind === kind);
  if (!entry) return null;
  const divider = <hr className="smenu-divider" />;
  return (
    <>
      {kind === "unpin" && divider}
      <PinRow entry={entry} onClick={onClick} />
      {kind === "pin" && divider}
    </>
  );
};

/** "This dictation only" (009): this choice does not move the pin. */
const DictationOnlyHint: React.FC = () => {
  const { t } = useTranslation();
  const screen = useIndicatorScreen();
  return (
    <span className="smenu-hint" data-testid={pinTestIds(screen).hint}>
      {t("overlay.destination.menu.thisDictationOnly")}
    </span>
  );
};

const AppRow: React.FC<{
  entry: MenuEntry;
  testId: string;
  pinActive: boolean;
  onChoose: (choice: MenuChoice) => void;
}> = ({ entry, testId, pinActive, onChoose }) => (
  <button
    type="button"
    role="menuitemradio"
    aria-checked={entry.current}
    className={`smenu-item ${entry.current ? "current" : ""}`}
    data-testid={testId}
    data-instance-id={`app-${entry.token}`}
    onMouseDown={keepFocus}
    onClick={() => onChoose({ kind: "application", token: entry.token })}
  >
    <AppIcon name={entry.display_name} iconRef={entry.icon_ref} />
    <span className="smenu-name">{entry.display_name}</span>
    {pinActive && !entry.pinned && <DictationOnlyHint />}
    {entry.current && <Check className="smenu-check" aria-hidden="true" />}
  </button>
);

const MenuLabel: React.FC<{ testId: string; text: string }> = ({
  testId,
  text,
}) => (
  <p className="smenu-label" data-testid={testId}>
    {text}
  </p>
);

const MenuEntries: React.FC<{
  menu: Extract<DestinationMenuState, { status: "open" }>["menu"];
  pinActive: boolean;
  onChoose: (choice: MenuChoice) => void;
}> = ({ menu, pinActive, onChoose }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const sections = menuSections(menu);
  return (
    <>
      {sections.frontmost && (
        <>
          <MenuLabel
            testId={`${menuId}-now-label`}
            text={t("overlay.destination.menu.frontmostNow")}
          />
          <AppRow
            entry={sections.frontmost}
            testId={`${menuId}-frontmost`}
            pinActive={pinActive}
            onChoose={onChoose}
          />
        </>
      )}
      {sections.others.length > 0 && (
        <MenuLabel
          testId={`${menuId}-others-label`}
          text={t("overlay.destination.menu.otherApps")}
        />
      )}
      {sections.others.map((entry) => (
        <AppRow
          key={entry.token}
          entry={entry}
          testId={`${menuId}-app`}
          pinActive={pinActive}
          onChoose={onChoose}
        />
      ))}
      <hr className="smenu-divider" />
      <FrontmostAtEnd
        current={sections.frontmostAtEndCurrent}
        onChoose={onChoose}
      />
    </>
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

/** The destination menu panel in its loading, error or open state. */
export const DestinationMenuPanel: React.FC<DestinationMenuProps> = ({
  menu,
  feedbackKey,
  onChoose,
  pinEntries = [],
  pinActive = false,
  onPin = () => undefined,
  onUnpin = () => undefined,
}) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  return (
    <div
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
          <PinSection entries={pinEntries} kind="pin" onClick={onPin} />
          <MenuEntries
            menu={menu.menu}
            pinActive={pinActive}
            onChoose={onChoose}
          />
          <PinSection entries={pinEntries} kind="unpin" onClick={onUnpin} />
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
