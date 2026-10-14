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
import { type ApplyNote, applyNote } from "./preview-model";
import { SCREEN_IDS } from "./rules-model";
import {
  canApplyText,
  isPreviewCurrent,
  useReplacementText,
} from "./text-store";
import { CommitFailureNote } from "./commit-failure-note";

const BAR_ID = `${SCREEN_IDS.text}-apply-bar`;

export const ApplyBar: React.FC = () => {
  const { t } = useTranslation();
  const failure = useReplacementText((state) => state.failure);
  const note = useApplyNote();
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
          {t(note.key, note.options)}
        </p>
        <ApplyButtons />
      </div>
    </div>
  );
};

/** Cancel and Apply; Apply shows its loading state while committing. */
const ApplyButtons: React.FC = () => {
  const { t } = useTranslation();
  const enabled = useReplacementText(canApplyText);
  const applying = useReplacementText((state) => state.applying);
  const handleApply = () => void applyWithToast(t);
  return (
    <>
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
        onClick={handleApply}
      >
        {applying
          ? t("settings.replacements.bulk.applying")
          : t("settings.replacements.bulk.apply")}
      </Button>
    </>
  );
};

/** Apply the previewed text and confirm a successful Apply with a toast. */
const applyWithToast = async (t: (key: string) => string): Promise<void> => {
  if (await useReplacementText.getState().apply()) {
    toast.success(t("settings.replacements.bulk.applied"));
  }
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
 * Note of the apply bar for the current text state (see `applyNote`). Each selector returns a
 * stable value, so the bar re-renders only when the note can change.
 * @returns the note's translation key and options
 */
const useApplyNote = (): ApplyNote => {
  const refused = useReplacementText((state) => state.previewFailure !== null);
  const current = useReplacementText(isPreviewCurrent);
  const preview = useReplacementText((state) => state.preview?.preview ?? null);
  return applyNote({ refused, current, preview });
};
