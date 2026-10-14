// One-shot requests to start a shortcut recorder (004): "Change" on an
// inherited conflict asks one row to record again, "Add" asks the draft
// recorder to start. A request is consumed once its recorder handled it, and
// dropped when its target leaves the list, so a recorder that mounts later
// (a row re-added by the CLI, a keyboard implementation switch) never starts
// recording on its own.

/** A pending request to record `target` (a stored shortcut, or the draft). */
export interface RecordRequest {
  target: string;
  /** Unique per request, never reused, so a recorder handles each one once. */
  count: number;
}

/** The pending request, if any, and how many requests were issued. */
export interface RecordRequests {
  request: RecordRequest | null;
  issued: number;
}

export const NO_RECORD_REQUEST: RecordRequests = { request: null, issued: 0 };

/**
 * Issues a new request for `target`, replacing any pending one.
 * @param state Current requests.
 * @param target Recorder that must start.
 * @returns The requests with the new one pending.
 */
export const requestRecording = (
  state: RecordRequests,
  target: string,
): RecordRequests => {
  const issued = state.issued + 1;
  return { request: { target, count: issued }, issued };
};

/**
 * Consumes request `count` once its recorder handled it (started, or ignored
 * it because it was disabled). A newer request stays pending.
 * @param state Current requests.
 * @param count The handled request.
 */
export const markRequestHandled = (
  state: RecordRequests,
  count: number,
): RecordRequests =>
  state.request?.count === count ? { ...state, request: null } : state;

/**
 * Drops the pending request when its target is no longer listed.
 * @param state Current requests.
 * @param targets Recorders currently listed.
 */
export const dropUnlistedRequest = (
  state: RecordRequests,
  targets: ReadonlyArray<string>,
): RecordRequests =>
  state.request && !targets.includes(state.request.target)
    ? { ...state, request: null }
    : state;

/**
 * Request number a recorder receives: the pending one if it targets it, else
 * 0 (no request).
 * @param state Current requests.
 * @param target The recorder.
 */
export const requestFor = (state: RecordRequests, target: string): number =>
  state.request?.target === target ? state.request.count : 0;

/**
 * Whether a recorder must act on `request`: non-zero and not the one it
 * handled last (a re-render, or StrictMode's second effect run, never
 * replays it).
 * @param request Request number the recorder receives.
 * @param handled Last request number it handled.
 */
export const isNewRecordRequest = (request: number, handled: number): boolean =>
  request !== 0 && request !== handled;
