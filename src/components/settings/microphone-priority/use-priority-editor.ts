// State of the microphone priority editor (feature 001): the committed order from the settings
// store, the draft beside it, the status read and the Save / Discard / Reload actions. Save sends
// the complete order with the base revision through `set_microphone_priority`, the same
// validation and transaction as `handy settings microphones set-priority`.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { commands, type SettingsCommandError } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import {
  useMicrophoneStatus,
  type MicrophoneStatusHandle,
} from "@/hooks/use-microphone-status";
import { isSettingsCommandError } from "@/stores/settings-errors";
import { useSettingsStore } from "@/stores/settingsStore";
import { enumeratedNames } from "./availability";
import {
  EMPTY_DRAFT,
  SAVED_DRAFT,
  addChoice,
  baseRevision,
  canSave,
  editOrder,
  failSave,
  isDirty,
  moveChoice,
  rebaseDraft,
  removeChoice,
  sameOrder,
  saveErrorMessage,
  shownOrder,
  startSave,
  type PriorityDraft,
  type SavedPriority,
} from "./draft-state";
import {
  useEditorFocus,
  useFocusWhenBarCloses,
  type EditorFocus,
} from "./editor-focus";

/** Draft edits; each one changes only the draft. */
export interface DraftEdits {
  move: (from: number, to: number) => void;
  remove: (index: number) => void;
  add: (name: string) => void;
  discard: () => void;
}

/** What the views show of the draft. */
export interface DraftView {
  /** Order shown in the list (the draft, or the committed order). */
  order: readonly string[];
  dirty: boolean;
  saveEnabled: boolean;
}

/** Everything the editor views read and do. */
export interface PriorityEditor extends DraftEdits, DraftView {
  /** The settings are not loaded yet: the committed order and its revision are unknown. */
  loading: boolean;
  saved: SavedPriority;
  draft: PriorityDraft;
  /** Names of the enumerated devices, or null before a successful listing. */
  enumerated: readonly string[] | null;
  /** The last device listing failed (Add has nothing to offer until a Refresh succeeds). */
  listingFailed: boolean;
  status: MicrophoneStatusHandle;
  /** Focus requests (SC-003) and the element that holds their targets. */
  focus: EditorFocus;
  reload: () => Promise<void>;
  save: () => Promise<void>;
  refreshAll: () => Promise<void>;
}

const SAVED_KEY = "settings.sound.microphonePriority.draft.saved";

type SetDraft = Dispatch<SetStateAction<PriorityDraft>>;

/** Outcome of one write, with the typed refusal (null when untyped) on failure. */
export type WriteOutcome =
  | { ok: true }
  | { ok: false; error: SettingsCommandError | null };

/**
 * Submit the complete order with the revision it is based on (compare-and-set).
 * @param order complete draft order
 * @param revision base revision
 * @returns whether the write committed, with the refusal otherwise
 */
export const writePriority = async (
  order: readonly string[],
  revision: number | null,
): Promise<WriteOutcome> => {
  try {
    const result = await commands.setMicrophonePriority([...order], revision);
    if (result.status === "ok") return { ok: true };
    return { ok: false, error: result.error };
  } catch (error) {
    console.error("Microphone priority write failed:", error);
    return { ok: false, error: isSettingsCommandError(error) ? error : null };
  }
};

/**
 * Committed priority from the settings store, stable while its order and revision are unchanged.
 * @returns the committed order and its revision
 */
const useSavedPriority = (): SavedPriority => {
  const order = useSettingsStore(
    (state) => state.settings?.microphone_priority,
  );
  const revision = useSettingsStore((state) => state.revision);
  return useMemo(() => ({ order: order ?? [], revision }), [order, revision]);
};

/**
 * Draft edits bound to the committed priority. Discard removes the bar that had focus, so focus
 * moves to the section title.
 * @param saved committed priority
 * @param setDraft draft setter
 * @param focus focus requests
 * @returns the edit actions
 */
