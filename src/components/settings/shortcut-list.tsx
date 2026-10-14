// Settings row listing every shortcut of one action (1 to 4), with record,
// remove, add and reset controls, followed inside the same card by the
// alerts of inherited conflicts and system refusals. Every change goes
// through the backend `edit_binding_shortcuts` command, which validates,
// registers and saves; an added shortcut is first checked without saving.
import React from "react";
import { useTranslation } from "react-i18next";
import type { RefusedShortcut, ShortcutBinding } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { SettingContainer } from "../ui/SettingContainer";
import { addControlState, canCommit } from "./shortcut-draft-model";
import {
  type MessageNames,
  type ShortcutTestIds,
  describeRefusal,
  legacyWarningsOf,
  listHint,
  shortcutTestIds,
} from "./shortcut-list-feedback";
import {
  type TranslatedMessage,
  canAddShortcut,
  canRemoveShortcut,
  describeConflict,
  refusalFor,
  shortcutsOf,
  storedConflictFor,
} from "./shortcut-list-model";
import {
  LegacyConflictAlert,
  SystemRefusalAlert,
} from "./shortcut-list-alerts";
import {
  AddRow,
  DraftRow,
  ListHintText,
  ListMessages,
  ShortcutRow,
} from "./shortcut-list-rows";
import {
  type EditSubmitter,
  type RowRecordRequest,
  type ShortcutDraft,
  type StoredConflicts,
  useEditSubmitter,
  useMessageNames,
  useRowRecordRequest,
  useShortcutDraft,
  useStoredConflicts,
} from "./use-shortcut-list";

interface ShortcutListProps {
  actionId: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  /** The action keeps one shortcut (cancel): no Add, no Remove, no hint. */
  single?: boolean;
}

/** Everything one list renders, gathered from the settings and its hooks. */
interface ShortcutListModel {
  actionId: string;
  binding: ShortcutBinding;
  ids: ShortcutTestIds;
  names: MessageNames;
  stored: StoredConflicts;
  refused: RefusedShortcut[];
  submitter: EditSubmitter;
  draft: ShortcutDraft;
  rowRequest: RowRecordRequest | null;
  change: (shortcut: string) => void;
}

interface ListPartProps {
  list: ShortcutListModel;
  single: boolean;
}

/** Gathers the list model; null while the action's binding is unknown. */
const useShortcutList = (actionId: string): ShortcutListModel | null => {
  const { getSetting, refusedShortcuts } = useSettings();
  const bindings = getSetting("bindings");
  const stored = useStoredConflicts(
    JSON.stringify(bindings ?? {}),
    getSetting("keyboard_implementation"),
  );
  const names = useMessageNames();
  const submitter = useEditSubmitter(actionId, names);
  const draft = useShortcutDraft(actionId, submitter);
  const [rowRequest, change] = useRowRecordRequest();
  const binding = bindings?.[actionId];
  if (!binding) return null;
  const ids = shortcutTestIds(actionId);
  const refused = refusedShortcuts;
  return {
    actionId,
    binding,
    ids,
    names,
    stored,
    refused,
    submitter,
    draft,
    rowRequest,
    change,
  };
};

/**
 * Inline message under the list: the refusal of the last edit, else the
 * refusal of the checked draft. A system refusal has its own alert.
 */
const inlineError = (list: ShortcutListModel): TranslatedMessage | null => {
  const { feedback } = list.submitter;
  if (feedback.error) return feedback.error;
  const { state } = list.draft;
  if (state.phase !== "invalid" || state.refusal === null) return null;
  if (state.refusal.code === "registration_failed") return null;
  return describeRefusal(state.refusal, state.shortcut, list.names);
};

/** Warning text of a stored shortcut that conflicts with another one. */
const conflictOf = (
  list: ShortcutListModel,
  shortcut: string,
): TranslatedMessage | undefined => {
  const stored = storedConflictFor(
    list.stored.conflicts,
    list.actionId,
    shortcut,
  );
  return stored
    ? describeConflict(stored.conflict, list.names.actionName)
    : undefined;
};

