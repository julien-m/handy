/**
 * Status wording of the vocabulary editor (AC-003): the tone the mockups give each of the five
 * statuses, the title and body keys, and the not-sent terms grouped by reason.
 */
import type { ExcludedTerm, VocabularyStatusReport } from "@/bindings";

/** Status box look, from the existing Alert and info-box recipes. */
export type StatusTone = "success" | "neutral" | "info" | "warning" | "error";

/**
 * Tone of the status box.
 * @param report status for the selected model
 * @returns the tone the mockups give each of the five statuses
 */
export const statusTone = (report: VocabularyStatusReport): StatusTone => {
  switch (report.status) {
    case "used_during_recognition":
      return "success";
    case "unknown_until_model_loads":
      return "info";
    case "no_term_sent":
      // Business rule: an empty list is a neutral fact; invalid or oversized terms need action.
      return report.reason === "empty_vocabulary" ? "neutral" : "warning";
    default:
      return "neutral";
  }
};

/** Translation keys of the status title and body. */
export interface StatusText {
  title: string;
  body: string;
}

const STATUS_ROOT = "settings.vocabulary.status";

/**
 * Translation keys for the status title and body.
 * @param report status for the selected model
 * @returns keys under `settings.vocabulary.status`
 */
export const statusText = (report: VocabularyStatusReport): StatusText => {
  switch (report.status) {
    case "used_during_recognition":
      return { title: `${STATUS_ROOT}.used.title`, body: usedBodyKey(report) };
    case "no_term_sent":
      return {
        title: `${STATUS_ROOT}.noTermSent.title`,
        body: `${STATUS_ROOT}.noTermSent.${report.reason ?? "empty_vocabulary"}`,
      };
    case "not_supported":
      return {
        title: `${STATUS_ROOT}.notSupported.title`,
        body: `${STATUS_ROOT}.notSupported.body`,
      };
    case "not_used_streaming":
      return {
        title: `${STATUS_ROOT}.streaming.title`,
        body: `${STATUS_ROOT}.streaming.body`,
      };
    default:
      return {
        title: `${STATUS_ROOT}.unknown.title`,
        body: `${STATUS_ROOT}.unknown.${report.reason ?? "model_not_loaded"}`,
      };
  }
};

const usedBodyKey = (report: VocabularyStatusReport): string =>
  sentTotal(report) === 1
    ? `${STATUS_ROOT}.used.sentOne`
    : `${STATUS_ROOT}.used.sent`;

/**
 * Terms counted in "N of M terms are sent".
 * @param report status for the selected model
 * @returns sent plus not-sent terms (every stored occurrence)
 */
export const sentTotal = (report: VocabularyStatusReport): number =>
  report.terms_sent.length + report.terms_not_sent.length;

/** Why a group of saved terms is not sent, as the status lists them. */
export type NotSentGroupReason =
  | "over_budget"
  | "invalid"
  | "duplicate"
  | "over_capacity";

export interface NotSentGroup {
  reason: NotSentGroupReason;
  terms: string[];
}

const groupOf = (excluded: ExcludedTerm): NotSentGroupReason | null => {
  switch (excluded.reason) {
    case "over_budget":
    case "duplicate":
    case "over_capacity":
      return excluded.reason;
    // A model or mode without context excludes every term; the status title already says so.
    case "no_context":
      return null;
    default:
      return "invalid";
  }
};

const GROUP_ORDER: readonly NotSentGroupReason[] = [
  "over_budget",
  "invalid",
  "duplicate",
  "over_capacity",
];

/**
 * Not-sent terms grouped by reason, in a stable order, each group in list order.
 * @param excluded terms the next request leaves out
 * @returns non-empty groups only
 */
export const notSentGroups = (
  excluded: readonly ExcludedTerm[],
): NotSentGroup[] => {
  const groups = new Map<NotSentGroupReason, string[]>();
  for (const term of excluded) {
    const reason = groupOf(term);
    if (!reason) continue;
    const group = groups.get(reason);
    // Appended in place: with Whisper about 4,950 of 5,000 terms can be left out.
    if (group) group.push(term.term);
    else groups.set(reason, [term.term]);
  }
  return GROUP_ORDER.filter((reason) => groups.has(reason)).map((reason) => ({
    reason,
    terms: groups.get(reason) ?? [],
  }));
};

/**
 * Most terms named per not-sent group; the rest is counted ("and 4,930 more"), so a 5,000-term
 * vocabulary over a Whisper budget never turns the status into a wall of text (native session
 * N7 review). Twenty names fit a few lines of the status box.
 */
export const NOT_SENT_SHOWN = 20;

/** Terms of a not-sent group as the status shows them. */
export interface NotSentPreview {
  shown: string[];
  hidden: number;
}

/**
 * The first `NOT_SENT_SHOWN` terms of a group, in list order, and how many are left out.
 * @param group one group of `notSentGroups`
 * @returns the named terms and the count of the others
 */
export const notSentPreview = (group: NotSentGroup): NotSentPreview => ({
  shown: group.terms.slice(0, NOT_SENT_SHOWN),
  hidden: Math.max(0, group.terms.length - NOT_SENT_SHOWN),
});
