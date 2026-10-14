/**
 * Deletion confirmation of one replacement rule (contract state `confirm-delete`, AC-001). The
 * rule leaves the draft only; Save commits the deletion.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../../ui/Dialog";
import { Button } from "../../ui/Button";
import type { DraftRule } from "./rules-model";

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
    >
      <p className="text-sm">
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
      <Button variant="secondary" onClick={onCancel}>
        {t("settings.replacements.deleteDialog.cancel")}
      </Button>
      <Button variant="danger" onClick={onConfirm}>
        {t("settings.replacements.deleteDialog.confirm")}
      </Button>
    </>
  );
};
