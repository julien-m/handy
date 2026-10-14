/**
 * Line-by-line preview of a bulk text (contract nodes `replacement-settings-bulk.preview`,
 * `.preview.summary`, `.preview.lines`, `.preview.line`): each listed line with its outcome
 * (add, update, delete, unchanged, ignored, error) and the summary that says whether Apply is
 * available (AC-003).
 */
import React from "react";
import { useTranslation } from "react-i18next";
import type {
  PreviewLine,
  PreviewStatus,
  ReplacementPreview,
} from "@/bindings";
import Badge from "../../ui/Badge";
import { Alert } from "../../ui/Alert";
import {
  hasPreviewErrors,
  ignoredKey,
  previewCounts,
  previewKindKey,
  previewLineKey,
} from "./preview-model";
import { SCREEN_IDS, issueKey } from "./rules-model";
import { isPreviewCurrent, useReplacementText } from "./text-store";

const PREVIEW_ID = `${SCREEN_IDS.text}-preview`;
const LINE_ID = `${PREVIEW_ID}-line`;

const BADGES: Record<
  PreviewStatus,
  "success" | "info" | "warning" | "secondary" | "error"
> = {
  add: "success",
  update: "info",
  delete: "warning",
  unchanged: "secondary",
  ignored: "secondary",
  error: "error",
};

export const PreviewPanel: React.FC = () => {
  const { t } = useTranslation();
  const preview = useReplacementText((state) => state.preview?.preview ?? null);
  const failure = useReplacementText((state) => state.previewFailure);
  const current = useReplacementText(isPreviewCurrent);
  return (
    <div
      className="border border-mid-gray/20 rounded-lg p-4 space-y-3"
      data-testid={PREVIEW_ID}
      aria-busy={!current}
    >
      <div className="flex items-center justify-between gap-2">
        <h4 className="text-sm font-medium" data-testid={`${PREVIEW_ID}-title`}>
          {t("settings.replacements.preview.title")}
        </h4>
        {preview && <PreviewSummary preview={preview} />}
      </div>
      {failure && <Alert variant="error">{t(failure.key)}</Alert>}
      {!current && !failure && (
        <p className="text-sm text-mid-gray">
          {t("settings.replacements.preview.updating")}
        </p>
      )}
      {preview && <PreviewLines preview={preview} />}
    </div>
  );
};

const PreviewSummary: React.FC<{ preview: ReplacementPreview }> = ({
  preview,
}) => {
  const { t } = useTranslation();
  const counts = previewCounts(preview);
  const errors = Math.max(counts.errors, preview.set_issues.length);
  const [variant, text] = hasPreviewErrors(preview)
    ? ([
        "error",
        errors === 1
          ? t("settings.replacements.preview.errorOne")
          : t("settings.replacements.preview.errorMany", { count: errors }),
      ] as const)
    : preview.is_noop
      ? (["secondary", t("settings.replacements.preview.noChanges")] as const)
      : ([
          "info",
          t("settings.replacements.preview.changes", { ...counts }),
        ] as const);
  return (
    <span data-testid={`${PREVIEW_ID}-summary`}>
      <Badge variant={variant}>{text}</Badge>
    </span>
  );
};

const PreviewLines: React.FC<{ preview: ReplacementPreview }> = ({
  preview,
}) => {
  const { t } = useTranslation();
  return (
    <>
      <ul className="space-y-2" data-testid={`${PREVIEW_ID}-lines`}>
        {preview.set_issues.map((kind) => (
          <li key={kind} className="text-sm text-red-400">
            {t(issueKey("rules", kind))}
          </li>
        ))}
        {preview.lines.map((line) => (
          <PreviewRow key={previewLineKey(line)} line={line} />
        ))}
      </ul>
      {preview.omitted_lines > 0 && (
        <p className="text-xs text-mid-gray">
          {t("settings.replacements.preview.omitted", {
            count: preview.omitted_lines,
          })}
        </p>
      )}
    </>
  );
};

const PreviewRow: React.FC<{ line: PreviewLine }> = ({ line }) => {
  const { t } = useTranslation();
  return (
    <li
      className="flex items-center gap-3"
      data-testid={LINE_ID}
      data-instance-id={`preview-line-${line.line_number ?? previewLineKey(line)}`}
    >
      <span
        className="w-8 shrink-0 text-end text-sm text-mid-gray"
        data-testid={`${LINE_ID}-number`}
      >
        {line.line_number ?? "–"}
      </span>
      <span data-testid={`${LINE_ID}-kind`} className="shrink-0">
        <Badge variant={BADGES[line.status]}>
          {t(previewKindKey(line.status))}
        </Badge>
      </span>
      <span
        className={`min-w-0 break-words text-sm ${line.status === "error" ? "text-red-400" : "text-mid-gray"}`}
        data-testid={`${LINE_ID}-text`}
      >
        {previewLineText(line, t)}
      </span>
    </li>
  );
};

/**
 * Text of a preview row: why it is ignored, why it is in error, or the rule it yields.
 * @param line preview row
 * @param t translation function
 * @returns the row's text
 */
const previewLineText = (
  line: PreviewLine,
  t: (key: string, options?: Record<string, unknown>) => string,
): string => {
  if (line.status === "ignored") return t(ignoredKey(line.ignored));
  if (line.status === "error") {
    const reasons = line.issues.map((issue) =>
      t(issueKey(issue.field, issue.kind)),
    );
    if (line.duplicate_lines.length > 0) {
      reasons.push(
        t("settings.replacements.preview.duplicateOf", {
          lines: line.duplicate_lines.join(", "),
        }),
      );
    }
    return reasons.join(" ");
  }
  return `${line.recognized ?? ""} → ${line.replacement ?? ""}`;
};
