/**
 * Focus targets of the microphone priority editor (feature 001, SC-003): after Remove, focus goes
 * to the row taking the removed position, else the previous row, else the Add control, and the
 * section title is always the last resort, focused without scrolling. The DOM behavior is checked in
 * `tests/microphone-priority.spec.ts`. Run with `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import {
  ADD_FOCUS_KEY,
  STABLE_FOCUS_KEY,
  focusAfterRemove,
  focusOptionsOf,
  rowFocusKey,
} from "./editor-focus";

const order = ["USB headset", "MacBook", "Studio Display"];
const json = (value: unknown) => JSON.stringify(value);

// @spec AC-001: Keyboard users keep focus after Remove — .specs/features/001-microphone-priority/spec.md#ac-001
describe("focus after Remove", () => {
  test("the next row takes focus, then the previous, then Add, then the title", () => {
    expect(json(focusAfterRemove(order, 1))).toBe(
      json([
        rowFocusKey("Studio Display", "remove"),
        rowFocusKey("USB headset", "remove"),
        ADD_FOCUS_KEY,
        STABLE_FOCUS_KEY,
      ]),
    );
  });

  test("the last row: the previous row", () => {
    expect(focusAfterRemove(order, 2)[0]).toBe(
      rowFocusKey("MacBook", "remove"),
    );
  });

  test("the only row: the Add control", () => {
    expect(json(focusAfterRemove(["USB headset"], 0))).toBe(
      json([ADD_FOCUS_KEY, STABLE_FOCUS_KEY]),
    );
  });

  test("keys name the row and the control", () => {
    expect(
      rowFocusKey("MacBook", "up") === rowFocusKey("MacBook", "down"),
    ).toBe(false);
    expect(
      rowFocusKey("MacBook", "remove") ===
        rowFocusKey("Studio Display", "remove"),
    ).toBe(false);
  });
});

// @spec AC-001: Keyboard users keep focus after Remove — .specs/features/001-microphone-priority/spec.md#ac-001
describe("focus options", () => {
  test("focus returned to the section title does not scroll the page", () => {
    expect(json(focusOptionsOf(STABLE_FOCUS_KEY))).toBe(
      json({ preventScroll: true }),
    );
  });

  test("a row control or Add scrolls into view as usual", () => {
    expect(focusOptionsOf(ADD_FOCUS_KEY)).toBe(undefined);
    expect(focusOptionsOf(rowFocusKey("MacBook", "remove"))).toBe(undefined);
  });
});
