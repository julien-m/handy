/**
 * Visibility of "Show the bubble when no text field is focused" (010 AC-007): only when the
 * settings snapshot carries the key (destination-return gate open) and the OS is macOS.
 * Run with `bun run test:output-settings`.
 */
import { describe, expect, test } from "bun:test";
import { showsNoFieldBubbleToggle } from "./recovery-bubble-toggle-state";

describe("no-text-field bubble toggle", () => {
  test("shown on macOS when the open gate provides the key", () => {
    expect(showsNoFieldBubbleToggle(true, "macos")).toBe(true);
    expect(showsNoFieldBubbleToggle(false, "macos")).toBe(true);
  });
  test("hidden with the gate closed or off macOS", () => {
    expect(showsNoFieldBubbleToggle(undefined, "macos")).toBe(false);
    expect(showsNoFieldBubbleToggle(null, "macos")).toBe(false);
    expect(showsNoFieldBubbleToggle(false, "windows")).toBe(false);
    expect(showsNoFieldBubbleToggle(false, "linux")).toBe(false);
  });
});
