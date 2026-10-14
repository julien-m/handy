/**
 * Text view of the replacement rules (feature 002, AC-003): the exchange text, its line-by-line
 * preview from `preview_word_replacements` (replace mode, bounded to 1,000,000 bytes) and Apply,
 * which commits exactly that input at the revision its preview was computed at.
 */
import { create } from "zustand";
import {
  commands,
  type ReplacementPreviewView,
  type ReplacementRule,
} from "@/bindings";
import { exchangeText } from "./rules-model";
import {
  type CommitFailure,
  UNEXPECTED_FAILURE,
  canApplyPreview,
  commitFailure,
} from "./preview-model";
import { useReplacementEditor } from "./editor-store";

export interface TextStore {
  /** Text being edited. */
  text: string;
  /** Text of the saved rules when the view opened or was last applied. */
  baseText: string;
  /** Preview of `previewText`, null until the first preview answers. */
  preview: ReplacementPreviewView | null;
  previewText: string | null;
  previewing: boolean;
  /** Typed refusal of the last preview (for example above 1,000,000 bytes). */
  previewFailure: CommitFailure | null;
  applying: boolean;
  applied: boolean;
  failure: CommitFailure | null;
  open: (rules: ReadonlyArray<ReplacementRule>) => void;
  setText: (text: string) => void;
  requestPreview: () => Promise<void>;
  apply: () => Promise<boolean>;
  cancel: () => void;
}

let previewSequence = 0;

export const useReplacementText = create<TextStore>()(() => ({
  text: "",
  baseText: "",
  preview: null,
  previewText: null,
  previewing: false,
  previewFailure: null,
  applying: false,
  applied: false,
  failure: null,
  open,
  setText: (text) => patch({ text, applied: false, failure: null }),
  requestPreview,
  apply,
  cancel: () => open(useReplacementEditor.getState().saved),
}));

const state = (): TextStore => useReplacementText.getState();
const patch = (next: Partial<TextStore>): void =>
  useReplacementText.setState(next);

/**
 * Whether the text differs from the saved rules' text.
 * @param store text state
 * @returns true when leaving the view would lose edits
 */
export const isTextDirty = (store: TextStore): boolean =>
  store.text !== store.baseText;

/**
 * Whether the preview on screen belongs to the current text.
 * @param store text state
 * @returns true when the preview is current
 */
export const isPreviewCurrent = (store: TextStore): boolean =>
  store.preview !== null && store.previewText === store.text;

/**
 * Whether Apply may commit: a current preview without errors that changes the rules.
 * @param store text state
 * @returns true when Apply is enabled
 */
export const canApplyText = (store: TextStore): boolean =>
  isPreviewCurrent(store) &&
  store.preview !== null &&
  canApplyPreview(store.preview.preview) &&
  !store.previewing &&
  !store.applying &&
  // After a revision conflict, Reload previews again before any Apply.
  store.failure?.conflict !== true;

/**
 * Start the view on the text of `rules`, discarding any text edit.
 * @param rules saved rules
 */
function open(rules: ReadonlyArray<ReplacementRule>): void {
  previewSequence += 1;
  const text = exchangeText(rules);
  patch({
    text,
    baseText: text,
    preview: null,
    previewText: null,
    previewing: false,
    previewFailure: null,
    applying: false,
    failure: null,
  });
}

/**
 * Preview the current text against the saved rules; a newer request wins.
 * Side effects: one `preview_word_replacements` call.
 */
async function requestPreview(): Promise<void> {
  previewSequence += 1;
  const sequence = previewSequence;
  const { text } = state();
  patch({ previewing: true });
  let outcome: Partial<TextStore>;
  try {
    const result = await commands.previewWordReplacements(text, "replace");
    outcome =
      result.status === "ok"
        ? { preview: result.data, previewFailure: null, failure: null }
        : { preview: null, previewFailure: commitFailure(result.error) };
  } catch (error) {
    console.error("Failed to preview replacement rules:", error);
    outcome = { preview: null, previewFailure: UNEXPECTED_FAILURE };
  }
  // A newer request (or a reopened view) owns the preview; this answer is stale.
  if (sequence !== previewSequence) return;
  patch({ ...outcome, previewText: text, previewing: false });
}

/**
 * Apply the previewed text in one commit at its preview's revision (AC-003).
 * Side effects: one `apply_word_replacements` call; the list editor takes the saved rules.
 * @returns true when the rules were replaced
 */
async function apply(): Promise<boolean> {
  const current = state();
  if (!canApplyText(current) || current.preview === null) return false;
  patch({ applying: true, failure: null });
  try {
    const result = await commands.applyWordReplacements(
      current.text,
      "replace",
      current.preview.revision,
    );
    if (result.status === "error")
      return failApply(commitFailure(result.error));
    useReplacementEditor.getState().acceptCommitted(result.data);
    open(result.data.rules);
    patch({ applied: true });
    return true;
  } catch (error) {
    console.error("Failed to apply replacement rules:", error);
    return failApply(UNEXPECTED_FAILURE);
  }
}

function failApply(failure: CommitFailure): false {
  patch({ applying: false, failure });
  if (failure.uncertain) void useReplacementEditor.getState().loadRules();
  return false;
}
