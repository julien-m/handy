// Shortcut controls of one action without their settings row, as the prompt
// editor shows them (005): each stored shortcut with its warnings and remove
// button, then a recorder that adds one, within the action's limits (0 to 2
// without default for a prompt target `prompt:<id>`). Every change goes
// through the backend `edit_binding_shortcuts` command, which validates,
// registers and saves. The dictation settings rows (004) use `ShortcutList`.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ShortcutEdit } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { ResetButton } from "../ui/ResetButton";
import { ShortcutInput } from "./ShortcutInput";
import { ShortcutRow } from "./shortcut-row";
import type { ListState, MessageNames } from "./shortcut-list-feedback";
import {
  type ShortcutLimits,
  type TranslatedMessage,
  canAddShortcut,
  canRemoveShortcut,
  describeConflict,
  describeEditError,
  limitsFor,
  refusalFor,
  shortcutsOf,
  storedConflictFor,
} from "./shortcut-list-model";
import { useMessageNames, useStoredConflicts } from "./use-shortcut-list";

/** Test identifiers of a list. */
export interface ShortcutListTestIds {
  list: string;
  /** Row of the shortcut at `index`; its remove button adds `-remove`. */
  row: (index: number) => string;
  add: string;
}

interface ShortcutEditState {
  state: ListState;
  error: TranslatedMessage | null;
  submit: (edit: ShortcutEdit) => Promise<void>;
}

/** Sends one edit of `actionId`'s list and keeps its outcome. */
const useShortcutEdit = (
  actionId: string,
  names: MessageNames,
): ShortcutEditState => {
  const { editBindingShortcuts } = useSettings();
  const [state, setState] = useState<ListState>("idle");
  const [error, setError] = useState<TranslatedMessage | null>(null);
  const submit = async (edit: ShortcutEdit) => {
    setState("loading");
    setError(null);
    try {
      const failure = await editBindingShortcuts(actionId, edit);
      setError(failure ? describeEditError(failure, names.actionName) : null);
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
  return { state, error, submit };
};

interface ShortcutListEditorProps {
  actionId: string;
  testIds: ShortcutListTestIds;
  /** `column` (stacked rows) or `row` (prompt editor, 005). */
  layout?: "column" | "row";
}

interface AddControlsProps {
  actionId: string;
  limits: ShortcutLimits;
  count: number;
  busy: boolean;
  testId: string;
  submit: (edit: ShortcutEdit) => Promise<void>;
}

/** Label of the add recorder: add, or why no shortcut can be added. */
const useAddLabel = (canAdd: boolean, limits: ShortcutLimits): string => {
  const { t } = useTranslation();
  return canAdd
    ? t("settings.general.shortcut.list.add")
    : t("settings.general.shortcut.list.addLimit", { limit: limits.max });
};

const AddControls: React.FC<AddControlsProps> = (props) => {
  const { actionId, limits, count, busy, testId, submit } = props;
  const { t } = useTranslation();
  const canAdd = canAddShortcut(count, limits);
  const label = useAddLabel(canAdd, limits);
  return (
    <div className="flex items-center gap-1">
      <ShortcutInput
        shortcutId={actionId}
        value={null}
        disabled={!canAdd || busy}
        label={label}
        testId={testId}
        onCapture={(shortcut) => submit({ kind: "add", shortcut })}
      />
      {limits.resettable && (
        <ResetButton
          onClick={() => void submit({ kind: "reset" })}
          disabled={busy}
          ariaLabel={t("settings.general.shortcut.list.reset")}
        />
      )}
    </div>
  );
};

interface ListMessagesProps {
  listId: string;
  conflictsFailed: boolean;
  error: TranslatedMessage | null;
  align: string;
}

const ListMessages: React.FC<ListMessagesProps> = ({
  listId,
  conflictsFailed,
  error,
  align,
}) => {
  const { t } = useTranslation();
  return (
    <>
      {conflictsFailed && (
        <p
          role="status"
          className={`text-xs text-yellow-600 max-w-xs ${align}`}
          data-testid={`${listId}-conflicts-unavailable`}
        >
          {t("settings.general.shortcut.list.errors.conflictsUnavailable")}
        </p>
      )}
      {error && (
        <p
          role="alert"
          className={`text-xs text-red-500 max-w-xs ${align}`}
          data-testid={`${listId}-error`}
        >
          {t(error.key, error.params)}
        </p>
      )}
    </>
  );
};

/** Everything one list shows and the edit it sends. */
const useShortcutListModel = (actionId: string) => {
  const { getSetting, refusedShortcuts } = useSettings();
  const names = useMessageNames();
  const edit = useShortcutEdit(actionId, names);
  const bindings = getSetting("bindings");
  const stored = useStoredConflicts(
    JSON.stringify(bindings ?? {}),
    getSetting("keyboard_implementation"),
  );
  const binding = bindings?.[actionId];
  const conflictOf = (shortcut: string) => {
    const found = storedConflictFor(stored.conflicts, actionId, shortcut);
    return found
      ? describeConflict(found.conflict, names.actionName)
      : undefined;
  };
  return {
    ...edit,
    shortcuts: binding ? shortcutsOf(binding) : [],
    limits: limitsFor(actionId),
    busy: edit.state === "loading",
    conflictsFailed: stored.failed,
    conflictOf,
    refusalOf: (shortcut: string) =>
      refusalFor(refusedShortcuts, actionId, shortcut),
  };
};

type ShortcutListModel = ReturnType<typeof useShortcutListModel>;

interface ListPartProps {
  actionId: string;
  testIds: ShortcutListTestIds;
  list: ShortcutListModel;
}

/** One row per stored shortcut, with its warnings. */
const ShortcutRows: React.FC<ListPartProps> = ({ actionId, testIds, list }) => (
  <>
    {list.shortcuts.map((shortcut, index) => (
      <ShortcutRow
        key={shortcut}
        actionId={actionId}
        rowId={testIds.row(index)}
        shortcut={shortcut}
        canRemove={canRemoveShortcut(list.shortcuts.length, list.limits)}
        busy={list.busy}
        conflict={list.conflictOf(shortcut)}
        refusal={list.refusalOf(shortcut)}
        onEdit={list.submit}
      />
    ))}
  </>
);

// @spec AC-006: Prompt shortcuts reuse the shared list — .specs/features/005-custom-ai-prompts/spec.md#ac-006
/**
 * Shortcut controls of `actionId` without their settings row. Stored
 * shortcuts that already conflict (kept as before an upgrade) or that the
 * system refused when post-processing was enabled carry a warning and stay
 * editable.
 */
export const ShortcutListEditor: React.FC<ShortcutListEditorProps> = ({
  actionId,
  testIds,
  layout = "column",
}) => {
  const list = useShortcutListModel(actionId);
  const isRow = layout === "row";
  const layoutClasses = isRow
    ? "flex flex-wrap items-center gap-2"
    : "flex flex-col items-end gap-1";
  return (
    <div
      className={layoutClasses}
      data-testid={testIds.list}
      data-state={list.state}
      aria-busy={list.busy}
    >
      <ShortcutRows actionId={actionId} testIds={testIds} list={list} />
      <AddControls
        actionId={actionId}
        limits={list.limits}
        count={list.shortcuts.length}
        busy={list.busy}
        testId={testIds.add}
        submit={list.submit}
      />
      <ListMessages
        listId={testIds.list}
        conflictsFailed={list.conflictsFailed}
        error={list.error}
        align={isRow ? "basis-full" : "text-right"}
      />
    </div>
  );
};
