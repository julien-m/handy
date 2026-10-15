// Destination menu of the recording indicator (feature 006): the application frontmost now,
// the other running applications with windows, then "Frontmost app at the end", the current
// choice checked. Pointer-operated: no item takes keyboard focus from the user's application.
import { AppWindow, Check, Pin, PinOff } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import type { MenuChoice, MenuEntry } from "@/bindings";
import { AppIcon, keepFocus } from "./destination-badge";
import { menuSections } from "./destination-model";
import type { PinEntry } from "./pin-state";
import type { DestinationMenuState } from "./use-destination";

const TEST_ID = "recording-indicator-destination-menu-menu";

interface DestinationMenuProps {
  menu: Exclude<DestinationMenuState, { status: "closed" }>;
  feedbackKey: string | null;
  onChoose: (choice: MenuChoice) => void;
  /** Pin entries of 009, listed above the application list. */
  pinEntries?: PinEntry[];
  onPin?: () => void;
  onUnpin?: () => void;
}

// @spec AC-001: Pin entries above the application list — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
/** "Pin <destination>" and "Unpin <app>" (009), above the application list, then a divider. */
const PinRows: React.FC<{
  entries: PinEntry[];
  onPin: () => void;
  onUnpin: () => void;
}> = ({ entries, onPin, onUnpin }) => {
  const { t } = useTranslation();
  if (entries.length === 0) return null;
  return (
    <>
      {entries.map((entry) => {
        const Glyph = entry.kind === "pin" ? Pin : PinOff;
        return (
          <button
            key={entry.kind}
            type="button"
            role="menuitem"
            className="smenu-item"
            data-testid={entry.testId}
            onMouseDown={keepFocus}
            onClick={entry.kind === "pin" ? onPin : onUnpin}
          >
            <Glyph className="smenu-glyph" aria-hidden="true" />
            <span className="smenu-name">
              {t(entry.labelKey, { name: entry.name })}
            </span>
          </button>
        );
      })}
      <hr className="smenu-divider" />
    </>
  );
};

const AppRow: React.FC<{
  entry: MenuEntry;
  testId: string;
  onChoose: (choice: MenuChoice) => void;
}> = ({ entry, testId, onChoose }) => (
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
  onChoose: (choice: MenuChoice) => void;
}> = ({ menu, onChoose }) => {
  const { t } = useTranslation();
  const sections = menuSections(menu);
  return (
    <>
      {sections.frontmost && (
        <>
          <MenuLabel
            testId={`${TEST_ID}-now-label`}
            text={t("overlay.destination.menu.frontmostNow")}
          />
          <AppRow
            entry={sections.frontmost}
            testId={`${TEST_ID}-frontmost`}
            onChoose={onChoose}
          />
        </>
      )}
      {sections.others.length > 0 && (
        <MenuLabel
          testId={`${TEST_ID}-others-label`}
          text={t("overlay.destination.menu.otherApps")}
        />
      )}
      {sections.others.map((entry) => (
        <AppRow
          key={entry.token}
          entry={entry}
          testId={`${TEST_ID}-app`}
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
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={current}
      className={`smenu-item ${current ? "current" : ""}`}
      data-testid={`${TEST_ID}-frontmost-at-end`}
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
  onPin = () => undefined,
  onUnpin = () => undefined,
}) => {
  const { t } = useTranslation();
  return (
    <div
      className="smenu"
      role="menu"
      aria-label={t("overlay.destination.menu.title")}
      data-testid={TEST_ID}
    >
      {menu.status === "loading" && (
        <p className="smenu-status">{t("overlay.destination.menu.loading")}</p>
      )}
      {menu.status === "error" && (
        <p className="smenu-status">{t("overlay.destination.menu.failed")}</p>
      )}
      {menu.status === "open" && (
        <>
          <PinRows entries={pinEntries} onPin={onPin} onUnpin={onUnpin} />
          <MenuEntries menu={menu.menu} onChoose={onChoose} />
        </>
      )}
      {feedbackKey && (
        <p className="smenu-feedback" role="status">
          {t(feedbackKey)}
        </p>
      )}
    </div>
  );
};