/** The stored shortcuts, first one first. */
const ShortcutRows: React.FC<ListPartProps & { busy: boolean }> = ({
  list,
  single,
  busy,
}) => {
  const shortcuts = shortcutsOf(list.binding);
  const { rowRequest } = list;
  return (
    <>
      {shortcuts.map((shortcut) => (
        <ShortcutRow
          key={shortcut}
          actionId={list.actionId}
          shortcut={shortcut}
          ids={list.ids}
          hasRemove={!single}
          canRemove={canRemoveShortcut(shortcuts.length)}
          busy={busy}
          conflict={conflictOf(list, shortcut)}
          refusal={refusalFor(list.refused, list.actionId, shortcut)}
          recordRequest={
            rowRequest?.shortcut === shortcut ? rowRequest.count : 0
          }
          onEdit={list.submitter.submit}
        />
      ))}
    </>
  );
};

// @spec AC-001: Settings list with add and remove controls — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/** Right-hand column: shortcuts, draft, Add and Reset, hint and messages. */
const ShortcutColumn: React.FC<ListPartProps> = ({ list, single }) => {
  const { submitter, draft, ids } = list;
  const count = shortcutsOf(list.binding).length;
  const busy = submitter.feedback.state === "loading";
  const hasRoom = canAddShortcut(count);
  const isDraftOpen = draft.state.phase !== "closed";
  const reset = () => {
    draft.discard();
    void submitter.submit({ kind: "reset" });
  };
  return (
    <div
      className="flex flex-col items-end gap-1"
      data-testid={ids.list}
      data-state={submitter.feedback.state}
      aria-busy={busy}
    >
      <ShortcutRows list={list} single={single} busy={busy} />
      {!single && (
        <DraftRow
          actionId={list.actionId}
          ids={ids}
          draft={draft}
          busy={busy}
        />
      )}
      <AddRow
        ids={ids}
        addState={single ? null : addControlState(draft.state, hasRoom, busy)}
        isFull={!hasRoom}
        onAdd={isDraftOpen ? () => void draft.commit() : draft.open}
        onReset={reset}
        resetDisabled={busy}
      />
      {!single && (
        <ListHintText hint={listHint(count)} count={count} ids={ids} />
      )}
      <ListMessages
        ids={ids}
        conflictsFailed={list.stored.failed}
        error={inlineError(list)}
      />
    </div>
  );
};

/** Retries the refused draft, else the refused edit. */
const retryOf = (list: ShortcutListModel) => (): void => {
  const { draft, submitter } = list;
  if (draft.state.phase === "invalid" && canCommit(draft.state)) {
    void draft.commit();
    return;
  }
  const refusal = submitter.feedback.systemRefusal;
  if (refusal) void submitter.submit(refusal.edit);
};

/** Inherited conflict and system refusal alerts, inside the same card. */
const ListAlerts: React.FC<Pick<ListPartProps, "list">> = ({ list }) => {
  const busy = list.submitter.feedback.state === "loading";
  const refusal = list.submitter.feedback.systemRefusal;
  const warnings = legacyWarningsOf(
    list.stored.conflicts,
    list.actionId,
    list.names,
  );
  return (
    <>
      {warnings.map((warning) => (
        <LegacyConflictAlert
          key={warning.shortcut}
          warning={warning}
          ids={list.ids}
          busy={busy}
          onChange={list.change}
        />
      ))}
      {refusal && (
        <SystemRefusalAlert
          refusal={refusal}
          ids={list.ids}
          names={list.names}
          busy={busy}
          onRetry={retryOf(list)}
        />
      )}
    </>
  );
};

/**
 * Lists and edits every shortcut of `actionId`. Stored shortcuts that
 * already conflict (kept as before an upgrade) or that the system refused
 * when post-processing was enabled carry a warning and stay editable.
 */
export const ShortcutList: React.FC<ShortcutListProps> = ({
  actionId,
  descriptionMode = "tooltip",
  grouped = false,
  single = false,
}) => {
  const { t } = useTranslation();
  const list = useShortcutList(actionId);
  if (!list) return null;
  return (
    <>
      <SettingContainer
        title={list.names.actionName(actionId)}
        description={t(
          `settings.general.shortcut.bindings.${actionId}.description`,
          list.binding.description,
        )}
        descriptionMode={descriptionMode}
        grouped={grouped}
        layout="horizontal"
        testId={list.ids.container}
      >
        <ShortcutColumn list={list} single={single} />
      </SettingContainer>
      <ListAlerts list={list} />
    </>
  );
};