const useDraftEdits = (
  saved: SavedPriority,
  setDraft: SetDraft,
  focus: EditorFocus,
): DraftEdits => {
  const edit = useCallback(
    (change: (order: readonly string[]) => readonly string[]) =>
      setDraft((draft) =>
        editOrder(draft, saved, change(shownOrder(draft, saved))),
      ),
    [saved, setDraft],
  );
  return {
    move: (from, to) => edit((order) => moveChoice(order, from, to)),
    remove: (index) => edit((order) => removeChoice(order, index)),
    add: (name) => setDraft((draft) => addChoice(draft, saved, name)),
    discard: () => {
      setDraft(EMPTY_DRAFT);
      focus.request([]);
    },
  };
};

/** Store, status, notification and focus actions a write needs afterwards. */
export interface WriteEffects {
  refreshSettings: () => Promise<void>;
  /** Committed order in the settings store (after a reread), or null before the settings load. */
  committedOrder: () => readonly string[] | null;
  refreshStatus: () => Promise<void>;
  notify: (key: string, kind: "success" | "error") => void;
  /** Move focus to the section title once the Unsaved changes bar is gone. */
  focusStable: () => void;
}

/**
 * Close the draft of a committed write, once the committed order is reread: the editor is marked
 * saved (or, for a value stored with a failed live effect, the failure is reported in a toast).
 * The bar that held focus is gone, so focus moves to the section title.
 * @param error null for a confirmed write, else the refusal of a value that was still stored
 * @param setDraft draft setter
 * @param effects store and status actions
 */
const closeCommitted = (
  error: SettingsCommandError | null,
  setDraft: SetDraft,
  effects: WriteEffects,
): void => {
  setDraft(error === null ? SAVED_DRAFT : EMPTY_DRAFT);
  effects.focusStable();
  if (error === null) effects.notify(SAVED_KEY, "success");
  else effects.notify(saveErrorMessage(error).key, "error");
  void effects.refreshStatus();
};

/**
 * Apply the outcome of a write. Success rereads the committed order before dropping the draft (no
 * flash of the old order). An unknown outcome rereads first: if the sent order is now committed,
 * the write did happen and is confirmed like a success; otherwise it is a refusal. Refusals keep
 * the draft and the focused Save (now Retry); a conflict rereads so the base revision is current.
 * @param outcome result of the write
 * @param sent complete order the write sent
 * @param setDraft draft setter
 * @param effects store and status actions
 */
export const settleWrite = async (
  outcome: WriteOutcome,
  sent: readonly string[],
  setDraft: SetDraft,
  effects: WriteEffects,
): Promise<void> => {
  const error = outcome.ok ? null : outcome.error;
  if (outcome.ok || error?.outcome === "committed") {
    await effects.refreshSettings();
    return closeCommitted(error, setDraft, effects);
  }
  if (error?.outcome === "unknown") {
    await effects.refreshSettings();
    const committed = effects.committedOrder();
    if (committed !== null && sameOrder(committed, sent)) {
      return closeCommitted(null, setDraft, effects);
    }
  }
  setDraft((draft) => failSave(draft, error));
  if (error?.code === "revision_conflict") await effects.refreshSettings();
};

/**
 * Save action guarded against a second dispatch while a write is in flight, and against a write
 * without a base revision (see canSave).
 * @param draft editor state
 * @param saved committed priority
 * @param setDraft draft setter
 * @param effects store and status actions
 * @returns the Save action
 */
const useSave = (
  draft: PriorityDraft,
  saved: SavedPriority,
  setDraft: SetDraft,
  effects: WriteEffects,
): (() => Promise<void>) => {
  const inFlight = useRef(false);
  return useCallback(async () => {
    if (inFlight.current || !canSave(draft, saved)) return;
    inFlight.current = true;
    setDraft(startSave);
    const sent = draft.order ?? [];
    const outcome = await writePriority(sent, baseRevision(draft, saved));
    await settleWrite(outcome, sent, setDraft, effects);
    inFlight.current = false;
  }, [draft, saved, setDraft, effects]);
};

