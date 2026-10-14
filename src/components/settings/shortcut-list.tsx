// Settings row listing every shortcut of one action (1 to 4), with record,
// remove, add and reset controls. Every change goes through the backend
// `edit_binding_shortcuts` command, which validates, registers and saves.
import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, X } from "lucide-react";
import { commands } from "@/bindings";
import type { ShortcutEdit, StoredShortcutConflict } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { ResetButton } from "../ui/ResetButton";
import { SettingContainer } from "../ui/SettingContainer";
import { ShortcutInput } from "./ShortcutInput";
import {
  MAX_SHORTCUTS_PER_ACTION,
  type TranslatedMessage,
  canAddShortcut,
  canRemoveShortcut,
  describeConflict,
  describeEditError,
  refusalFor,
  shortcutsOf,
  storedConflictFor,
} from "./shortcut-list-model";

interface ShortcutListProps {
  actionId: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

type ListState = "idle" | "loading" | "error" | "success";

interface ShortcutRowProps {
  actionId: string;
  index: number;
  shortcut: string;
  canRemove: boolean;
  busy: boolean;
  conflict?: TranslatedMessage;
  /** The system refused this shortcut: it is stored but does nothing. */
  refusal?: TranslatedMessage;
  onEdit: (edit: ShortcutEdit) => Promise<void>;
}

interface RowWarningProps {
  message: TranslatedMessage;
  testId: string;
}

/** Warning icon of a row, explained by its accessible label and tooltip. */
const RowWarning: React.FC<RowWarningProps> = ({ message, testId }) => {
  const { t } = useTranslation();
  const text = t(message.key, message.params);
  return (
    <span
      role="img"
      className="text-yellow-500"
      data-testid={testId}
      aria-label={text}
      title={text}
    >
      <AlertTriangle size={14} />
    </span>
  );
};

const ShortcutRow: React.FC<ShortcutRowProps> = ({
  actionId,
  index,
  shortcut,
  canRemove,
  busy,
  conflict,
  refusal,
  onEdit,
}) => {
  const { t } = useTranslation();
  const removeLabel = canRemove
    ? t("settings.general.shortcut.list.remove")
    : t("settings.general.shortcut.list.removeLast");
  const rowId = `shortcut-row-${actionId}-${index}`;
  return (
    <div className="flex items-center gap-1" data-testid={rowId}>
      {refusal && <RowWarning message={refusal} testId={`${rowId}-refused`} />}
      {conflict && (
        <RowWarning message={conflict} testId={`${rowId}-conflict`} />
      )}
      <ShortcutInput
        shortcutId={actionId}
        value={shortcut}
        disabled={busy}
        label={t("settings.general.shortcut.list.change")}
        testId={`${rowId}-recorder`}
        onCapture={(next) =>
          onEdit({ kind: "replace", previous: shortcut, shortcut: next })
        }
      />
      <button
        type="button"
        className={`p-1 rounded-md ${canRemove && !busy ? "hover:bg-logo-primary/30 cursor-pointer" : "opacity-50 cursor-not-allowed"}`}
        data-testid={`${rowId}-remove`}
        aria-label={removeLabel}
        aria-disabled={!canRemove || busy}
        title={removeLabel}
        onClick={() => {
          if (canRemove && !busy) void onEdit({ kind: "remove", shortcut });
        }}
      >
        <X size={14} />
      </button>
    </div>
  );
};

interface StoredConflicts {
  conflicts: StoredShortcutConflict[];
  /** Whether the last refresh failed; the last known list is kept. */
  failed: boolean;
}

// @spec AC-002: Legacy conflict warnings stay current — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/**
 * Stored conflicts, refreshed whenever any action's shortcuts or the
 * keyboard implementation change, so a conflict resolved from another
 * action's recorder clears its warning here too.
 * @param bindingsKey Serialized shortcuts of every action.
 * @param implementation Active keyboard implementation.
 */
const useStoredConflicts = (
  bindingsKey: string,
  implementation: string | undefined,
): StoredConflicts => {
  const [state, setState] = useState<StoredConflicts>({
    conflicts: [],
    failed: false,
  });
  useEffect(() => {
    let isCurrent = true;
    commands
      .getShortcutConflicts()
      .then((conflicts) => {
        if (isCurrent) setState({ conflicts, failed: false });
      })
      .catch((failure) => {
        console.error("Failed to load shortcut conflicts:", failure);
        if (isCurrent) setState((previous) => ({ ...previous, failed: true }));
      });
    return () => {
      isCurrent = false;
    };
  }, [bindingsKey, implementation]);
  return state;
};

// @spec AC-001: Settings list with add and remove controls — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/**
 * Lists and edits every shortcut of `actionId`. Stored shortcuts that
 * already conflict (kept as before an upgrade) or that the system refused
 * when post-processing was enabled carry a warning and stay editable.
 */
export const ShortcutList: React.FC<ShortcutListProps> = ({
  actionId,
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, editBindingShortcuts, refusedShortcuts } = useSettings();
  const [state, setState] = useState<ListState>("idle");
  const [error, setError] = useState<TranslatedMessage | null>(null);
  const bindings = getSetting("bindings");
  const binding = bindings?.[actionId];
  const { conflicts, failed: conflictsFailed } = useStoredConflicts(
    JSON.stringify(bindings ?? {}),
    getSetting("keyboard_implementation"),
  );
  const actionName = useCallback(
    (id: string) => t(`settings.general.shortcut.bindings.${id}.name`, id),
    [t],
  );

  const submit = async (edit: ShortcutEdit) => {
    setState("loading");
    setError(null);
    try {
      const failure = await editBindingShortcuts(actionId, edit);
      setError(failure ? describeEditError(failure, actionName) : null);
      setState(failure ? "error" : "success");
    } catch (unexpected) {
      console.error("Failed to edit shortcuts:", unexpected);
      setError({
        key: "settings.general.shortcut.list.errors.generic",
        params: {},
      });
      setState("error");
    }
  };

  if (!binding) return null;
  const shortcuts = shortcutsOf(binding);
  const busy = state === "loading";
  const canAdd = canAddShortcut(shortcuts.length);
  const listId = `shortcut-list-${actionId}`;
  const conflictOf = (shortcut: string) => {
    const stored = storedConflictFor(conflicts, actionId, shortcut);
    return stored ? describeConflict(stored.conflict, actionName) : undefined;
  };

  return (
    <SettingContainer
      title={actionName(actionId)}
      description={t(
        `settings.general.shortcut.bindings.${actionId}.description`,
        binding.description,
      )}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div
        className="flex flex-col items-end gap-1"
        data-testid={listId}
        data-state={state}
        aria-busy={busy}
      >
        {shortcuts.map((shortcut, index) => (
          <ShortcutRow
            key={shortcut}
            actionId={actionId}
            index={index}
            shortcut={shortcut}
            canRemove={canRemoveShortcut(shortcuts.length)}
            busy={busy}
            conflict={conflictOf(shortcut)}
            refusal={refusalFor(refusedShortcuts, actionId, shortcut)}
            onEdit={submit}
          />
        ))}
        <div className="flex items-center gap-1">
          <ShortcutInput
            shortcutId={actionId}
            value={null}
            disabled={!canAdd || busy}
            label={
              canAdd
                ? t("settings.general.shortcut.list.add")
                : t("settings.general.shortcut.list.addLimit", {
                    limit: MAX_SHORTCUTS_PER_ACTION,
                  })
            }
            testId={`${listId}-add`}
            onCapture={(shortcut) => submit({ kind: "add", shortcut })}
          />
          <ResetButton
            onClick={() => void submit({ kind: "reset" })}
            disabled={busy}
            ariaLabel={t("settings.general.shortcut.list.reset")}
          />
        </div>
        {conflictsFailed && (
          <p
            role="status"
            className="text-xs text-yellow-600 max-w-xs text-right"
            data-testid={`${listId}-conflicts-unavailable`}
          >
            {t("settings.general.shortcut.list.errors.conflictsUnavailable")}
          </p>
        )}
        {error && (
          <p
            role="alert"
            className="text-xs text-red-500 max-w-xs text-right"
            data-testid={`${listId}-error`}
          >
            {t(error.key, error.params)}
          </p>
        )}
      </div>
    </SettingContainer>
  );
};
