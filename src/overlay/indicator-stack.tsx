// Layout of the recording indicator around the existing Handy pill (features 005, 006, 007):
// from the screen edge outwards, the indicator (with the destination badge), the notices, the
// prompt pill and the open menu. Only one menu is open at a time; the stack reports its size so
// the native window fits it. Without prompt, destination or notice it renders the pill alone.
import type React from "react";
import type { OSType } from "@/lib/utils/keyboard";
import { DestinationBadge } from "./destination-badge";
import { DestinationMenuPanel } from "./destination-menu";
import { pinEntries } from "./pin-state";
import { useDictationCount, useMenuTimer } from "./menu-timer";
import type { OverlayNotice } from "./notice-model";
import { OverlayNotices } from "./overlay-notices";
import { PromptPill } from "./prompt-pill";
import { pillVisible } from "./prompt-model";
import { useOsType } from "@/hooks/useOsType";
import { useDestination, type DestinationControls } from "./use-destination";
import {
  useDictationPrompt,
  type PromptControls,
} from "./use-dictation-prompt";
import { useOverlayExtent } from "./use-overlay-extent";
import { useOverlayNotices } from "./use-overlay-notices";
import {
  type IndicatorScreen,
  IndicatorScreenContext,
  indicatorScreen,
} from "./indicator-screen";
import "./indicator.css";

/** Grace period before the destination menu closes once the pointer left the overlay. */
const LEAVE_CLOSE_MS = 400;

interface IndicatorStackProps {
  position: "top" | "bottom";
  direction: "ltr" | "rtl";
  form: "compact" | "live";
  showIndicator: boolean;
  recording: boolean;
  /** Number of the dictation shown; a new one cancels pending menu timers. */
  dictation: number;
  destination: DestinationControls;
  prompt: PromptControls;
  notices: OverlayNotice[];
  osType: OSType;
  children: React.ReactNode;
}

interface IndicatorProps {
  form: IndicatorStackProps["form"];
  destination: DestinationControls;
  onToggleMenu: () => void;
  children: React.ReactNode;
}

/** The indicator pill, merged with the destination badge when there is one. */
const Indicator: React.FC<IndicatorProps> = (props) => {
  const { form, destination, children } = props;
  const { badge } = destination;
  if (badge.kind === "hidden") {
    return form === "compact" ? (
      <div className="ov-fade show">{children}</div>
    ) : (
      <>{children}</>
    );
  }
  const fade = form === "compact" ? "ov-fade show" : "";
  return (
    <div className={`sindicator with-dest ${form} ${fade}`}>
      <div className="sdest-segment">
        <DestinationBadge
          badge={badge}
          menuOpen={destination.menu.status !== "closed"}
          onToggleMenu={props.onToggleMenu}
          onRemove={destination.remove}
        />
      </div>
      {form === "compact" && <span className="sdivider" aria-hidden="true" />}
      {children}
    </div>
  );
};

/**
 * One menu open at a time, and the destination menu closing once the pointer has left the
 * overlay for a while; that delayed close is cancelled on unmount and at each new dictation.
 */
const useMenuCoordination = (
  destination: DestinationControls,
  prompt: PromptControls,
  dictation: number,
) => {
  const leaveTimer = useMenuTimer(dictation);
  const promptControls: PromptControls = {
    ...prompt,
    openMenu: () => {
      destination.closeMenu();
      prompt.openMenu();
    },
  };
  const toggleDestination = () => {
    prompt.closeMenu();
    destination.toggleMenu();
  };
  const closeAll = () => {
    destination.closeMenu();
    prompt.closeMenu();
  };
  const onLeave = () =>
    leaveTimer.schedule(destination.closeMenu, LEAVE_CLOSE_MS);
  const onEnter = leaveTimer.cancel;
  return { promptControls, toggleDestination, closeAll, onLeave, onEnter };
};

type Menus = ReturnType<typeof useMenuCoordination>;

interface StackPartProps {
  props: IndicatorStackProps;
  menus: Menus;
}

