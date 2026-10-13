import React, { useState } from "react";
import { SettingContainer } from "./SettingContainer";
import { ResetButton } from "./ResetButton";

interface SliderProps {
  value: number;
  /** Called once per gesture, when the user releases the slider (or the key, or leaves it). */
  onChange: (value: number) => void;
  min: number;
  max: number;
  step?: number;
  disabled?: boolean;
  label: string;
  description: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  showValue?: boolean;
  formatValue?: (value: number) => string;
  onReset?: () => void;
  isResetting?: boolean;
}

export const Slider: React.FC<SliderProps> = ({
  value,
  onChange,
  min,
  max,
  step = 0.01,
  disabled = false,
  label,
  description,
  descriptionMode = "tooltip",
  grouped = false,
  showValue = true,
  formatValue = (v) => v.toFixed(2),
  onReset,
  isResetting = false,
}) => {
  // Dragging only moves a local draft; the value is committed once, on release. Each commit is a
  // durable settings write (and a UI event), so committing every input event would write dozens
  // of revisions per second.
  const [draft, setDraft] = useState<number | null>(null);
  const shown = draft ?? value;
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setDraft(parseFloat(e.target.value));
  };
  const commit = () => {
    if (draft !== null && draft !== value) onChange(draft);
    setDraft(null);
  };

  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      disabled={disabled}
    >
      <div className="w-full">
        <div className="flex items-center space-x-1 h-6">
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={shown}
            onChange={handleChange}
            onPointerUp={commit}
            onKeyUp={commit}
            onBlur={commit}
            disabled={disabled}
            className="flex-grow h-2 rounded-lg appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-logo-primary disabled:opacity-50 disabled:cursor-not-allowed"
            style={{
              background: `linear-gradient(to right, var(--color-background-ui) ${
                ((shown - min) / (max - min)) * 100
              }%, rgba(128, 128, 128, 0.2) ${
                ((shown - min) / (max - min)) * 100
              }%)`,
            }}
          />
          {showValue && (
            <span className="text-sm font-medium text-text/90 w-12 text-end">
              {formatValue(shown)}
            </span>
          )}
          {onReset && (
            <ResetButton onClick={onReset} disabled={disabled || isResetting} />
          )}
        </div>
      </div>
    </SettingContainer>
  );
};
