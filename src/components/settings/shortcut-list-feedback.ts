// Pure presentation rules of a shortcut list: what the list shows after an
// edit (success, inline refusal, or a system refusal with retry), the
// warnings of inherited conflicts, the limit hints and the stable test ids
// of the Penflow contract (shortcut-settings screens).
import type {
  ShortcutEdit,
  ShortcutEditError,
  StoredShortcutConflict,
} from "@/bindings";
import type { OSType } from "../../lib/utils/keyboard";
import type { DraftState, RecorderStartFailure } from "./shortcut-draft-model";
import type { ShortcutScreen } from "./shortcut-screen";
import {
  MAX_SHORTCUTS_PER_ACTION,
  type TranslatedMessage,
  describeEditError,
} from "./shortcut-list-model";

const LIST_KEY = "settings.general.shortcut.list";

/** Visual state of a list (Behavioral AC async_action). */
export type ListState = "idle" | "loading" | "error" | "success";

/** The system refused to register a shortcut; the edit can be retried. */
export interface SystemRefusal {
  /** Refused shortcut in stored syntax, or null for a reset. */
  shortcut: string | null;
  /** The system's own message, in English. */
  reason: string;
  edit: ShortcutEdit;
}

/** What a list shows about its last edit. */
export interface ListFeedback {
  state: ListState;
  /** Inline refusal shown under the list, if any. */
  error: TranslatedMessage | null;
  systemRefusal: SystemRefusal | null;
}

export const IDLE_FEEDBACK: ListFeedback = {
  state: "idle",
  error: null,
  systemRefusal: null,
};

export const LOADING_FEEDBACK: ListFeedback = {
  state: "loading",
  error: null,
  systemRefusal: null,
};

/** The edit call itself failed: no typed reason, nothing known to be saved. */
export const UNEXPECTED_FEEDBACK: ListFeedback = {
  state: "error",
  error: { key: `${LIST_KEY}.errors.generic`, params: {} },
  systemRefusal: null,
};

/** Translates names for messages: action ids and stored shortcuts. */
export interface MessageNames {
  actionName: (actionId: string) => string;
  shortcutLabel: (shortcut: string) => string;
}

/**
 * The shortcut an edit adds or changes, or null for a removal or a reset.
 * @param edit The edit.
 */
export const candidateOf = (edit: ShortcutEdit): string | null =>
  edit.kind === "add" || edit.kind === "replace" ? edit.shortcut : null;

/**
 * Inline message for a refused shortcut. A conflict names the candidate and
 * the action that owns it, and says that nothing was saved.
 * @param refusal Typed refusal of the check or of the edit.
 * @param candidate Refused shortcut in stored syntax, if known.
 * @param names Display names.
 */
export const describeRefusal = (
  refusal: ShortcutEditError,
  candidate: string | null,
  names: MessageNames,
): TranslatedMessage => {
  if (refusal.code !== "shortcut_conflict" || candidate === null) {
    return describeEditError(refusal, names.actionName);
  }
  const isOverlap = refusal.conflict.kind === "modifier_overlap";
  return {
    key: `${LIST_KEY}.${isOverlap ? "draftOverlap" : "draftConflict"}`,
    params: {
      shortcut: names.shortcutLabel(candidate),
      action: names.actionName(refusal.conflict.owning_action),
    },
  };
};

/**
 * What the list shows after an edit: success, a system refusal (alert with
 * retry), or an inline refusal. Saved shortcuts are unchanged on refusal.
 * @param edit The edit that was submitted.
 * @param refusal Its refusal, or null when it was saved.
 * @param names Display names.
 */
export const feedbackAfterEdit = (
  edit: ShortcutEdit,
  refusal: ShortcutEditError | null,
  names: MessageNames,
): ListFeedback => {
  if (refusal === null) return { ...IDLE_FEEDBACK, state: "success" };
  if (refusal.code === "registration_failed") {
    const systemRefusal = {
      shortcut: candidateOf(edit),
      reason: refusal.message,
      edit,
    };
    return { state: "error", error: null, systemRefusal };
  }
  const error = describeRefusal(refusal, candidateOf(edit), names);
  return { state: "error", error, systemRefusal: null };
};

/**
 * The system refusal of the draft's last Add, while the draft still shows it
 * (refused, not recorded again nor discarded): its alert and Try again stay
 * available even after another edit of the list replaced the feedback.
 * @param state The draft.
 * @returns The refusal to show, or null.
 */
export const draftSystemRefusal = (state: DraftState): SystemRefusal | null => {
  const { phase, refusal, shortcut } = state;
  if (phase !== "invalid" || shortcut === null) return null;
  if (refusal?.code !== "registration_failed") return null;
  return { shortcut, reason: refusal.message, edit: { kind: "add", shortcut } };
};

/**
 * Whether two edits are the same request (same kind and shortcuts).
 * @param edit An edit.
 * @param other Another edit, if any.
 */
