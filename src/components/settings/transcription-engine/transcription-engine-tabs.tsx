// @spec FR-002: Engine tabs — .specs/features/024-transcription-engine/spec.md#fr-002
/** Local / API tabs of Settings › Models; each tab activates its own engine. */
import React, { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cloud, HardDrive } from "lucide-react";
import type { TranscriptionEngine } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { ApiEnginePanel } from "./api-engine-panel";
import { LocalEnginePanel } from "./local-engine-panel";
import {
  adjacentTab,
  ENGINE_TABS,
  selectedEngine,
  tabOfEngine,
  type EngineTab,
} from "./transcription-engine-options";

const TAB_ICONS: Record<EngineTab, React.ElementType> = {
  local: HardDrive,
  api: Cloud,
};

interface TranscriptionEngineTabsProps {
  /** Content of the Local tab: the model search, filters and lists. */
  children: React.ReactNode;
}

export const TranscriptionEngineTabs: React.FC<
  TranscriptionEngineTabsProps
> = ({ children }) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const engine = selectedEngine(getSetting("transcription_engine"));
  const [tab, setTab] = useState<EngineTab>(() => tabOfEngine(engine));
  const baseId = useId();
  const isBusy = isUpdating("transcription_engine");

  const handleActivate = (next: TranscriptionEngine) => {
    if (next !== engine) updateSetting("transcription_engine", next);
  };

  // Arrow keys move to the adjacent tab (wrapping); Home/End reach the ends.
  const handleTabKey = (event: React.KeyboardEvent, current: EngineTab) => {
    const targets: Record<string, EngineTab> = {
      ArrowRight: adjacentTab(current, 1),
      ArrowLeft: adjacentTab(current, -1),
      Home: ENGINE_TABS[0],
      End: ENGINE_TABS[ENGINE_TABS.length - 1],
    };
    const next = targets[event.key];
    if (next === undefined) return;
    event.preventDefault();
    setTab(next);
    document.getElementById(`${baseId}-tab-${next}`)?.focus();
  };

  return (
    <div className="flex flex-col gap-4" data-testid="transcription-engine">
      <div className="flex flex-col gap-2">
        <div
          role="tablist"
          aria-label={t("settings.models.engine.title")}
          className="flex gap-2"
        >
          {ENGINE_TABS.map((id) => {
            const Icon = TAB_ICONS[id];
            const isSelected = tab === id;
            const holdsActiveEngine = tabOfEngine(engine) === id;
            return (
              <button
                key={id}
                id={`${baseId}-tab-${id}`}
                type="button"
                role="tab"
                aria-selected={isSelected}
                aria-controls={`${baseId}-panel`}
                tabIndex={isSelected ? 0 : -1}
                onClick={() => setTab(id)}
                onKeyDown={(event) => handleTabKey(event, id)}
                data-testid={`engine-tab-${id}`}
                className={`flex items-center gap-2 rounded-lg border px-4 py-2 text-sm font-medium transition-colors cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-logo-primary ${
                  isSelected
                    ? "border-logo-primary/50 bg-logo-primary/15 text-text"
                    : "border-mid-gray/20 bg-mid-gray/10 text-text/70 hover:bg-mid-gray/20"
                }`}
              >
                <Icon className="w-4 h-4" aria-hidden="true" />
                {t(`settings.models.engine.tabs.${id}`)}
                {holdsActiveEngine && (
                  <span
                    className="w-2 h-2 rounded-full bg-logo-primary"
                    title={t("settings.models.engine.active")}
                    data-testid={`engine-tab-${id}-active-dot`}
                  >
                    <span className="sr-only">
                      {t("settings.models.engine.active")}
                    </span>
                  </span>
                )}
              </button>
            );
          })}
        </div>
        <p className="text-xs text-text/60">
          {t("settings.models.engine.description")}
        </p>
      </div>
      <div
        id={`${baseId}-panel`}
        role="tabpanel"
        aria-labelledby={`${baseId}-tab-${tab}`}
      >
        {tab === "local" ? (
          <LocalEnginePanel
            isActive={engine === "local"}
            isBusy={isBusy}
            onActivate={() => handleActivate("local")}
          >
            {children}
          </LocalEnginePanel>
        ) : (
          <ApiEnginePanel
            engine={engine}
            isBusy={isBusy}
            onActivate={handleActivate}
          />
        )}
      </div>
    </div>
  );
};
