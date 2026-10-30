/**
 * Settings lists that can hold hundreds of entries (replacement rules, vocabulary terms) keep a
 * fixed height of about five rows and scroll inside, so the settings page does not grow with
 * them (decision `bounded_lists`, Julien, 2026-10-30).
 */
import { type RefObject, useEffect, useRef } from "react";

/** Rows visible before the list scrolls. */
export const BOUNDED_LIST_ROWS = 5;

/**
 * Scrolls the list to its end when entries are appended to a list that already had some, so a
 * rule or term just added stays visible. The first load (from empty) keeps the top in view.
 * @param list the scrolling element
 * @param count number of entries shown
 */
export const useRevealAppended = (
  list: RefObject<HTMLElement>,
  count: number,
): void => {
  const previous = useRef(count);
  useEffect(() => {
    const before = previous.current;
    previous.current = count;
    if (before > 0 && count > before && list.current) {
      list.current.scrollTop = list.current.scrollHeight;
    }
  }, [list, count]);
};
