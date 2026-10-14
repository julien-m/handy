// Tests of the history limit field validation (008 validation display); run
// with `bun run test:output-settings`.
import { describe, expect, test } from "bun:test";
import {
  HISTORY_LIMIT_RANGE,
  type HistoryLimitEvent,
  SHOWN_STORED,
  historyLimitStep,
  isBlurWithinWindow,
  isCommitKey,
  parseHistoryLimit,
} from "./history-limit-model";

describe("history limit field", () => {
  test("whole numbers in range are accepted", () => {
    expect(parseHistoryLimit("0")).toEqual({ isValid: true, value: 0 });
    expect(parseHistoryLimit(" 25 ")).toEqual({ isValid: true, value: 25 });
    expect(parseHistoryLimit(String(HISTORY_LIMIT_RANGE.max))).toEqual({
      isValid: true,
      value: 1000,
    });
  });

  test("negative, fractional, empty or too large values are refused", () => {
    for (const text of ["-5", "2.5", "", "abc", "1e3", "1001"]) {
      expect(parseHistoryLimit(text)).toEqual({ isValid: false });
    }
  });
});

/** Plays events on the field as the component does; returns the saves and the last field. */
const play = (stored: number, events: HistoryLimitEvent[]) => {
  const saves: number[] = [];
  let field = SHOWN_STORED;
  for (const event of events) {
    const step = historyLimitStep(field, event, stored);
    field = step.field;
    if (step.save !== null) saves.push(step.save);
  }
  return { saves, field };
};
const typed = (text: string): HistoryLimitEvent => ({ type: "typed", text });
const COMMITTED: HistoryLimitEvent = { type: "committed" };
/** Blur to another place of the window. */
const LEFT_INSIDE: HistoryLimitEvent = { type: "left", withinWindow: true };
/** Blur because the window lost focus (another app, closed to the tray). */
const WINDOW_LOST: HistoryLimitEvent = { type: "left", withinWindow: false };

describe("saving the history limit (blur or Enter)", () => {
  test("typing 1 then 12 sends nothing until the field is left, then 12 once", () => {
    const whileTyping = play(5, [typed("1"), typed("12")]);
    expect(whileTyping.saves).toEqual([]);
    expect(whileTyping.field).toEqual({ draft: "12" });
    const left = play(5, [typed("1"), typed("12"), COMMITTED]);
    expect(left.saves).toEqual([12]);
    expect(left.field).toEqual(SHOWN_STORED);
  });

  test("a refused value stays in the field with its error and is never sent", () => {
    const refused = play(5, [typed("-5"), COMMITTED, COMMITTED]);
    expect(refused.saves).toEqual([]);
    expect(refused.field).toEqual({ draft: "-5" });
    expect(parseHistoryLimit(refused.field.draft ?? "").isValid).toBe(false);
  });

  test("Enter commits like leaving the field; other keys do not", () => {
    expect(isCommitKey("Enter")).toBe(true);
    expect(isCommitKey("1")).toBe(false);
    expect(isCommitKey("Tab")).toBe(false);
    // Enter, then the blur that follows: one save only.
    expect(play(5, [typed("40"), COMMITTED, LEFT_INSIDE]).saves).toEqual([40]);
  });

  test("an unchanged value is not sent again", () => {
    expect(play(5, [typed("5"), COMMITTED]).saves).toEqual([]);
    expect(play(5, [COMMITTED]).saves).toEqual([]);
    expect(play(5, [typed(" 5 "), COMMITTED]).field).toEqual(SHOWN_STORED);
  });

  test("a value corrected after a refusal is saved", () => {
    const corrected = play(5, [
      typed("1001"),
      COMMITTED,
      typed("100"),
      COMMITTED,
    ]);
    expect(corrected.saves).toEqual([100]);
  });
});

describe("blur caused by the window losing focus", () => {
  test("typing 1 on the way to 12, then switching app, saves nothing", () => {
    const away = play(5, [typed("1"), WINDOW_LOST]);
    expect(away.saves).toEqual([]);
    expect(away.field).toEqual({ draft: "1" });
  });

  test("back in the window the draft is kept and finished normally", () => {
    const back = play(5, [typed("1"), WINDOW_LOST, typed("12"), LEFT_INSIDE]);
    expect(back.saves).toEqual([12]);
    const enter = play(5, [typed("1"), WINDOW_LOST, typed("12"), COMMITTED]);
    expect(enter.saves).toEqual([12]);
  });

  test("leaving the field for another place of the window saves", () => {
    expect(play(5, [typed("40"), LEFT_INSIDE]).saves).toEqual([40]);
  });

  test("a blur stays in the window when the document keeps focus or another element takes it", () => {
    expect(isBlurWithinWindow(true, null)).toBe(true);
    expect(isBlurWithinWindow(false, { tagName: "BUTTON" })).toBe(true);
    expect(isBlurWithinWindow(false, null)).toBe(false);
    expect(isBlurWithinWindow(false, undefined)).toBe(false);
  });
});
