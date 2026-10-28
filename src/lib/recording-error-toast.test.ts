/**
 * Toasts of the backend's `recording-error` event (001, native session N1): a recording cut
 * short by a lost microphone is never a toast of the settings window, except through the
 * overlay-none fallback, so it is reported once. Run with `bun run test:indicator`.
 */
import { beforeEach, describe, expect, mock, test } from "bun:test";
import type { TFunction } from "i18next";

const shown: { kind: string; title: string; duration?: number }[] = [];
const record =
  (kind: string) => (title: string, options?: { duration?: number }) =>
    shown.push({ kind, title, duration: options?.duration });
mock.module("sonner", () => ({
  toast: {
    error: record("error"),
    warning: record("warning"),
    info: record("info"),
  },
}));
mock.module("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));

const { showInputLostToast, showRecordingErrorToast } = await import(
  "./recording-error-toast"
);
const { NOTICE_MIN_MS } = await import("@/overlay/notice-model");
// The keys stand in for their translations: only which toast appears is checked.
const t = ((key: string) => key) as unknown as TFunction;

beforeEach(() => {
  shown.length = 0;
});

describe("recording-error toasts", () => {
  test("a recording cut short is left to the indicator: no settings toast", () => {
    showRecordingErrorToast({ error_type: "input_lost" }, t);
    expect(shown).toEqual([]);
  });
  test("start errors keep their toast", () => {
    showRecordingErrorToast({ error_type: "no_input_device" }, t);
    expect(shown.map((toast) => toast.title)).toEqual([
      "errors.noInputDeviceTitle",
    ]);
  });
  test("the overlay-none fallback shows one warning for at least 4 s", () => {
    showInputLostToast(t);
    expect(shown).toEqual([
      {
        kind: "warning",
        title: "errors.inputLostTitle",
        duration: NOTICE_MIN_MS,
      },
    ]);
  });
});
