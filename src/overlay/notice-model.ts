// Pure rules of the transient notices on the recording indicator: the AI failure of 005
// (AC-005), the Screen Sharing hand-over of 007 (AC-012), the recording cut short by a lost
// microphone of 001 (FR-004) and the failed insertion (`paste-error`, native session N17). Each
// stays at least four seconds; none ever claims that an insertion succeeded. The destination
// reason of 006 is presented by 010's recovery bubble instead; `paste-error` is emitted only when
// no bubble shows the failure, so a failure is reported once.
import type { OverlayStyle, PromptProcessingFailed } from "@/bindings";

/** Minimum display time of every indicator notice (005 AC-005, 007 AC-012). */
export const NOTICE_MIN_MS = 4000;

/** Which feature raised a notice; a newer notice of the same kind replaces the older one. */
export type NoticeKind =
  | "ai_failed"
  | "hand_over"
  | "input_lost"
  | "paste_failed";

/** Recording error event of the backend (`managers::audio_priority`, `actions`). */
export const RECORDING_ERROR_EVENT = "recording-error";
/** Failed insertion event of the backend (`destination::delivery`, `smart_paste::app`, 010). */
export const PASTE_ERROR_EVENT = "paste-error";
/** Payload of `paste-error`: whether the transcript is in history (`null` from the legacy paths). */
export type PasteErrorPayload = { history_saved: boolean } | null;

/** `error_type` of the `recording-error` event of a recording cut short by a lost microphone. */
export const INPUT_LOST_ERROR = "input_lost";

/** One notice on the indicator. `appName` fills `{{app}}`; `null` names no application. */
export interface OverlayNotice {
  kind: NoticeKind;
  key: string;
  appName: string | null;
  until: number;
  testId: string;
}

// @spec AC-005: AI failure notice at least 4 s — .specs/features/005-custom-ai-prompts/spec.md#ac-005
/** Notice of the AI step failure: the original text was used; nothing about the insertion. */
export const aiFailureNotice = (
  event: PromptProcessingFailed,
  now: number,
): OverlayNotice => ({
  kind: "ai_failed",
  key: "overlay.fallbackNotice",
  appName: null,
  until: now + Math.max(event.notice_ms, NOTICE_MIN_MS),
  testId: "recording-indicator-ai-failed-notice",
});

// @spec AC-012: Hand-over message on the indicator — .specs/features/007-smart-paste/spec.md#ac-012
/** Notice of the Screen Sharing hand-over (007): text placed on the clipboard. */
export const handOverNotice = (now: number): OverlayNotice => ({
  kind: "hand_over",
  key: "smartPaste.handOver",
  appName: null,
  until: now + NOTICE_MIN_MS,
  testId: "recording-indicator-hand-over-notice",
});

// @spec FR-004: The user is told the recording was cut short — .specs/features/001-microphone-priority/spec.md#fr-004
/** Notice of a recording cut short by a lost microphone (001): the captured audio is transcribed. */
export const inputLostNotice = (now: number): OverlayNotice => ({
  kind: "input_lost",
  key: "errors.inputLost",
  appName: null,
  until: now + NOTICE_MIN_MS,
  testId: "recording-indicator-input-lost-notice",
});

// @spec FR-002: The bubble replaces paste-error, never both — .specs/features/010-recovery-bubble/spec.md#fr-002
/**
 * Notice of a failed insertion when no recovery bubble shows it (N17): never reserved to the
 * settings window, which may be closed. Says when the text is not in history either.
 * @param payload `paste-error` payload
 * @param now current time (ms)
 * @returns the notice
 */
export const pasteFailedNotice = (
  payload: PasteErrorPayload,
  now: number,
): OverlayNotice => ({
  kind: "paste_failed",
  key: pasteFailedKey(payload),
  appName: null,
  until: now + NOTICE_MIN_MS,
  testId: "recording-indicator-paste-failed-notice",
});

/**
 * Message of a failed insertion: not saved to history either, or only not pasted.
 * @param payload `paste-error` payload
 * @returns the translation key
 */
export const pasteFailedKey = (payload: PasteErrorPayload): string =>
  payload?.history_saved === false
    ? "errors.pasteFailedNotSaved"
    : "errors.pasteFailed";

/**
 * Whether the settings window shows an indicator notice as a toast: only when the overlay style
 * is none (no indicator window). Otherwise the indicator alone shows it, never both.
 * @param style overlay style of the settings, `undefined` before they load
 * @returns true when the toast replaces the indicator notice
 */
export const noticeFallsBackToToast = (
  style: OverlayStyle | undefined,
): boolean => style === "none";

/** Add `notice`, replacing an older notice of the same kind and dropping expired ones. */
export const addNotice = (
  notices: readonly OverlayNotice[],
  notice: OverlayNotice,
  now: number,
): OverlayNotice[] => [
  ...liveNotices(notices, now).filter((item) => item.kind !== notice.kind),
  notice,
];

/** Notices still inside their display time. */
export const liveNotices = (
  notices: readonly OverlayNotice[],
  now: number,
): OverlayNotice[] => notices.filter((notice) => notice.until > now);

/** How long the window must stay up for the notices (0 when none is left). */
export const holdRemaining = (
  notices: readonly OverlayNotice[],
  now: number,
): number =>
  notices.reduce((longest, notice) => Math.max(longest, notice.until - now), 0);

/** Milliseconds until the next notice expires, or `null` without notices. */
export const nextExpiry = (
  notices: readonly OverlayNotice[],
  now: number,
): number | null =>
  notices.length === 0
    ? null
    : Math.max(0, Math.min(...notices.map((notice) => notice.until - now)));
