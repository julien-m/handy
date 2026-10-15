// Pure rules of the transient notices on the recording indicator: the AI failure of 005
// (AC-005), the destination reason of 006 (AC-005) and the Screen Sharing hand-over of 007
// (AC-012). Each stays at least four seconds; none ever claims that an insertion succeeded.
import type { PromptProcessingFailed } from "@/bindings";

/** Minimum display time of every indicator notice (005 AC-005, 006 AC-005). */
export const NOTICE_MIN_MS = 4000;

/** Which feature raised a notice; a newer notice of the same kind replaces the older one. */
export type NoticeKind = "ai_failed" | "destination" | "hand_over";

/** One notice on the indicator. `appName` fills `{{app}}`; `null` names no application. */
export interface OverlayNotice {
  kind: NoticeKind;
  key: string;
  appName: string | null;
  until: number;
  testId: string;
}

/** The fields of 006's `insertion-outcome` event the indicator reads (metadata only). */
export interface OutcomeFacts {
  dictationId: number;
  failed: boolean;
  reason:
    | "app_closed"
    | "activation_failed"
    | "insertion_failed"
    | "pinned_app_not_running"
    | null;
  historySaved: boolean;
}

const REASONS = [
  "app_closed",
  "activation_failed",
  "insertion_failed",
  "pinned_app_not_running",
] as const;

// @spec AC-003: Pinned app not running reason shown — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-003
const REASON_KEYS: Record<NonNullable<OutcomeFacts["reason"]>, string> = {
  app_closed: "appClosed",
  activation_failed: "activationFailed",
  insertion_failed: "insertionFailed",
  pinned_app_not_running: "pinnedAppNotRunning",
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;

const reasonOf = (value: unknown): OutcomeFacts["reason"] =>
  REASONS.find((reason) => reason === value) ?? null;

/** Read an `insertion-outcome` payload; `null` when it is not one (never trusted blindly). */
export const parseOutcome = (payload: unknown): OutcomeFacts | null => {
  if (!isRecord(payload) || typeof payload.dictation_id !== "number") {
    return null;
  }
  return {
    dictationId: payload.dictation_id,
    failed: payload.status === "failed",
    reason: reasonOf(payload.reason),
    historySaved: payload.history_status === "saved",
  };
};

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

// @spec AC-005: Reason shown at least 4 s — .specs/features/006-destination-application/spec.md#ac-005
/**
 * Notice of a destination failure (app closed, pinned app not running (009), activation failed,
 * insertion failed) saying
 * whether the transcript is in History; `null` when the outcome is not such a failure.
 */
export const destinationNotice = (
  facts: OutcomeFacts,
  appName: string | null,
  now: number,
): OverlayNotice | null => {
  if (!facts.failed || facts.reason === null) return null;
  const history = facts.historySaved ? "Kept" : "NotKept";
  return {
    kind: "destination",
    key: `overlay.destination.notice.${REASON_KEYS[facts.reason]}${history}`,
    appName,
    until: now + NOTICE_MIN_MS,
    testId: "recording-indicator-destination-unavailable-notice",
  };
};

// @spec AC-012: Hand-over message on the indicator — .specs/features/007-smart-paste/spec.md#ac-012
/** Notice of the Screen Sharing hand-over (007): text placed on the clipboard. */
export const handOverNotice = (now: number): OverlayNotice => ({
  kind: "hand_over",
  key: "smartPaste.handOver",
  appName: null,
  until: now + NOTICE_MIN_MS,
  testId: "recording-indicator-hand-over-notice",
});

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
