// Keyboard focus of the microphone priority editor (feature 001, SC-003): after an action removes
// or moves the control that had focus, focus goes to a logical control and never falls on <body>.
// Targets carry `data-focus-key`; a request lists them by preference and the first focusable one
// wins, with the section title (tabIndex -1) as the stable fallback.
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import type { RefObject } from "react";

/** Focus key of the priority section title, the stable target after Discard, Reload and Save. */
export const STABLE_FOCUS_KEY = "microphone-priority-title";
/** Focus key of the Add microphone control. */
export const ADD_FOCUS_KEY = "microphone-priority-add";

/** Row control a focus key can name. */
export type RowFocusControl = "up" | "down" | "remove";

/**
 * Focus key of one row control. Names are unique in a valid order, and the separator cannot be
 * part of a device name shown on one line.
 * @param name exact microphone name of the row
 * @param control control of that row
 * @returns the key carried by the control's `data-focus-key`
 */
export const rowFocusKey = (name: string, control: RowFocusControl): string =>
  `${name}\n${control}`;

/**
 * Where focus goes after Remove: the row that takes the removed position, else the previous row,
 * else the Add control (each on its Remove control), then the section title.
 * @param order order before the removal
 * @param index removed position
 * @returns focus keys by preference
 */
export const focusAfterRemove = (
  order: readonly string[],
  index: number,
): string[] => {
  const keys: string[] = [];
  if (index + 1 < order.length)
    keys.push(rowFocusKey(order[index + 1], "remove"));
  if (index > 0) keys.push(rowFocusKey(order[index - 1], "remove"));
  return [...keys, ADD_FOCUS_KEY, STABLE_FOCUS_KEY];
};

/**
 * The element that takes focus for a target: the target itself when focusable, else its first
 * focusable descendant (the Add wrapper holds the dropdown button); never a disabled control.
 * @param holder element carrying the focus key
 * @returns the element to focus, or null
 */
const focusableIn = (holder: HTMLElement): HTMLElement | null => {
  const selector = "button, [tabindex]";
  const target = holder.matches(selector)
    ? holder
    : holder.querySelector<HTMLElement>(selector);
  return target === null || target.matches(":disabled") ? null : target;
};

/**
 * Focus the first available target inside the editor. Focus that already moved outside the
 * editor (and is not on <body>) is left alone.
 * @param root editor element
 * @param keys focus keys by preference; the section title is always tried last
 * @returns the focused element, or null when nothing was focused
 */
export const focusFirstTarget = (
  root: HTMLElement,
  keys: readonly string[],
): HTMLElement | null => {
  const active = root.ownerDocument.activeElement;
  const lost = active === null || active === root.ownerDocument.body;
  if (!lost && !root.contains(active)) return null;
  const holders = Array.from(
    root.querySelectorAll<HTMLElement>("[data-focus-key]"),
  );
  for (const key of [...keys, STABLE_FOCUS_KEY]) {
    const holder = holders.find((element) => element.dataset.focusKey === key);
    const target = holder ? focusableIn(holder) : null;
    if (target !== null) {
      target.focus();
      return target;
    }
  }
  return null;
};

/** Focus requests of the editor. */
export interface EditorFocus {
  /** Element that contains every focus target (the editor screen). */
  rootRef: RefObject<HTMLDivElement>;
  /** Move focus once the next render is committed (the target may only exist then). */
  request: (keys: readonly string[]) => void;
}

/**
 * Focus requests applied after the render that follows them.
 * Side effects: moves the document focus in a layout effect.
 * @returns the root ref and the request action
 */
export const useEditorFocus = (): EditorFocus => {
  const rootRef = useRef<HTMLDivElement>(null);
  // A new object per request, so a request always renders, even when the draft did not change.
  const [pending, setPending] = useState<{ keys: readonly string[] } | null>(
    null,
  );
  useLayoutEffect(() => {
    if (pending !== null && rootRef.current !== null) {
      focusFirstTarget(rootRef.current, pending.keys);
    }
  }, [pending]);
  const request = useCallback(
    (keys: readonly string[]) => setPending({ keys }),
    [],
  );
  return { rootRef, request };
};
