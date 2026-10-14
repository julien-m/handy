// Commits of the prompt editor (005 AC-001, AC-004): a prompt's name and text
// and its provider override are separate backend commands, sent in order;
// the outcome says what was saved so the editor never claims more. A refusal
// whose outcome is `committed` or `unknown` may have been stored: the library
// is reread before deciding, never treated as a plain refusal.
import { commands } from "@/bindings";
import type { PromptLibraryView, SettingsCommandError } from "@/bindings";
import { type PromptDraft, isContentDirty } from "./prompt-draft-model";
import {
  type ProviderDraft,
  overrideOf,
  providerDraftOf,
  sameProviderDraft,
} from "./prompt-provider-model";

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
 * Id of the prompt a creation stored: the prompt with the sent name and text.
 * @param library Library reread after the creation.
 * @param draft Values sent.
 */
export const createdPromptIdOf = (
  library: PromptLibraryView,
  draft: PromptDraft,
): string | null =>
  library.prompts.find(
    (prompt) =>
      prompt.name === draft.name.trim() && prompt.prompt === draft.text,
  )?.id ?? null;

/**
 * Whether prompt `id` has the name and text of `draft`.
 * @param library Library reread after the update.
 * @param id Updated prompt.
 * @param draft Values sent.
 */
export const contentStoredIn = (
  library: PromptLibraryView,
  id: string,
  draft: PromptDraft,
): boolean =>
  library.prompts.some(
    (prompt) =>
      prompt.id === id &&
      prompt.name === draft.name.trim() &&
      prompt.prompt === draft.text,
  );

/**
 * Whether prompt `id` has the provider choice `provider`.
 * @param library Library reread after the change.
 * @param id Changed prompt.
 * @param provider Provider choice sent.
 */
export const overrideStoredIn = (
  library: PromptLibraryView,
  id: string,
  provider: ProviderDraft,
): boolean =>
  library.prompts.some(
    (prompt) =>
      prompt.id === id &&
      sameProviderDraft(providerDraftOf(prompt.provider_override), provider),
  );

/**
 * What a refused command stored anyway. Only a `committed` outcome (stored,
 * a live effect failed) or an `unknown` one (may be stored) is checked, by
 * rereading the library; a library that cannot be read proves nothing.
 * @param error The refusal.
 * @param find Looks for the change in the reread library.
 * @returns What `find` found, or null.
 * @throws Error when the library command cannot be invoked at all.
 */
const storedDespite = async <T>(
  error: SettingsCommandError,
  find: (library: PromptLibraryView) => T | null,
): Promise<T | null> => {
  if (error.outcome !== "committed" && error.outcome !== "unknown") {
    return null;
  }
  const result = await commands.getPromptLibrary();
  return result.status === "ok" ? find(result.data) : null;
};

/** `true` when the check passes, else null (for `storedDespite`). */
const found = (isStored: boolean): true | null => (isStored ? true : null);

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
  if (result.status === "ok") return null;
  const isStored = await storedDespite(result.error, (library) =>
    found(overrideStoredIn(library, id, draft.provider)),
  );
  return isStored ? null : result.error;
};

/**
 * Id of the prompt `addPostProcessPrompt` created, or its refusal.
 * @throws Error when a command cannot be invoked at all.
 */
const addPrompt = async (
  draft: PromptDraft,
): Promise<{ id: string } | { error: SettingsCommandError }> => {
  const created = await commands.addPostProcessPrompt(
    draft.name.trim(),
    draft.text,
  );
  if (created.status === "ok") return { id: created.data.id };
  const id = await storedDespite(created.error, (library) =>
    createdPromptIdOf(library, draft),
  );
  return id ? { id } : { error: created.error };
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
  const created = await addPrompt(draft);
  if ("error" in created) return refused(created.error, null, false);
  const { id } = created;
  const overrideError = await commitOverride(id, draft, defaults);
  return overrideError
    ? refused(overrideError, id, true)
    : { status: "saved", id };
};

/**
 * Saves the name and text of prompt `id`.
 * @returns The refusal, or null once stored.
 * @throws Error when a command cannot be invoked at all.
 */
const updateContent = async (
  id: string,
  draft: PromptDraft,
): Promise<SettingsCommandError | null> => {
  const result = await commands.updatePostProcessPrompt(
    id,
    draft.name.trim(),
    draft.text,
  );
  if (result.status === "ok") return null;
  const isStored = await storedDespite(result.error, (library) =>
    found(contentStoredIn(library, id, draft)),
  );
  return isStored ? null : result.error;
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
    const error = await updateContent(id, draft);
    if (error) return refused(error, id, false);
  }
  const overrideError = await commitOverride(id, draft, saved);
  return overrideError
    ? refused(overrideError, id, contentChanged)
    : { status: "saved", id };
};
