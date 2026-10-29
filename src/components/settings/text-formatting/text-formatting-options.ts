// Options of the "Text formatting" section of the Advanced settings page (015 smart
// capitalization; smart paragraphs and smart spacing come in their own features). Each option
// is one boolean setting with its own independent switch row: adding an option is one entry
// here (and its setting key in the union), nothing else in the section changes.
import { CaseSensitive, type LucideIcon } from "lucide-react";

/** Boolean settings shown in the Text formatting section. */
export type TextFormattingKey = "smart_capitalization";

/** One switch row of the section. */
export interface TextFormattingOption {
  /** The boolean setting the switch writes through the settings service. */
  key: TextFormattingKey;
  /** Small decorative icon before the label. */
  icon: LucideIcon;
  /** i18n key of the label. */
  labelKey: string;
  /** i18n key of the description shown by the info button. */
  descriptionKey: string;
  /** Test-id suffix of the row, after the screen prefix. */
  testId: string;
}

// @spec AC-008: Text formatting switch row — .specs/features/015-smart-capitalization/spec.md#ac-008
/** The section's rows, in display order. */
export const TEXT_FORMATTING_OPTIONS: readonly TextFormattingOption[] = [
  {
    key: "smart_capitalization",
    icon: CaseSensitive,
    labelKey: "settings.advanced.textFormatting.smartCapitalization.label",
    descriptionKey:
      "settings.advanced.textFormatting.smartCapitalization.description",
    testId: "smart-capitalization",
  },
];
