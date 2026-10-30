/**
 * Visibility of the experimental "Verify the paste" switch (007 AC-019): only while smart
 * paste is the paste method. Run with `bun run test:output-settings`.
 */
import { describe, expect, test } from "bun:test";
import { showsPasteVerificationToggle } from "./paste-verification-toggle-state";

describe("paste verification toggle", () => {
  test("shown while smart paste is the paste method", () => {
    expect(showsPasteVerificationToggle("smart_paste")).toBe(true);
  });
  test("hidden for every other paste method or without a value", () => {
    for (const method of [
      "ctrl_v",
      "direct",
      "none",
      "shift_insert",
      "ctrl_shift_v",
      "external_script",
    ]) {
      expect(showsPasteVerificationToggle(method)).toBe(false);
    }
    expect(showsPasteVerificationToggle(undefined)).toBe(false);
    expect(showsPasteVerificationToggle(null)).toBe(false);
  });
});
