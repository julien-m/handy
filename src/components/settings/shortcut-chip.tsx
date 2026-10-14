// Key chip of a shortcut recorder in capture mode: shows a shortcut (or the
// keys being recorded) and starts recording when activated. A shortcut list
// can also start it (an "Add shortcut" draft, a "Change" action) and give it
// a validation state through `ShortcutChipContext`; the chip reports back
// each start request it handled, every recording change and a refused start.
import React, { createContext, useContext, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useOsType } from "../../hooks/useOsType";
import { formatKeyCombination } from "../../lib/utils/keyboard";
import { isNewRecordRequest } from "./record-request-model";
import type {
  RecorderStartFailure,
  RecorderValidity,
} from "./shortcut-draft-model";

/** What a shortcut list asks of the chips it renders. */
export interface ShortcutChipControl {
  /** Each new non-zero value asks the chip to focus and start recording. */
  recordRequest: number;
  /** Told once a request was handled (recording started, or not possible). */
  onRequestHandled?: (request: number) => void;
  /** Validation state of a draft recorder, shown by its colors. */
  validity?: RecorderValidity;
  /** Told whenever recording starts or stops. */
  onRecordingChange?: (isRecording: boolean) => void;
  /** Told when the system refused to start recording, with the reason. */
  onStartFailed?: (failure: RecorderStartFailure) => void;
}

export const ShortcutChipContext = createContext<ShortcutChipControl>({
  recordRequest: 0,
});

/** Starts recording; resolves with the reason when it could not start. */
export type StartRecording = () => Promise<RecorderStartFailure | null>;

/**
 * Text of a recorder chip: the keys being recorded ("Press keys…" before the
 * first one), else the shortcut it shows, else the add label.
 * @param isRecording Whether the chip records.
 * @param keys Keys recorded so far, in stored syntax ("" for none).
 * @param value Shortcut shown while idle, or null.
 */
export const useChipText = (
  isRecording: boolean,
  keys: string,
  value: string | null,
): string => {
  const { t } = useTranslation();
  const osType = useOsType();
  const shown = isRecording ? keys : value;
  if (shown) return formatKeyCombination(shown, osType);
  return t(
    isRecording
      ? "settings.general.shortcut.pressKeys"
      : "settings.general.shortcut.list.add",
  );
};

/**
 * Cancels a capture when the user clicks outside the chip.
 * @param isRecording Whether the chip records (listens only then).
 * @param chip The chip element.
 * @param cancel Stops the capture without saving.
 */
export const useCancelOnClickOutside = (
  isRecording: boolean,
  chip: React.RefObject<HTMLElement | null>,
  cancel: () => void,
): void => {
  useEffect(() => {
    if (!isRecording) return;
    const onClick = (event: MouseEvent) => {
      if (chip.current && !chip.current.contains(event.target as Node)) {
        cancel();
      }
    };
    window.addEventListener("click", onClick);
    return () => window.removeEventListener("click", onClick);
  }, [isRecording, chip, cancel]);
};

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
  onStart: StartRecording;
  onLeave: () => void;
  chipRef: (element: HTMLButtonElement | null) => void;
}

/**
 * Handles each new start request of the list once, then reports it handled
 * so the list forgets it, and reports recording changes. The latest
 * callbacks are read through a ref so a re-render never replays a request.
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
    if (!isNewRecordRequest(request, handledRequest.current)) return;
    handledRequest.current = request;
    latest.current.start();
    latest.current.control.onRequestHandled?.(request);
  }, [control.recordRequest]);
  useEffect(() => {
    // Report changes only: the first render is not a stop, and reporting it
    // would close a draft recorder before it starts recording.
    if (wasRecording.current === isRecording) return;
    wasRecording.current = isRecording;
    latest.current.control.onRecordingChange?.(isRecording);
  }, [isRecording]);
};

/** Starts recording from a click or a list request; reports a refused start. */
const useStart = (
  props: ShortcutChipProps,
  control: ShortcutChipControl,
  button: React.RefObject<HTMLButtonElement | null>,
): (() => void) => {
  const { disabled, isRecording, onStart } = props;
  return () => {
    if (disabled || isRecording) return;
    // Focus first: leaving the focused chip is what cancels a capture.
    button.current?.focus();
    void onStart().then((failure) => {
      if (failure) control.onStartFailed?.(failure);
    });
  };
};

/**
 * Focusable chip used by both keyboard implementations' recorders. A
 * disabled chip stays focusable (`aria-disabled`) so its tooltip can
 * explain why it is unavailable.
 */
export const ShortcutChip: React.FC<ShortcutChipProps> = (props) => {
  const { isRecording, disabled, label } = props;
  const control = useContext(ShortcutChipContext);
  const button = useRef<HTMLButtonElement | null>(null);
  const start = useStart(props, control, button);
  useListControl(control, isRecording, start);
  const { validity } = control;
  return (
    <button
      type="button"
      ref={(element) => {
        button.current = element;
        props.chipRef(element);
      }}
      data-testid={props.testId}
      data-state={isRecording ? "recording" : "idle"}
      data-validity={validity}
      aria-label={label}
      aria-disabled={disabled}
      aria-invalid={validity === "invalid" || undefined}
      title={label}
      className={shortcutChipClass({ isRecording, disabled, validity })}
      onClick={start}
      onBlur={() => isRecording && props.onLeave()}
    >
      {props.text}
    </button>
  );
};
