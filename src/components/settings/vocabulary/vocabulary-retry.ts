/**
 * Retry of a failed vocabulary commit (feature 003): where the focus goes once the error and its
 * Retry button leave, and what a Retry of a failed Apply does with the preview reread after the
 * failure (sent, or nothing sent with the reason shown).
 */
import type { VocabularyStore } from "@/stores/vocabulary-store";
import {
  applyRetryBlock,
  type ApplyRetryBlock,
} from "./vocabulary-paste-model";

/** How a Retry of a failed Apply reports its outcome. */
export interface ApplyRetryFeedback {
  /** Nothing was sent: the preview cannot be applied or adds nothing. */
  blocked: (block: ApplyRetryBlock) => void;
  /** The terms were added (the paste box closes, focus returns to "Paste a list…"). */
  applied: () => void;
}

/**
 * Retry of a failed Add or removal. A successful Retry removes the error and its Retry button,
 * which had the focus: the term input takes it.
 * @param store editor store
 * @param focusInput moves the focus to the term input
 * @returns the Retry action
 */
export const retryThenFocus =
  (store: VocabularyStore, focusInput: () => void) =>
  async (): Promise<void> => {
    if (await store.getState().retry()) focusInput();
  };

/**
 * Retry of a failed Apply, on the preview reread after the failure. A preview that cannot be
 * applied, or that adds nothing, sends nothing and reports why; a successful Retry is reported
 * like a successful Apply.
 * @param store editor store
 * @param feedback outcome reports
 */
export const retryApply = async (
  store: VocabularyStore,
  feedback: ApplyRetryFeedback,
): Promise<void> => {
  const state = store.getState();
  const fresh = state.previewText === state.pasteText;
  const block = applyRetryBlock(state.pasteText, state.preview, fresh);
  if (block !== null) return feedback.blocked(block);
  if (await state.retry()) feedback.applied();
};
