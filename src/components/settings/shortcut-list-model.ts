// Pure rules of the shortcut list UI: list limits and the translation of
// backend refusals, stored conflicts and shortcuts the system refused into
// user messages.
import type {
  RefusedShortcut,
  ShortcutBinding,
  ShortcutEditError,
  StoredShortcutConflict,
} from "@/bindings";
import { ShortcutEditFailure } from "../../lib/shortcut-edit-failure";
import {
  SettingsCommandFailure,
  settingsErrorKey,
} from "../../stores/settings-errors";

/** Mirrors MAX_SHORTCUTS_PER_ACTION in src-tauri/src/shortcut/action_shortcuts.rs. */
export const MAX_SHORTCUTS_PER_ACTION = 4;

/** A translation key with its interpolation values. */
export interface TranslatedMessage {
  key: string;
  params: Record<string, string | number>;
}

/**
 * Every shortcut of an action, first one first.
 * @param binding Stored binding of the action.
 * @returns The ordered shortcuts.
 */
export const shortcutsOf = (binding: ShortcutBinding): string[] => [
  binding.current_binding,
  ...(binding.additional_bindings ?? []),
];

/**
 * Whether one more shortcut can be added.
 * @param count Current number of shortcuts of the action.
 */
export const canAddShortcut = (count: number): boolean =>
  count < MAX_SHORTCUTS_PER_ACTION;

/**
 * Whether a shortcut can be removed: each action keeps at least one.
 * @param count Current number of shortcuts of the action.
 */
export const canRemoveShortcut = (count: number): boolean => count > 1;

const LIST_KEY = "settings.general.shortcut.list";

/**
 * Message explaining why the backend refused an edit.
 * @param error Typed refusal from `edit_binding_shortcuts`.
 * @param actionName Translated display name of an action id.
 * @returns The translation key and its values.
 */
export const describeEditError = (
  error: ShortcutEditError,
  actionName: (actionId: string) => string,
): TranslatedMessage => {
  switch (error.code) {
    case "shortcut_conflict":
      return describeConflict(error.conflict, actionName);
    case "limit_exceeded":
      return { key: `${LIST_KEY}.addLimit`, params: { limit: error.limit } };
    case "last_shortcut":
      return { key: `${LIST_KEY}.removeLast`, params: {} };
    case "invalid_shortcut":
      return {
        key: `${LIST_KEY}.errors.invalid`,
        params: { error: error.message },
      };
    case "registration_failed":
      return {
        key: `${LIST_KEY}.errors.registration`,
        params: { error: error.message },
      };
    case "settings_refused":
      return { key: settingsErrorKey(error.error), params: {} };
    default:
      return { key: `${LIST_KEY}.errors.generic`, params: {} };
  }
};

/**
 * Reason of a failed shortcut change for the legacy recorders: the typed
 * backend refusal when there is one, else the raw error text.
 * @param error Error thrown by `updateBinding` or `resetBinding`.
 * @param actionName Translated display name of an action id.
 * @returns A translation to show, or the untranslatable raw text.
 */
export const describeShortcutFailure = (
  error: unknown,
  actionName: (actionId: string) => string,
): TranslatedMessage | string => {
  if (error instanceof ShortcutEditFailure) {
    return describeEditError(error.editError, actionName);
  }
  if (error instanceof SettingsCommandFailure) {
    return describeSettingsFailure(error);
  }
  return String(error);
};

/**
 * Message for a refusal of the settings service itself (busy, persistence),
 * translated from its typed code, never from its English text.
 * @param failure Typed settings command failure.
 * @returns The translation key, without values.
 */
export const describeSettingsFailure = (
  failure: SettingsCommandFailure,
): TranslatedMessage => ({ key: settingsErrorKey(failure.detail), params: {} });

/**
 * Message naming the action that already owns a combination.
 * @param conflict Conflict reported by the shared checker.
 * @param actionName Translated display name of an action id.
 */
export const describeConflict = (
  conflict: StoredShortcutConflict["conflict"],
  actionName: (actionId: string) => string,
): TranslatedMessage => ({
  key:
    conflict.kind === "duplicate"
      ? `${LIST_KEY}.errors.conflict`
      : `${LIST_KEY}.errors.overlap`,
  params: {
    action: actionName(conflict.owning_action),
    shortcut: conflict.owning_shortcut,
  },
});

/**
 * Stored conflict reported for one shortcut of one action, if any.
 * @param conflicts Conflicts from `get_shortcut_conflicts`.
 * @param actionId Action owning the shortcut.
 * @param shortcut Stored shortcut text.
 */
export const storedConflictFor = (
  conflicts: ReadonlyArray<StoredShortcutConflict>,
  actionId: string,
  shortcut: string,
): StoredShortcutConflict | undefined =>
  conflicts.find(
    (entry) => entry.action_id === actionId && entry.shortcut === shortcut,
  );

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/**
 * Warning for a stored shortcut the system refused when post-processing was
 * enabled, if it is still in the action's list. It stays editable.
 * @param refusals Shortcuts refused at the last activation.
 * @param actionId Action owning the shortcut.
 * @param shortcut Stored shortcut text.
 */
export const refusalFor = (
  refusals: ReadonlyArray<RefusedShortcut>,
  actionId: string,
  shortcut: string,
): TranslatedMessage | undefined => {
  const refusal = refusals.find(
    (entry) => entry.action_id === actionId && entry.shortcut === shortcut,
  );
  return refusal
    ? {
        key: `${LIST_KEY}.errors.registration`,
        params: { error: refusal.reason },
      }
    : undefined;
};

/**
 * Refusals whose shortcut is still stored for its action: a shortcut that
 * was changed or removed is no longer refused.
 * @param refusals Shortcuts refused at the last activation.
 * @param bindings Stored bindings of every action.
 */
export const keepStoredRefusals = (
  refusals: ReadonlyArray<RefusedShortcut>,
  bindings: Partial<Record<string, ShortcutBinding>>,
): RefusedShortcut[] =>
  refusals.filter((refusal) => {
    const binding = bindings[refusal.action_id];
    return binding ? shortcutsOf(binding).includes(refusal.shortcut) : false;
  });
