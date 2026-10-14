// Tests of the history limit field validation (008 validation display); run
// with `bun run test:output-settings`.
import { describe, expect, test } from "bun:test";
import { HISTORY_LIMIT_RANGE, parseHistoryLimit } from "./history-limit-model";

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