/** The open destination menu and the prompt pill (with its menu), above the indicator. */
const MenuItems: React.FC<StackPartProps> = ({ props, menus }) => {
  const { destination, prompt, showIndicator } = props;
  const { view } = prompt;
  const { menu } = destination;
  const showPrompt = showIndicator && pillVisible(view, props.recording);
  return (
    <>
      {showIndicator && menu.status !== "closed" && (
        <DestinationMenuPanel
          menu={menu}
          feedbackKey={destination.feedbackKey}
          onChoose={destination.choose}
          pinEntries={
            menu.status === "open"
              ? pinEntries(destination.snapshot, menu.menu)
              : []
          }
          pinActive={Boolean(destination.snapshot?.pin)}
          onPin={destination.pin}
          onUnpin={destination.unpin}
          onToggleFavorite={destination.toggleFavorite}
        />
      )}
      {showPrompt && view && (
        <PromptPill
          controls={menus.promptControls}
          view={view}
          osType={props.osType}
          hoverOpens={menu.status === "closed"}
          dictation={props.dictation}
        />
      )}
    </>
  );
};

/** The stack items, from the top of the screen down (reversed for a top placement). */
const StackItems: React.FC<StackPartProps> = ({ props, menus }) => (
  <>
    <MenuItems props={props} menus={menus} />
    <OverlayNotices notices={props.notices} />
    {props.showIndicator && (
      <Indicator
        form={props.form}
        destination={props.destination}
        onToggleMenu={menus.toggleDestination}
      >
        {props.children}
      </Indicator>
    )}
  </>
);

/**
 * Contract screen the stack shows (its test-id prefix).
 * @param props stack props
 * @returns the screen shown
 */
export const stackScreen = (
  props: Pick<
    IndicatorStackProps,
    "showIndicator" | "recording" | "destination" | "prompt" | "notices"
  >,
): IndicatorScreen =>
  indicatorScreen({
    showIndicator: props.showIndicator,
    recording: props.recording,
    promptView: props.prompt.view,
    promptMenuOpen: props.prompt.menuOpen,
    destinationMenuOpen: props.destination.menu.status !== "closed",
    pinned: Boolean(props.destination.snapshot?.pin),
    badge: props.destination.badge,
    notices: props.notices,
  });

/** The full indicator stack inside the overlay stage. */
export const IndicatorStack: React.FC<IndicatorStackProps> = (props) => {
  const extentRef = useOverlayExtent();
  const { destination, prompt, dictation } = props;
  const menus = useMenuCoordination(destination, prompt, dictation);
  const screen = stackScreen(props);
  return (
    <IndicatorScreenContext.Provider value={screen}>
      <div
        dir={props.direction}
        className={`ov-stage ${props.position}`}
        data-testid={`${screen}-screen`}
        onMouseDown={(event) =>
          event.target === event.currentTarget && menus.closeAll()
        }
        onMouseLeave={menus.onLeave}
        onMouseEnter={menus.onEnter}
      >
        <div ref={extentRef} className="ov-stack" data-testid={`${screen}-bar`}>
          <StackItems props={props} menus={menus} />
        </div>
      </div>
    </IndicatorScreenContext.Provider>
  );
};

/** Inputs of [`useIndicatorExtras`] read from the overlay state. */
interface ExtrasInput {
  isVisible: boolean;
  state: "recording" | "streaming" | "transcribing" | "processing";
  phase: string;
  position: "top" | "bottom";
  direction: "ltr" | "rtl";
}

/**
 * Prompt (005), destination (006) and notice (005, 007) state of the overlay, packed as
 * the props of [`IndicatorStack`] (all but `form` and `children`).
 */
export const useIndicatorExtras = (input: ExtrasInput) => {
  const { isVisible, state, phase } = input;
  const recording =
    isVisible &&
    (state === "recording" || (state === "streaming" && phase !== "working"));
  const destination = useDestination(isVisible);
  const prompt = useDictationPrompt(isVisible, recording);
  const notices = useOverlayNotices();
  const osType = useOsType();
  const dictation = useDictationCount();
  const stack = {
    position: input.position,
    direction: input.direction,
    showIndicator: isVisible,
    recording,
    dictation,
    destination,
    prompt,
    notices,
    osType,
  };
  return {
    stack,
    /** Contract screen shown, for the test ids of the Handy pill rendered by the caller. */
    screen: stackScreen(stack),
    withDestination: destination.badge.kind !== "hidden",
    hasNotices: notices.length > 0,
  };
};
