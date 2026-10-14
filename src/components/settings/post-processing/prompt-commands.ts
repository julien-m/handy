// Commits of the prompt editor (005 AC-001, AC-004): a prompt's name and text
// and its provider override are separate backend commands, sent in order;
// the outcome says what was saved so the editor never claims more.
import { commands } from "@/bindings";
import type { SettingsCommandError } from "@/bindings";
import { type PromptDraft, isContentDirty } from "./prompt-draft-model";
import { overrideOf, sameProviderDraft } from "./prompt-provider-model";

/** What a save committed. */
export type CommitOutcome =
  | { status: "saved"; id: string }
  /**
   * Refused. `id` is the prompt when it exists (a new prompt created before
   * its override was refused); `contentSaved` says whether the name and
   * text were committed before the refusal.
   */
  | {
      status: "refused";
      error: SettingsCommandError;
      id: string | null;
      contentSaved: boolean;
    };

const refused = (
  error: SettingsCommandError,
  id: string | null,
  contentSaved: boolean,
): CommitOutcome => ({ status: "refused", error, id, contentSaved });

/**
 * Sets the override of `id` when it differs from the saved one.
 * @returns The refusal, or null once saved or unchanged.
 */
const commitOverride = async (
  id: string,
  draft: PromptDraft,
  saved: PromptDraft,
): Promise<SettingsCommandError | null> => {
  if (sameProviderDraft(draft.provider, saved.provider)) return null;
  const result = await commands.setPostProcessPromptProviderOverride(
    id,
    overrideOf(draft.provider),
  );
  return result.status === "error" ? result.error : null;
};

/**
 * Creates a prompt, then sets its override. The name is sent trimmed (the
 * backend stores it so) and the text verbatim.
 * @param draft Values of the form.
 * @param defaults Values a new prompt starts with (the default provider).
 * @throws Error when a command cannot be invoked at all.
 */
export const createPrompt = async (
  draft: PromptDraft,
  defaults: PromptDraft,
): Promise<CommitOutcome> => {
  const created = await commands.addPostProcessPrompt(
    draft.name.trim(),
    draft.text,
  );
  if (created.status === "error") return refused(created.error, null, false);
  const id = created.data.id;
  const overrideError = await commitOverride(id, draft, defaults);
  return overrideError
    ? refused(overrideError, id, true)
    : { status: "saved", id };
};

/**
 * Saves the changed name and text of prompt `id`, then its override.
 * @param id Prompt to update.
 * @param draft Values of the form.
 * @param saved Values last saved.
 * @throws Error when a command cannot be invoked at all.
 */
export const updatePrompt = async (
  id: string,
  draft: PromptDraft,
  saved: PromptDraft,
): Promise<CommitOutcome> => {
  const contentChanged = isContentDirty(draft, saved);
  if (contentChanged) {
    const result = await commands.updatePostProcessPrompt(
      id,
      draft.name.trim(),
      draft.text,
    );
    if (result.status === "error") return refused(result.error, id, false);
  }
  const overrideError = await commitOverride(id, draft, saved);
  return overrideError
    ? refused(overrideError, id, contentChanged)
    : { status: "saved", id };
};
