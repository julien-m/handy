/**
 * Text view of the replacement rules (screen `replacement-settings-bulk`): the exchange text,
 * its format hint and the live preview, refreshed after a pause in typing (AC-003). The text is
 * read-only while Apply runs, so the applied rules never overwrite text typed meanwhile.
 */
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Textarea } from "../../ui/Textarea";
import { SCREEN_IDS } from "./rules-model";
import { PreviewPanel } from "./preview-panel";
import { useReplacementText } from "./text-store";

/** Pause before previewing typed or pasted text; one preview per pause, not per key. */
const PREVIEW_DELAY_MS = 300;

export const BulkEditor: React.FC = () => {
  const { t } = useTranslation();
  const text = useReplacementText((state) => state.text);
  const applying = useReplacementText((state) => state.applying);
  usePreviewWhileTyping(text);
  return (
    <div className="space-y-3">
      <Textarea
        className="w-full min-h-[160px]"
        spellCheck={false}
        aria-label={t("settings.replacements.bulk.editor")}
        value={text}
        readOnly={applying}
        data-testid={`${SCREEN_IDS.text}-editor`}
        onChange={(event) =>
          useReplacementText.getState().setText(event.target.value)
        }
      />
      <p
        className="text-sm text-mid-gray"
        data-testid={`${SCREEN_IDS.text}-hint`}
      >
        {t("settings.replacements.bulk.hint")}
      </p>
      <PreviewPanel />
    </div>
  );
};

/**
 * Preview `text` after a pause in typing, unless its preview is already shown.
 * @param text current editor text
 */
const usePreviewWhileTyping = (text: string): void => {
  useEffect(() => {
    if (useReplacementText.getState().previewText === text) return;
    const timer = window.setTimeout(() => {
      void useReplacementText.getState().requestPreview();
    }, PREVIEW_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [text]);
};
