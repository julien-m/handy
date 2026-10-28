/**
 * Saved vocabulary terms as removable chips (legacy and not-sent terms flagged), the empty state,
 * the Add row and the term count.
 */
import React, { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { BookA, ClipboardPaste, Plus, TriangleAlert, X } from "lucide-react";
import type { Occurrence, SettingsCommandError } from "@/bindings";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import {
  canAddTerm,
  chipFlagKey,
  FOCUS_RING,
  removalFocusPosition,
  termInputState,
  formatTermCount,
  VOCABULARY_MAX_TERMS,
  type RemovalTarget,
  type VocabularyScreen,
} from "./vocabulary-model";

interface TermChipProps {
  occurrence: Occurrence;
  screen: VocabularyScreen;
  overBudget: boolean;
  busy: boolean;
  onRemove: (occurrence: Occurrence) => void;
  removeRef: (element: HTMLButtonElement | null) => void;
}

const RemoveButton: React.FC<{
  term: string;
  screen: VocabularyScreen;
  busy: boolean;
  onClick: () => void;
  buttonRef: (element: HTMLButtonElement | null) => void;
}> = ({ term, screen, busy, onClick, buttonRef }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      ref={buttonRef}
      className={`p-1 rounded-md cursor-pointer hover:bg-mid-gray/20 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed ${FOCUS_RING}`}
      data-testid={`${screen}-term-remove`}
      aria-label={t("settings.vocabulary.remove", { term })}
      disabled={busy}
      onClick={onClick}
    >
      <X className="w-3.5 h-3.5" aria-hidden />
    </button>
  );
};

