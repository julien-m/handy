// Draft of the microphone priority editor (feature 001): edits change only the draft, Save
// commits the complete order with the revision the draft is based on, and a change saved
// elsewhere keeps the draft and raises a conflict until Reload, Discard or a successful Save.
import type { ErrorReason, SettingsCommandError } from "@/bindings";
import { settingsErrorKey } from "@/stores/settings-errors";

/** Translation key under `settings.sound.microphonePriority` plus its values. */
export interface TranslatedMessage {
  key: string;
  params?: Record<string, string>;
}

/** Committed order and the settings revision it was read at. */
export interface SavedPriority {
  order: readonly string[];
  revision: number | null;
}

/** Write state of the Save control. */
export type SavePhase = "idle" | "saving" | "failed";

/** Refused addition: the chosen name stays in the Add control with its message. */
export interface AddError {
  name: string;
  message: TranslatedMessage;
}

/** Editor state kept beside the committed settings. */
export interface PriorityDraft {
  /** Edited order, or null while the editor shows the committed order. */
  order: readonly string[] | null;
  /** Committed order and revision the edit is based on (null before the first edit). */
  base: SavedPriority | null;
  /** The committed order changed elsewhere since `base` was read. */
  conflict: boolean;
  savePhase: SavePhase;
  saveError: TranslatedMessage | null;
  addError: AddError | null;
}

export const EMPTY_DRAFT: PriorityDraft = {
  order: null,
  base: null,
  conflict: false,
  savePhase: "idle",
  saveError: null,
  addError: null,
};

/** Translation namespace of the editor. */
export const PRIORITY_KEY_ROOT = "settings.sound.microphonePriority";

/** Translation keys of the draft messages. */
export const PRIORITY_KEYS = {
  duplicateChoice: `${PRIORITY_KEY_ROOT}.duplicateChoice`,
  duplicate: `${PRIORITY_KEY_ROOT}.errors.duplicate`,
  empty: `${PRIORITY_KEY_ROOT}.errors.empty`,
} as const;

/**
 * Whether two orders hold the same names in the same positions.
 * @param left first order
 * @param right second order
 * @returns true when both are equal element by element
 */
export const sameOrder = (
  left: readonly string[],
  right: readonly string[],
): boolean =>
  left.length === right.length &&
  left.every((name, index) => name === right[index]);

/**
 * The order the editor shows.
 * @param draft editor state
 * @param saved committed priority
 * @returns the edited order, or the committed one when nothing is edited
 */
export const shownOrder = (
  draft: PriorityDraft,
  saved: SavedPriority,
): readonly string[] => draft.order ?? saved.order;

/**
 * Whether the draft differs from what is committed (the Unsaved changes bar is shown).
 * @param draft editor state
 * @param saved committed priority
 * @returns true when Save would change the committed order
 */
export const isDirty = (draft: PriorityDraft, saved: SavedPriority): boolean =>
  draft.order !== null && !sameOrder(draft.order, saved.order);

/**
 * First reason an order cannot be saved: a blank name, then a repeated exact name. Mirrors the
 * backend validation so Save is disabled before a refused write.
 * @param order candidate order
 * @returns the message to show, or null when the order is valid
 */
export const orderError = (
  order: readonly string[],
): TranslatedMessage | null => {
  if (order.some((name) => name.trim() === "")) {
    return { key: PRIORITY_KEYS.empty };
  }
  const unique = new Set(order);
  return unique.size === order.length ? null : { key: PRIORITY_KEYS.duplicate };
};

/**
 * Whether Save may dispatch a write.
 * @param draft editor state
 * @param saved committed priority
 * @returns true for a changed, valid draft with no write in flight
 */
export const canSave = (draft: PriorityDraft, saved: SavedPriority): boolean =>
  draft.savePhase !== "saving" &&
  isDirty(draft, saved) &&
  orderError(draft.order ?? []) === null;

/**
 * Apply an edited order: the first edit records the committed order it starts from. Edits are
 * ignored while a write is in flight, so the order being saved is the order shown.
 * @param draft editor state
 * @param saved committed priority
 * @param next new draft order
 * @returns the edited state, with earlier save and add messages cleared
 */
export const editOrder = (
  draft: PriorityDraft,
  saved: SavedPriority,
  next: readonly string[],
): PriorityDraft => {
  if (draft.savePhase === "saving") return draft;
  return {
    ...draft,
    order: next,
    base: draft.base ?? saved,
    savePhase: "idle",
    saveError: null,
    addError: null,
  };
};

