// Contract screen shown by a settings page that draws several Penflow screens at once (the base
// screen and its state screens, `penflow/flow-ui-contract/contract.json`). Every test id of the
// page carries the prefix of the screen actually shown, so each one is unique in the tree. Each
// part of the page reports the state screen it shows (or none); the page shows the first
// reported screen in its priority order, else its base screen.
import {
  type Context,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

/** The page's screen and how each part reports the screen it shows. */
export interface ScreenScope<S extends string> {
  screen: S;
  report: (source: string, screen: S | null) => void;
}

/** Order of the state screens of one page, and its base screen. */
export interface ScreenOrder<S extends string> {
  /** State screens, the one that wins first. */
  priority: readonly S[];
  base: S;
}

/**
 * Screen shown for the reported screens.
 * @param order the page's screens
 * @param reported screen reported by each part (null: none)
 * @returns the first reported state screen in priority order, else the base screen
 */
export const firstReported = <S extends string>(
  order: ScreenOrder<S>,
  reported: readonly (S | null)[],
): S =>
  order.priority.find((screen) => reported.includes(screen)) ?? order.base;

/**
 * Scope of a page root: gathers what each part reports.
 * @param order the page's screens
 * @returns the scope to provide to the page's parts
 */
export const useScreenScope = <S extends string>(
  order: ScreenOrder<S>,
): ScreenScope<S> => {
  const [reported, setReported] = useState<Record<string, S | null>>({});
  const report = useCallback((source: string, screen: S | null) => {
    // Same screen: keep the object so the page does not render again.
    setReported((current) =>
      current[source] === screen ? current : { ...current, [source]: screen },
    );
  }, []);
  const screen = firstReported(order, Object.values(reported));
  return useMemo(() => ({ screen, report }), [screen, report]);
};

/**
 * Reports one part's screen to the page and returns the screen shown. The part's own screen
 * also counts at once, so its own test ids never wait for the page to render again; outside a
 * page, the part decides alone.
 * Side effects: reports `own` for `source` while mounted, and null when unmounted.
 * @param context the page's scope
 * @param order the page's screens
 * @param source a stable name of the part
 * @param own the state screen the part shows, or null
 * @returns the test-id prefix of the screen shown
 */
export const useReportedScreen = <S extends string>(
  context: Context<ScreenScope<S> | null>,
  order: ScreenOrder<S>,
  source: string,
  own: S | null,
): S => {
  const scope = useContext(context);
  const report = scope?.report;
  useEffect(() => {
    report?.(source, own);
  }, [report, source, own]);
  useEffect(() => () => report?.(source, null), [report, source]);
  return firstReported(order, [scope?.screen ?? null, own]);
};
