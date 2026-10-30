/**
 * Paste box of the vocabulary editor: one term per line, a backend preview of the lines to add,
 * skip or fix, and the Apply bar. Nothing is saved until Apply commits the whole candidate.
 */
import React, { useId } from "react";
import { useTranslation } from "react-i18next";
import type { VocabularyImportPreview } from "@/bindings";
import Badge from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Textarea } from "../../ui/Textarea";
import { FOCUS_RING, VOCABULARY_SCREENS } from "./vocabulary-model";
import {
  applyLabelKey,
  applyNote,
  applyState,
  lineIssueKey,
  previewRows,
  previewSummary,
  type PreviewRow,
} from "./vocabulary-paste-model";

const SCREEN = VOCABULARY_SCREENS.paste;

/** Badge geometry with the warning and error recipes the success badge uses (`*-500/20`). */
const PILL =
  "inline-flex items-center px-3 py-1 rounded-full text-xs font-medium";
const WARNING_PILL = `${PILL} bg-yellow-500/20 text-yellow-400`;
const ERROR_PILL = `${PILL} bg-red-500/20 text-red-400`;

const KindBadge: React.FC<{ kind: PreviewRow["kind"] }> = ({ kind }) => {
  const { t } = useTranslation();
  const label = t(`settings.vocabulary.pasteBox.kind.${kind}`);
  const testId = `${SCREEN}-preview-line-kind`;
  if (kind === "error") {
    return (
      <span className={ERROR_PILL} data-testid={testId}>
        {label}
      </span>
    );
  }
  return (
    <span data-testid={testId}>
      <Badge variant={kind === "add" ? "success" : "secondary"}>{label}</Badge>
    </span>
  );
};

const useRowText = (row: PreviewRow): string => {
  const { t } = useTranslation();
  if (row.kind === "error" && row.issue) {
    return t(lineIssueKey(row.issue), { line: row.line });
  }
  if (row.kind === "duplicate" && row.sameAs) {
    return t("settings.vocabulary.pasteBox.sameAs", {
      term: row.term,
      original: row.sameAs,
    });
  }
  if (row.kind === "duplicate") {
    return t("settings.vocabulary.pasteBox.alreadyListed", { term: row.term });
  }
  return row.term ?? "";
};

const PreviewLine: React.FC<{ row: PreviewRow }> = ({ row }) => {
  const text = useRowText(row);
  return (
    <li
      className="flex items-center gap-3 px-1 py-2"
      data-testid={`${SCREEN}-preview-line`}
      data-instance-id={`vocabulary-line-${row.line}`}
      data-kind={row.kind}
    >
      <KindBadge kind={row.kind} />
      <span
        className={`text-sm ${row.kind === "error" ? "text-red-400" : "text-mid-gray"}`}
        data-testid={`${SCREEN}-preview-line-text`}
      >
        {text}
      </span>
    </li>
  );
};

const SummaryPill: React.FC<{ preview: VocabularyImportPreview }> = ({
  preview,
}) => {
  const { t } = useTranslation();
  const summary = previewSummary(preview.preview);
  const root = "settings.vocabulary.pasteBox.summary";
  const parts = [t(`${root}.add`, { n: summary.added })];
  if (summary.duplicates > 0) {
    const key = summary.duplicates === 1 ? "duplicateOne" : "duplicateMany";
    parts.push(t(`${root}.${key}`, { n: summary.duplicates }));
  }
  if (summary.errors > 0) {
    const key = summary.errors === 1 ? "errorOne" : "errorMany";
    parts.push(t(`${root}.${key}`, { n: summary.errors }));
  }
  const label = parts.join(" · ");
  const testId = `${SCREEN}-preview-summary`;
  if (summary.tone === "warning") {
    return (
      <span className={WARNING_PILL} data-testid={testId}>
        {label}
      </span>
    );
  }
  return (
    <span data-testid={testId}>
      <Badge variant={summary.tone}>{label}</Badge>
    </span>
  );
};

const PreviewLines: React.FC<{ rows: PreviewRow[] }> = ({ rows }) => {
  const { t } = useTranslation();
  if (rows.length === 0) {
    return (
      <p className="text-sm text-mid-gray">
        {t("settings.vocabulary.pasteBox.previewEmpty")}
      </p>
    );
  }
  return (
    <ul data-testid={`${SCREEN}-preview-lines`}>
      {rows.map((row) => (
        <PreviewLine key={row.line} row={row} />
      ))}
    </ul>
  );
};

