/**
 * Tauri window visibility source of the status polling (feature 001, AC-002): a minimized window
 * counts as hidden, subscriptions that resolve late or fail are handled, and the event the
 * frontend listens to is the one `show_main_window` emits. The polling itself is covered in
 * `status-polling.test.ts`. Run with `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import {
  MAIN_WINDOW_SHOWN_EVENT,
  windowVisibility,
  type WindowPort,
} from "./window-visibility";

/** Let pending subscriptions settle (they resolve in microtasks). */
const answered = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** A visible, focused window whose listeners do nothing. */
const basePort = (): WindowPort => ({
  isVisible: async () => true,
  isMinimized: async () => false,
  onFocusChanged: async () => () => undefined,
  onShown: async () => () => undefined,
});

// @spec AC-002: Status read on open and every 5 s while visible — .specs/features/001-microphone-priority/spec.md#ac-002
describe("window visibility source", () => {
  test("an unsubscribe before the focus listener is ready releases it once ready", async () => {
    const released: string[] = [];
    let ready: (release: () => void) => void = () => undefined;
    const port: WindowPort = {
      ...basePort(),
      onFocusChanged: () => new Promise((resolve) => (ready = resolve)),
    };
    const unsubscribe = windowVisibility(port).subscribe(() => undefined);
    unsubscribe();
    ready(() => void released.push("focus listener"));
    await answered();
    expect(released).toEqual(["focus listener"]);
  });

  test("failed subscriptions are reported", async () => {
    const warnings: string[] = [];
    const refuse = async (): Promise<() => void> => {
      throw new Error("listen refused");
    };
    const port: WindowPort = {
      ...basePort(),
      onFocusChanged: refuse,
      onShown: refuse,
    };
    windowVisibility(port, (message) => warnings.push(message)).subscribe(
      () => undefined,
    );
    await answered();
    expect(warnings).toEqual([
      "Window focus subscription failed:",
      "Window shown subscription failed:",
    ]);
  });

  test("a minimized window is not visible; isMinimized is asked only when shown", async () => {
    const state = { visible: true, minimized: true, minimizedAsks: 0 };
    const port: WindowPort = {
      ...basePort(),
      isVisible: async () => state.visible,
      isMinimized: async () => {
        state.minimizedAsks += 1;
        return state.minimized;
      },
    };
    const source = windowVisibility(port);
    expect(await source.isVisible()).toBe(false);
    state.visible = false;
    expect(await source.isVisible()).toBe(false);
    expect(state.minimizedAsks).toBe(1);
    state.visible = true;
    state.minimized = false;
    expect(await source.isVisible()).toBe(true);
  });

  test("the event name matches the one show_main_window emits", () => {
    const lib = readFileSync(
      new URL("../../../../src-tauri/src/lib.rs", import.meta.url),
      "utf8",
    );
    expect(lib).toContain(
      `const MAIN_WINDOW_SHOWN_EVENT: &str = "${MAIN_WINDOW_SHOWN_EVENT}";`,
    );
    expect(lib).toContain("app.emit(MAIN_WINDOW_SHOWN_EVENT, ())");
  });
});
