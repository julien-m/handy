// Pure state machine of the "Add shortcut" draft of one action: the recorder
// opens empty, captures a combination, is checked by the backend without
// saving, and only a valid draft can be added. The settings list renders the
// recorder and the Add control from these states.
import type { ShortcutEditError } from "@/bindings";

/** Where the draft stands; `closed` means no recorder is shown. */
export type DraftPhase =
  | "closed"
  | "recording"
  | "checking"
  | "valid"
  | "invalid"
  | "committing";

/** The draft recorder of one action. */
export interface DraftState {
  phase: DraftPhase;
  /** Captured combination in stored syntax, or null before a capture. */
  shortcut: string | null;
  /** Why the draft cannot be added (check or commit refusal). */
  refusal: ShortcutEditError | null;
}

export type DraftEvent =
  | { type: "open" }
  | { type: "captured"; shortcut: string }
  | { type: "checked"; shortcut: string; refusal: ShortcutEditError | null }
  | { type: "stopped" }
  | { type: "commit" }
  | { type: "committed" }
  // A null refusal means the call itself failed (no typed reason).
  | { type: "commitFailed"; refusal: ShortcutEditError | null }
  | { type: "discard" };

/** Visual state of the draft recorder (Behavioral AC has_validation). */
export type RecorderValidity = "empty" | "valid" | "invalid";

/** Visual state of the Add control (Behavioral AC is_submittable). */
export type AddControlState = "enabled" | "disabled" | "loading";

export const CLOSED_DRAFT: DraftState = {
  phase: "closed",
  shortcut: null,
  refusal: null,
};

const RECORDING_DRAFT: DraftState = {
  phase: "recording",
  shortcut: null,
  refusal: null,
};

/**
 * Next draft after `event`. Events that make no sense in the current phase
 * leave it unchanged, so late answers can never resurrect a closed draft.
 * @param state Current draft.
 * @param event What happened.
 * @returns The next draft.
 */
export const draftReducer = (
  state: DraftState,
  event: DraftEvent,
): DraftState => {
  switch (event.type) {
    case "open":
      return state.phase === "closed" ? RECORDING_DRAFT : state;
    case "captured":
      return state.phase === "closed" || state.phase === "committing"
        ? state
        : { phase: "checking", shortcut: event.shortcut, refusal: null };
    case "checked":
      return afterCheck(state, event.shortcut, event.refusal);
    case "stopped":
      // Recording ended without a capture (focus left the recorder): an
      // empty draft closes, a captured one stays for Add or discard.
      return state.phase === "recording" ? CLOSED_DRAFT : state;
    default:
      return afterCommitEvent(state, event);
  }
};

/**
 * Applies a backend check answer. Business rule: an answer for an older
 * capture (the user recorded again meanwhile) is ignored.
 */
const afterCheck = (
  state: DraftState,
  shortcut: string,
  refusal: ShortcutEditError | null,
): DraftState => {
  if (state.phase !== "checking" || state.shortcut !== shortcut) return state;
  return { phase: refusal ? "invalid" : "valid", shortcut, refusal };
};

const afterCommitEvent = (state: DraftState, event: DraftEvent): DraftState => {
  switch (event.type) {
    case "commit":
      return canCommit(state)
        ? { ...state, phase: "committing", refusal: null }
        : state;
    case "committed":
    case "discard":
      return CLOSED_DRAFT;
    case "commitFailed":
      return state.phase === "committing"
        ? { ...state, phase: "invalid", refusal: event.refusal }
        : state;
    default:
      return state;
  }
};

/**
 * Whether the draft can be committed: a valid draft, or one whose last
 * commit failed for a reason worth retrying (system or settings service).
 * @param state Current draft.
 */
export const canCommit = (state: DraftState): boolean =>
  state.phase === "valid" ||
  (state.phase === "invalid" && isRetryableRefusal(state.refusal));

/**
 * Refusals that may succeed unchanged on a second attempt: the system's
 * registration (another app may release the combination) and the settings
 * service (busy dictation, persistence). Validation refusals never change.
 * @param refusal Refusal of an edit.
 */
export const isRetryableRefusal = (
  refusal: ShortcutEditError | null,
): boolean =>
  refusal?.code === "registration_failed" ||
  refusal?.code === "settings_refused";

/**
 * Visual state of the draft recorder, or undefined when it shows no
 * validation state (closed, or waiting for the check or the commit).
 * @param state Current draft.
 */
export const recorderValidity = (
  state: DraftState,
): RecorderValidity | undefined => {
  switch (state.phase) {
    case "recording":
      return "empty";
    case "valid":
      return "valid";
    case "invalid":
      return "invalid";
    default:
      return undefined;
  }
};

/**
 * Visual state of the Add control. Without a draft, Add opens the recorder
 * when the action has room; with a draft, only a valid one can be added.
 * @param state Current draft.
 * @param hasRoom Whether the action is below its shortcut limit.
 * @param isBusy Whether another edit of the action is in progress.
 */
export const addControlState = (
  state: DraftState,
  hasRoom: boolean,
  isBusy: boolean,
): AddControlState => {
  if (state.phase === "committing") return "loading";
  if (!hasRoom || isBusy) return "disabled";
  return state.phase === "closed" || state.phase === "valid"
    ? "enabled"
    : "disabled";
};
