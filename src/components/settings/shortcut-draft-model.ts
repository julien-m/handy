// Pure state machine of the "Add shortcut" draft of one action: the recorder
// opens empty, captures a combination, is checked by the backend without
// saving, and only a valid draft can be added. The settings list renders the
// recorder and the Add control from these states. A recorder the system
// refuses to start (macOS Secure Input) leaves the draft "blocked": empty, with
// the reason shown and Discard as a way out. While a draft is open, Add gives
// way to explicit Save and Discard buttons and Reset is unavailable, so a draft
// is never mistaken for a saved shortcut nor cleared without a word (native
// session finding N8).
import type { ShortcutEditError } from "@/bindings";

/**
 * Where the draft stands; `closed` means no recorder is shown and `blocked`
 * that the recorder could not start.
 */
export type DraftPhase =
  | "closed"
  | "recording"
  | "blocked"
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
  /** Why the recorder did not start on its last attempt, if it did not. */
  startFailure: RecorderStartFailure | null;
}

/** Why a recorder could not start recording. */
export type RecorderStartFailure =
  | { kind: "secure_input" }
  | { kind: "failed"; message: string };

export type DraftEvent =
  | { type: "open" }
  | { type: "captured"; shortcut: string }
  | { type: "checked"; shortcut: string; refusal: ShortcutEditError | null }
  | { type: "stopped" }
  | { type: "startFailed"; failure: RecorderStartFailure }
  | { type: "recordingStarted" }
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
  startFailure: null,
};

const RECORDING_DRAFT: DraftState = { ...CLOSED_DRAFT, phase: "recording" };

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
        : { ...CLOSED_DRAFT, phase: "checking", shortcut: event.shortcut };
    case "checked":
      return afterCheck(state, event.shortcut, event.refusal);
    case "stopped":
      // Recording ended without a capture (focus left the recorder): an
      // empty draft closes, a captured one stays for Add or discard.
      return state.phase === "recording" ? CLOSED_DRAFT : state;
    default:
      return afterRecorderEvent(state, event);
  }
};

/**
 * Applies a recorder start answer. Business rule: a refused start never
 * leaves an empty draft "recording" with no way out; it becomes `blocked`
 * (empty, reason shown, Discard offered) until a start succeeds. A captured
 * draft keeps its combination and only shows the reason.
 */
const afterRecorderEvent = (
  state: DraftState,
  event: DraftEvent,
): DraftState => {
  if (state.phase === "closed" || state.phase === "committing") {
    return afterCommitEvent(state, event);
  }
  switch (event.type) {
    case "startFailed":
      return state.phase === "recording" || state.phase === "blocked"
        ? { ...state, phase: "blocked", startFailure: event.failure }
        : { ...state, startFailure: event.failure };
    case "recordingStarted":
      return state.phase === "blocked"
        ? { ...state, phase: "recording", startFailure: null }
        : { ...state, startFailure: null };
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
  const phase = refusal ? "invalid" : "valid";
  return { ...CLOSED_DRAFT, phase, shortcut, refusal };
};

const afterCommitEvent = (state: DraftState, event: DraftEvent): DraftState => {
  switch (event.type) {
    case "commit":
      return canCommit(state)
        ? { ...state, phase: "committing", refusal: null, startFailure: null }
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
    case "blocked":
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

/** Controls under a shortcut list (native session finding N8). */
export interface ListControls {
  /** `add` shows "Add shortcut"; `draft` shows Save and Discard instead. */
  mode: "add" | "draft";
  /** State of Add, or of Save during a draft: only a valid draft is saved. */
  submit: AddControlState;
  /** Discard is offered for an open draft, except while it is being saved. */
  canDiscard: boolean;
  /** Reset never clears a draft: it waits until the draft is saved or discarded. */
  isResetEnabled: boolean;
}

/**
 * Controls under the list for the current draft.
 * @param state Current draft.
 * @param hasRoom Whether the action is below its shortcut limit.
 * @param isBusy Whether another edit of the action is in progress.
 * @returns Which buttons show and which are enabled.
 */
export const listControls = (
  state: DraftState,
  hasRoom: boolean,
  isBusy: boolean,
): ListControls => {
  const isDraftOpen = state.phase !== "closed";
  return {
    mode: isDraftOpen ? "draft" : "add",
    submit: addControlState(state, hasRoom, isBusy),
    canDiscard: isDraftOpen && state.phase !== "committing",
    isResetEnabled: !isDraftOpen && !isBusy,
  };
};