const TermChip: React.FC<TermChipProps> = (props) => {
  const { occurrence, screen, overBudget, onRemove } = props;
  const { t } = useTranslation();
  const flagKey = chipFlagKey(occurrence, overBudget);
  const flag = flagKey ? t(flagKey) : undefined;
  return (
    <li
      className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg border bg-mid-gray/10 border-mid-gray/20 text-sm font-medium ${flag ? "opacity-60" : ""}`}
      data-testid={`${screen}-term`}
      data-instance-id={`term-${occurrence.index}`}
      data-flag={occurrence.issue ?? (overBudget ? "over_budget" : undefined)}
      title={flag}
    >
      {occurrence.issue && (
        <TriangleAlert className="w-3.5 h-3.5 text-yellow-500" aria-hidden />
      )}
      <span>{occurrence.term}</span>
      {flag && <span className="sr-only">{flag}</span>}
      <RemoveButton
        term={occurrence.term}
        screen={screen}
        busy={props.busy}
        onClick={() => onRemove(occurrence)}
        buttonRef={props.removeRef}
      />
    </li>
  );
};

const EmptyTerms: React.FC<{ screen: VocabularyScreen }> = ({ screen }) => {
  const { t } = useTranslation();
  return (
    <div
      className="flex flex-col items-center gap-2 p-6 bg-mid-gray/5 rounded-md border border-mid-gray/20 text-sm text-center"
      data-testid={`${screen}-empty`}
    >
      <BookA className="w-6 h-6 text-mid-gray" aria-hidden />
      <p data-testid={`${screen}-empty-title`}>
        {t("settings.vocabulary.empty.title")}
      </p>
      <p className="text-mid-gray" data-testid={`${screen}-empty-body`}>
        {t("settings.vocabulary.empty.body")}
      </p>
    </div>
  );
};

export interface TermListProps {
  terms: Occurrence[];
  screen: VocabularyScreen;
  overBudget: Set<number>;
  busy: boolean;
  onRemove: (occurrence: Occurrence) => Promise<boolean>;
  focusInput: () => void;
}

/**
 * Once a removal settles, focus the next term (removed) or the same term (failed, after the list
 * is reread), else the term input. The target is set only after the commit answers, so a reload
 * during a failure never consumes it early.
 */
const useRemovalFocus = (terms: Occurrence[], focusInput: () => void) => {
  const removeButtons = useRef<Array<HTMLButtonElement | null>>([]);
  const target = useRef<RemovalTarget | null>(null);
  const [settled, setSettled] = useState(0);
  useEffect(() => {
    const done = target.current;
    if (done === null) return;
    target.current = null;
    const position = removalFocusPosition(terms, done);
    const next = position === null ? null : removeButtons.current[position];
    if (next) next.focus();
    else focusInput();
  }, [settled, terms, focusInput]);
  const settle = (done: RemovalTarget) => {
    target.current = done;
    setSettled((count) => count + 1);
  };
  return { removeButtons, settle };
};

interface TermListBodyProps extends TermListProps {
  remove: (occurrence: Occurrence) => void;
  removeButtons: React.MutableRefObject<Array<HTMLButtonElement | null>>;
}

const TermList: React.FC<TermListBodyProps> = (props) => {
  const { terms, screen, overBudget, busy, removeButtons } = props;
  const { t } = useTranslation();
  return (
    <ul
      className="flex flex-wrap gap-2"
      data-testid={`${screen}-list`}
      aria-label={t("settings.vocabulary.listLabel")}
    >
      {terms.map((occurrence, position) => (
        <TermChip
          key={`${occurrence.index}:${occurrence.term}`}
          occurrence={occurrence}
          screen={screen}
          overBudget={overBudget.has(occurrence.index)}
          busy={busy}
          onRemove={props.remove}
          removeRef={(element) => (removeButtons.current[position] = element)}
        />
      ))}
    </ul>
  );
};

/**
 * Saved terms, or the empty state when there are none. Removal focus lives here so it survives
 * the switch to the empty state after the last term goes.
 * @param props terms, screen prefix, over-budget positions, busy flag and callbacks
 */
export const VocabularyTerms: React.FC<TermListProps> = (props) => {
  const { terms, onRemove, focusInput } = props;
  const { removeButtons, settle } = useRemovalFocus(terms, focusInput);
  const remove = async (occurrence: Occurrence) => {
    const removed = await onRemove(occurrence);
    settle({ index: occurrence.index, term: occurrence.term, removed });
  };
  if (terms.length === 0) return <EmptyTerms screen={props.screen} />;
  return (
    <TermList
      {...props}
      remove={(occurrence) => void remove(occurrence)}
      removeButtons={removeButtons}
    />
  );
};

export interface AddRowProps {
  screen: VocabularyScreen;
  draft: string;
  busy: boolean;
  refusal: SettingsCommandError | null;
  errorId: string | undefined;
  inputId: string;
  /** Id of "Paste a list…", which gets focus back when the paste box closes. */
  pasteId: string;
  onDraft: (draft: string) => void;
  onAdd: () => void;
  onPaste: () => void;
}

const TermInput: React.FC<AddRowProps & { enabled: boolean }> = (props) => {
  const { screen, draft, refusal, enabled } = props;
  const { t } = useTranslation();
  return (
    <Input
      id={props.inputId}
      className="flex-1 min-w-40 max-w-60"
      value={draft}
      placeholder={t("settings.vocabulary.placeholder")}
      aria-label={t("settings.vocabulary.inputLabel")}
      aria-invalid={refusal !== null}
      aria-describedby={props.errorId}
      data-testid={`${screen}-new-term`}
      data-state={termInputState(draft, refusal !== null)}
      onChange={(event) => props.onDraft(event.target.value)}
      onKeyDown={(event) => {
        if (event.key !== "Enter") return;
        event.preventDefault();
        if (enabled) props.onAdd();
      }}
    />
  );
};

/**
 * Add a term from the Add button. Add turns disabled while the commit runs, so the focus moves to
 * the term input first instead of falling on the page.
 * @param props the term input's id and the Add callback
 */
const addFromButton = (props: Pick<AddRowProps, "inputId" | "onAdd">) => {
  document.getElementById(props.inputId)?.focus();
  props.onAdd();
};

/** Why Add is unavailable, for screen readers (a commit is running, or the input is empty). */
const AddHint: React.FC<{ id: string; busy: boolean }> = ({ id, busy }) => {
  const { t } = useTranslation();
  const hint = busy
    ? "settings.vocabulary.busy"
    : "settings.vocabulary.addHint";
  return (
    <span id={id} className="sr-only">
      {t(hint)}
    </span>
  );
};

/** Add, with an accessible explanation while it is unavailable. */
const AddButton: React.FC<AddRowProps & { enabled: boolean }> = (props) => {
  const { screen, busy, enabled } = props;
  const { t } = useTranslation();
  const hintId = useId();
  return (
    <>
      <Button
        variant="secondary"
        className={`inline-flex items-center gap-1 ${FOCUS_RING}`}
        data-testid={`${screen}-add`}
        disabled={!enabled}
        aria-describedby={enabled ? undefined : hintId}
        onClick={() => addFromButton(props)}
      >
        <Plus className="w-4 h-4" aria-hidden />
        {t("settings.vocabulary.add")}
      </Button>
      {!enabled && <AddHint id={hintId} busy={busy} />}
    </>
  );
};

/**
 * Term input, Add and "Paste a list…".
 * @param props screen prefix, draft, busy flag, current refusal and callbacks
 */
export const VocabularyAddRow: React.FC<AddRowProps> = (props) => {
  const { t } = useTranslation();
  const enabled = canAddTerm(props.draft, props.busy);
  return (
    <div className="flex flex-wrap items-center gap-2">
      <TermInput {...props} enabled={enabled} />
      <AddButton {...props} enabled={enabled} />
      <Button
        variant="ghost"
        id={props.pasteId}
        className={`inline-flex items-center gap-1 ${FOCUS_RING}`}
        data-testid={`${props.screen}-paste`}
        disabled={props.busy}
        onClick={props.onPaste}
      >
        <ClipboardPaste className="w-4 h-4" aria-hidden />
        {t("settings.vocabulary.paste")}
      </Button>
    </div>
  );
};

/**
 * "N of 5,000 terms" under the list, the limit formatted for the interface language with Latin
 * digits, as the limit messages.
 * @param props screen prefix and stored occurrence count
 */
export const VocabularyCount: React.FC<{
  screen: VocabularyScreen;
  total: number;
}> = ({ screen, total }) => {
  const { t, i18n } = useTranslation();
  return (
    <p className="text-xs text-mid-gray" data-testid={`${screen}-count`}>
      {t("settings.vocabulary.count", {
        total,
        max: formatTermCount(VOCABULARY_MAX_TERMS, i18n.language),
      })}
    </p>
  );
};
