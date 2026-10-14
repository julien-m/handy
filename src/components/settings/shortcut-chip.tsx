// Key chip of a shortcut recorder in capture mode: shows a shortcut (or the
// keys being recorded) and starts recording when activated.
import React from "react";

interface ShortcutChipProps {
  text: string;
  isRecording: boolean;
  disabled: boolean;
  label?: string;
  testId?: string;
  onStart: () => void;
  onLeave: () => void;
  chipRef: (element: HTMLButtonElement | null) => void;
}

/**
 * Focusable chip used by both keyboard implementations' recorders. A
 * disabled chip stays focusable (`aria-disabled`) so its tooltip can
 * explain why it is unavailable.
 */
export const ShortcutChip: React.FC<ShortcutChipProps> = ({
  text,
  isRecording,
  disabled,
  label,
  testId,
  onStart,
  onLeave,
  chipRef,
}) => {
  const stateClass = isRecording
    ? "border-logo-primary bg-logo-primary/30"
    : "bg-mid-gray/10 border-mid-gray/80";
  const interactiveClass = disabled
    ? "opacity-50 cursor-not-allowed"
    : "hover:bg-logo-primary/10 hover:border-logo-primary cursor-pointer";
  return (
    <button
      type="button"
      ref={chipRef}
      data-testid={testId}
      data-state={isRecording ? "recording" : "idle"}
      aria-label={label}
      aria-disabled={disabled}
      title={label}
      className={`px-2 py-1 text-sm font-semibold border rounded-md focus-visible:outline focus-visible:outline-2 focus-visible:outline-logo-primary ${stateClass} ${interactiveClass}`}
      onClick={() => {
        if (!disabled && !isRecording) onStart();
      }}
      onBlur={() => {
        if (isRecording) onLeave();
      }}
    >
      {text}
    </button>
  );
};
