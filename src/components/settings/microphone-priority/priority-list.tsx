// Ordered list of configured microphones (feature 001), or its empty state. Pointer reorder uses
// the row handles; a keyboard move keeps focus on the moved row's Move up / Move down control.
import React, { useLayoutEffect, useRef } from "react";
import type { RefObject } from "react";
import { useTranslation } from "react-i18next";
import { Mic } from "lucide-react";
import type { Availability } from "@/bindings";
import { PRIORITY_KEY_ROOT } from "./draft-state";
import {
  PriorityRow,
  type PriorityRowProps,
  type RowControl,
} from "./priority-row";
import { usePointerReorder, type PointerReorder } from "./use-pointer-reorder";

/** Props of the priority list. */
export interface PriorityListProps {
  order: readonly string[];
  /** Availability of each entry of `order`, same positions. */
  availability: readonly Availability[];
  busy: boolean;
  onMove: (from: number, to: number) => void;
  onRemove: (index: number) => void;
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
 * Move focus back to the control a keyboard move started from, once the reordered rows are in
 * the DOM (React may move the focused row's node, which drops its focus).
 * @param listRef the list element
 * @param pendingFocus focus key to restore, cleared once used
 * @param order current order (the effect runs after each reorder)
 */
const useRestoreFocus = (
  listRef: RefObject<HTMLOListElement | null>,
  pendingFocus: React.MutableRefObject<string | null>,
  order: readonly string[],
): void => {
  useLayoutEffect(() => {
    const key = pendingFocus.current;
    if (key === null || listRef.current === null) return;
    pendingFocus.current = null;
    const buttons = listRef.current.querySelectorAll<HTMLButtonElement>(
      "button[data-focus-key]",
    );
    Array.from(buttons)
      .find((button) => button.dataset.focusKey === key)
      ?.focus();
  }, [listRef, pendingFocus, order]);
};

/**
 * Props of row `index`: its drag state and its controls.
 * @param props list props
 * @param reorder pointer reorder state
 * @param index row position
 * @param onKeyboardMove move that restores focus afterwards
 * @returns the row props
 */
const rowPropsAt = (
  props: PriorityListProps,
  reorder: PointerReorder,
  index: number,
  onKeyboardMove: PriorityRowProps["onMove"],
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
    onMove: onKeyboardMove,
    onRemove: props.onRemove,
  };
};

/** Ordered, reorderable list of the configured microphones. */
export const PriorityList: React.FC<PriorityListProps> = (props) => {
  const { t } = useTranslation();
  const listRef = useRef<HTMLOListElement>(null);
  const pendingFocus = useRef<string | null>(null);
  const reorder = usePointerReorder(props.onMove);
  useRestoreFocus(listRef, pendingFocus, props.order);
  if (props.order.length === 0) return <EmptyPriority />;
  const { drag } = reorder;
  const moveFromKeyboard = (from: number, to: number, control: RowControl) => {
    pendingFocus.current = `${props.order[from]}\n${control}`;
    props.onMove(from, to);
  };
  return (
    <ol
      ref={listRef}
      aria-label={t(`${PRIORITY_KEY_ROOT}.title`)}
      className="border border-mid-gray/20 rounded-lg divide-y divide-mid-gray/20"
      data-testid="microphone-settings-choices"
      data-state={drag && drag.over !== drag.from ? "drop-target" : undefined}
    >
      {props.order.map((name, index) => (
        <PriorityRow
          key={name}
          {...rowPropsAt(props, reorder, index, moveFromKeyboard)}
        />
      ))}
    </ol>
  );
};
