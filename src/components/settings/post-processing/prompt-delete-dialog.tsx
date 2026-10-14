// Delete confirmation of a prompt (005 AC-001, AC-006): names the shortcuts
// that will be removed and the default that becomes None, then deletes. A
// refusal (a dictation in progress while the prompt has shortcuts) is
// reported translated and nothing is removed.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { commands } from "@/bindings";
import type { ListedPrompt } from "@/bindings";
import { useOsType } from "../../../hooks/useOsType";
import { formatKeyCombination } from "../../../lib/utils/keyboard";
import { settingsErrorKey } from "../../../stores/settings-errors";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { PROMPT_KEYS } from "./prompt-library-model";

const KEY = `${PROMPT_KEYS}.deleteDialog`;

/** Deletion state of the edited prompt. */
export interface PromptDeletion {
  isConfirming: boolean;
  isDeleting: boolean;
  /** Translation key of the last refusal. */
  error: string | null;
  ask: () => void;
  dismiss: () => void;
  confirm: () => Promise<void>;
}

/** Shortcuts shown as the user sees them on their keyboard. */
const useShortcutText = (): ((shortcuts: string[]) => string) => {
  const osType = useOsType();
  return (shortcuts) =>
    shortcuts
      .map((shortcut) => formatKeyCombination(shortcut, osType))
      .join(", ");
};

/** Announces a deletion with the shortcuts it removed. */
const useDeletedToast = (): ((
  prompt: ListedPrompt,
  removed: string[],
) => void) => {
  const { t } = useTranslation();
  const shortcutText = useShortcutText();
  return (prompt, removed) => {
    const shortcuts = shortcutText(removed);
    toast.success(
      shortcuts
        ? t(`${KEY}.deletedWithShortcuts`, { name: prompt.name, shortcuts })
        : t(`${KEY}.deleted`, { name: prompt.name }),
    );
  };
};

/**
 * Deletes `prompt` and announces it.
 * @returns The translation key of the refusal, or null once deleted.
 */
const deletePrompt = async (
  prompt: ListedPrompt,
  announce: (prompt: ListedPrompt, removed: string[]) => void,
): Promise<string | null> => {
  try {
    const result = await commands.deletePostProcessPrompt(prompt.id);
    if (result.status === "error") return settingsErrorKey(result.error);
    announce(prompt, result.data);
    return null;
  } catch (failure) {
    console.error("Failed to delete the prompt:", failure);
    return "errors.settings.generic";
  }
};

// @spec AC-006: Deleting a prompt removes its shortcuts after confirmation — .specs/features/005-custom-ai-prompts/spec.md#ac-006
/**
 * Confirms and deletes `prompt`, then reports the removed shortcuts.
 * @param prompt Prompt to delete (null while the library loads).
 * @param onDeleted Called once the prompt is deleted.
 */
export const usePromptDeletion = (
  prompt: ListedPrompt | undefined,
  onDeleted: () => void,
): PromptDeletion => {
  const announce = useDeletedToast();
  const [isConfirming, setIsConfirming] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const confirm = async () => {
    if (!prompt) return;
    setIsDeleting(true);
    const refusal = await deletePrompt(prompt, announce);
    setError(refusal);
    setIsDeleting(false);
    setIsConfirming(false);
    if (!refusal) onDeleted();
  };
  const ask = () => {
    setError(null);
    setIsConfirming(true);
  };
  const dismiss = () => setIsConfirming(false);
  return { isConfirming, isDeleting, error, ask, dismiss, confirm };
};

interface PromptDeleteDialogProps {
  testId: string;
  prompt: ListedPrompt;
  isDefault: boolean;
  deletion: PromptDeletion;
}

/** Cancel and Delete of the confirmation. */
const DialogFooter: React.FC<PromptDeleteDialogProps> = ({
  testId,
  deletion,
}) => {
  const { t } = useTranslation();
  return (
    <>
      <Button
        variant="secondary"
        onClick={deletion.dismiss}
        disabled={deletion.isDeleting}
      >
        {t(`${KEY}.cancel`)}
      </Button>
      <Button
        variant="danger"
        onClick={() => void deletion.confirm()}
        disabled={deletion.isDeleting}
        data-testid={`${testId}-delete-confirm`}
      >
        {t(`${KEY}.confirm`)}
      </Button>
    </>
  );
};

/** What the deletion removes: shortcuts, the default, for good. */
const DialogBody: React.FC<PromptDeleteDialogProps> = (props) => {
  const { testId, prompt, isDefault } = props;
  const { t } = useTranslation();
  const shortcutText = useShortcutText();
  return (
    <div className="space-y-2 text-sm" data-testid={`${testId}-delete-dialog`}>
      {prompt.shortcuts.length > 0 && (
        <p>
          {t(`${KEY}.shortcuts`, { shortcuts: shortcutText(prompt.shortcuts) })}
        </p>
      )}
      {isDefault && <p>{t(`${KEY}.default`)}</p>}
      <p className="text-mid-gray">{t(`${KEY}.body`)}</p>
    </div>
  );
};

/** The confirmation dialog. */
export const PromptDeleteDialog: React.FC<PromptDeleteDialogProps> = (
  props,
) => {
  const { prompt, deletion } = props;
  const { t } = useTranslation();
  return (
    <Dialog
      open={deletion.isConfirming}
      onOpenChange={(open) => {
        if (!open) deletion.dismiss();
      }}
      title={t(`${KEY}.title`, { name: prompt.name })}
      closeLabel={t(`${KEY}.close`)}
      dismissible={!deletion.isDeleting}
      footer={<DialogFooter {...props} />}
    >
      <DialogBody {...props} />
    </Dialog>
  );
};
