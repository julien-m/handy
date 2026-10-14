// "In use now" row (feature 001, AC-002): the microphone actually used, shown apart from the
// saved order — the current input and its reason, the last recording's input labelled as such,
// or "not used yet" after a launch.
import React from "react";
import { useTranslation } from "react-i18next";
import Badge from "../../ui/Badge";
import { SettingContainer } from "../../ui/SettingContainer";
import {
  describeEffective,
  type EffectiveView,
  type StatusView,
} from "./availability";
import { PRIORITY_KEY_ROOT } from "./draft-state";

/** Props of the In use now row. */
export interface EffectiveInputProps {
  /** Last status view (the last successful read and whether the latest failed). */
  view: StatusView;
}

/** Reason pill: in use now, or the last recording's reason labelled as such. */
const ReasonPill: React.FC<{ view: EffectiveView }> = ({ view }) => {
  const { t } = useTranslation();
  if (view.reason === null) return null;
  const reason = t(view.reason.key, view.reason.params);
  const text = view.current
    ? reason
    : t(`${PRIORITY_KEY_ROOT}.inUse.lastRecording`, { reason });
  return (
    <span data-testid="microphone-settings-in-use-reason">
      <Badge variant={view.current ? "info" : "secondary"}>{text}</Badge>
    </span>
  );
};

/** The microphone in use, apart from the saved order. */
export const EffectiveInputRow: React.FC<EffectiveInputProps> = (props) => {
  const { t } = useTranslation();
  const view = describeEffective(props.view);
  const muted = view.name === null && view.reason === null;
  return (
    <SettingContainer
      title={t(`${PRIORITY_KEY_ROOT}.inUse.title`)}
      description={t(`${PRIORITY_KEY_ROOT}.inUse.description`)}
      descriptionMode="tooltip"
      grouped={true}
      testId="microphone-settings-in-use"
      titleTestId="microphone-settings-in-use-title"
    >
      <div className="flex flex-col items-end gap-1 text-end">
        <span
          className={`text-sm ${muted ? "text-mid-gray" : ""}`}
          data-testid="microphone-settings-in-use-device"
        >
          {view.name ?? (view.label ? t(view.label.key) : "")}
        </span>
        <ReasonPill view={view} />
      </div>
    </SettingContainer>
  );
};
