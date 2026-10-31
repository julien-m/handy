/**
 * Settings lists that can hold hundreds of entries (replacement rules, vocabulary terms) keep a
 * fixed height of about five rows and scroll inside, so the settings page does not grow with
 * them (decision `bounded_lists`, Julien, 2026-10-30).
 */
import { type RefObject, useEffect, useRef } from "react";

/** Rows visible before the list scrolls. */
export const BOUNDED_LIST_ROWS = 5;

/**
 * Brings an entry just added into view: the list is sorted (decision `alphabetical_display`), so
 * the new entry can be anywhere. The first load (from empty) keeps the top in view.
 * @param list the scrolling element, whose children are the entries in display order
 * @param keys stable keys of the entries, in display order
 */
export const useRevealAdded = (
  list: RefObject<HTMLElement>,
  keys: ReadonlyArray<string>,
): void => {
  const previous = useRef(keys);
  useEffect(() => {
    const before = previous.current;
    previous.current = keys;
    if (before.length === 0 || keys.length <= before.length) return;
    const known = new Set(before);
    const position = keys.findIndex((key) => !known.has(key));
    const entry = position < 0 ? null : list.current?.children[position];
    entry?.scrollIntoView({ block: "nearest" });
  }, [list, keys]);
};
