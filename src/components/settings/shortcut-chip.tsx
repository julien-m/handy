// Key chip of a shortcut recorder in capture mode: shows a shortcut (or the
// keys being recorded) and starts recording when activated. A shortcut list
// can also start it (an "Add shortcut" draft, a "Change" action) and give it
// a validation state through `ShortcutChipContext`.
import React, { createContext, useContext, useEffect, useRef } from "react";
import type { RecorderValidity } from "./shortcut-draft-model";

/** What a shortcut list asks of the chips it renders. */
export interface ShortcutChipControl {
  /** Each new non-zero value asks the chip to focus and start recording. */
  recordRequest: number;
  /** Validation state of a draft recorder, shown by its colors. */
  validity?: RecorderValidity;
  /** Told whenever recording starts or stops. */
  onRecordingChange?: (isRecording: boolean) => void;
}

export const ShortcutChipContext = createContext<ShortcutChipControl>({
  recordRequest: 0,
});

interface ChipLook {
  isRecording: boolean;
  disabled: boolean;
  validity?: RecorderValidity;
}

/**
 * Classes of a shortcut chip, shared with the list's "Add shortcut" control
 * so both keep Handy's chip look.
 * @param look Recording, disabled and validation state.
 */
export const shortcutChipClass = ({
  isRecording,
  disabled,
  validity,
}: ChipLook): string => {
  const stateClass = isRecording
    ? "border-logo-primary bg-logo-primary/30"
    : validity === "invalid"
      ? "border-red-500 bg-red-500/20"
      : "bg-mid-gray/10 border-mid-gray/80";
  const interactiveClass = disabled
    ? "opacity-50 cursor-not-allowed"
    : "hover:bg-logo-primary/10 hover:border-logo-primary cursor-pointer";
  return `px-2 py-1 text-sm font-semibold border rounded-md focus-visible:outline focus-visible:outline-2 focus-visible:outline-logo-primary ${stateClass} ${interactiveClass}`;
};

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
 * Starts recording whenever the list sends a new request, and reports
 * recording changes. The latest callbacks are read through a ref so a
 * re-render never replays a request.
 */
const useListControl = (
  control: ShortcutChipControl,
  isRecording: boolean,
  start: () => void,
): void => {
  const latest = useRef({ control, start });
  latest.current = { control, start };
  const wasRecording = useRef(isRecording);
  const handledRequest = useRef(0);
  useEffect(() => {
    // Each request starts recording once, even when React runs the effect
    // twice (StrictMode): a second start would ask the backend twice.
    const request = control.recordRequest;
    if (request === 0 || request === handledRequest.current) return;
    handledRequest.current = request;
    latest.current.start();
  }, [control.recordRequest]);
  useEffect(() => {
    // Report changes only: the first render is not a stop, and reporting it
    // would close a draft recorder before it starts recording.
    if (wasRecording.current === isRecording) return;
    wasRecording.current = isRecording;
    latest.current.control.onRecordingChange?.(isRecording);
  }, [isRecording]);
};

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
  const control = useContext(ShortcutChipContext);
  const button = useRef<HTMLButtonElement | null>(null);
  useListControl(control, isRecording, () => {
    if (disabled || isRecording) return;
    // Focus first: leaving the focused chip is what cancels a capture.
    button.current?.focus();
    onStart();
  });
  return (
    <button
      type="button"
      ref={(element) => {
        button.current = element;
        chipRef(element);
      }}
      data-testid={testId}
      data-state={isRecording ? "recording" : "idle"}
      data-validity={control.validity}
      aria-label={label}
      aria-disabled={disabled}
      aria-invalid={control.validity === "invalid" || undefined}
      title={label}
      className={shortcutChipClass({
        isRecording,
        disabled,
        validity: control.validity,
      })}
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
