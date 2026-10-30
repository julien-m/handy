/**
 * The « Insert without Enter » mark of the recording indicator (014 AC-004): shown with its
 * accessible name only while the dictation's snapshot says so, with or without the destination
 * badge. Run with `bun test src/overlay/enter-skipped-render.test.tsx`.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import en from "../i18n/locales/en/translation.json";
import {
  chatgpt,
  destination,
  initIndicatorI18n,
  render,
  session,
} from "./indicator-test-fixtures";

beforeAll(initIndicatorI18n);

const MARK = "recording-indicator-enter-skipped";
const label = en.overlay.dictationKeys.enterSkipped;

describe("Insert without Enter mark", () => {
  test("shown with its accessible name when the key was pressed", () => {
    const skipped = { ...session(chatgpt), enter_skipped: true };
    const markup = render({ destination: destination(skipped) });
    expect(markup.includes(`data-testid="${MARK}"`)).toBe(true);
    expect(markup.includes(`aria-label="${label}"`)).toBe(true);
  });

  test("shown without a badge (return off, nothing pinned)", () => {
    const hidden = {
      ...session(chatgpt),
      return_enabled: false,
      enter_skipped: true,
    };
    const markup = render({ destination: destination(hidden) });
    expect(markup.includes(`data-testid="${MARK}"`)).toBe(true);
  });

  test("absent otherwise", () => {
    const markup = render({ destination: destination(session(chatgpt)) });
    expect(markup.includes(MARK)).toBe(false);
  });
});
