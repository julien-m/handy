// Options of the "Text formatting" section of the Advanced settings page (017 smart spacing,
// first; 015 smart capitalization; 016 smart paragraphs). Each option is one boolean setting
// with its own independent switch row: adding an option is one entry here (and its setting key
// in the union), nothing else in the section changes.
import { CaseSensitive, Pilcrow, Space, type LucideIcon } from "lucide-react";
import { smartParagraphsDescriptionKey } from "../smart-paragraphs-state";

/** Boolean settings shown in the Text formatting section. */
export type TextFormattingKey =
  | "smart_spacing"
  | "smart_capitalization"
  | "smart_paragraphs_enabled";

/** One switch row of the section. */
export interface TextFormattingOption {
  /** The boolean setting the switch writes through the settings service. */
  key: TextFormattingKey;
  /** Small decorative icon before the label. */
  icon: LucideIcon;
  /** i18n key of the label. */
  labelKey: string;
  /**
   * i18n key of the description shown by the info button, for the system the app runs on
   * (`type()` of `@tauri-apps/plugin-os`).
   */
  descriptionKey: (osType: string) => string;
  /** Test-id suffix of the row, after the screen prefix. */
  testId: string;
}

// @spec AC-008: Text formatting switch row — .specs/features/015-smart-capitalization/spec.md#ac-008
/** The section's rows, in display order. */
export const TEXT_FORMATTING_OPTIONS: readonly TextFormattingOption[] = [
  // @spec AC-014: First row of Text formatting — .specs/features/017-smart-spacing/spec.md#ac-014
  {
    key: "smart_spacing",
    icon: Space,
    labelKey: "settings.advanced.textFormatting.smartSpacing.label",
    descriptionKey: () =>
      "settings.advanced.textFormatting.smartSpacing.description",
    testId: "smart-spacing",
  },
  {
    key: "smart_capitalization",
    icon: CaseSensitive,
    labelKey: "settings.advanced.textFormatting.smartCapitalization.label",
    descriptionKey: () =>
      "settings.advanced.textFormatting.smartCapitalization.description",
    testId: "smart-capitalization",
  },
  // @spec AC-010: Text formatting row, FR-007: Translated row — .specs/features/016-smart-paragraphs/spec.md#ac-010
  {
    key: "smart_paragraphs_enabled",
    icon: Pilcrow,
    labelKey: "settings.advanced.smartParagraphs.title",
    descriptionKey: smartParagraphsDescriptionKey,
    testId: "smart-paragraphs",
  },
];
