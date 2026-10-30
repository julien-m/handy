/**
 * Rules of the no-pin notice of 014 (AC-007): « Send to the App in Front » pressed with nothing
 * pinned. Run with `bun run test:indicator`.
 */
import { describe, expect, test } from "bun:test";
import en from "@/i18n/locales/en/translation.json";
import {
  addNotice,
  holdRemaining,
  inputLostNotice,
  NOTICE_MIN_MS,
  noPinNotice,
} from "./notice-model";

// @spec AC-007: No-pin notice on the indicator — .specs/features/014-in-dictation-keys/spec.md#ac-007
describe("no-pin notice (014 AC-007)", () => {
  test("the key pressed without a pin says so for 4 s", () => {
    const notice = noPinNotice(250);
    expect(notice.kind).toBe("no_pin");
    expect(notice.until).toBe(250 + NOTICE_MIN_MS);
    expect(notice.testId).toBe("recording-indicator-no-pin-notice");
    expect(notice.key).toBe("overlay.dictationKeys.noPin");
    expect(en.overlay.dictationKeys.noPin).toBe("No pinned app");
  });
  test("a second press replaces the notice and restarts its 4 s", () => {
    const first = addNotice([inputLostNotice(0)], noPinNotice(0), 0);
    const again = addNotice(first, noPinNotice(1500), 1500);
    expect(again.map((notice) => notice.kind)).toEqual([
      "input_lost",
      "no_pin",
    ]);
    expect(holdRemaining(again, 1500)).toBe(NOTICE_MIN_MS);
  });
});
