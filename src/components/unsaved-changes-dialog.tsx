/**
 * Section navigation that respects unsaved edits (feature 002, has_dirty_state): leaving a
 * section whose editor holds a draft asks to stay or to discard the edits first.
 */
import React, { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "./ui/Dialog";
import { Button } from "./ui/Button";
import {
  discardUnsavedChanges,
  hasUnsavedChanges,
} from "@/stores/unsaved-changes";

/** Navigation request and the confirmation dialog to render with it. */
export interface GuardedNavigation<T> {
  readonly request: (target: T) => void;
  readonly dialog: React.ReactNode;
}

/**
 * Wrap a navigation setter so a draft is never dropped silently.
 * @param navigate the unguarded setter (for example the section state setter)
 * @returns the guarded request and the dialog element to render
 */
export const useGuardedNavigation = <T,>(
  navigate: (target: T) => void,
): GuardedNavigation<T> => {
  const [pending, setPending] = useState<{ target: T } | null>(null);
  const request = useCallback(
    (target: T) => {
      if (hasUnsavedChanges()) setPending({ target });
      else navigate(target);
    },
    [navigate],
  );
  const leave = () => {
    if (pending === null) return;
    discardUnsavedChanges();
    setPending(null);
    navigate(pending.target);
  };
  const dialog = (
    <UnsavedChangesDialog
      open={pending !== null}
      onStay={() => setPending(null)}
      onDiscard={leave}
    />
  );
  return { request, dialog };
};

interface UnsavedChangesDialogProps {
  open: boolean;
  onStay: () => void;
  onDiscard: () => void;
}

const UnsavedChangesDialog: React.FC<UnsavedChangesDialogProps> = ({
  open,
  onStay,
  onDiscard,
}) => {
  const { t } = useTranslation();
  return (
    <Dialog
      open={open}
      title={t("settings.unsavedChanges.title")}
      closeLabel={t("settings.unsavedChanges.stay")}
      onOpenChange={(next) => {
        if (!next) onStay();
      }}
      footer={
        <>
          <Button variant="secondary" onClick={onStay}>
            {t("settings.unsavedChanges.stay")}
          </Button>
          <Button variant="danger" onClick={onDiscard}>
            {t("settings.unsavedChanges.discard")}
          </Button>
        </>
      }
    >
      <p className="text-sm">{t("settings.unsavedChanges.description")}</p>
    </Dialog>
  );
};
