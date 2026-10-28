/**
 * Deletion confirmation of one replacement rule (contract state `confirm-delete`, AC-001). The
 * rule leaves the draft only; Save commits the deletion.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../../ui/Dialog";
import { Button } from "../../ui/Button";
import { type DraftRule, SCREEN_IDS } from "./rules-model";

/** Test id of the deletion dialog (contract node `replacement-settings.delete-dialog`). */
const DIALOG_ID = `${SCREEN_IDS.list}-delete-dialog`;

interface DeleteRuleDialogProps {
  rule: DraftRule | null;
  onCancel: () => void;
  onConfirm: () => void;
}

export const DeleteRuleDialog: React.FC<DeleteRuleDialogProps> = ({
  rule,
  onCancel,
  onConfirm,
}) => {
  const { t } = useTranslation();
  return (
    <Dialog
      open={rule !== null}
      title={t("settings.replacements.deleteDialog.title")}
      closeLabel={t("settings.replacements.deleteDialog.cancel")}
      onOpenChange={(open) => {
        if (!open) onCancel();
      }}
      footer={<DeleteRuleFooter onCancel={onCancel} onConfirm={onConfirm} />}
      testId={DIALOG_ID}
    >
      <p className="text-sm" data-testid={`${DIALOG_ID}-body`}>
        {t("settings.replacements.deleteDialog.body", {
          recognized: rule?.recognized ?? "",
          replacement: rule?.replacement ?? "",
        })}
      </p>
    </Dialog>
  );
};

const DeleteRuleFooter: React.FC<
  Pick<DeleteRuleDialogProps, "onCancel" | "onConfirm">
> = ({ onCancel, onConfirm }) => {
  const { t } = useTranslation();
  return (
    <>
      <Button
        variant="secondary"
        onClick={onCancel}
        data-testid={`${DIALOG_ID}-cancel`}
      >
        {t("settings.replacements.deleteDialog.cancel")}
      </Button>
      <Button
        variant="danger"
        onClick={onConfirm}
        data-testid={`${DIALOG_ID}-confirm`}
      >
        {t("settings.replacements.deleteDialog.confirm")}
      </Button>
    </>
  );
};
