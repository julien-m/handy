// "Return to the dictation's application" (006): a toggle with its inline
// description, an info button with the details, and, where destination
// return is unavailable, the reason in place of the description. A build
// that does not offer destination return shows no row at all.
import React, { useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { useDestinationSupport } from "../../hooks/use-destination-support";
import { SettingContainer } from "../ui/SettingContainer";
import { SwitchControl } from "../ui/ToggleSwitch";
import { Tooltip } from "../ui/Tooltip";
import { type ReturnToAppView, returnToAppView } from "./return-to-app-model";
import { useReportedOutputScreen } from "./output-screen";

const KEY = "settings.advanced.returnToApp";

interface InfoButtonProps {
  label: string;
  details: string;
  testId: string;
}

/** Info icon next to the toggle; hover, focus or click shows the details. */
const InfoButton: React.FC<InfoButtonProps> = ({ label, details, testId }) => {
  const [isOpen, setIsOpen] = useState(false);
  const anchor = useRef<HTMLButtonElement>(null);
  return (
    <button
      type="button"
      ref={anchor}
      className="relative text-mid-gray hover:text-logo-primary cursor-help focus-visible:outline focus-visible:outline-2 focus-visible:outline-logo-primary rounded-full"
      aria-label={label}
      aria-expanded={isOpen}
      data-testid={testId}
      data-state={isOpen ? "details-shown" : "closed"}
      onClick={() => setIsOpen((open) => !open)}
      onMouseEnter={() => setIsOpen(true)}
      onMouseLeave={() => setIsOpen(false)}
      onBlur={() => setIsOpen(false)}
    >
      <svg
        className="w-4 h-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
      {isOpen && (
        <Tooltip targetRef={anchor} position="top">
          <span className="block text-sm text-center leading-relaxed">
            {details}
          </span>
        </Tooltip>
      )}
    </button>
  );
};

interface ReturnToAppSettingProps {
  grouped?: boolean;
}

interface ReturnToggleProps {
  view: ReturnToAppView;
  description: string;
  /** Test id of the row, under the output screen shown. */
  testId: string;
}

/**
 * The toggle, off and disabled with its reason where unsupported; screen
 * readers get the description (or the reason) through a hidden copy.
 */
const ReturnToggle: React.FC<ReturnToggleProps> = ({
  view,
  description,
  testId,
}) => {
  const { t } = useTranslation();
  const { updateSetting, isUpdating } = useSettings();
  const descriptionId = useId();
  const isBusy = isUpdating("return_to_dictation_app");
  return (
    <>
      <span id={descriptionId} className="sr-only">
        {description}
      </span>
      <SwitchControl
        checked={view.checked}
        disabled={view.state !== "available" || isBusy}
        ariaLabel={t(`${KEY}.title`)}
        ariaDescribedBy={descriptionId}
        testId={`${testId}-toggle`}
        onChange={(enabled) =>
          void updateSetting("return_to_dictation_app", enabled)
        }
      />
    </>
  );
};

// @spec AC-007: Unavailable on this system with the reason — .specs/features/006-destination-application/spec.md#ac-007
/**
 * The setting row; the toggle is disabled with its reason where unsupported.
 * Nothing is shown while support is unknown or when this build does not offer
 * destination return (rollout gate closed: the settings stay as on `dev`).
 */
export const ReturnToAppSetting: React.FC<ReturnToAppSettingProps> = (
  props,
) => {
  const { t } = useTranslation();
  const stored = useSettings().getSetting("return_to_dictation_app");
  const view = returnToAppView(useDestinationSupport(), stored);
  // Unavailable destination return is the output-settings-unavailable screen (006).
  const screen = useReportedOutputScreen(
    "return-app",
    view.state === "unavailable" ? "output-settings-unavailable" : null,
  );
  const testId = `${screen}-return-app`;
  if (view.state === "hidden" || view.state === "loading") return null;
  const description = t(view.descriptionKey);
  return (
    <SettingContainer
      title={t(`${KEY}.title`)}
      description={description}
      descriptionMode="inline"
      grouped={props.grouped ?? false}
      testId={testId}
    >
      <div className="flex items-center gap-3" data-state={view.state}>
        <InfoButton
          label={t(`${KEY}.detailsLabel`)}
          details={t(`${KEY}.details`)}
          testId={`${testId}-info`}
        />
        <ReturnToggle view={view} description={description} testId={testId} />
      </div>
    </SettingContainer>
  );
};