export const isSameEdit = (
  edit: ShortcutEdit,
  other: ShortcutEdit | undefined,
): boolean => JSON.stringify(edit) === JSON.stringify(other);

/**
 * Inline reason of a recorder that could not start: macOS Secure Input, or
 * the backend's own message.
 * @param failure Why the recorder did not start.
 */
export const describeStartFailure = (
  failure: RecorderStartFailure,
): TranslatedMessage =>
  failure.kind === "secure_input"
    ? { key: "secureInput.recorderBlocked", params: {} }
    : {
        key: "settings.general.shortcut.errors.set",
        params: { error: failure.message },
      };

/** Warning of a stored shortcut that conflicts since before the update. */
export interface LegacyWarning {
  shortcut: string;
  title: TranslatedMessage;
}

/**
 * Inherited conflict warnings of one action, one per stored shortcut that
 * conflicts with another action's (or its own) stored shortcut.
 * @param conflicts Conflicts from `get_shortcut_conflicts`.
 * @param actionId Action whose list is shown.
 * @param names Display names.
 */
export const legacyWarningsOf = (
  conflicts: ReadonlyArray<StoredShortcutConflict>,
  actionId: string,
  names: MessageNames,
): LegacyWarning[] =>
  conflicts
    .filter((entry) => entry.action_id === actionId)
    .map((entry) => ({
      shortcut: entry.shortcut,
      title: {
        key: `${LIST_KEY}.${entry.conflict.kind === "modifier_overlap" ? "legacyOverlap" : "legacyDuplicate"}`,
        params: {
          shortcut: names.shortcutLabel(entry.shortcut),
          action: names.actionName(entry.conflict.owning_action),
        },
      },
    }));

/** Hint under a list: why Add or Remove is unavailable. */
export type ListHint = "last" | "limit" | null;

/**
 * Hint for a list of `count` shortcuts: the limit when full, the
 * keep-one rule when a single shortcut is left.
 * @param count Number of stored shortcuts.
 * @param limit Most shortcuts the action may have.
 */
export const listHint = (
  count: number,
  limit: number = MAX_SHORTCUTS_PER_ACTION,
): ListHint => {
  if (count >= limit) return "limit";
  if (count <= 1) return "last";
  return null;
};

/** Product names of the systems; they are not translated. */
const SYSTEM_NAMES: Partial<Record<OSType, string>> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux",
};

/**
 * Title of a system refusal: the system and the refused shortcut, or a
 * generic title when either is unknown.
 * @param os Platform of the settings window.
 * @param shortcut Refused shortcut, already formatted, or null.
 */
export const systemRefusalTitle = (
  os: OSType,
  shortcut: string | null,
): TranslatedMessage => {
  const system = SYSTEM_NAMES[os];
  if (system && shortcut) {
    return { key: `${LIST_KEY}.systemRefused`, params: { system, shortcut } };
  }
  return { key: `${LIST_KEY}.systemRefusedGeneric`, params: {} };
};

/** Contract name of each action in the shortcut-settings test ids. */
const ACTION_SLUGS: Record<string, string> = {
  transcribe: "dictation",
  transcribe_with_post_process: "ai-dictation",
  cancel: "cancel",
};

/**
 * Stable test ids of one action's list, `<screen>-<action>-…`: every element
 * carries the prefix of the contract screen shown (base, conflict or limit),
 * so each test id is unique in the tree.
 * @param actionId Action id.
 * @param screen Test-id prefix of the screen shown.
 */
export const shortcutTestIds = (
  actionId: string,
  screen: ShortcutScreen = "shortcut-settings",
) => {
  const slug = ACTION_SLUGS[actionId] ?? actionId;
  const base = `${screen}-${slug}`;
  return {
    container: base,
    list: `${base}-list`,
    shortcut: `${base}-shortcut`,
    remove: `${base}-shortcut-remove`,
    refused: `${base}-shortcut-refused`,
    conflictIcon: `${base}-shortcut-conflict`,
    recording: `${base}-recording`,
    discard: `${base}-recording-discard`,
    unsaved: `${base}-recording-unsaved`,
    add: `${base}-add`,
    reset: `${base}-reset`,
    last: `${base}-last`,
    conflictsUnavailable: `${base}-conflicts-unavailable`,
    limit: `${base}-limit`,
    error: `${base}-conflict`,
    legacy: `${base}-legacy`,
    change: `${base}-change`,
    system: `${base}-os`,
    retry: `${base}-os-retry`,
  };
};

export type ShortcutTestIds = ReturnType<typeof shortcutTestIds>;

/**
 * Instance id of one stored shortcut (`instance_id_template` `shortcut-{keys}`
 * of the contract): the stored binding, never text typed by the user.
 * @param shortcut Stored shortcut, for example `option+space`.
 */
export const shortcutInstanceId = (shortcut: string): string =>
  `shortcut-${shortcut}`;
