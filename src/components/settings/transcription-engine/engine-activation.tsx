// @spec FR-009: Sign-in required to select ChatGPT — .specs/features/024-transcription-engine/spec.md#fr-009
/** Activate button or "Active" badge shared by the engine panels. */
import React from "react";
import { useTranslation } from "react-i18next";
import { Check } from "lucide-react";
import Badge from "../../ui/Badge";
import { Button } from "../../ui/Button";

interface EngineActivationProps {
  isActive: boolean;
  isBusy: boolean;
  /** False while the engine cannot be selected (e.g. no sign-in); the button stays disabled. */
  isSelectable?: boolean;
  /** Id of the text explaining why the engine cannot be selected yet. */
  describedBy?: string;
  onActivate: () => void;
  testId: string;
}

export const EngineActivation: React.FC<EngineActivationProps> = ({
  isActive,
  isBusy,
  isSelectable = true,
  describedBy,
  onActivate,
  testId,
}) => {
  const { t } = useTranslation();
  if (isActive) {
    return (
      <Badge variant="primary" testId={`${testId}-active`}>
        <Check className="w-3 h-3 mr-1" aria-hidden="true" />
        {t("settings.models.engine.active")}
      </Badge>
    );
  }
  return (
    <Button
      variant="primary-soft"
      size="sm"
      onClick={onActivate}
      disabled={isBusy || !isSelectable}
      aria-describedby={isSelectable ? undefined : describedBy}
      data-testid={`${testId}-activate`}
    >
      {t("settings.models.engine.activate")}
    </Button>
  );
};
