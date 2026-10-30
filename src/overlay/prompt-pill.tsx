// Prompt pill of the recording indicator (feature 005): the active prompt with its shortcut
// badge, above the indicator. Hovering it unfolds the prompt list (a click also toggles it, as a
// pointer fallback); the choice applies to this dictation only. Nothing takes keyboard focus.
import { Ban, Check } from "lucide-react";
import type React from "react";
import { useRef } from "react";
import { useMenuTimer } from "./menu-timer";
import { useTranslation } from "react-i18next";
import type { DictationPromptView, PromptLibraryView } from "@/bindings";
import type { OSType } from "@/lib/utils/keyboard";
import { keepFocus } from "./destination-badge";
import {
  defaultPromptName,
  findPrompt,
  promptMenuRows,
  promptShortcut,
  type PromptMenuRow,
} from "./prompt-model";
import type { PromptControls } from "./use-dictation-prompt";
import { useIndicatorScreen } from "./indicator-screen";

/**
 * Test id of the prompt menu, under the screen shown (`<screen>-menu`).
 * @returns the menu's test id
 */
const useMenuId = (): string => `${useIndicatorScreen()}-menu`;
/** Grace period before a hover menu closes, so the pointer can travel to it. */
const HOVER_CLOSE_MS = 250;

const ShortcutBadge: React.FC<{ text: string; testId: string }> = ({
  text,
  testId,
}) => (
  <span className="skbd" data-testid={testId}>
    {text}
  </span>
);

const PromptRow: React.FC<{
  row: PromptMenuRow;
  onSelect: (promptId: string | null) => void;
}> = ({ row, onSelect }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const isNone = row.id === null;
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={row.current}
      disabled={!row.usable}
      className={`smenu-item ${row.current ? "current" : ""}`}
      data-testid={isNone ? `${menuId}-none` : `${menuId}-prompt`}
      data-instance-id={isNone ? undefined : `prompt-${row.id}`}
      onMouseDown={keepFocus}
      onClick={() => onSelect(row.id)}
    >
      <span className="smenu-slot">
        {isNone && <Ban className="smenu-glyph" aria-hidden="true" />}
        {row.shortcut && (
          <ShortcutBadge
            text={row.shortcut}
            testId={`${menuId}-prompt-shortcut`}
          />
        )}
      </span>
      <span className="smenu-name">{row.name ?? t("overlay.prompt.none")}</span>
      {row.current && <Check className="smenu-check" aria-hidden="true" />}
    </button>
  );
};

const PromptRows: React.FC<{
  controls: PromptControls;
  library: PromptLibraryView;
  view: DictationPromptView;
  osType: OSType;
}> = ({ controls, library, view, osType }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const defaultName = defaultPromptName(library) ?? t("overlay.prompt.none");
  return (
    <>
      <div className="smenu-scroll">
        {promptMenuRows(library, view, osType).map((row) => (
          <PromptRow
            key={row.id ?? "none"}
            row={row}
            onSelect={controls.select}
          />
        ))}
      </div>
      {/* @spec AC-009: a plain dictation's choice is kept — .specs/features/005-custom-ai-prompts/spec.md#ac-009 */}
      <p className="smenu-note" data-testid={`${menuId}-note`}>
        {view.remembered
          ? t("overlay.prompt.noteRemembered")
          : t("overlay.prompt.note", { name: defaultName })}
      </p>
    </>
  );
};

const PromptMenu: React.FC<{
  controls: PromptControls;
  view: DictationPromptView;
  osType: OSType;
}> = ({ controls, view, osType }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const library = controls.library;
  return (
    <div
      className="smenu"
      role="menu"
      aria-label={t("overlay.prompt.menuTitle")}
      data-testid={menuId}
    >
      <p className="smenu-label" data-testid={`${menuId}-label`}>
        {t("overlay.prompt.menuTitle")}
      </p>
      {library.status === "ready" ? (
        <PromptRows
          controls={controls}
          library={library.library}
          view={view}
          osType={osType}
        />
      ) : (
        <p className="smenu-status">
          {t(
            library.status === "loading"
              ? "overlay.prompt.loading"
              : "overlay.prompt.failed",
          )}
        </p>
      )}
      {controls.feedbackKey && (
        <p className="smenu-feedback" role="status">
          {t(controls.feedbackKey)}
        </p>
      )}
    </div>
  );
};

