// Root of the shortcut settings page (feature 004): gathers the state reported by each shortcut
// list and gives every part of the page the test-id prefix of the contract screen shown.
import React, { useCallback, useMemo, useState } from "react";
import {
  type ShortcutListState,
  ShortcutScreenContext,
  shortcutScreenOf,
} from "./shortcut-screen";

/**
 * Screen root with its `-screen` and `-general` test ids.
 * @param props.general the General group (shortcut lists and activation)
 * @param props.children the rest of the page
 */
export const ShortcutScreenRoot: React.FC<{
  general: React.ReactNode;
  children: React.ReactNode;
}> = ({ general, children }) => {
  const [states, setStates] = useState<Record<string, ShortcutListState>>({});
  const report = useCallback((actionId: string, state: ShortcutListState) => {
    // Same state: keep the object so the page does not render again.
    setStates((current) =>
      current[actionId] === state ? current : { ...current, [actionId]: state },
    );
  }, []);
  const screen = shortcutScreenOf(Object.values(states));
  const scope = useMemo(() => ({ screen, report }), [screen, report]);
  return (
    <ShortcutScreenContext.Provider value={scope}>
      <div
        className="max-w-3xl w-full mx-auto space-y-6"
        data-testid={`${screen}-screen`}
      >
        <div data-testid={`${screen}-general`}>{general}</div>
        {children}
      </div>
    </ShortcutScreenContext.Provider>
  );
};
