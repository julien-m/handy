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
import { addableNames } from "./availability";
import {
  EMPTY_DRAFT,
  addChoice,
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

/** Draft edits; each one changes only the draft. */
export interface DraftEdits {
  move: (from: number, to: number) => void;
  remove: (index: number) => void;
  add: (name: string) => void;
  discard: () => void;
}

/** Everything the editor views read and do. */
export interface PriorityEditor extends DraftEdits {
  /** The settings are not loaded yet: the committed order is unknown. */
  loading: boolean;
  saved: SavedPriority;
  draft: PriorityDraft;
  /** Order shown in the list (the draft, or the committed order). */
  order: readonly string[];
  dirty: boolean;
  saveEnabled: boolean;
  /** Names of the enumerated devices, or null before the first listing. */
  enumerated: readonly string[] | null;
  status: MicrophoneStatusHandle;
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
 * Draft edits bound to the committed priority.
 * @param saved committed priority
 * @param setDraft draft setter
 * @returns the edit actions
 */
const useDraftEdits = (
  saved: SavedPriority,
  setDraft: SetDraft,
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
    discard: () => setDraft(EMPTY_DRAFT),
  };
};

/** Store and status actions a write needs afterwards. */
export interface WriteEffects {
  refreshSettings: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  notify: (key: string, kind: "success" | "error") => void;
}

/**
 * Apply the outcome of a write. Success rereads the committed order before dropping the draft
 * (no flash of the old order). A value stored with a failed live effect is no longer a draft, so
 * it is reported in a toast. Other refusals keep the draft; a conflict or an unknown outcome
 * rereads the settings so the base revision is current.
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
    setDraft(EMPTY_DRAFT);
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
 * Save action guarded against a second dispatch while a write is in flight.
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
    const revision = draft.base?.revision ?? saved.revision;
    const outcome = await writePriority(draft.order ?? [], revision);
    await settleWrite(outcome, setDraft, effects);
    inFlight.current = false;
  }, [draft, saved, setDraft, effects]);
};

/**
 * Store, status and notification actions of the editor.
 * @param status status handle
 * @returns the effects a write and a reload use
 */
const useWriteEffects = (status: MicrophoneStatusHandle): WriteEffects => {
  const { t } = useTranslation();
  const { refreshSettings } = useSettings();
  const { refresh } = status;
  return useMemo(
    () => ({
      refreshSettings,
      refreshStatus: refresh,
      notify: (key, kind) =>
        kind === "success" ? toast.success(t(key)) : toast.error(t(key)),
    }),
    [refreshSettings, refresh, t],
  );
};

/**
 * Microphone priority editor state for the audio settings.
 * Side effects: reads the microphone status every 5 s while mounted (see useMicrophoneStatus).
 * @returns the editor state and actions
 */
export const usePriorityEditor = (): PriorityEditor => {
  const { audioDevices, refreshAudioDevices } = useSettings();
  const saved = useSavedPriority();
  const loading = useSettingsStore((state) => state.settings === null);
  const status = useMicrophoneStatus();
  const [draft, setDraft] = useState<PriorityDraft>(EMPTY_DRAFT);
  useEffect(() => setDraft((current) => rebaseDraft(current, saved)), [saved]);
  const effects = useWriteEffects(status);
  const save = useSave(draft, saved, setDraft, effects);
  const reload = useCallback(async () => {
    await effects.refreshSettings();
    setDraft(EMPTY_DRAFT);
  }, [effects]);
  const refreshAll = useCallback(async () => {
    await Promise.all([status.refresh(), refreshAudioDevices()]);
  }, [status, refreshAudioDevices]);
  return {
    ...useDraftEdits(saved, setDraft),
    loading,
    saved,
    draft,
    order: shownOrder(draft, saved),
    dirty: isDirty(draft, saved),
    saveEnabled: canSave(draft, saved),
    enumerated: audioDevices.length > 0 ? addableNames(audioDevices) : null,
    status,
    reload,
    save,
    refreshAll,
  };
};
