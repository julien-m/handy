// Layout of the recording indicator around the existing Handy pill (features 005, 006, 007):
// from the screen edge outwards, the indicator (with the destination badge), the notices, the
// prompt pill and the open menu. Only one menu is open at a time; the stack reports its size so
// the native window fits it. Without prompt, destination or notice it renders the pill alone.
import type React from "react";
import { useRef } from "react";
import type { OSType } from "@/lib/utils/keyboard";
import { DestinationBadge } from "./destination-badge";
import { DestinationMenuPanel } from "./destination-menu";
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
import "./indicator.css";

/** Grace period before the destination menu closes once the pointer left the overlay. */
const LEAVE_CLOSE_MS = 400;

interface IndicatorStackProps {
  position: "top" | "bottom";
  direction: "ltr" | "rtl";
  form: "compact" | "live";
  showIndicator: boolean;
  recording: boolean;
  destination: DestinationControls;
  prompt: PromptControls;
  notices: OverlayNotice[];
  osType: OSType;
  children: React.ReactNode;
}

/** The indicator pill, merged with the destination badge when there is one. */
const Indicator: React.FC<{
  form: IndicatorStackProps["form"];
  destination: DestinationControls;
  onToggleMenu: () => void;
  children: React.ReactNode;
}> = ({ form, destination, onToggleMenu, children }) => {
  const { badge } = destination;
  if (badge.kind === "hidden") {
    return form === "compact" ? (
      <div className="ov-fade show">{children}</div>
    ) : (
      <>{children}</>
    );
  }
  return (
    <div
      className={`sindicator with-dest ${form} ${form === "compact" ? "ov-fade show" : ""}`}
    >
      <div className="sdest-segment">
        <DestinationBadge
          badge={badge}
          menuOpen={destination.menu.status !== "closed"}
          onToggleMenu={onToggleMenu}
          onRemove={destination.remove}
        />
      </div>
      {form === "compact" && <span className="sdivider" aria-hidden="true" />}
      {children}
    </div>
  );
};

const useMenuCoordination = (
  destination: DestinationControls,
  prompt: PromptControls,
) => {
  const leaveTimer = useRef<ReturnType<typeof setTimeout>>();
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
  const onLeave = () => {
    clearTimeout(leaveTimer.current);
    leaveTimer.current = setTimeout(destination.closeMenu, LEAVE_CLOSE_MS);
  };
  const onEnter = () => clearTimeout(leaveTimer.current);
  return { promptControls, toggleDestination, closeAll, onLeave, onEnter };
};

type Menus = ReturnType<typeof useMenuCoordination>;

/** The stack items, from the top of the screen down (reversed for a top placement). */
const StackItems: React.FC<{ props: IndicatorStackProps; menus: Menus }> = ({
  props,
  menus,
}) => {
  const { destination, prompt, showIndicator } = props;
  const view = prompt.view;
  const destinationMenu = destination.menu;
  const showPrompt = showIndicator && pillVisible(view, props.recording);
  return (
    <>
      {showIndicator && destinationMenu.status !== "closed" && (
        <DestinationMenuPanel
          menu={destinationMenu}
          feedbackKey={destination.feedbackKey}
          onChoose={destination.choose}
        />
      )}
      {showPrompt && view && (
        <PromptPill
          controls={menus.promptControls}
          view={view}
          osType={props.osType}
          hoverOpens={destinationMenu.status === "closed"}
        />
      )}
      <OverlayNotices notices={props.notices} />
      {showIndicator && (
        <Indicator
          form={props.form}
          destination={destination}
          onToggleMenu={menus.toggleDestination}
        >
          {props.children}
        </Indicator>
      )}
    </>
  );
};

/** The full indicator stack inside the overlay stage. */
export const IndicatorStack: React.FC<IndicatorStackProps> = (props) => {
  const extentRef = useOverlayExtent();
  const menus = useMenuCoordination(props.destination, props.prompt);
  return (
    <div
      dir={props.direction}
      className={`ov-stage ${props.position}`}
      data-testid="recording-indicator-screen"
      onMouseDown={(event) =>
        event.target === event.currentTarget && menus.closeAll()
      }
      onMouseLeave={menus.onLeave}
      onMouseEnter={menus.onEnter}
    >
      <div
        ref={extentRef}
        className="ov-stack"
        data-testid="recording-indicator-bar"
      >
        <StackItems props={props} menus={menus} />
      </div>
    </div>
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
 * Prompt (005), destination (006) and notice (005, 006, 007) state of the overlay, packed as
 * the props of [`IndicatorStack`] (all but `form` and `children`).
 */
export const useIndicatorExtras = (input: ExtrasInput) => {
  const { isVisible, state, phase } = input;
  const recording =
    isVisible &&
    (state === "recording" || (state === "streaming" && phase !== "working"));
  const destination = useDestination(isVisible);
  const prompt = useDictationPrompt(isVisible, recording);
  const notices = useOverlayNotices(destination.latest);
  const osType = useOsType();
  const stack = {
    position: input.position,
    direction: input.direction,
    showIndicator: isVisible,
    recording,
    destination,
    prompt,
    notices,
    osType,
  };
  return {
    stack,
    withDestination: destination.badge.kind !== "hidden",
    hasNotices: notices.length > 0,
  };
};
