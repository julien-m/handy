import React from "react";
import { SettingContainer } from "./SettingContainer";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  isUpdating?: boolean;
  label: string;
  description: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  tooltipPosition?: "top" | "bottom";
  /** Test identifier of the row; the switch gets `<testId>-toggle`. */
  testId?: string;
}

interface SwitchControlProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  /** Accessible name, needed when no visible label wraps the switch. */
  ariaLabel?: string;
  /** Points to the visible text that explains the switch. */
  ariaDescribedBy?: string;
  testId?: string;
}

/** The switch itself, without its settings row: a checkbox styled as a toggle. */
export const SwitchControl: React.FC<SwitchControlProps> = ({
  checked,
  onChange,
  disabled = false,
  ariaLabel,
  ariaDescribedBy,
  testId,
}) => (
  <label
    className={`flex items-center ${disabled ? "cursor-not-allowed" : "cursor-pointer"}`}
  >
    <input
      type="checkbox"
      value=""
      className="sr-only peer"
      checked={checked}
      disabled={disabled}
      aria-label={ariaLabel}
      aria-describedby={ariaDescribedBy}
      data-testid={testId}
      onChange={(e) => onChange(e.target.checked)}
    />
    <div className="relative w-11 h-6 bg-mid-gray/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-logo-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-background-ui peer-disabled:opacity-50"></div>
  </label>
);

export const ToggleSwitch: React.FC<ToggleSwitchProps> = ({
  checked,
  onChange,
  disabled = false,
  isUpdating = false,
  label,
  description,
  descriptionMode = "tooltip",
  grouped = false,
  tooltipPosition = "top",
  testId,
}) => {
  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      disabled={disabled}
      tooltipPosition={tooltipPosition}
      testId={testId}
    >
      <SwitchControl
        checked={checked}
        onChange={onChange}
        disabled={disabled || isUpdating}
        ariaLabel={label}
        testId={testId ? `${testId}-toggle` : undefined}
      />
      {isUpdating && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-4 h-4 border-2 border-logo-primary border-t-transparent rounded-full animate-spin"></div>
        </div>
      )}
    </SettingContainer>
  );
};
