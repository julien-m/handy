/** Activate button or "Active" badge shared by the engine panels. */
import React from "react";
import { useTranslation } from "react-i18next";
import { Check } from "lucide-react";
import Badge from "../../ui/Badge";
import { Button } from "../../ui/Button";

interface EngineActivationProps {
  isActive: boolean;
  isBusy: boolean;
  onActivate: () => void;
  testId: string;
}

export const EngineActivation: React.FC<EngineActivationProps> = ({
  isActive,
  isBusy,
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
      disabled={isBusy}
      data-testid={`${testId}-activate`}
    >
      {t("settings.models.engine.activate")}
    </Button>
  );
};
