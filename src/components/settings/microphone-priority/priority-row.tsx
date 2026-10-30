// One configured microphone of the priority list (feature 001): drag handle, rank, exact name,
// availability pill (text, not only color) and the Move up / Move down / Remove controls.
import React from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, GripVertical, X } from "lucide-react";
import type { Availability } from "@/bindings";
import Badge from "../../ui/Badge";
import { AVAILABILITY_TONE } from "./availability";
import { PRIORITY_KEY_ROOT } from "./draft-state";
import { rowFocusKey } from "./editor-focus";
import {
  microphoneInstanceId,
  useMicrophoneScreen,
  type MicrophoneScreen,
} from "./screen-id";
import type { PointerReorder } from "./use-pointer-reorder";

/** Row control that keeps focus after a keyboard move. */
export type RowControl = "up" | "down";

/** Props of one priority row. */
export interface PriorityRowProps {
  name: string;
  index: number;
  count: number;
  availability: Availability;
  /** A write is in flight: every control is inert. */
  busy: boolean;
  dragging: boolean;
  dropTarget: boolean;
  rowRef: (element: HTMLElement | null) => void;
  handle: ReturnType<PointerReorder["handleProps"]>;
  onMove: (from: number, to: number, control: RowControl) => void;
  onRemove: (index: number) => void;
}

/** Props of one icon control of a row. */
interface RowButtonProps {
  label: string;
  testId: string;
  inert: boolean;
  focusKey?: string;
  onActivate: () => void;
  children: React.ReactNode;
}

// Edge controls stay focusable (aria-disabled, not disabled) so focus is kept after a move.
const RowButton: React.FC<RowButtonProps> = (props) => (
  <button
    type="button"
    className={`p-1 rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary ${props.inert ? "opacity-50 cursor-not-allowed" : "hover:bg-logo-primary/30 cursor-pointer"}`}
    data-testid={props.testId}
    data-focus-key={props.focusKey}
    aria-label={props.label}
    aria-disabled={props.inert}
    title={props.label}
    onClick={() => {
      if (!props.inert) props.onActivate();
    }}
  >
    {props.children}
  </button>
);

/**
 * Visual state of a row for the drag and drop contract.
 * @param props row props
 * @returns the classes and `data-state` of the row
 */
const rowState = (props: PriorityRowProps) => {
  if (props.dragging) return { state: "dragging", classes: "opacity-50" };
  if (props.dropTarget) {
    return { state: "drop-target", classes: "bg-logo-primary/10" };
  }
  return { state: undefined, classes: "" };
};

type Translate = ReturnType<typeof useTranslation>["t"];

/**
 * Move up or Move down of one row; the first row cannot move up, the last cannot move down.
 * @param props row props
 * @param t translation function
 * @param control direction of the move
 * @param screen test-id prefix of the screen shown
 * @returns the control's props
 */
const moveSpec = (
  props: PriorityRowProps,
  t: Translate,
  control: RowControl,
  screen: MicrophoneScreen,
): RowButtonProps => {
  const { name, index, count, busy } = props;
  const up = control === "up";
  const to = up ? index - 1 : index + 1;
  return {
    label: t(`${PRIORITY_KEY_ROOT}.${up ? "moveUp" : "moveDown"}`, { name }),
    testId: `${screen}-choice-${up ? "move-up" : "move-down"}`,
    focusKey: rowFocusKey(name, control),
    inert: busy || (up ? index === 0 : index === count - 1),
    onActivate: () => props.onMove(index, to, control),
    children: up ? <ArrowUp size={14} /> : <ArrowDown size={14} />,
  };
};

/**
 * Move up, Move down and Remove of one row; nothing acts while a write is in flight.
 * @param props row props
 * @param t translation function
 * @param screen test-id prefix of the screen shown
 * @returns the three controls, in reading order
 */
const controlSpecs = (
  props: PriorityRowProps,
  t: Translate,
  screen: MicrophoneScreen,
): RowButtonProps[] => [
  moveSpec(props, t, "up", screen),
  moveSpec(props, t, "down", screen),
  {
    label: t(`${PRIORITY_KEY_ROOT}.remove`, { name: props.name }),
    testId: `${screen}-choice-remove`,
    focusKey: rowFocusKey(props.name, "remove"),
    inert: props.busy,
    onActivate: () => props.onRemove(props.index),
    children: <X size={14} />,
  },
];

/** Controls of one row. */
const RowControls: React.FC<PriorityRowProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  return (
    <div className="flex items-center gap-1">
      {controlSpecs(props, t, screen).map((spec) => (
        <RowButton key={spec.testId} {...spec} />
      ))}
    </div>
  );
};

/** Drag handle (pointer only; keyboard users use Move up / Move down). */
const RowHandle: React.FC<PriorityRowProps> = ({ busy, handle }) => (
  <span
    aria-hidden="true"
    className={`text-mid-gray touch-none ${busy ? "cursor-not-allowed" : "cursor-grab"}`}
    {...(busy ? {} : handle)}
  >
    <GripVertical size={15} />
  </span>
);

/** Rank, exact name and availability of one row. */
const RowLabel: React.FC<PriorityRowProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  return (
    <>
      <span
        className="w-4 text-sm text-mid-gray"
        data-testid={`${screen}-choice-rank`}
      >
        {props.index + 1}
      </span>
      <span
        className="flex-1 min-w-0 truncate text-sm"
        data-testid={`${screen}-choice-name`}
        title={props.name}
      >
        {props.name}
      </span>
      <span data-testid={`${screen}-choice-status`}>
        <Badge variant={AVAILABILITY_TONE[props.availability]}>
          {t(`${PRIORITY_KEY_ROOT}.status.${props.availability}`)}
        </Badge>
      </span>
    </>
  );
};

/** One configured microphone with its status and controls. */
export const PriorityRow: React.FC<PriorityRowProps> = (props) => {
  const { state, classes } = rowState(props);
  const screen = useMicrophoneScreen();
  return (
    <li
      ref={props.rowRef}
      className={`flex items-center gap-3 px-4 py-2 ${classes}`}
      data-testid={`${screen}-choice`}
      data-instance-id={microphoneInstanceId(props.name)}
      data-name={props.name}
      data-state={state}
    >
      <RowHandle {...props} />
      <RowLabel {...props} />
      <RowControls {...props} />
    </li>
  );
};
