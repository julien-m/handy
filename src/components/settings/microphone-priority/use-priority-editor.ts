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
  saveErrorMessage,
  shownOrder,
  startSave,
  type PriorityDraft,
  type SavedPriority,
} from "./draft-state";
import { useEditorFocus, type EditorFocus } from "./editor-focus";

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
  refreshStatus: () => Promise<void>;
  notify: (key: string, kind: "success" | "error") => void;
  /** Move focus to the section title once the Unsaved changes bar is gone. */
  focusStable: () => void;
}

/**
 * Apply the outcome of a write. Success rereads the committed order before dropping the draft
 * (no flash of the old order) and marks the editor saved. A value stored with a failed live effect
 * is no longer a draft, so it is reported in a toast. Either way the bar that held focus is gone,
 * so focus moves to the section title. Other refusals keep the draft and the focused Save (now
 * Retry); a conflict or an unknown outcome rereads the settings so the base revision is current.
 * @param outcome result of the write
 * @param setDraft draft setter
 * @param effects store and status actions
 */
export const settleWrite = async (
  outcome: WriteOutcome,
  setDraft: SetDraft,
  effects: WriteEffects,
): Promise<void> => {
  if (outcome.ok || outcome.error?.outcome === "committed") {
    await effects.refreshSettings();
    setDraft(outcome.ok ? SAVED_DRAFT : EMPTY_DRAFT);
    effects.focusStable();
    if (outcome.ok) effects.notify(SAVED_KEY, "success");
    else effects.notify(saveErrorMessage(outcome.error).key, "error");
    void effects.refreshStatus();
    return;
  }
  const { error } = outcome;
  setDraft((draft) => failSave(draft, error));
  if (error?.code === "revision_conflict" || error?.outcome === "unknown") {
    await effects.refreshSettings();
  }
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
    const outcome = await writePriority(
      draft.order ?? [],
      baseRevision(draft, saved),
    );
    await settleWrite(outcome, setDraft, effects);
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
 * @returns the names the Add control offers and draft-only choices are classified against
 */
const useEnumerated = (): readonly string[] | null => {
  const devices = useSettingsStore((state) => state.audioDevices);
  const failed = useSettingsStore((state) => state.audioDevicesFailed);
  return useMemo(() => enumeratedNames(devices, failed), [devices, failed]);
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
 * useMicrophoneStatus) and moves focus after actions that remove the focused control.
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
  return {
    ...useDraftEdits(saved, setDraft, focus),
    ...useRefreshActions(status, effects, setDraft),
    ...draftView(draft, saved, loading),
    loading,
    saved,
    draft,
    enumerated: useEnumerated(),
    status,
    focus,
    save: useSave(draft, saved, setDraft, effects),
  };
};
