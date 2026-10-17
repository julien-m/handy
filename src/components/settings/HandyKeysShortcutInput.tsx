// Shortcut recorder of the HandyKeys keyboard implementation: the backend
// records the keys (`start_handy_keys_recording`) and streams them as
// `handy-keys-event`; the combination goes to `onCapture`, which saves it and
// reports refusals. Leaving the chip (blur, or a click elsewhere) cancels.
// The backend refuses to start while macOS Secure Input is active; the chip
// then reports the reason to its list (the draft shows it, with Discard).
import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";
import { toast } from "sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SECURE_INPUT_HELP_URL } from "../SecureInputWarning";
import {
  ShortcutChip,
  useCancelOnClickOutside,
  useChipText,
} from "./shortcut-chip";
import type { RecorderStartFailure } from "./shortcut-draft-model";
import { startHandyKeys, useMounted } from "./recorder-lifecycle";

interface HandyKeysShortcutInputProps {
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

interface HandyKeysEvent {
  modifiers: string[];
  key: string | null;
  is_key_down: boolean;
  hotkey_string: string;
}

/**
 * Keys of one capture. Keyed and modifier-only captures are tracked apart so
 * a combo commits only on its key's release and a modifier-only shortcut
 * only once every modifier is released. Committing on the *first* release
 * (the old behavior) silently saved just the modifier whenever the key event
 * never arrived — e.g. while macOS Secure Input is active (issue #1578).
 */
interface CaptureRefs {
  keyed: string;
  modifierOnly: string;
}

/** Keys captured after an event, and the combination to commit, if any. */
interface CaptureStep {
  refs: CaptureRefs;
  commit: string | null;
}

/**
 * Next capture step after `event`: a press updates the captured keys, a
 * release may commit.
 * @param event Key event from the backend.
 * @param refs Keys captured so far.
 */
const captureStep = (event: HandyKeysEvent, refs: CaptureRefs): CaptureStep => {
  const { hotkey_string, is_key_down, key, modifiers } = event;
  if (is_key_down && hotkey_string) {
    const pressed = key
      ? { ...refs, keyed: hotkey_string }
      : { ...refs, modifierOnly: hotkey_string };
    return { refs: pressed, commit: null };
  }
  // The main key was released: commit the keyed combo. The release event's
  // hotkey_string still contains the key, so it works even if the key-down
  // was missed. Never fall back to a modifier-only capture here: that's how
  // bindings got silently overwritten with just the modifier (issue #1578).
  if (!is_key_down && key) {
    return { refs, commit: refs.keyed || hotkey_string || null };
  }
  // Every modifier released without a main key ever going down: commit as a
  // modifier-only shortcut.
  const isModifierOnly = !key && modifiers.length === 0 && !refs.keyed;
  const commit = !is_key_down && isModifierOnly ? refs.modifierOnly : "";
  return { refs, commit: commit || null };
};

/** Toast of a refused start (the inline reason is shown by the list). */
const notifyStartFailure = (
  failure: RecorderStartFailure,
  t: TFunction,
): void => {
  if (failure.kind === "secure_input") {
    toast.error(t("secureInput.recorderBlocked"), {
      action: {
        label: t("secureInput.learnMore"),
        onClick: () => openUrl(SECURE_INPUT_HELP_URL),
      },
    });
    return;
  }
  toast.error(
    t("settings.general.shortcut.errors.set", { error: failure.message }),
  );
};

/**
 * Listens to the backend's key events while `isRecording`: shows the keys
 * held and calls `finish` with the combination to commit. Unmounting stops
 * the backend recording so no recording loop is orphaned. `finish` is read
 * through a ref: a re-render of the list (a new inline `onCapture`) never
 * restarts the listener, which would stop the backend mid-recording.
 */
const useBackendKeys = (
  isRecording: boolean,
  finish: (shortcut: string) => Promise<void>,
): string => {
  const [currentKeys, setCurrentKeys] = useState("");
  const latestFinish = useRef(finish);
  latestFinish.current = finish;
  useEffect(() => {
    if (!isRecording) return setCurrentKeys("");
    let refs: CaptureRefs = { keyed: "", modifierOnly: "" };
    let isDone = false;
    const unlisten = listen<HandyKeysEvent>("handy-keys-event", (event) => {
      if (isDone) return;
      const { hotkey_string, is_key_down } = event.payload;
      if (is_key_down && hotkey_string) setCurrentKeys(hotkey_string);
      const step = captureStep(event.payload, refs);
      refs = step.refs;
      if (step.commit === null) return;
      isDone = true;
      void latestFinish.current(step.commit);
    });
    return () => {
      isDone = true;
      void unlisten.then((stop) => stop());
      commands.stopHandyKeysRecording().catch(console.error);
    };
  }, [isRecording]);
  return currentKeys;
};

/**
 * Hands the combination to `onCapture` (which saves it and reports
 * refusals), then stops recording even if it failed.
 */
const captureThenStop = async (
  onCapture: (shortcut: string) => Promise<void>,
  shortcut: string,
  stop: () => Promise<void>,
  t: TFunction,
): Promise<void> => {
  try {
    await onCapture(shortcut);
  } catch (error) {
    console.error("Failed to change binding:", error);
    toast.error(
      t("settings.general.shortcut.errors.set", { error: String(error) }),
    );
  } finally {
    await stop();
  }
};

/**
 * Recording state of the chip, from the backend's start answer to the stop.
 * A start that succeeds after the chip unmounted is stopped at once.
 */
const useRecording = (
  shortcutId: string,
  onCapture: (shortcut: string) => Promise<void>,
) => {
  const { t } = useTranslation();
  const mounted = useMounted();
  const [isRecording, setIsRecording] = useState(false);
  const stop = useCallback(async () => {
    await commands.stopHandyKeysRecording().catch(console.error);
    setIsRecording(false);
  }, []);
  const start = useCallback(async () => {
    const answer = await startHandyKeys(shortcutId, () => mounted.current);
    if (answer.status === "recording") setIsRecording(true);
    if (answer.status !== "refused") return null;
    notifyStartFailure(answer.failure, t);
    return answer.failure;
  }, [shortcutId, t, mounted]);
  const finish = useCallback(
    (shortcut: string) => captureThenStop(onCapture, shortcut, stop, t),
    [onCapture, stop, t],
  );
  return { isRecording, start, stop, finish };
};

export const HandyKeysShortcutInput: React.FC<HandyKeysShortcutInputProps> = (
  props,
) => {
  const chip = useRef<HTMLElement | null>(null);
  const recording = useRecording(props.shortcutId, props.onCapture);
  const { isRecording, stop } = recording;
  const keys = useBackendKeys(isRecording, recording.finish);
  useCancelOnClickOutside(isRecording, chip, stop);
  const text = useChipText(isRecording, keys, props.value ?? null);
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
