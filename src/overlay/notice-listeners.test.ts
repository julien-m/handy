/**
 * Wiring of the cut-short recording report (001, native session N1): the `recording-error`
 * listeners of the indicator (`subscribeNoticeSources`) and of the overlay-none fallback
 * (`subscribeNoneNotices`), with the settings window's own `recording-error` toast. Only
 * `input_lost` raises the indicator notice or the fallback toast, and for every overlay style the
 * user gets exactly one report. No DOM renderer is installed, so the effects' bodies are tested
 * directly. Run with `bun run test:indicator`.
 */
import { beforeEach, describe, expect, mock, test } from "bun:test";
import type { TFunction } from "i18next";
import type { OverlayStyle } from "@/bindings";
import type { RecordingErrorEvent } from "@/lib/types/events";

type Handler = (event: { payload: unknown }) => void;
const handlers = new Map<string, Handler[]>();
const subscribe = (event: string, handler: Handler) => {
  handlers.set(event, [...(handlers.get(event) ?? []), handler]);
  return Promise.resolve(() => undefined);
};
const toasts: string[] = [];
const record = (title: string) => {
  toasts.push(title);
};
// The real modules need Tauri IPC and a DOM; the listeners under test only need these.
mock.module("@tauri-apps/api/event", () => ({ listen: subscribe }));
mock.module("@/bindings", () => ({
  commands: {},
  events: {
    promptProcessingFailed: {
      listen: (handler: Handler) =>
        subscribe("prompt-processing-failed", handler),
    },
  },
}));
const descriptions: string[] = [];
const recordError = (title: string, options?: { description?: string }) => {
  toasts.push(title);
  if (options?.description) descriptions.push(options.description);
};
mock.module("sonner", () => ({
  toast: { error: recordError, warning: record, info: record },
}));
mock.module("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));

const { subscribeNoticeSources } = await import("./use-overlay-notices");
const { subscribeNoneNotices } = await import(
  "@/hooks/use-overlay-none-notices"
);
const { showRecordingErrorToast } = await import("@/lib/recording-error-toast");
const {
  NO_PIN_EVENT,
  noticeFallsBackToToast,
  PASTE_ERROR_EVENT,
  pasteFailedNotice,
  RECORDING_ERROR_EVENT,
} = await import("./notice-model");
// The keys stand in for their translations: only which report appears is checked.
const t = ((key: string) => key) as unknown as TFunction;

/** Subscribe the three listeners for `style`; returns the kinds of indicator notices pushed. */
const wire = (style: OverlayStyle): string[] => {
  handlers.clear();
  const notices: string[] = [];
  void subscribeNoticeSources((build) => {
    const notice = build(0);
    if (notice) notices.push(notice.kind);
  });
  void subscribeNoneNotices(t, () => noticeFallsBackToToast(style));
  return notices;
};

/** Emit `recording-error` to every listener, the settings window's toast included (App.tsx). */
const emit = (payload: RecordingErrorEvent) => {
  for (const handler of handlers.get(RECORDING_ERROR_EVENT) ?? []) {
    handler({ payload });
  }
  showRecordingErrorToast(payload, t);
};

beforeEach(() => {
  toasts.length = 0;
  descriptions.length = 0;
});

describe("recording-error listeners (001 FR-004, native session N1)", () => {
  test("both surfaces listen to recording-error", () => {
    wire("minimal");
    expect(handlers.get(RECORDING_ERROR_EVENT)?.length).toBe(2);
  });

  for (const style of ["minimal", "live"] as const) {
    test(`${style}: input_lost is one indicator notice and no toast`, () => {
      const notices = wire(style);
      emit({ error_type: "input_lost" });
      expect(notices).toEqual(["input_lost"]);
      expect(toasts).toEqual([]);
    });
  }

  test("none: input_lost is one toast; the indicator window never shows", () => {
    const notices = wire("none");
    emit({ error_type: "input_lost" });
    expect(toasts).toEqual(["errors.inputLostTitle"]);
    // The overlay pushes its notice, but `hold_overlay_for_notice` refuses to show the window
    // when the style is none, so the toast is the only report.
    expect(notices).toEqual(["input_lost"]);
  });

  for (const style of ["none", "minimal", "live"] as const) {
    test(`${style}: other recording errors raise no notice and keep their single toast`, () => {
      const notices = wire(style);
      for (const error_type of ["no_input_device", "recording_failed"]) {
        emit({ error_type });
      }
      expect(notices).toEqual([]);
      expect(toasts).toEqual([
        "errors.noInputDeviceTitle",
        "errors.recordingFailed",
      ]);
    });
  }
});

/** Emit `paste-error` to every listener (the settings window has no listener of its own). */
const emitPasteError = (payload: { history_saved: boolean } | null) => {
  for (const handler of handlers.get(PASTE_ERROR_EVENT) ?? []) {
    handler({ payload });
  }
};

describe("paste-error listeners (010 FR-002, native session N17)", () => {
  test("only the indicator and the overlay-none fallback listen to paste-error", () => {
    wire("minimal");
    expect(handlers.get(PASTE_ERROR_EVENT)?.length).toBe(2);
  });

  for (const style of ["minimal", "live"] as const) {
    test(`${style}: a failed insertion is one indicator notice and no toast`, () => {
      const notices = wire(style);
      emitPasteError({ history_saved: true });
      expect(notices).toEqual(["paste_failed"]);
      expect(toasts).toEqual([]);
    });
  }

  test("none: a failed insertion is one toast; the indicator window never shows", () => {
    const notices = wire("none");
    emitPasteError({ history_saved: false });
    expect(toasts).toEqual(["errors.pasteFailedTitle"]);
    expect(descriptions).toEqual(["errors.pasteFailedNotSaved"]);
    // As for input_lost: `hold_overlay_for_notice` refuses to show the window with style none.
    expect(notices).toEqual(["paste_failed"]);
  });

  test("the notice says whether the text is in history and lasts four seconds", () => {
    const saved = pasteFailedNotice({ history_saved: true }, 1000);
    expect(saved.key).toBe("errors.pasteFailed");
    expect(saved.until).toBe(5000);
    expect(saved.testId).toBe("recording-indicator-paste-failed-notice");
    expect(pasteFailedNotice(null, 0).key).toBe("errors.pasteFailed");
    expect(pasteFailedNotice({ history_saved: false }, 0).key).toBe(
      "errors.pasteFailedNotSaved",
    );
  });
});

describe("no-pin listener (014 AC-007)", () => {
  test("the key pressed without a pin is one indicator notice and no toast", () => {
    const notices = wire("minimal");
    for (const handler of handlers.get(NO_PIN_EVENT) ?? []) {
      handler({ payload: null });
    }
    expect(notices).toEqual(["no_pin"]);
    expect(toasts).toEqual([]);
  });
});