/**
 * Store, status, notification and focus actions of the editor.
 * @param status status handle
 * @param focus focus requests
 * @returns the effects a write and a reload use
 */
const useWriteEffects = (
  status: MicrophoneStatusHandle,
  focus: EditorFocus,
): WriteEffects => {
  const { t } = useTranslation();
  const { refreshSettings } = useSettings();
  const { refresh } = status;
  const { request } = focus;
  return useMemo(
    () => ({
      refreshSettings,
      committedOrder: () =>
        useSettingsStore.getState().settings?.microphone_priority ?? null,
      refreshStatus: refresh,
      notify: (key, kind) =>
        kind === "success" ? toast.success(t(key)) : toast.error(t(key)),
      focusStable: () => request([]),
    }),
    [refreshSettings, refresh, request, t],
  );
};

/**
 * Reload (conflict banner) and the availability refresh.
 * @param status status handle
 * @param effects store and focus actions
 * @param setDraft draft setter
 * @returns the reload and refreshAll actions
 */
const useRefreshActions = (
  status: MicrophoneStatusHandle,
  effects: WriteEffects,
  setDraft: SetDraft,
): Pick<PriorityEditor, "reload" | "refreshAll"> => {
  const { refreshAudioDevices } = useSettings();
  const reload = useCallback(async () => {
    await effects.refreshSettings();
    // A write in flight settles the draft itself; Reload is inactive meanwhile.
    setDraft((draft) => (draft.savePhase === "saving" ? draft : EMPTY_DRAFT));
    effects.focusStable();
  }, [effects, setDraft]);
  const refreshAll = useCallback(async () => {
    await Promise.all([status.refresh(), refreshAudioDevices()]);
  }, [status, refreshAudioDevices]);
  return { reload, refreshAll };
};

/**
 * Enumerated device names from the store, null while unknown (not listed yet, or listing failed).
 * @returns the names the Add control offers and draft-only choices are classified against, and
 *   whether the last listing failed
 */
const useEnumerated = (): Pick<
  PriorityEditor,
  "enumerated" | "listingFailed"
> => {
  const devices = useSettingsStore((state) => state.audioDevices);
  const failed = useSettingsStore((state) => state.audioDevicesFailed);
  return useMemo(
    () => ({
      enumerated: enumeratedNames(devices, failed),
      listingFailed: failed,
    }),
    [devices, failed],
  );
};

/**
 * What the views show of the draft. Nothing can be saved before the settings are loaded.
 * @param draft editor state
 * @param saved committed priority
 * @param loading whether the settings are still loading
 * @returns the shown order, dirtiness and whether Save may dispatch
 */
const draftView = (
  draft: PriorityDraft,
  saved: SavedPriority,
  loading: boolean,
): DraftView => ({
  order: shownOrder(draft, saved),
  dirty: isDirty(draft, saved),
  saveEnabled: !loading && canSave(draft, saved),
});

/**
 * Microphone priority editor state for the audio settings.
 * Side effects: reads the microphone status every 5 s while mounted and visible (see
 * useMicrophoneStatus) and moves focus after actions that remove the focused control, or when the
 * Unsaved changes bar closes on its own.
 * @returns the editor state and actions
 */
export const usePriorityEditor = (): PriorityEditor => {
  const saved = useSavedPriority();
  const loading = useSettingsStore((state) => state.settings === null);
  const status = useMicrophoneStatus();
  const focus = useEditorFocus();
  const [draft, setDraft] = useState<PriorityDraft>(EMPTY_DRAFT);
  useEffect(() => setDraft((current) => rebaseDraft(current, saved)), [saved]);
  const effects = useWriteEffects(status, focus);
  const view = draftView(draft, saved, loading);
  useFocusWhenBarCloses(view.dirty, focus.request);
  return {
    ...useDraftEdits(saved, setDraft, focus),
    ...useRefreshActions(status, effects, setDraft),
    ...view,
    ...useEnumerated(),
    loading,
    saved,
    draft,
    status,
    focus,
    save: useSave(draft, saved, setDraft, effects),
  };
};
