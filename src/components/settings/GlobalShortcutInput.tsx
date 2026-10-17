// Shortcut recorder of the Tauri keyboard implementation: while its chip
// records, every global shortcut is suspended and the keys pressed in the
// settings window are collected; once all are released the combination is
// handed to `onCapture`, which saves it and reports refusals. Leaving the
// chip (blur, or a click elsewhere) cancels without saving.
import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  type OSType,
  getKeyName,
  normalizeKey,
} from "../../lib/utils/keyboard";
import { useOsType } from "../../hooks/useOsType";
import { commands } from "@/bindings";
import {
  resumeOnUnmount,
  suspendForCapture,
  useMounted,
} from "./recorder-lifecycle";
import {
  ShortcutChip,
  useCancelOnClickOutside,
  useChipText,
} from "./shortcut-chip";

interface GlobalShortcutInputProps {
  shortcutId: string;
  disabled?: boolean;
  /** Receives the recorded combination; saving it is the caller's job. */
  onCapture: (shortcut: string) => Promise<void>;
  /** The shortcut shown on the chip; null shows the add label. */
  value?: string | null;
  label?: string;
  testId?: string;
  /** Instance id of a repeated chip (`shortcut-{keys}` of the Penflow contract). */
  instanceId?: string;
}

/** Keys held down and every key recorded during one capture. */
interface CaptureKeys {
  pressed: string[];
  recorded: string[];
}

const NO_KEYS: CaptureKeys = { pressed: [], recorded: [] };

const MODIFIERS = [
  "ctrl",
  "control",
  "shift",
  "alt",
  "option",
  "meta",
  "command",
  "cmd",
  "super",
  "win",
  "windows",
];

/** Stored syntax of the recorded keys: modifiers first, then the main key. */
const combinationOf = (recorded: ReadonlyArray<string>): string => {
  const isModifier = (key: string) => MODIFIERS.includes(key.toLowerCase());
  return [...recorded]
    .sort((a, b) => Number(isModifier(b)) - Number(isModifier(a)))
    .join("+");
};

const keyOf = (event: KeyboardEvent, osType: OSType): string =>
  normalizeKey(getKeyName(event, osType));

const keysAfterDown = (keys: CaptureKeys, key: string): CaptureKeys =>
  keys.pressed.includes(key)
    ? keys
    : {
        pressed: [...keys.pressed, key],
        recorded: keys.recorded.includes(key)
          ? keys.recorded
          : [...keys.recorded, key],
      };

/** Window key listeners of one capture step (they see the current keys). */
const captureListeners = (
  keys: CaptureKeys,
  osType: OSType,
  setKeys: React.Dispatch<React.SetStateAction<CaptureKeys>>,
  finish: (shortcut: string) => void,
) => ({
  keydown: (event: KeyboardEvent) => {
    if (event.repeat) return; // ignore auto-repeat
    event.preventDefault();
    setKeys((current) => keysAfterDown(current, keyOf(event, osType)));
  },
  keyup: (event: KeyboardEvent) => {
    event.preventDefault();
    const key = keyOf(event, osType);
    const pressed = keys.pressed.filter((held) => held !== key);
    setKeys((current) => ({ ...current, pressed }));
    // Every key released: the recorded combination is complete.
    if (pressed.length === 0 && keys.recorded.length > 0) {
      finish(combinationOf(keys.recorded));
    }
  },
});

/**
 * Collects keys while `isRecording`; once every key is released, `finish`
 * receives the combination. Window listeners live as long as the capture.
 */
const useKeyCapture = (
  isRecording: boolean,
  finish: (shortcut: string) => void,
): CaptureKeys => {
  const osType = useOsType();
  const [keys, setKeys] = useState<CaptureKeys>(NO_KEYS);
  useEffect(() => {
    if (!isRecording) return setKeys(NO_KEYS);
    const { keydown, keyup } = captureListeners(keys, osType, setKeys, finish);
    window.addEventListener("keydown", keydown);
    window.addEventListener("keyup", keyup);
    return () => {
      window.removeEventListener("keydown", keydown);
      window.removeEventListener("keyup", keyup);
    };
  }, [isRecording, keys, osType, finish]);
  return keys;
};

/**
 * Recording state of the chip: start suspends every global shortcut so none
 * fires (or swallows the keys) while recording; stop resumes them, and so
 * does unmounting while recording or while a start is answered.
 */
const useRecording = (onCapture: (shortcut: string) => Promise<void>) => {
  const mounted = useMounted();
  const isSuspending = useRef(false);
  const [isRecording, setIsRecording] = useState(false);
  useEffect(() => resumeOnUnmount(isSuspending), []);
  const stop = useCallback(async () => {
    isSuspending.current = false;
    await commands.resumeAllBindings().catch(console.error);
    setIsRecording(false);
  }, []);
  const start = useCallback(async () => {
    const answer = await suspendForCapture(() => mounted.current);
    isSuspending.current = answer.status === "recording";
    setIsRecording(isSuspending.current);
    return null;
  }, [mounted]);
  const finish = useCallback(
    (shortcut: string) => {
      // Saving is the caller's job (it reports refusals); the shortcuts
      // resume once it answered, even if it failed.
      void onCapture(shortcut).catch(console.error).finally(stop);
    },
    [onCapture, stop],
  );
  return { isRecording, start, stop, finish };
};

export const GlobalShortcutInput: React.FC<GlobalShortcutInputProps> = (
  props,
) => {
  const chip = useRef<HTMLElement | null>(null);
  const recording = useRecording(props.onCapture);
  const { isRecording, stop } = recording;
  const keys = useKeyCapture(isRecording, recording.finish);
  useCancelOnClickOutside(isRecording, chip, stop);
  const keysText = keys.recorded.join("+");
  const text = useChipText(isRecording, keysText, props.value ?? null);
  return (
    <ShortcutChip
      text={text}
      isRecording={isRecording}
      disabled={props.disabled ?? false}
      label={props.label}
      testId={props.testId}
      instanceId={props.instanceId}
      onStart={recording.start}
      onLeave={stop}
      chipRef={(element) => {
        chip.current = element;
      }}
    />
  );
};
