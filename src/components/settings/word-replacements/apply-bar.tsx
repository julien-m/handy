/**
 * Apply bar of the text view (contract nodes `replacement-settings-bulk.apply-bar`, `.cancel`,
 * `.apply`): Apply commits exactly the previewed text, only while the preview is current and
 * error-free, and shows its loading state; Cancel drops the text edits and returns to the list.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "../../ui/Button";
import { useReplacementEditor } from "./editor-store";
import { firstErrorLine, hasPreviewErrors } from "./preview-model";
import { SCREEN_IDS } from "./rules-model";
import {
  type TextStore,
  canApplyText,
  isPreviewCurrent,
  useReplacementText,
} from "./text-store";
import { CommitFailureNote } from "./commit-failure-note";

const BAR_ID = `${SCREEN_IDS.text}-apply-bar`;

export const ApplyBar: React.FC = () => {
  const { t } = useTranslation();
  const enabled = useReplacementText(canApplyText);
  const applying = useReplacementText((state) => state.applying);
  const failure = useReplacementText((state) => state.failure);
  const note = useReplacementText((state) => applyNote(state, t));
  const apply = async () => {
    if (await useReplacementText.getState().apply()) {
      toast.success(t("settings.replacements.bulk.applied"));
    }
  };
  return (
    <div
      className="bg-background border border-mid-gray/20 rounded-lg p-4 space-y-3"
      data-testid={BAR_ID}
    >
      {failure && <CommitFailureNote failure={failure} onReload={reloadText} />}
      <div className="flex flex-wrap items-center justify-end gap-2">
        <p
          className="flex-1 min-w-48 text-sm text-mid-gray"
          data-testid={`${BAR_ID}-note`}
        >
          {note}
        </p>
        <Button
          variant="secondary"
          disabled={applying}
          data-testid={`${SCREEN_IDS.text}-cancel`}
          onClick={cancelText}
        >
          {t("settings.replacements.bulk.cancel")}
        </Button>
        <Button
          variant="primary"
          disabled={!enabled}
          aria-busy={applying}
          data-testid={`${SCREEN_IDS.text}-apply`}
          onClick={() => void apply()}
        >
          {applying
            ? t("settings.replacements.bulk.applying")
            : t("settings.replacements.bulk.apply")}
        </Button>
      </div>
    </div>
  );
};

/** Drop the text edits and return to the list. */
const cancelText = (): void => {
  useReplacementText.getState().cancel();
  useReplacementEditor.getState().setView("list");
};

/** After a conflict: reread the saved rules, then preview the same text against them. */
const reloadText = async (): Promise<void> => {
  await useReplacementEditor.getState().loadRules();
  await useReplacementText.getState().requestPreview();
};

/**
 * What the apply bar says about the current preview.
 * @param state text state
 * @param t translation function
 * @returns the note
 */
const applyNote = (
  state: TextStore,
  t: (key: string, options?: Record<string, unknown>) => string,
): string => {
  const preview = state.preview?.preview;
  if (state.previewFailure !== null) {
    return t("settings.replacements.commit.unchanged");
  }
  if (!isPreviewCurrent(state) || preview === undefined) {
    return t("settings.replacements.preview.updating");
  }
  if (hasPreviewErrors(preview)) {
    const line = firstErrorLine(preview);
    return preview.error_lines === 1 && line !== null
      ? t("settings.replacements.bulk.fixLine", { line })
      : t("settings.replacements.bulk.fixLines", {
          count: Math.max(preview.error_lines, 1),
        });
  }
  return preview.is_noop
    ? t("settings.replacements.bulk.noChanges")
    : t("settings.replacements.bulk.ready");
};
