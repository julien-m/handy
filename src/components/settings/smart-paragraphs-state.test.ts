/**
 * Description of the smart paragraphs switch per platform (016 AC-007). Run with
 * `bun run test:smart-paragraphs`.
 */
import { describe, expect, test } from "bun:test";
import { smartParagraphsDescriptionKey } from "./smart-paragraphs-state";

describe("smart paragraphs description", () => {
  test("macOS segments every language", () => {
    expect(smartParagraphsDescriptionKey("macos")).toBe(
      "settings.advanced.smartParagraphs.description",
    );
  });
  test("other systems say that Chinese and Japanese stay unchanged", () => {
    for (const osType of ["windows", "linux"]) {
      expect(smartParagraphsDescriptionKey(osType)).toBe(
        "settings.advanced.smartParagraphs.descriptionLimited",
      );
    }
  });
});
