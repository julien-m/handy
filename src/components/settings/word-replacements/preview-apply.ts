/**
 * Apply of a previewed input (text view Apply and file import, feature 002 AC-003/AC-005) bound
 * to the preview's revision. The backend compares the global settings revision, so a change of
 * another setting after the preview also refuses the Apply with `revision_conflict`; the input is
 * then previewed again and, only when the fresh preview equals the one the user saw, applied once
 * at the fresh revision. Any other difference stays a conflict: nothing is applied unseen.
 */
import {
  commands,
  type ImportMode,
  type ReplacementPreviewView,
  type ReplacementRulesView,
  type Result,
  type SettingsCommandError,
} from "@/bindings";
import { samePreview } from "./preview-model";

/**
 * Apply `input` at the revision of `preview`, retried once at a newer revision when the fresh
 * preview of the same input is unchanged.
 * Side effects: one `apply_word_replacements` call; after a conflict, one preview and at most
 * one more Apply.
 * @param input text or file content that was previewed
 * @param mode import mode of the preview
 * @param preview preview shown to the user
 * @returns the saved rules, or the refusal of the last Apply
 * @throws when a command fails instead of returning a typed refusal
 */
export const applyPreviewed = async (
  input: string,
  mode: ImportMode,
  preview: ReplacementPreviewView,
): Promise<Result<ReplacementRulesView, SettingsCommandError>> => {
  const first = await commands.applyWordReplacements(
    input,
    mode,
    preview.revision,
  );
  if (first.status === "ok" || first.error.code !== "revision_conflict") {
    return first;
  }
  const fresh = await commands.previewWordReplacements(input, mode);
  // Only an identical outcome may move to the new revision; otherwise the conflict stands.
  if (
    fresh.status === "error" ||
    !samePreview(preview.preview, fresh.data.preview)
  ) {
    return first;
  }
  return commands.applyWordReplacements(input, mode, fresh.data.revision);
};
