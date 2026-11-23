// @spec FR-002: API engines panel — .specs/features/024-transcription-engine/spec.md#fr-002
/** "API" tab: one card per online transcription engine, plus upcoming providers. */
import React from "react";
import { useTranslation } from "react-i18next";
import type { TranscriptionEngine } from "@/bindings";
import { EngineActivation } from "./engine-activation";
import { useChatGptSignIn } from "./use-chatgpt-sign-in";

interface ApiEnginePanelProps {
  engine: TranscriptionEngine;
  isBusy: boolean;
  onActivate: (engine: TranscriptionEngine) => void;
}

const CARD = "flex flex-col gap-2 rounded-xl border-2 px-4 py-3";

export const ApiEnginePanel: React.FC<ApiEnginePanelProps> = ({
  engine,
  isBusy,
  onActivate,
}) => {
  const { t } = useTranslation();
  const signIn = useChatGptSignIn();
  const isChatGptActive = engine === "chatgpt";

  return (
    <div className="flex flex-col gap-3" data-testid="engine-panel-api">
      <div
        className={`${CARD} ${isChatGptActive ? "border-logo-primary/50 bg-logo-primary/10" : "border-mid-gray/20"}`}
        data-testid="engine-card-chatgpt"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex flex-col gap-1 min-w-0">
            <h3 className="text-base font-semibold text-text">
              {t("settings.models.engine.providers.chatgpt.title")}
            </h3>
            <p className="text-sm text-text/60">
              {t("settings.models.engine.providers.chatgpt.description")}
            </p>
          </div>
          <EngineActivation
            isActive={isChatGptActive}
            isBusy={isBusy}
            onActivate={() => onActivate("chatgpt")}
            testId="engine-card-chatgpt"
          />
        </div>
        <div className="flex flex-col gap-1 text-xs">
          <p className="text-text/60">
            {t("settings.models.engine.chatgptNotice")}
          </p>
          {signIn !== null && (
            <p
              className={
                signIn === "signed_in"
                  ? "text-text/60"
                  : "text-red-600 dark:text-red-400"
              }
              role="status"
              data-testid={`transcription-engine-signin-${signIn}`}
            >
              {signIn === "signed_in"
                ? t("settings.models.engine.status.signedIn")
                : t("settings.models.engine.status.signedOut")}
            </p>
          )}
        </div>
      </div>
      <div
        className={`${CARD} border-dashed border-mid-gray/20 opacity-60`}
        data-testid="engine-card-upcoming"
      >
        <h3 className="text-base font-semibold text-text">
          {t("settings.models.engine.providers.upcoming.title")}
        </h3>
        <p className="text-sm text-text/60">
          {t("settings.models.engine.providers.upcoming.description")}
        </p>
      </div>
    </div>
  );
};