/**
 * Hover opens the menu and leaving closes it after a grace period. A click opens it too (the
 * pointer fallback where hover events do not reach a non-key panel) and closes only a menu that
 * a click opened, so a click on a hover-opened menu never folds it under the pointer. The
 * pending close is cancelled on unmount and when a new dictation starts.
 */
const useHoverMenu = (
  controls: PromptControls,
  hoverOpens: boolean,
  dictation: number,
) => {
  const timer = useMenuTimer(dictation);
  const openedByClick = useRef(false);
  const enter = () => {
    timer.cancel();
    // While the destination menu is open the pointer crosses the pill to reach it.
    if (controls.menuOpen || !hoverOpens) return;
    openedByClick.current = false;
    controls.openMenu();
  };
  const leave = () => timer.schedule(controls.closeMenu, HOVER_CLOSE_MS);
  const click = () => {
    if (controls.menuOpen && openedByClick.current) return controls.closeMenu();
    timer.cancel();
    openedByClick.current = !controls.menuOpen;
    if (!controls.menuOpen) controls.openMenu();
  };
  return { enter, leave, click };
};

const PillLabel: React.FC<{ name: string; hint: boolean }> = ({
  name,
  hint,
}) => {
  const { t } = useTranslation();
  const screen = useIndicatorScreen();
  return (
    <span className="sprompt-text">
      <span className="sprompt-name">{name}</span>
      {hint && (
        <span className="sprompt-hint" data-testid={`${screen}-prompt-hint`}>
          {t("overlay.prompt.hint")}
        </span>
      )}
    </span>
  );
};

interface PromptPillProps {
  controls: PromptControls;
  view: DictationPromptView;
  osType: OSType;
  hoverOpens: boolean;
  /** Number of the dictation shown; a new one cancels a pending close. */
  dictation: number;
}

/** Shortcut of the dictation's prompt, once the library is read. */
const pillShortcut = (
  controls: PromptControls,
  view: DictationPromptView,
  osType: OSType,
): string | null => {
  const { library } = controls;
  const prompts = library.status === "ready" ? library.library : null;
  return promptShortcut(findPrompt(prompts, view.prompt_id), osType);
};

/** The pill itself: the prompt's shortcut badge, its name and the hover hint. */
const PillButton: React.FC<PromptPillProps & { onClick: () => void }> = (
  props,
) => {
  const { t } = useTranslation();
  const shortcut = pillShortcut(props.controls, props.view, props.osType);
  const name = props.view.prompt_name ?? t("overlay.prompt.none");
  const open = props.controls.menuOpen;
  const screen = useIndicatorScreen();
  return (
    <button
      type="button"
      className={`sprompt ${open ? "open" : ""}`}
      data-testid={`${screen}-prompt`}
      aria-label={t("overlay.prompt.label", { name })}
      aria-haspopup="menu"
      aria-expanded={open}
      onMouseDown={keepFocus}
      onClick={props.onClick}
    >
      {shortcut && (
        <ShortcutBadge text={shortcut} testId={`${screen}-prompt-shortcut`} />
      )}
      <PillLabel name={name} hint={!open} />
    </button>
  );
};

/** The prompt pill and, while hovered, its menu (unfolding above it at the bottom of the screen). */
export const PromptPill: React.FC<PromptPillProps> = (props) => {
  const { controls, view, osType } = props;
  const hover = useHoverMenu(controls, props.hoverOpens, props.dictation);
  return (
    <div
      className="sprompt-zone"
      onMouseEnter={hover.enter}
      onMouseLeave={hover.leave}
    >
      {controls.menuOpen && (
        <PromptMenu controls={controls} view={view} osType={osType} />
      )}
      <PillButton {...props} onClick={hover.click} />
    </div>
  );
};
