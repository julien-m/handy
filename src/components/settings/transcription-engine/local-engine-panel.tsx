// @spec FR-008: Local model list locked as fallback — .specs/features/024-transcription-engine/spec.md#fr-008
/** "Local" tab: the on-device model list, with its activation when an API engine is active. */
import React from "react";
import { useTranslation } from "react-i18next";
import { EngineActivation } from "./engine-activation";

interface LocalEnginePanelProps {
  isActive: boolean;
  isBusy: boolean;
  onActivate: () => void;
  /** The model search, filters and lists. */
  children: React.ReactNode;
}

export const LocalEnginePanel: React.FC<LocalEnginePanelProps> = ({
  isActive,
  isBusy,
  onActivate,
  children,
}) => {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-4" data-testid="engine-panel-local">
      {!isActive && (
        <div
          className="flex items-center justify-between gap-4 rounded-lg border border-mid-gray/20 px-4 py-3"
          data-testid="models-fallback-notice"
        >
          <p className="text-sm text-text/70">
            {t("settings.models.engine.fallbackNotice")}
          </p>
          <EngineActivation
            isActive={false}
            isBusy={isBusy}
            onActivate={onActivate}
            testId="engine-local"
          />
        </div>
      )}
      {children}
    </div>
  );
};
