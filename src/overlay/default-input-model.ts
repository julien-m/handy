// Pure model of the "Remember this field / terminal" entry of the badge menu (013): the label of
// an offer, the feedback of an answer, and the settlement of a remember request whose failure
// never throws. No React here, so every rule is unit-testable.
import type {
  DefaultInputKind,
  RememberAnswer,
  RememberOffer,
  RememberRefusal,
} from "@/bindings";

const KEY = "overlay.destination.menu.rememberInput";

/** A translated sentence: its key and its interpolation values. */
export interface TranslatedText {
  key: string;
  values: Record<string, string>;
}

/** What the entry shows for one offer. */
export interface EntryLabel extends TranslatedText {
  kind: DefaultInputKind;
}

/** Translation key suffix of each refusal of the backend. */
const REFUSALS: Record<RememberRefusal, string> = {
  stale: "stale",
  not_text_field: "notTextField",
  password_field: "passwordField",
  unrecognizable: "unrecognizable",
  terminal_unreadable: "terminalUnreadable",
  full: "full",
  failed: "failed",
};

// @spec AC-001: Field or terminal entry per destination — .specs/features/013-default-input-per-app/spec.md#ac-001
/**
 * Label of the entry: "Remember this field for <app>" or "Remember this terminal", with an
 * "again" variant when the application already has a remembered input.
 * @param offer the backend's offer for the open menu
 * @returns the label key, its values and the kind of target (for the glyph)
 */
export const offerLabel = (offer: RememberOffer): EntryLabel => {
  const again = offer.remembered ? "Again" : "";
  return {
    key: `${KEY}.${offer.kind}${again}`,
    values: offer.kind === "field" ? { app: offer.app_name } : {},
    kind: offer.kind,
  };
};

/**
 * Feedback of a remember answer: the saved kind for the application, or the refusal reason.
 * @param answer the backend's answer
 * @returns the feedback sentence
 */
export const answerFeedback = (answer: RememberAnswer): TranslatedText =>
  answer.status === "saved"
    ? {
        key: `${KEY}.saved.${answer.value.kind}`,
        values: { app: answer.value.app_name },
      }
    : { key: `${KEY}.refused.${REFUSALS[answer.value]}`, values: {} };

/** Outcome of a remember request: the feedback, and the offer it leaves (null: unchanged). */
export interface RememberOutcome {
  feedback: TranslatedText;
  offer: RememberOffer | null;
}

/**
 * Settles a remember request: a saved answer replaces the offer (now remembered); a refusal or
 * a rejected call keeps it and says why ("Could not save the field" when the call failed).
 * @param pending the remember call in flight
 * @returns the feedback and the offer to show afterwards
 */
export const settleRemember = async (
  pending: Promise<RememberAnswer>,
): Promise<RememberOutcome> => {
  try {
    const answer = await pending;
    return {
      feedback: answerFeedback(answer),
      offer: answer.status === "saved" ? answer.value : null,
    };
  } catch (error: unknown) {
    console.warn("Remember default input failed:", error);
    return {
      feedback: answerFeedback({ status: "refused", value: "failed" }),
      offer: null,
    };
  }
};

/**
 * Key of the menu a request belongs to; an answer for another key arrived late and is ignored.
 * @param dictationId dictation of the open menu
 * @param selectionVersion selection version of the open menu
 * @returns the request key
 */
export const menuRequestKey = (
  dictationId: number,
  selectionVersion: number,
): string => `${dictationId}:${selectionVersion}`;
