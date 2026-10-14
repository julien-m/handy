/**
 * Body of the import dialog: the merge/replace mode (each choice previews again) and the preview
 * counts with the first change as an example (contract nodes `.dialog.merge`, `.dialog.replace`,
 * `.dialog.updates`, `.dialog.adds`, `.dialog.errors`, `.dialog.example`).
 */
import React from "react";
import { useTranslation } from "react-i18next";
import type {
  ImportMode as Mode,
  ReplacementPreview,
  ReplacementRule,
} from "@/bindings";
import Badge from "../../ui/Badge";
import {
  firstErrorLine,
  hasPreviewErrors,
  importExample,
  previewCounts,
} from "./preview-model";
import { SCREEN_IDS, issueKey } from "./rules-model";
import { useReplacementFiles } from "./file-store";
import { useReplacementEditor } from "./editor-store";

const DIALOG_ID = `${SCREEN_IDS.import}-dialog`;
const MODES: ReadonlyArray<Mode> = ["merge", "replace"];

export const ImportMode: React.FC = () => {
  const { t } = useTranslation();
  const mode = useReplacementFiles((state) => state.mode);
  const busy = useReplacementFiles((state) => state.phase === "applying");
  return (
    <fieldset className="space-y-2">
      <legend
        className="text-sm font-semibold mb-2"
        data-testid={`${DIALOG_ID}-mode-label`}
      >
        {t("settings.replacements.importDialog.mode")}
      </legend>
      {MODES.map((value) => (
        <label
          key={value}
          className="flex items-center gap-2 text-sm cursor-pointer"
        >
          <input
            type="radio"
            name="replacement-import-mode"
            className="appearance-none w-4 h-4 shrink-0 rounded-full border border-mid-gray/80 bg-mid-gray/10 checked:border-4 checked:border-logo-primary cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-logo-primary"
            checked={mode === value}
            disabled={busy}
            data-testid={`${DIALOG_ID}-${value}`}
            onChange={() => void useReplacementFiles.getState().setMode(value)}
          />
          {t(`settings.replacements.importDialog.${value}`)}
        </label>
      ))}
    </fieldset>
  );
};

export const ImportSummary: React.FC = () => {
  const { t } = useTranslation();
  const preview = useReplacementFiles(
    (state) => state.preview?.preview ?? null,
  );
  const previewing = useReplacementFiles(
    (state) => state.phase === "previewing",
  );
  const saved = useReplacementEditor((state) => state.saved);
  return (
    <div
      className="p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20 space-y-2"
      data-testid={`${DIALOG_ID}-summary`}
      aria-busy={previewing}
    >
      {previewing || preview === null ? (
        <p className="text-sm text-mid-gray">
          {t("settings.replacements.importDialog.previewing")}
        </p>
      ) : (
        <>
          <SummaryCounts preview={preview} />
          <SummaryDetail preview={preview} saved={saved} />
        </>
      )}
    </div>
  );
};

/** Count badge: the "One" text for exactly 1, the "Many" text otherwise (including 0). */
const countText = (
  t: Translate,
  name: "update" | "add" | "delete" | "error",
  count: number,
): string =>
  count === 1
    ? t(`settings.replacements.importDialog.${name}One`)
    : t(`settings.replacements.importDialog.${name}Many`, { count });

const SummaryCounts: React.FC<{ preview: ReplacementPreview }> = ({
  preview,
}) => {
  const { t } = useTranslation();
  const counts = previewCounts(preview);
  const errors = Math.max(counts.errors, preview.set_issues.length);
  return (
    <div className="flex flex-wrap gap-2">
      <span data-testid={`${DIALOG_ID}-updates`}>
        <Badge variant="info">{countText(t, "update", counts.updates)}</Badge>
      </span>
      <span data-testid={`${DIALOG_ID}-adds`}>
        <Badge variant="success">{countText(t, "add", counts.adds)}</Badge>
      </span>
      {counts.deletes > 0 && (
        <Badge variant="warning">
          {countText(t, "delete", counts.deletes)}
        </Badge>
      )}
      <span data-testid={`${DIALOG_ID}-errors`}>
        <Badge variant={errors > 0 ? "error" : "secondary"}>
          {countText(t, "error", errors)}
        </Badge>
      </span>
    </div>
  );
};

interface SummaryDetailProps {
  preview: ReplacementPreview;
  saved: ReadonlyArray<ReplacementRule>;
}

type Translate = (key: string, options?: Record<string, unknown>) => string;

/** The first error, else the first change, else "nothing to import". */
const SummaryDetail: React.FC<SummaryDetailProps> = ({ preview, saved }) => {
  const { t } = useTranslation();
  const hasError = hasPreviewErrors(preview);
  return (
    <p
      className={`text-sm ${hasError ? "text-red-400" : "text-mid-gray"}`}
      data-testid={`${DIALOG_ID}-example`}
    >
      {hasError ? errorText(t, preview) : exampleText(t, preview, saved)}
    </p>
  );
};

/** The whole-set issue, else the first error line and its first reason. */
const errorText = (t: Translate, preview: ReplacementPreview): string => {
  if (preview.set_issues.length > 0) {
    return t(issueKey("rules", preview.set_issues[0]));
  }
  const line = preview.lines.find((row) => row.status === "error");
  const issue = line?.issues[0];
  const reason = issue
    ? t(issueKey(issue.field, issue.kind))
    : t("settings.replacements.preview.duplicateOf", {
        lines: line?.duplicate_lines.join(", ") ?? "",
      });
  return t("settings.replacements.importDialog.firstError", {
    line: firstErrorLine(preview),
    reason,
  });
};

const EXAMPLE_KEYS = {
  update: "exampleUpdate",
  add: "exampleAdd",
  delete: "exampleDelete",
} as const;

/** The first update, addition or deletion, else "nothing to import". */
const exampleText = (
  t: Translate,
  preview: ReplacementPreview,
  saved: ReadonlyArray<ReplacementRule>,
): string => {
  const example = importExample(preview, saved);
  if (example === null)
    return t("settings.replacements.importDialog.noChanges");
  return t(
    `settings.replacements.importDialog.${EXAMPLE_KEYS[example.status]}`,
    {
      ...example,
    },
  );
};
