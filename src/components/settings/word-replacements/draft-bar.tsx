/**
 * Unsaved-changes bar of the list (contract nodes `replacement-settings.draft`, `.discard`,
 * `.save`): shown while the draft differs from the saved rules. Save is enabled only for a valid
 * changed draft, shows its loading state while committing, and a refusal keeps the draft with
 * its reason; a revision conflict offers Reload instead of overwriting (has_dirty_state,
 * is_submittable, async_action).
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "../../ui/Button";
import { canSave, isListDirty, useReplacementEditor } from "./editor-store";
import { SCREEN_IDS } from "./rules-model";
import { CommitFailureNote } from "./commit-failure-note";

const DRAFT_ID = `${SCREEN_IDS.list}-draft`;

export const DraftBar: React.FC = () => {
  const { t } = useTranslation();
  const dirty = useReplacementEditor(isListDirty);
  const enabled = useReplacementEditor(canSave);
  const saving = useReplacementEditor((state) => state.commit === "saving");
  const failure = useReplacementEditor((state) => state.failure);
  if (!dirty && failure === null) return null;
  const save = async () => {
    if (await useReplacementEditor.getState().save()) {
      toast.success(t("settings.replacements.draft.saved"));
    }
  };
  return (
    <div
      className="bg-background border border-mid-gray/20 rounded-lg p-4 space-y-3"
      data-testid={DRAFT_ID}
    >
      {failure && (
        <CommitFailureNote
          failure={failure}
          onReload={() => void useReplacementEditor.getState().loadRules()}
        />
      )}
      <div className="flex flex-wrap items-center justify-end gap-2">
        <p
          className="flex-1 min-w-48 text-sm text-mid-gray"
          data-testid={`${DRAFT_ID}-note`}
        >
          {t("settings.replacements.draft.note")}
        </p>
        <Button
          variant="secondary"
          disabled={saving}
          data-testid={`${SCREEN_IDS.list}-discard`}
          onClick={() => useReplacementEditor.getState().discard()}
        >
          {t("settings.replacements.draft.discard")}
        </Button>
        <Button
          variant="primary"
          disabled={!enabled}
          aria-busy={saving}
          data-testid={`${SCREEN_IDS.list}-save`}
          onClick={() => void save()}
        >
          {saving
            ? t("settings.replacements.draft.saving")
            : t("settings.replacements.draft.save")}
        </Button>
      </div>
    </div>
  );
};
