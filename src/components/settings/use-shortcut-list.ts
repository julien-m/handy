// State of one action's shortcut list: its stored conflicts, the edits sent
// to the backend with their feedback, the "Add shortcut" draft, and the
// one-shot requests to record a row again ("Change" on an inherited
// conflict).
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
  type RecorderStartFailure,
  canCommit,
  draftReducer,
} from "./shortcut-draft-model";
import {
  NO_RECORD_REQUEST,
  dropUnlistedRequest,
  markRequestHandled,
  requestFor,
  requestRecording,
} from "./record-request-model";
import {
  IDLE_FEEDBACK,
  LOADING_FEEDBACK,
  type ListFeedback,
  type MessageNames,
  UNEXPECTED_FEEDBACK,
  feedbackAfterEdit,
} from "./shortcut-list-feedback";
import { promptNameOfTarget } from "./shortcut-list-model";

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

/**
 * Display names for messages, in the current language and platform. A
 * prompt target (005) is named after its prompt, a dictation action by its
 * translated name.
 */
export const useMessageNames = (): MessageNames => {
  const { t } = useTranslation();
  const os = useOsType();
  const prompts = useSettings().getSetting("post_process_prompts");
  return useMemo(
    () => ({
      actionName: (id: string) => {
        const promptName = promptNameOfTarget(prompts ?? [], id);
        return promptName === null
          ? t(`settings.general.shortcut.bindings.${id}.name`, id)
          : t("settings.postProcessing.promptLibrary.shortcutOwner", {
              name: promptName,
            });
      },
      shortcutLabel: (shortcut: string) => formatKeyCombination(shortcut, os),
    }),
    [t, os, prompts],
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
  /** Non-zero while the recorder must start (Add pressed), until handled. */
  recordRequest: number;
  open: () => void;
  capture: (shortcut: string) => Promise<void>;
  stop: () => void;
  commit: () => Promise<void>;
  discard: () => void;
  /** The recorder reports a start request handled, started or refused. */
  recorder: RecorderReports;
}

/** What the draft recorder reports back to the draft. */
export interface RecorderReports {
  requestHandled: (count: number) => void;
  started: () => void;
  startFailed: (failure: RecorderStartFailure) => void;
}

const DRAFT_TARGET = "draft";

/** The draft recorder's start request (one-shot) and its reports. */
const useDraftRecorder = (
  dispatch: Dispatch<DraftEvent>,
): [number, () => void, RecorderReports] => {
  const [requests, setRequests] = useState(NO_RECORD_REQUEST);
  const request = useCallback(
    () => setRequests((state) => requestRecording(state, DRAFT_TARGET)),
    [],
  );
  const reports = useMemo<RecorderReports>(
    () => ({
      requestHandled: (count) =>
        setRequests((state) => markRequestHandled(state, count)),
      started: () => dispatch({ type: "recordingStarted" }),
      startFailed: (failure) => dispatch({ type: "startFailed", failure }),
    }),
    [dispatch],
  );
  return [requestFor(requests, DRAFT_TARGET), request, reports];
};

/** Records a combination in the draft, then has the backend check it. */
const useDraftCapture = (
  actionId: string,
  clear: () => void,
  dispatch: Dispatch<DraftEvent>,
): ((shortcut: string) => Promise<void>) =>
  useCallback(
    async (shortcut: string) => {
      clear();
      dispatch({ type: "captured", shortcut });
      const refusal = await checkDraft(actionId, shortcut);
      dispatch({ type: "checked", shortcut, refusal });
    },
    [actionId, clear, dispatch],
  );

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
  const [recordRequest, requestStart, recorder] = useDraftRecorder(dispatch);
  const { submit, clear } = submitter;
  const open = useCallback(() => {
    clear();
    dispatch({ type: "open" });
    requestStart();
  }, [clear, requestStart]);
  const capture = useDraftCapture(actionId, clear, dispatch);
  const commit = useDraftCommit(state, dispatch, submit);
  const stop = useCallback(() => dispatch({ type: "stopped" }), []);
  const discard = useCallback(() => {
    clear();
    dispatch({ type: "discard" });
  }, [clear]);
  const actions = { open, capture, stop, commit, discard };
  return { state, recordRequest, ...actions, recorder };
};

/** Adds the draft; a refusal keeps it, with its reason, for a retry or a new recording. */
const useDraftCommit = (
  state: DraftState,
  dispatch: Dispatch<DraftEvent>,
  submit: EditSubmitter["submit"],
): (() => Promise<void>) =>
  useCallback(async () => {
    // Only a committable draft is sent: two calls in one render (Add and
    // Try again) never submit the same shortcut twice.
    if (state.shortcut === null || !canCommit(state)) return;
    dispatch({ type: "commit" });
    const outcome = await submit({ kind: "add", shortcut: state.shortcut });
    if (outcome.saved) {
      dispatch({ type: "committed" });
    } else {
      dispatch({ type: "commitFailed", refusal: outcome.refusal });
    }
  }, [state, dispatch, submit]);

/** Requests to record a stored shortcut again ("Change"). */
export interface RowRecordRequests {
  /** Request number of the row showing `shortcut` (0: none). */
  requestFor: (shortcut: string) => number;
  change: (shortcut: string) => void;
  handled: (count: number) => void;
}

/**
 * Requests to record a stored shortcut again, sent by the "Change" button
 * of an inherited conflict warning. Each one is consumed once its row
 * handled it, and dropped when its shortcut leaves the list.
 * @param shortcuts Shortcuts the list shows.
 */
export const useRowRecordRequests = (
  shortcuts: ReadonlyArray<string>,
): RowRecordRequests => {
  const [requests, setRequests] = useState(NO_RECORD_REQUEST);
  const listed = shortcuts.join("\n");
  useEffect(() => {
    setRequests((state) => dropUnlistedRequest(state, listed.split("\n")));
  }, [listed]);
  return {
    requestFor: (shortcut) =>
      shortcuts.includes(shortcut) ? requestFor(requests, shortcut) : 0,
    change: (shortcut) =>
      setRequests((state) => requestRecording(state, shortcut)),
    handled: (count) =>
      setRequests((state) => markRequestHandled(state, count)),
  };
};
