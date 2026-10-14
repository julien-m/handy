/**
 * Rules of the editor's guard dialog (feature 002, AC-001): a guarded transition (List/Text,
 * Import…) with unsaved edits waits in the dialog for Save, Discard or Cancel. While a commit is
 * in flight the guard is inactive: no new transition is taken, and Discard and Cancel do nothing,
 * so the lock of a running Save or Apply is never lifted. A transition runs only after the Save
 * that the same request started, and never after a Cancel.
 */

/** A transition waiting for the user's choice. */
export interface PendingTransition {
  readonly action: () => void;
}

/** What the guard reads and does in the editor. */
export interface DraftGateDeps {
  /** The current view holds unsaved edits. */
  isDirty: () => boolean;
  /** A Save or an Apply is in flight. */
  isBusy: () => boolean;
  /** Drop the edits of both views. */
  discard: () => void;
  /** Save (list) or Apply (text) the current view; true when committed. */
  commit: () => Promise<boolean>;
  /** Called with the waiting transition each time it changes (null: dialog closed). */
  onChange: (pending: PendingTransition | null) => void;
}

/** The guard's actions. */
export interface DraftGateController {
  request: (action: () => void) => void;
  cancel: () => void;
  discard: () => void;
  save: () => Promise<void>;
}

/**
 * Create the guard of the editor's transitions.
 * Side effects: runs the transitions, discards or commits the edits through `deps`.
 * @param deps editor state and actions
 * @returns the request, Cancel, Discard and Save actions
 */
export const createDraftGate = (deps: DraftGateDeps): DraftGateController => {
  let pending: PendingTransition | null = null;
  const settle = (next: PendingTransition | null) => {
    pending = next;
    deps.onChange(next);
  };
  return {
    request: (action) => {
      if (deps.isBusy()) return;
      if (deps.isDirty()) settle({ action });
      else action();
    },
    cancel: () => {
      if (!deps.isBusy()) settle(null);
    },
    discard: () => {
      if (pending === null || deps.isBusy()) return;
      const { action } = pending;
      deps.discard();
      settle(null);
      action();
    },
    save: () => saveThenRun(deps, () => pending, settle),
  };
};

/**
 * Commit the edits, then run the transition that was waiting when Save was chosen, unless the
 * dialog was cancelled or replaced meanwhile. A refused commit closes the dialog and keeps the
 * edits with their error.
 */
const saveThenRun = async (
  deps: DraftGateDeps,
  current: () => PendingTransition | null,
  settle: (next: PendingTransition | null) => void,
): Promise<void> => {
  const request = current();
  if (request === null || deps.isBusy()) return;
  const committed = await deps.commit();
  if (current() !== request) return;
  settle(null);
  if (committed) request.action();
};
