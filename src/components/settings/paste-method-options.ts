/**
 * Paste methods offered by the settings selector for one platform, and whether the macOS beta
 * reliable paste switch is still shown. Pure: the selector maps translation keys to labels.
 * Smart paste (feature 007) is offered only when the backend reports it available (macOS, once
 * dictation delivers through it); it then replaces the macOS beta reliable paste.
 */
import type { PasteMethod } from "@/bindings";
import type { OSType } from "@/lib/utils/keyboard";

/** One selector entry: a stored value, its translation key and whether it can be chosen. */
export interface PasteMethodChoice {
  value: PasteMethod;
  labelKey: string;
  disabled: boolean;
}

const OPTION_KEYS = "settings.advanced.pasteMethod.options";

const choice = (
  value: PasteMethod,
  label: string,
  disabled = false,
): PasteMethodChoice => ({
  value,
  labelKey: `${OPTION_KEYS}.${label}`,
  disabled,
});

/**
 * Smart paste first on macOS once available. A saved value that cannot be chosen here stays
 * visible but disabled, so the selector always shows the stored setting.
 */
const smartPasteChoices = (
  osType: OSType,
  selected: PasteMethod | null,
  smartPasteAvailable: boolean,
): PasteMethodChoice[] => {
  const offered = osType === "macos" && smartPasteAvailable;
  if (!offered && selected !== "smart_paste") return [];
  return [choice("smart_paste", "smartPaste", !offered)];
};

/**
 * Selector entries for `osType`.
 * @param osType platform of the settings window
 * @param selected stored paste method, or null while the settings are loading
 * @param smartPasteAvailable backend answer of `is_smart_paste_available`
 * @returns entries in display order
 */
export const pasteMethodChoices = (
  osType: OSType,
  selected: PasteMethod | null,
  smartPasteAvailable: boolean,
): PasteMethodChoice[] => {
  const isMac = osType === "macos";
  const choices = smartPasteChoices(osType, selected, smartPasteAvailable);
  choices.push(choice("ctrl_v", "clipboard"));
  // Direct input is not offered on macOS; an existing selection stays visible, disabled.
  if (!isMac || selected === "direct") {
    choices.push(choice("direct", "direct", isMac));
  }
  choices.push(choice("none", "none"));
  if (osType === "windows" || osType === "linux") {
    choices.push(choice("ctrl_shift_v", "clipboardCtrlShiftV"));
    choices.push(choice("shift_insert", "clipboardShiftInsert"));
  }
  if (osType === "linux") {
    choices.push(choice("external_script", "externalScript"));
  }
  return choices;
};

/**
 * Whether the beta reliable paste switch is shown: on macOS and Windows, except on macOS once
 * smart paste replaces it (Windows keeps it).
 * @param osType platform of the settings window
 * @param smartPasteAvailable backend answer of `is_smart_paste_available`
 */
export const showsReliablePasteToggle = (
  osType: OSType,
  smartPasteAvailable: boolean,
): boolean =>
  osType === "windows" || (osType === "macos" && !smartPasteAvailable);
