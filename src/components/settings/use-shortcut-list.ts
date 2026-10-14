// State of one action's shortcut list: its stored conflicts, the edits sent
// to the backend with their feedback, the "Add shortcut" draft, and the
// requests to record a row again ("Change" on an inherited conflict).
import {
  type Dispatch,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import type {
  ShortcutEdit,
  ShortcutEditError,
  StoredShortcutConflict,
} from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import { formatKeyCombination } from "../../lib/utils/keyboard";
import {
  CLOSED_DRAFT,
  type DraftEvent,
  type DraftState,
  draftReducer,
} from "./shortcut-draft-model";
import {
  IDLE_FEEDBACK,
  LOADING_FEEDBACK,
  type ListFeedback,
  type MessageNames,
  UNEXPECTED_FEEDBACK,
  feedbackAfterEdit,
} from "./shortcut-list-feedback";

export interface StoredConflicts {
  conflicts: StoredShortcutConflict[];
  /** Whether the last refresh failed; the last known list is kept. */
  failed: boolean;
}

// @spec AC-002: Legacy conflict warnings stay current — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/**
 * Stored conflicts, refreshed whenever any action's shortcuts or the
 * keyboard implementation change, so a conflict resolved from another
 * action's recorder clears its warning here too.
 * @param bindingsKey Serialized shortcuts of every action.
 * @param implementation Active keyboard implementation.
 */
export const useStoredConflicts = (
  bindingsKey: string,
  implementation: string | undefined,
): StoredConflicts => {
  const [state, setState] = useState<StoredConflicts>({
    conflicts: [],
    failed: false,
  });
  useEffect(() => {
    let isCurrent = true;
    commands
      .getShortcutConflicts()
      .then((conflicts) => {
        if (isCurrent) setState({ conflicts, failed: false });
      })
      .catch((failure) => {
        console.error("Failed to load shortcut conflicts:", failure);
        if (isCurrent) setState((previous) => ({ ...previous, failed: true }));
      });
    return () => {
      isCurrent = false;
    };
  }, [bindingsKey, implementation]);
  return state;
};

/** Display names for messages, in the current language and platform. */
export const useMessageNames = (): MessageNames => {
  const { t } = useTranslation();
  const os = useOsType();
  return useMemo(
    () => ({
      actionName: (id: string) =>
        t(`settings.general.shortcut.bindings.${id}.name`, id),
      shortcutLabel: (shortcut: string) => formatKeyCombination(shortcut, os),
    }),
    [t, os],
  );
};

/** Result of one submitted edit. */
export interface SubmitOutcome {
  saved: boolean;
  /** Typed refusal, or null when saved or when the call itself failed. */
  refusal: ShortcutEditError | null;
}

/** Edits of one action and what the list shows about the last one. */
export interface EditSubmitter {
  feedback: ListFeedback;
  submit: (edit: ShortcutEdit) => Promise<SubmitOutcome>;
  clear: () => void;
}

// @spec AC-004: Rejections leave saved shortcuts unchanged — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
/**
 * Sends edits through `edit_binding_shortcuts`, which validates, registers
 * and saves, and keeps the feedback of the last one (async_action states).
 * @param actionId Action whose list is edited.
 * @param names Display names for refusal messages.
 */
export const useEditSubmitter = (
  actionId: string,
  names: MessageNames,
): EditSubmitter => {
  const { editBindingShortcuts } = useSettings();
  const [feedback, setFeedback] = useState<ListFeedback>(IDLE_FEEDBACK);
  const submit = useCallback(
    async (edit: ShortcutEdit): Promise<SubmitOutcome> => {
      setFeedback(LOADING_FEEDBACK);
      try {
        const refusal = await editBindingShortcuts(actionId, edit);
        setFeedback(feedbackAfterEdit(edit, refusal, names));
        return { saved: refusal === null, refusal };
      } catch (unexpected) {
        console.error("Failed to edit shortcuts:", unexpected);
        setFeedback(UNEXPECTED_FEEDBACK);
        return { saved: false, refusal: null };
      }
    },
    [actionId, editBindingShortcuts, names],
  );
  const clear = useCallback(() => setFeedback(IDLE_FEEDBACK), []);
  return { feedback, submit, clear };
};

/**
 * Backend check of a recorded draft, without saving. A failed call is
 * logged and treated as "no refusal known": Add then sends the real edit,
 * which runs the same checks before anything is saved.
 */
const checkDraft = async (
  actionId: string,
  shortcut: string,
): Promise<ShortcutEditError | null> => {
  try {
    return await commands.checkBindingShortcutEdit(actionId, {
      kind: "add",
      shortcut,
    });
  } catch (failure) {
    console.error("Failed to check the recorded shortcut:", failure);
    return null;
  }
};

/** The "Add shortcut" draft of one action and its controls. */
export interface ShortcutDraft {
  state: DraftState;
  /** Increments each time the recorder must start (Add pressed). */
  recordRequest: number;
  open: () => void;
  capture: (shortcut: string) => Promise<void>;
  stop: () => void;
  commit: () => Promise<void>;
  discard: () => void;
}

// @spec AC-001: Add records with the existing recorder — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/**
 * Draft recorder of the Add control: open, capture, check, then add.
 * @param actionId Action the draft is added to.
 * @param submitter Edits of the same action.
 */
export const useShortcutDraft = (
  actionId: string,
  submitter: EditSubmitter,
): ShortcutDraft => {
  const [state, dispatch] = useReducer(draftReducer, CLOSED_DRAFT);
  const [recordRequest, setRecordRequest] = useState(0);
  const { submit, clear } = submitter;
  const open = useCallback(() => {
    clear();
    dispatch({ type: "open" });
    setRecordRequest((count) => count + 1);
  }, [clear]);
  const capture = useCallback(
    async (shortcut: string) => {
      clear();
      dispatch({ type: "captured", shortcut });
      const refusal = await checkDraft(actionId, shortcut);
      dispatch({ type: "checked", shortcut, refusal });
    },
    [actionId, clear],
  );
  const commit = useDraftCommit(state, dispatch, submit);
  const stop = useCallback(() => dispatch({ type: "stopped" }), []);
  const discard = useCallback(() => {
    clear();
    dispatch({ type: "discard" });
  }, [clear]);
  return { state, recordRequest, open, capture, stop, commit, discard };
};

/** Adds the draft; a refusal keeps it, with its reason, for a retry or a new recording. */
const useDraftCommit = (
  state: DraftState,
  dispatch: Dispatch<DraftEvent>,
  submit: EditSubmitter["submit"],
): (() => Promise<void>) =>
  useCallback(async () => {
    if (state.shortcut === null) return;
    dispatch({ type: "commit" });
    const outcome = await submit({ kind: "add", shortcut: state.shortcut });
    if (outcome.saved) {
      dispatch({ type: "committed" });
    } else {
      dispatch({ type: "commitFailed", refusal: outcome.refusal });
    }
  }, [state.shortcut, dispatch, submit]);

/** A request to record one stored shortcut again. */
export interface RowRecordRequest {
  shortcut: string;
  count: number;
}

/**
 * Requests to record a stored shortcut again, sent by the "Change" button
 * of an inherited conflict warning.
 * @returns The current request and the function that sends one.
 */
export const useRowRecordRequest = (): [
  RowRecordRequest | null,
  (shortcut: string) => void,
] => {
  const [request, setRequest] = useState<RowRecordRequest | null>(null);
  const change = useCallback((shortcut: string) => {
    setRequest((previous) => ({
      shortcut,
      count: (previous?.count ?? 0) + 1,
    }));
  }, []);
  return [request, change];
};
