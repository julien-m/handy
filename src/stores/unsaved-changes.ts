/**
 * Unsaved-edit guards of the settings window: an editor with a draft (the replacement rules,
 * feature 002) registers a guard, and section navigation asks before leaving it, so a draft is
 * never dropped silently. The guard only reports and discards; saving stays in the editor.
 */
import { create } from "zustand";

/** One editor's draft, as navigation sees it. */
export interface UnsavedChangesGuard {
  /** Whether the editor holds edits that are not saved. */
  readonly isDirty: () => boolean;
  /** Drop the edits; the editor shows its saved state again. */
  readonly discard: () => void;
}

interface UnsavedChangesStore {
  guards: ReadonlyArray<UnsavedChangesGuard>;
}

const useUnsavedChangesStore = create<UnsavedChangesStore>()(() => ({
  guards: [],
}));

/**
 * Register an editor's guard until the returned function is called (on unmount).
 * @param guard the editor's draft probe and discard action
 * @returns the unregister function
 */
export const registerUnsavedChangesGuard = (
  guard: UnsavedChangesGuard,
): (() => void) => {
  useUnsavedChangesStore.setState((state) => ({
    guards: [...state.guards, guard],
  }));
  return () =>
    useUnsavedChangesStore.setState((state) => ({
      guards: state.guards.filter((registered) => registered !== guard),
    }));
};

/**
 * Whether any registered editor holds unsaved edits.
 * @returns true when leaving the section needs a confirmation
 */
export const hasUnsavedChanges = (): boolean =>
  useUnsavedChangesStore.getState().guards.some((guard) => guard.isDirty());

/** Discard the edits of every registered editor (the user chose to leave). */
export const discardUnsavedChanges = (): void => {
  for (const guard of useUnsavedChangesStore.getState().guards) {
    if (guard.isDirty()) guard.discard();
  }
};
