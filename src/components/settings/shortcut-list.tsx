// Settings row listing every shortcut of one action, with record, remove,
// add and reset controls: 1 to 4 shortcuts for a dictation action (004), 0 to
// 2 without default for a prompt target `prompt:<id>` (005). Every change
// goes through the backend `edit_binding_shortcuts` command, which
// validates, registers and saves.
import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import type { ShortcutEdit, StoredShortcutConflict } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { ResetButton } from "../ui/ResetButton";
import { SettingContainer } from "../ui/SettingContainer";
import { ShortcutInput } from "./ShortcutInput";
import { ShortcutRow } from "./shortcut-row";
import {
  type ShortcutLimits,
  type TranslatedMessage,
  canAddShortcut,
  canRemoveShortcut,
  describeConflict,
  describeEditError,
  limitsFor,
  promptIdOf,
  promptNameOfTarget,
  refusalFor,
  shortcutsOf,
  storedConflictFor,
} from "./shortcut-list-model";

interface ShortcutListProps {
  actionId: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/** Test identifiers of a list; the defaults are 004's. */
export interface ShortcutListTestIds {
  list: string;
  /** Row of the shortcut at `index`; its remove button adds `-remove`. */
  row: (index: number) => string;
  add: string;
}

type ListState = "idle" | "loading" | "error" | "success";

const defaultTestIds = (actionId: string): ShortcutListTestIds => ({
  list: `shortcut-list-${actionId}`,
  row: (index) => `shortcut-row-${actionId}-${index}`,
  add: `shortcut-list-${actionId}-add`,
});

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

/**
 * Display name of an action in messages: a prompt target is named after
 * its prompt, a dictation action by its translated name.
 */
const useActionName = (): ((actionId: string) => string) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const prompts = getSetting("post_process_prompts");
  return useCallback(
    (actionId: string) => {
      const promptName = promptNameOfTarget(prompts ?? [], actionId);
      return promptName === null
        ? t(`settings.general.shortcut.bindings.${actionId}.name`, actionId)
        : t("settings.postProcessing.promptLibrary.shortcutOwner", {
            name: promptName,
          });
    },
    [t, prompts],
  );
};

interface ShortcutEditState {
  state: ListState;
  error: TranslatedMessage | null;
  submit: (edit: ShortcutEdit) => Promise<void>;
}

/** Sends one edit of `actionId`'s list and keeps its outcome. */
const useShortcutEdit = (
  actionId: string,
  actionName: (actionId: string) => string,
): ShortcutEditState => {
  const { editBindingShortcuts } = useSettings();
  const [state, setState] = useState<ListState>("idle");
  const [error, setError] = useState<TranslatedMessage | null>(null);
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
  return { state, error, submit };
};

interface ShortcutListEditorProps {
  actionId: string;
  testIds?: ShortcutListTestIds;
  /** `column` (settings rows, 004) or `row` (prompt editor, 005). */
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
  const actionName = useActionName();
  const edit = useShortcutEdit(actionId, actionName);
  const bindings = getSetting("bindings");
  const stored = useStoredConflicts(
    JSON.stringify(bindings ?? {}),
    getSetting("keyboard_implementation"),
  );
  const binding = bindings?.[actionId];
  const conflictOf = (shortcut: string) => {
    const found = storedConflictFor(stored.conflicts, actionId, shortcut);
    return found ? describeConflict(found.conflict, actionName) : undefined;
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
  testIds = defaultTestIds(actionId),
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

// @spec AC-001: Settings list with add and remove controls — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/** Settings row listing and editing every shortcut of `actionId`. */
export const ShortcutList: React.FC<ShortcutListProps> = ({
  actionId,
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const binding = getSetting("bindings")?.[actionId];
  // A dictation action always has a stored binding once settings loaded; a
  // prompt target has none until its first shortcut.
  if (!binding && promptIdOf(actionId) === null) return null;
  return (
    <SettingContainer
      title={t(`settings.general.shortcut.bindings.${actionId}.name`, actionId)}
      description={t(
        `settings.general.shortcut.bindings.${actionId}.description`,
        binding?.description ?? "",
      )}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <ShortcutListEditor actionId={actionId} />
    </SettingContainer>
  );
};
