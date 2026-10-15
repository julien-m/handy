// Pure model of the recovery bubble (010): what the reason line and the explanation say for one
// projection, and when a pointer gesture on the handle becomes a native drag. The projection
// comes from the native `RecoveryManager`; the full transcript never reaches this window.
import type { RecoveryBubbleView } from "@/bindings";

/** Icon of the reason line. */
export type ReasonIcon = "warning" | "error" | "info";
/** One translated sentence of the explanation. */
export interface ExplanationLine {
  key: string;
  values?: Record<string, string>;
}
/** Everything the bubble says for one projection. */
export interface BubbleTexts {
  reasonKey: string;
  reasonValues: Record<string, string>;
  icon: ReasonIcon;
  /** Shown in the bubble itself, not only in the explanation: a late read means the text may
   * have been pasted after all, so the user checks before copying again. */
  lateReadKey: string | null;
  explanation: ExplanationLine[];
}

const KEY = "recoveryBubble";
/** Reasons that name the destination application. */
const NAMED: Partial<Record<RecoveryBubbleView["reason"], string>> = {
  app_closed: "appClosed",
  pinned_app_not_running: "pinnedAppNotRunning",
  activation_failed: "activationFailed",
};
const ICONS: Record<RecoveryBubbleView["reason"], ReasonIcon> = {
  app_closed: "warning",
  pinned_app_not_running: "warning",
  activation_failed: "warning",
  insertion_failed: "error",
  no_text_field: "info",
};

/** Pixels the pointer moves on the handle, button held, before the native drag starts. */
export const DRAG_THRESHOLD_PX = 3;

// @spec AC-003: The reason in one sentence — .specs/features/010-recovery-bubble/spec.md#ac-003
const reasonLine = (
  view: RecoveryBubbleView,
): { key: string; values: Record<string, string> } => {
  const named = NAMED[view.reason];
  if (named === undefined) {
    const plain =
      view.reason === "insertion_failed" ? "insertionFailed" : "noTextField";
    return { key: `${KEY}.reason.${plain}`, values: {} };
  }
  if (view.app_name === null) {
    return { key: `${KEY}.reason.${named}Unnamed`, values: {} };
  }
  return { key: `${KEY}.reason.${named}`, values: { app: view.app_name } };
};

const line = (key: string, values?: Record<string, string>): ExplanationLine =>
  values === undefined
    ? { key: `${KEY}.explanation.${key}` }
    : { key: `${KEY}.explanation.${key}`, values };

/** Where the user can put the text: open the app (named or not), or click in a field. */
const advice = (view: RecoveryBubbleView): ExplanationLine[] => {
  const ending = view.can_drag ? "Drag" : "Copy";
  if (view.reason === "no_text_field") {
    return [line(`focusField${ending}`), line("noFieldSetting")];
  }
  if (view.app_name === null) return [line(`openUnnamed${ending}`)];
  return [line(`openApp${ending}`, { app: view.app_name })];
};

// @spec AC-005: History said kept only when saved — .specs/features/010-recovery-bubble/spec.md#ac-005
const history = (view: RecoveryBubbleView, full: boolean): ExplanationLine => {
  if (view.history === "not_kept") return line("historyNotKept");
  return line(full ? "historyKeptFull" : "historyKept");
};

// @spec AC-001: Partial-insertion detail only for method failures — .specs/features/010-recovery-bubble/spec.md#ac-001
const explanation = (view: RecoveryBubbleView): ExplanationLine[] => {
  if (view.detail === "not_inserted") {
    return [line("notInserted"), history(view, false), ...advice(view)];
  }
  const sent =
    view.detail === "nothing_sent" ? "nothingSent" : "mayHaveReached";
  return [line(sent), history(view, true)];
};

/** The texts of `view`. */
export const bubbleTexts = (view: RecoveryBubbleView): BubbleTexts => {
  const reason = reasonLine(view);
  return {
    reasonKey: reason.key,
    reasonValues: reason.values,
    icon: ICONS[view.reason],
    // @spec AC-001: Late read visible in the bubble — .specs/features/010-recovery-bubble/spec.md#ac-001
    lateReadKey: view.late_read ? `${KEY}.explanation.lateRead` : null,
    explanation: explanation(view),
  };
};

/** A pointer point in CSS pixels. */
export interface Point {
  x: number;
  y: number;
}

// @spec FR-003: Drag gesture on the handle — .specs/features/010-recovery-bubble/spec.md#fr-003
/** True when the primary button is held and the pointer left the threshold around `start`. */
export const dragStarts = (
  start: Point | null,
  point: Point,
  buttons: number,
): boolean => {
  if (start === null || (buttons & 1) === 0) return false;
  return Math.hypot(point.x - start.x, point.y - start.y) > DRAG_THRESHOLD_PX;
};