const PreviewBox: React.FC<{ preview: VocabularyImportPreview | null }> = ({
  preview,
}) => {
  const { t } = useTranslation();
  const rows = preview ? previewRows(preview.preview) : [];
  return (
    <div
      className="flex flex-col gap-2 p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20"
      data-testid={`${SCREEN}-preview`}
      aria-live="polite"
    >
      <div className="flex items-center justify-between gap-2">
        <p className="text-sm" data-testid={`${SCREEN}-preview-title`}>
          {t("settings.vocabulary.pasteBox.previewTitle")}
        </p>
        {preview && <SummaryPill preview={preview} />}
      </div>
      <PreviewLines rows={rows} />
    </div>
  );
};

interface VocabularyPasteProps {
  text: string;
  /** Latest preview when it answers `text`, else null. */
  preview: VocabularyImportPreview | null;
  busy: boolean;
  onText: (text: string) => void;
}

/**
 * Paste editor and its preview, inside the vocabulary card.
 * @param props paste text, current preview, busy flag and text callback
 */
export const VocabularyPaste: React.FC<VocabularyPasteProps> = ({
  text,
  preview,
  busy,
  onText,
}) => {
  const { t } = useTranslation();
  return (
    <>
      <Textarea
        className="w-full"
        rows={5}
        value={text}
        autoFocus
        disabled={busy}
        aria-label={t("settings.vocabulary.pasteBox.label")}
        data-testid={`${SCREEN}-editor`}
        onChange={(event) => onText(event.target.value)}
      />
      <PreviewBox preview={preview} />
    </>
  );
};

interface ApplyBarProps {
  text: string;
  preview: VocabularyImportPreview | null;
  fresh: boolean;
  applying: boolean;
  onCancel: () => void;
  onApply: () => void;
}

/** Classes of a control made inactive with aria-disabled (the Button styles only `disabled`). */
const INACTIVE_CLASSES =
  "aria-disabled:opacity-50 aria-disabled:cursor-not-allowed";

/**
 * Apply. Inactive with aria-disabled rather than disabled, so a focused Apply keeps the focus
 * while it sends and after a failure (the note it points to says why it is inactive).
 */
const ApplyButton: React.FC<ApplyBarProps & { noteId: string }> = (props) => {
  const { preview, fresh, applying } = props;
  const { t } = useTranslation();
  const state = applyState(preview, fresh, applying);
  const added = fresh && preview ? preview.preview.added.length : 0;
  return (
    <Button
      variant="primary"
      className={INACTIVE_CLASSES}
      data-testid={`${SCREEN}-apply`}
      data-state={state}
      aria-disabled={state !== "enabled"}
      aria-busy={applying}
      aria-describedby={props.noteId}
      onClick={() => {
        if (state === "enabled") props.onApply();
      }}
    >
      {t(applyLabelKey(added, applying), { n: added })}
    </Button>
  );
};

/**
 * Note, Cancel and Apply under the paste card.
 * @param props paste text, latest preview and whether it is fresh, commit state and callbacks
 */
export const VocabularyApplyBar: React.FC<ApplyBarProps> = (props) => {
  const { t } = useTranslation();
  const noteId = useId();
  const note = applyNote(props.text, props.preview, props.fresh);
  return (
    <div
      className="flex flex-wrap items-center gap-2 px-4 py-3 bg-background border border-mid-gray/20 rounded-lg"
      data-testid={`${SCREEN}-apply-bar`}
    >
      <p
        id={noteId}
        className="flex-1 min-w-40 text-xs text-mid-gray"
        data-testid={`${SCREEN}-apply-bar-note`}
      >
        {t(note.key, note.values)}
      </p>
      <Button
        variant="secondary"
        className={FOCUS_RING}
        data-testid={`${SCREEN}-cancel`}
        disabled={props.applying}
        onClick={props.onCancel}
      >
        {t("settings.vocabulary.pasteBox.cancel")}
      </Button>
      <ApplyButton {...props} noteId={noteId} />
    </div>
  );
};