/**
 * Move one choice to another position.
 * @param order current order
 * @param from index of the moved choice
 * @param to index it ends at
 * @returns a new order (unchanged copy when an index is out of range)
 */
export const moveChoice = (
  order: readonly string[],
  from: number,
  to: number,
): string[] => {
  const next = [...order];
  const inRange = (index: number) => index >= 0 && index < order.length;
  if (!inRange(from) || !inRange(to) || from === to) return next;
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved);
  return next;
};

/**
 * Remove one choice; an unavailable choice can be removed like any other.
 * @param order current order
 * @param index position to remove
 * @returns a new order without that position
 */
export const removeChoice = (
  order: readonly string[],
  index: number,
): string[] => order.filter((_, position) => position !== index);

/**
 * Why a name cannot be added: blank, or already present with the exact same name.
 * @param order current draft order
 * @param name name chosen in the Add control
 * @returns the message to show, or null when the name can be added
 */
export const additionError = (
  order: readonly string[],
  name: string,
): TranslatedMessage | null => {
  if (name.trim() === "") return { key: PRIORITY_KEYS.empty };
  if (!order.includes(name)) return null;
  return { key: PRIORITY_KEYS.duplicateChoice, params: { name } };
};

/**
 * Add a choice at the end of the draft, or keep the refused name with its message.
 * @param draft editor state
 * @param saved committed priority
 * @param name name chosen in the Add control
 * @returns the updated state; a refusal leaves the order unchanged
 */
export const addChoice = (
  draft: PriorityDraft,
  saved: SavedPriority,
  name: string,
): PriorityDraft => {
  if (draft.savePhase === "saving") return draft;
  const order = shownOrder(draft, saved);
  const message = additionError(order, name);
  if (message) return { ...draft, addError: { name, message } };
  return editOrder(draft, saved, [...order, name]);
};

/**
 * Follow a new committed state while editing. Three cases: the committed order still equals the
 * base (another setting changed), so only the base revision advances; it now equals the draft (our
 * own Save, or the same order saved elsewhere), so nothing conflicts; otherwise the order changed
 * elsewhere, so the draft is kept, the conflict is raised and the base moves to the new revision,
 * which lets a later Save knowingly replace that order.
 * @param draft editor state
 * @param saved committed priority just read
 * @returns the rebased state
 */
export const rebaseDraft = (
  draft: PriorityDraft,
  saved: SavedPriority,
): PriorityDraft => {
  if (draft.order === null || draft.base === null) return draft;
  if (draft.base.revision === saved.revision) return draft;
  const foreign =
    !sameOrder(saved.order, draft.base.order) &&
    !sameOrder(saved.order, draft.order);
  return { ...draft, base: saved, conflict: draft.conflict || foreign };
};

/**
 * State while a write is in flight.
 * @param draft editor state
 * @returns the state with Save busy
 */
export const startSave = (draft: PriorityDraft): PriorityDraft => ({
  ...draft,
  savePhase: "saving",
  saveError: null,
});

const MICROPHONE_REASON_KEYS: Partial<Record<ErrorReason, string>> = {
  microphone_name_duplicate: PRIORITY_KEYS.duplicate,
  microphone_name_empty: PRIORITY_KEYS.empty,
};

/**
 * Translation of a refused write: a refused microphone name (the backend reports it in
 * `items[].code`, like the CLI's `details.items`) first, then the shared settings keys.
 * @param error typed refusal, or null for an untyped failure
 * @returns the message shown beside Save
 */
export const saveErrorMessage = (
  error: SettingsCommandError | null,
): TranslatedMessage => {
  if (error === null) return { key: "errors.settings.generic" };
  const reasons = [
    error.detail_reason,
    ...(error.items ?? []).map((item) => item.code),
  ];
  const key = reasons
    .map((reason) => (reason ? MICROPHONE_REASON_KEYS[reason] : undefined))
    .find((candidate) => candidate !== undefined);
  return { key: key ?? settingsErrorKey(error) };
};

/**
 * State after a refused write. A stale revision keeps the draft and raises the conflict (the
 * banner explains it); any other refusal keeps the draft with an error and a retry.
 * @param draft editor state
 * @param error typed refusal, or null for an untyped failure
 * @returns the state after the failure
 */
export const failSave = (
  draft: PriorityDraft,
  error: SettingsCommandError | null,
): PriorityDraft => {
  if (error?.code === "revision_conflict") {
    return { ...draft, savePhase: "idle", saveError: null, conflict: true };
  }
  return { ...draft, savePhase: "failed", saveError: saveErrorMessage(error) };
};
