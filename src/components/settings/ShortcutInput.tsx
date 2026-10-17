import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { GlobalShortcutInput } from "./GlobalShortcutInput";
import { HandyKeysShortcutInput } from "./HandyKeysShortcutInput";

interface ShortcutInputProps {
  shortcutId: string;
  disabled?: boolean;
  /**
   * Receives the recorded combination; the caller saves it (through the
   * shortcut list's edit) and reports refusals.
   */
  onCapture: (shortcut: string) => Promise<void>;
  /** The shortcut shown on the chip; null shows the add label. */
  value?: string | null;
  /** Accessible name and tooltip of the chip. */
  label?: string;
  /** Stable test identifier of the chip. */
  testId?: string;
  /** Instance id of a repeated chip (`shortcut-{keys}` of the Penflow contract). */
  instanceId?: string;
}

/**
 * Key chip of a shortcut recorder, using the implementation selected by the
 * keyboard_implementation setting.
 *
 * - "tauri" (default): Uses GlobalShortcutInput with JS keyboard events
 * - "handy_keys": Uses HandyKeysShortcutInput with backend key events
 */
export const ShortcutInput: React.FC<ShortcutInputProps> = (props) => {
  const { getSetting } = useSettings();
  const keyboardImplementation = getSetting("keyboard_implementation");

  // Default to Tauri implementation if not set
  if (keyboardImplementation === "handy_keys") {
    return <HandyKeysShortcutInput {...props} />;
  }

  return <GlobalShortcutInput {...props} />;
};
