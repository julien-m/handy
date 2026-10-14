// Availability and effective input of the microphone priority editor (feature 001), derived from
// the backend status read: what is configured stays apart from what is present and in use.
import type {
  AudioDevice,
  Availability,
  ChoiceStatus,
  EffectiveReason,
  EffectiveReport,
  MicrophoneStatus,
} from "@/bindings";
import { PRIORITY_KEY_ROOT, type TranslatedMessage } from "./draft-state";

/** Last status read of the editor. */
export interface StatusView {
  /** Last successful read, kept after a failed one (its effective input stays meaningful). */
  status: MicrophoneStatus | null;
  /** The last read failed or could not list the devices: every availability is unknown. */
  failed: boolean;
}

export const EMPTY_STATUS: StatusView = { status: null, failed: false };

/**
 * Merge one status read into the view.
 * @param previous view before the read
 * @param read status returned, or null when the command failed
 * @returns the new view; a failed command keeps the previous status but marks it failed
 */
export const mergeStatusRead = (
  previous: StatusView,
  read: MicrophoneStatus | null,
): StatusView =>
  read === null
    ? { status: previous.status, failed: true }
    : { status: read, failed: read.warning === "enumeration_failed" };

/**
 * Names the Add control offers: every enumerated device except the synthetic system default
 * entry that the device list prepends.
 * @param devices enumerated input devices
 * @returns their names, in enumeration order
 */
export const addableNames = (devices: readonly AudioDevice[]): string[] =>
  devices
    .filter((device) => !(device.is_default && device.index === "default"))
    .map((device) => device.name);

/**
 * Availability of one choice. The status read covers the committed order; a name only in the
 * draft is classified from the enumerated device names (one match available, several ambiguous).
 * @param name configured exact name
 * @param view last status read
 * @param enumerated names of the enumerated devices, or null when not listed yet
 * @returns the status shown beside the name
 */
export const availabilityOf = (
  name: string,
  view: StatusView,
  enumerated: readonly string[] | null,
): Availability => {
  if (view.failed) return "unknown";
  const read = view.status?.availability.find((entry) => entry.name === name);
  if (read) return read.status;
  if (enumerated === null) return "unknown";
  const matches = enumerated.filter((device) => device === name).length;
  if (matches === 0) return "unavailable";
  return matches === 1 ? "available" : "ambiguous";
};

/** Badge tone of each availability: color is never the only signal, the label is always shown. */
export const AVAILABILITY_TONE: Record<
  Availability,
  "success" | "warning" | "secondary"
> = {
  available: "success",
  unavailable: "warning",
  ambiguous: "warning",
  unknown: "secondary",
};

/** What the In use now row shows. */
export interface EffectiveView {
  /** Device name to show verbatim, or null when `label` describes it. */
  name: string | null;
  label: TranslatedMessage | null;
  reason: TranslatedMessage | null;
  /** Whether the input is in use now (otherwise it describes the last recording). */
  current: boolean;
}

/**
 * Why the effective input was chosen. A fallback names the first saved choice that was observed
 * unavailable or ambiguous before the device in use; without such evidence it stays generic.
 * @param reason backend reason
 * @param name device in use
 * @param availability saved choices with their status
 * @returns the reason message
 */
export const reasonMessage = (
  reason: EffectiveReason,
  name: string | null,
  availability: readonly ChoiceStatus[],
): TranslatedMessage => {
  if (reason !== "fallback")
    return { key: `${PRIORITY_KEY_ROOT}.reason.${reason}` };
  const end = availability.findIndex((entry) => entry.name === name);
  const earlier = end < 0 ? availability : availability.slice(0, end);
  const skipped = earlier.find((entry) => entry.status !== "available");
  if (skipped?.status === "unavailable" || skipped?.status === "ambiguous") {
    const suffix =
      skipped.status === "unavailable" ? "Unavailable" : "Ambiguous";
    return {
      key: `${PRIORITY_KEY_ROOT}.reason.fallback${suffix}`,
      params: { name: skipped.name },
    };
  }
  return { key: `${PRIORITY_KEY_ROOT}.reason.fallback` };
};

/**
 * Label of an effective input without a usable name.
 * @param report effective input report
 * @returns the label to show instead of a name
 */
const unnamedLabel = (report: EffectiveReport): TranslatedMessage => ({
  key:
    report.reason === "system_default"
      ? `${PRIORITY_KEY_ROOT}.inUse.systemDefault`
      : `${PRIORITY_KEY_ROOT}.inUse.unnamed`,
});

/**
 * Describe the input in use, or the one of the last recording, apart from the saved order.
 * @param view last status view
 * @returns the name, label and reason to show; without any successful read, "Loading…" or,
 * once a read failed, "Unknown"
 */
export const describeEffective = (view: StatusView): EffectiveView => {
  const { status } = view;
  const report = status?.effective_input;
  if (!report) {
    const key = view.failed
      ? `${PRIORITY_KEY_ROOT}.status.unknown`
      : "common.loading";
    return { name: null, label: { key }, reason: null, current: false };
  }
  if (report.scope === "not_used_yet") {
    const label = { key: `${PRIORITY_KEY_ROOT}.inUse.notUsedYet` };
    return { name: null, label, reason: null, current: false };
  }
  const reason = report.reason
    ? reasonMessage(report.reason, report.name, status.availability)
    : null;
  return {
    name: report.name,
    label: report.name === null ? unnamedLabel(report) : null,
    reason,
    current: report.scope === "current",
  };
};

/**
 * Whole seconds elapsed since the devices were observed, for "Checked N s ago".
 * @param observedAt RFC 3339 time of the observation
 * @param now current time in milliseconds
 * @returns seconds (never negative), or null when the time cannot be read
 */
export const secondsSince = (
  observedAt: string,
  now: number,
): number | null => {
  const observed = Date.parse(observedAt);
  if (Number.isNaN(observed)) return null;
  return Math.max(0, Math.floor((now - observed) / 1000));
};

/** Generation fence of the status reads: only the newest read may update the view. */
export interface ReadFence {
  /** Start a read and return its ticket (it supersedes every earlier read). */
  begin: () => number;
  /** Whether no read started, and no invalidation happened, since `ticket`. */
  isCurrent: (ticket: number) => boolean;
  /** Drop every pending read (the view closed). */
  invalidate: () => void;
}

/**
 * Create a read fence.
 * @returns a fence whose tickets increase with every read
 */
export const createReadFence = (): ReadFence => {
  let generation = 0;
  return {
    begin: () => {
      generation += 1;
      return generation;
    },
    isCurrent: (ticket) => ticket === generation,
    invalidate: () => {
      generation += 1;
    },
  };
};
