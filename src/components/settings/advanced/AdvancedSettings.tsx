import React from "react";
import { useTranslation } from "react-i18next";
import { ShowOverlay } from "../ShowOverlay";
import { ModelUnloadTimeoutSetting } from "../ModelUnloadTimeout";
import { CustomWords } from "../CustomWords";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { StartHidden } from "../StartHidden";
import { AutostartToggle } from "../AutostartToggle";
import { ShowTrayIcon } from "../ShowTrayIcon";
import { PasteMethodSetting } from "../PasteMethod";
import { TypingToolSetting } from "../TypingTool";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { ReturnToAppSetting } from "../return-to-app-setting";
import { DefaultInputsSetting } from "../default-inputs-setting";
import { NoFieldBubbleToggle } from "../recovery-bubble-toggle";
import { AutoSubmit } from "../AutoSubmit";
import { AutomaticEnter } from "../automatic-enter";
import { enterControl } from "../automatic-enter-state";
import { PostProcessingToggle } from "../PostProcessingToggle";
import { AppendTrailingSpace } from "../AppendTrailingSpace";
import { HistoryLimit } from "../HistoryLimit";
import { RecordingRetentionPeriodSelector } from "../RecordingRetentionPeriod";
import { ExperimentalToggle } from "../ExperimentalToggle";
import { useSettings } from "../../../hooks/useSettings";
import { KeyboardImplementationSelector } from "../debug/KeyboardImplementationSelector";
import { VoiceActivityDetection } from "../VoiceActivityDetection";
import { AccelerationSelector } from "../AccelerationSelector";
import { LazyStreamClose } from "../LazyStreamClose";
import { PasteVerificationToggle } from "../paste-verification-toggle";
import { FillerWordRemoval } from "../FillerWordRemoval";
import { ChineseScriptSetting } from "../ChineseScript";
import { VadBackendSelector } from "../VadBackendSelector";
import { WordReplacements } from "../word-replacements/word-replacements";
import { TextFormattingSection } from "../text-formatting/text-formatting-section";
import { useScreenScope } from "../page-screen";
import {
  OUTPUT_SCREENS,
  OutputScreenContext,
  useReportedOutputScreen,
} from "../output-screen";
import { useSettingsStore } from "../../../stores/settingsStore";

/** The revision conflict alert (008, above every page) shows the page's refused-write screen. */
const ConflictScreenReport: React.FC = () => {
  const hasConflict = useSettingsStore((state) => state.revisionConflict);
  useReportedOutputScreen(
    "conflict",
    hasConflict ? "settings-write-errors" : null,
  );
  return null;
};

export const AdvancedSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const experimentalEnabled = getSetting("experimental_enabled") || false;
  // 009: exactly one Enter control per build (Automatic Enter when the gate is open).
  const automaticEnter =
    enterControl({ automatic_enter: getSetting("automatic_enter") }) ===
    "automatic_enter";

  // 006, 007, 008, 009: the root, Output and History test ids follow the screen shown.
  const scope = useScreenScope(OUTPUT_SCREENS);
  const { screen } = scope;

  return (
    <OutputScreenContext.Provider value={scope}>
      <ConflictScreenReport />
      <div
        className="max-w-3xl w-full mx-auto space-y-6"
        data-testid={`${screen}-screen`}
      >
        <SettingsGroup title={t("settings.advanced.groups.app")}>
          <StartHidden descriptionMode="tooltip" grouped={true} />
          <AutostartToggle descriptionMode="tooltip" grouped={true} />
          <ShowTrayIcon descriptionMode="tooltip" grouped={true} />
          <ShowOverlay descriptionMode="tooltip" grouped={true} />
          <ModelUnloadTimeoutSetting descriptionMode="tooltip" grouped={true} />
          <ExperimentalToggle descriptionMode="tooltip" grouped={true} />
        </SettingsGroup>

        <div data-testid={`${screen}-output`}>
          <SettingsGroup title={t("settings.advanced.groups.output")}>
            <PasteMethodSetting descriptionMode="inline" grouped={true} />
            <TypingToolSetting descriptionMode="tooltip" grouped={true} />
            <ClipboardHandlingSetting
              descriptionMode="tooltip"
              grouped={true}
            />
            <ReturnToAppSetting grouped={true} />
            <DefaultInputsSetting grouped={true} />
            {automaticEnter ? (
              <AutomaticEnter grouped={true} />
            ) : (
              <AutoSubmit descriptionMode="tooltip" grouped={true} />
            )}
            <NoFieldBubbleToggle grouped={true} />
          </SettingsGroup>
        </div>

        <TextFormattingSection />

        <SettingsGroup title={t("settings.advanced.groups.transcription")}>
          <VoiceActivityDetection descriptionMode="tooltip" grouped={true} />
          <FillerWordRemoval descriptionMode="tooltip" grouped={true} />
          <ChineseScriptSetting descriptionMode="tooltip" grouped={true} />
          <AppendTrailingSpace descriptionMode="tooltip" grouped={true} />
        </SettingsGroup>

        <CustomWords />

        <WordReplacements />

        <div data-testid={`${screen}-history`}>
          <SettingsGroup title={t("settings.advanced.groups.history")}>
            <HistoryLimit descriptionMode="tooltip" grouped={true} />
            <RecordingRetentionPeriodSelector
              descriptionMode="tooltip"
              grouped={true}
            />
          </SettingsGroup>
        </div>

        {experimentalEnabled && (
          <SettingsGroup title={t("settings.advanced.groups.experimental")}>
            <PostProcessingToggle descriptionMode="tooltip" grouped={true} />
            <KeyboardImplementationSelector
              descriptionMode="tooltip"
              grouped={true}
            />
            <AccelerationSelector descriptionMode="tooltip" grouped={true} />
            <LazyStreamClose descriptionMode="tooltip" grouped={true} />
            <PasteVerificationToggle descriptionMode="tooltip" grouped={true} />
            <VadBackendSelector descriptionMode="tooltip" grouped={true} />
          </SettingsGroup>
        )}
      </div>
    </OutputScreenContext.Provider>
  );
};
