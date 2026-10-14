// Microphone priority section of the audio settings (feature 001): title, explanation, the
// ordered list with availability, the Add and Clear controls and the availability check line.
import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../../ui/SettingContainer";
import { availabilityOf } from "./availability";
import { PRIORITY_KEY_ROOT } from "./draft-state";
import { AddControl, AddErrorText, CheckLine } from "./priority-controls";
import { PriorityList } from "./priority-list";
import type { PriorityEditor } from "./use-priority-editor";

/** Props of the priority section. */
export interface PrioritySectionProps {
  editor: PriorityEditor;
}

/** Availability check line bound to the editor. */
const EditorCheckLine: React.FC<PrioritySectionProps> = ({ editor }) => (
  <CheckLine
    failed={editor.status.failed}
    observedAt={editor.status.status?.availability_observed_at ?? null}
    showChecked={editor.order.length > 0}
    onRefresh={() => void editor.refreshAll()}
  />
);

/** Add microphone control bound to the editor. */
const EditorAddControl: React.FC<PrioritySectionProps> = ({ editor }) => (
  <AddControl
    enumerated={editor.enumerated}
    addError={editor.draft.addError}
    busy={editor.draft.savePhase === "saving"}
    onAdd={editor.add}
    onOpen={() => void editor.refreshAll()}
  />
);

/** The list with the availability of each draft entry, or a loading line before the settings. */
const EditorList: React.FC<PrioritySectionProps> = ({ editor }) => {
  const { t } = useTranslation();
  if (editor.loading) {
    return (
      <p
        className="text-sm text-mid-gray"
        data-testid="microphone-settings-loading"
        role="status"
      >
        {t("common.loading")}
      </p>
    );
  }
  return (
    <PriorityList
      order={editor.order}
      availability={editor.order.map((name) =>
        availabilityOf(name, editor.status, editor.enumerated),
      )}
      busy={editor.draft.savePhase === "saving"}
      onMove={editor.move}
      onRemove={editor.remove}
    />
  );
};

/**
 * Ordered microphone choices with their controls. A failed availability check shows its pill and
 * Retry above the Add control; otherwise Refresh sits beside it.
 */
export const PrioritySection: React.FC<PrioritySectionProps> = ({ editor }) => {
  const { t } = useTranslation();
  const failed = editor.status.failed;
  return (
    <SettingContainer
      title={t(`${PRIORITY_KEY_ROOT}.title`)}
      description={t(`${PRIORITY_KEY_ROOT}.description`)}
      descriptionMode="inline"
      layout="stacked"
      grouped={true}
      testId="microphone-settings-priority"
      titleTestId="microphone-settings-priority-title"
    >
      <div className="space-y-3">
        <EditorList editor={editor} />
        {failed && <EditorCheckLine editor={editor} />}
        <div className="flex flex-wrap items-center gap-3">
          <EditorAddControl editor={editor} />
          {!failed && <EditorCheckLine editor={editor} />}
        </div>
        {editor.draft.addError && (
          <AddErrorText error={editor.draft.addError} />
        )}
      </div>
    </SettingContainer>
  );
};
