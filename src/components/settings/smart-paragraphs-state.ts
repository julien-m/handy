// Description of the smart paragraphs switch (016) for the platform the app runs on.

const DESCRIPTION_KEY = "settings.advanced.smartParagraphs.description";
const LIMITED_DESCRIPTION_KEY =
  "settings.advanced.smartParagraphs.descriptionLimited";

// @spec AC-007: Limitation shown outside macOS — .specs/features/016-smart-paragraphs/spec.md#ac-007
/**
 * Business rule (016 AC-007): macOS segments every language with Apple NaturalLanguage; other
 * systems use Unicode segmentation and leave Chinese and Japanese unchanged, which the
 * description says there.
 * @param osType value of `type()` from `@tauri-apps/plugin-os`
 * @returns the translation key of the switch's description
 */
export const smartParagraphsDescriptionKey = (osType: string): string =>
  osType === "macos" ? DESCRIPTION_KEY : LIMITED_DESCRIPTION_KEY;
