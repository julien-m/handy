// Ordered list of configured microphones (feature 001), or its empty state. Pointer reorder uses
// the row handles; a keyboard move keeps focus on the moved row's Move up / Move down control and
// Remove moves focus to the next row, the previous one or the Add control (SC-003).
import React from "react";
import { useTranslation } from "react-i18next";
import { Mic } from "lucide-react";
import type { Availability } from "@/bindings";
import { PRIORITY_KEY_ROOT } from "./draft-state";
import { focusAfterRemove, rowFocusKey } from "./editor-focus";
import { PriorityRow, type PriorityRowProps } from "./priority-row";
import { usePointerReorder, type PointerReorder } from "./use-pointer-reorder";

/** Props of the priority list. */
export interface PriorityListProps {
  order: readonly string[];
  /** Availability of each entry of `order`, same positions. */
  availability: readonly Availability[];
  busy: boolean;
  onMove: (from: number, to: number) => void;
  onRemove: (index: number) => void;
  /** Move focus to the first available of these focus keys once the list is rendered again. */
  onFocusRequest: (keys: readonly string[]) => void;
}

/** Empty normal list: recordings use the system default (after any closed-lid override). */
export const EmptyPriority: React.FC = () => {
  const { t } = useTranslation();
  return (
    <div
      className="flex flex-col items-center gap-2 px-4 py-6 text-center border border-mid-gray/20 rounded-lg"
      data-testid="microphone-settings-empty-empty"
    >
      <Mic size={20} className="text-mid-gray" aria-hidden="true" />
      <p
        className="text-sm font-medium"
        data-testid="microphone-settings-empty-empty-title"
      >
        {t(`${PRIORITY_KEY_ROOT}.emptyTitle`)}
      </p>
      <p
        className="text-sm text-mid-gray"
        data-testid="microphone-settings-empty-empty-body"
      >
        {t(`${PRIORITY_KEY_ROOT}.emptyBody`)}
      </p>
    </div>
  );
};

/**
 * Props of row `index`: its drag state and its controls.
 * @param props list props
 * @param reorder pointer reorder state
 * @param index row position
 * @returns the row props
 */
const rowPropsAt = (
  props: PriorityListProps,
  reorder: PointerReorder,
  index: number,
): PriorityRowProps => {
  const { drag } = reorder;
  return {
    name: props.order[index],
    index,
    count: props.order.length,
    availability: props.availability[index] ?? "unknown",
    busy: props.busy,
    dragging: drag?.from === index,
    dropTarget: drag !== null && drag.from !== index && drag.over === index,
    rowRef: reorder.rowRef(index),
    handle: reorder.handleProps(index),
    onMove: (from, to, control) => {
      // React may move the focused row's node, which drops its focus: restore it after the move.
      props.onFocusRequest([rowFocusKey(props.order[from], control)]);
      props.onMove(from, to);
    },
    onRemove: (removed) => {
      props.onFocusRequest(focusAfterRemove(props.order, removed));
      props.onRemove(removed);
    },
  };
};

/** Ordered, reorderable list of the configured microphones. */
export const PriorityList: React.FC<PriorityListProps> = (props) => {
  const { t } = useTranslation();
  const reorder = usePointerReorder(props.onMove);
  if (props.order.length === 0) return <EmptyPriority />;
  const { drag } = reorder;
  return (
    <ol
      aria-label={t(`${PRIORITY_KEY_ROOT}.title`)}
      className="border border-mid-gray/20 rounded-lg divide-y divide-mid-gray/20"
      data-testid="microphone-settings-choices"
      data-state={drag && drag.over !== drag.from ? "drop-target" : undefined}
    >
      {props.order.map((name, index) => (
        <PriorityRow key={name} {...rowPropsAt(props, reorder, index)} />
      ))}
    </ol>
  );
};
