/**
 * Alphabetical display order of the replacement rules (002) and vocabulary terms (003)
 * (decision `alphabetical_display`, Julien, 2026-10-31). Only what the list shows is sorted: the
 * saved order is unchanged, since 003 sends terms in saved order when the model budget is short.
 */

/** An item with the position it has in its saved list. */
export interface Placed<T> {
  item: T;
  index: number;
}

/**
 * Items in alphabetical order of their text for `locale`, ignoring case and accents, with
 * numbers compared by value; equal texts keep their saved order.
 * @param items items in saved order
 * @param text sort text of an item
 * @param locale UI language
 * @returns each item with its saved index, in display order
 */
export const alphabetical = <T>(
  items: ReadonlyArray<T>,
  text: (item: T) => string,
  locale: string,
): Placed<T>[] => {
  const collator = new Intl.Collator(locale, {
    sensitivity: "base",
    numeric: true,
  });
  return items
    .map((item, index) => ({ item, index }))
    .sort(
      (a, b) =>
        collator.compare(text(a.item).trim(), text(b.item).trim()) ||
        a.index - b.index,
    );
};

/**
 * Display order that does not move while a row is edited: with no frozen order, alphabetical;
 * with one, the frozen keys still present, then keys added since, in saved order.
 * @param items items in saved order
 * @param key stable key of an item
 * @param text sort text of an item
 * @param locale UI language
 * @param frozen keys in the order shown when editing started, or null
 * @returns each item with its saved index, in display order
 */
export const displayOrder = <T>(
  items: ReadonlyArray<T>,
  key: (item: T) => string,
  text: (item: T) => string,
  locale: string,
  frozen: ReadonlyArray<string> | null,
): Placed<T>[] => {
  if (frozen === null) return alphabetical(items, text, locale);
  const placed = new Map(
    items.map((item, index) => [key(item), { item, index }] as const),
  );
  const kept = frozen.flatMap((k) => {
    const entry = placed.get(k);
    return entry ? [entry] : [];
  });
  const shown = new Set(frozen);
  const added = items
    .map((item, index) => ({ item, index }))
    .filter((entry) => !shown.has(key(entry.item)));
  return [...kept, ...added];
};
