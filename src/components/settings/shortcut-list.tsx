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
import { listControls } from "./shortcut-draft-model";
import {
  type MessageNames,
  type ShortcutTestIds,
  type SystemRefusal,
  describeRefusal,
  describeStartFailure,
  draftSystemRefusal,
  isSameEdit,
  legacyWarningsOf,
  listHint,
  shortcutTestIds,
} from "./shortcut-list-feedback";
import {
  type TranslatedMessage,
  canAddShortcut,
  canRemoveShortcut,
  describeConflict,
  limitsFor,
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
  type RowRecordRequests,
  type ShortcutDraft,
  type StoredConflicts,
  useEditSubmitter,
  useMessageNames,
  useRowRecordRequests,
  useShortcutDraft,
  useStoredConflicts,
} from "./use-shortcut-list";
import {
  type ShortcutListState,
  useReportedShortcutScreen,
} from "./shortcut-screen";

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
  rows: RowRecordRequests;
}

interface ListPartProps {
  list: ShortcutListModel;
  single: boolean;
}

/** Gathers the list model; null while the action's binding is unknown. */
const useShortcutList = (
  actionId: string,
  single: boolean,
): ShortcutListModel | null => {
  const { getSetting, refusedShortcuts } = useSettings();
  const bindings = getSetting("bindings");
  const stored = useStoredConflicts(
    JSON.stringify(bindings ?? {}),
    getSetting("keyboard_implementation"),
  );
  const names = useMessageNames();
  const submitter = useEditSubmitter(actionId, names);
  const draft = useShortcutDraft(actionId, submitter);
  const binding = bindings?.[actionId];
  const rows = useRowRecordRequests(binding ? shortcutsOf(binding) : []);
  const parts = binding
    ? { actionId, binding, names, stored, submitter, draft, rows }
    : null;
  const screen = useReportedShortcutScreen(
    actionId,
    parts ? listState(parts, single) : null,
  );
  if (!parts) return null;
  return {
    ...parts,
    ids: shortcutTestIds(actionId, screen),
    refused: refusedShortcuts,
  };
};

/** What decides a list's screen: the model without its test ids and refusals. */
type ListStateParts = Omit<ShortcutListModel, "ids" | "refused">;

/**
 * State of one list for the contract screens: an inline refusal, a system refusal or an
 * inherited conflict shows the conflict screen; a full list shows the limit screen.
 * @param parts the list model
 * @param single the action keeps one shortcut (no limit hint)
 * @returns the list's state
 */
const listState = (
  parts: ListStateParts,
  single: boolean,
): ShortcutListState => {
  const { actionId, names, stored } = parts;
  const shown =
    inlineError(parts) !== null ||
    shownRefusalOf(parts) !== null ||
    legacyWarningsOf(stored.conflicts, actionId, names).length > 0;
  if (shown) return "shortcut-settings-conflict";
  const count = shortcutsOf(parts.binding).length;
  const { max, min } = limitsFor(actionId);
  const full = !single && listHint(count, max, min) === "limit";
  return full ? "shortcut-settings-limit" : null;
};

/**
 * Inline message under the list: the refusal of the last edit, else why the
 * draft recorder could not start, else the refusal of the checked draft. A
 * system refusal has its own alert.
 */
const inlineError = (list: ListStateParts): TranslatedMessage | null => {
  const { feedback } = list.submitter;
  if (feedback.error) return feedback.error;
  const { state } = list.draft;
  if (state.startFailure) return describeStartFailure(state.startFailure);
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
  const { rows } = list;
  return (
    <>
      {shortcuts.map((shortcut) => (
        <ShortcutRow
          key={shortcut}
          actionId={list.actionId}
          shortcut={shortcut}
          ids={list.ids}
          hasRemove={!single}
          canRemove={canRemoveShortcut(
            shortcuts.length,
            limitsFor(list.actionId),
          )}
          busy={busy}
          conflict={conflictOf(list, shortcut)}
          refusal={refusalFor(list.refused, list.actionId, shortcut)}
          recordRequest={rows.requestFor(shortcut)}
          onRequestHandled={rows.handled}
          onEdit={list.submitter.submit}
        />
      ))}
    </>
  );
};

// @spec AC-001: Settings list with add and remove controls — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/**
 * Right-hand column: shortcuts, draft, Add (Save and Discard during a draft)
 * and Reset, hint and messages. Reset is disabled while a draft is open, so it
 * never clears one silently (native session finding N8).
 */
const ShortcutColumn: React.FC<ListPartProps> = ({ list, single }) => {
  const { submitter, draft, ids } = list;
  const count = shortcutsOf(list.binding).length;
  const busy = submitter.feedback.state === "loading";
  const limits = limitsFor(list.actionId);
  const hasRoom = canAddShortcut(count, limits);
  const controls = single ? null : listControls(draft.state, hasRoom, busy);
  const reset = () => void submitter.submit({ kind: "reset" });
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
        controls={controls}
        isFull={!hasRoom}
        limit={limits.max}
        onAdd={draft.open}
        onSave={() => void draft.commit()}
        onDiscard={draft.discard}
        onReset={reset}
        resetDisabled={controls ? !controls.isResetEnabled : busy}
      />
      {!single && (
        <ListHintText
          hint={listHint(count, limits.max, limits.min)}
          count={count}
          ids={ids}
        />
      )}
      <ListMessages
        ids={ids}
        conflictsFailed={list.stored.failed}
        error={inlineError(list)}
      />
    </div>
  );
};

/** The system refusal the list shows, and how Try again repeats it. */
interface ShownRefusal {
  refusal: SystemRefusal;
  retry: () => void;
}

/**
 * The system refusal to show: the last edit's, unless it is the draft's own
 * Add, in which case the draft's refusal is shown (it outlives the feedback
 * of later edits) and Try again commits the draft again.
 */
const shownRefusalOf = (list: ListStateParts): ShownRefusal | null => {
  const { draft, submitter } = list;
  const fromDraft = draftSystemRefusal(draft.state);
  const fromEdit = submitter.feedback.systemRefusal;
  if (fromEdit && !isSameEdit(fromEdit.edit, fromDraft?.edit)) {
    return {
      refusal: fromEdit,
      retry: () => void submitter.submit(fromEdit.edit),
    };
  }
  if (fromDraft)
    return { refusal: fromDraft, retry: () => void draft.commit() };
  return null;
};

/** Inherited conflict and system refusal alerts, inside the same card. */
const ListAlerts: React.FC<Pick<ListPartProps, "list">> = ({ list }) => {
  const busy = list.submitter.feedback.state === "loading";
  const shown = shownRefusalOf(list);
  const { conflicts } = list.stored;
  const warnings = legacyWarningsOf(conflicts, list.actionId, list.names);
  return (
    <>
      {warnings.map((warning) => (
        <LegacyConflictAlert
          key={warning.shortcut}
          warning={warning}
          ids={list.ids}
          busy={busy}
          onChange={list.rows.change}
        />
      ))}
      {shown && (
        <SystemRefusalAlert
          refusal={shown.refusal}
          ids={list.ids}
          names={list.names}
          busy={busy}
          onRetry={shown.retry}
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
  const list = useShortcutList(actionId, single);
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
