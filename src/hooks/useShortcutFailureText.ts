// Translated reason of a failed shortcut change, shared by the recorders
// that save through the legacy `change_binding`/`reset_binding` commands.
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { describeShortcutFailure } from "../components/settings/shortcut-list-model";

/**
 * Returns a function turning an error thrown by `updateBinding` or
 * `resetBinding` into a translated reason (the raw text when the backend
 * gave no typed refusal).
 */
export const useShortcutFailureText = (): ((error: unknown) => string) => {
  const { t } = useTranslation();
  return useCallback(
    (error: unknown) => {
      const failure = describeShortcutFailure(error, (actionId) =>
        t(`settings.general.shortcut.bindings.${actionId}.name`, actionId),
      );
      return typeof failure === "string"
        ? failure
        : t(failure.key, failure.params);
    },
    [t],
  );
};
