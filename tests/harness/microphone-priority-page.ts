// Page helpers shared by the Playwright specs of the microphone priority editor (feature 001):
// the `window.harness` API of `microphone-priority.tsx`, the English texts and locators of the
// editor, and where keyboard focus is. Runs in the Playwright test process, not in the page.
import { expect, type Locator, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";

/** The harness API the page exposes (see `fake-tauri.ts`). */
export interface Harness {
  backend: {
    statusCalls: number;
    saveCalls: unknown[];
    listingFails: boolean;
    windowVisible: boolean;
    windowMinimized: boolean;
    windowVisibilityFails: boolean;
    windowVisibilityChecks: number;
  };
  commitElsewhere: (order: string[]) => void;
  holdSave: () => void;
  releaseSave: (failure: string | null) => void;
  releaseSaveUnknown: (committed: boolean) => void;
  releaseSettings: () => void;
  setVisible: (visible: boolean) => void;
  setWindowShown: (shown: boolean) => void;
  setWindowMinimized: (minimized: boolean) => void;
  showWindowWithoutFocus: () => void;
  unmount: () => void;
}

declare global {
  interface Window {
    harness: Harness;
  }
}

const en = JSON.parse(
  readFileSync(
    new URL("../../src/i18n/locales/en/translation.json", import.meta.url),
    "utf8",
  ),
);
/** English texts of the editor (`settings.sound.microphonePriority`). */
export const text = en.settings.sound.microphonePriority;
export const HARNESS = "/tests/harness/microphone-priority.html";
export const USB = "USB headset";
export const MACBOOK = "MacBook Pro Microphone";
export const STUDIO = "Studio Display Microphone";
export const ORIGINAL = [USB, MACBOOK, STUDIO];

/** Open the harness and wait for the editor. */
export const open = async (page: Page, query = ""): Promise<void> => {
  await page.goto(`${HARNESS}${query}`);
  await expect(page.getByTestId("microphone-settings-screen")).toBeVisible();
};

/** The row of the exact microphone `name`. */
export const row = (page: Page, name: string): Locator =>
  page.locator(`li[data-name="${name}"]`);

/** One control (`move-up`, `move-down`, `remove`, ...) of the row of `name`. */
export const control = (page: Page, name: string, id: string): Locator =>
  row(page, name).getByTestId(`microphone-settings-choice-${id}`);

/** The order shown in the list. */
export const order = (page: Page): Promise<string[]> =>
  page
    .getByTestId("microphone-settings-choice")
    .evaluateAll((rows) =>
      rows.map((item) => item.getAttribute("data-name") ?? ""),
    );

/** Where focus is: the focused element's test ID (or role) and the row that holds it. */
export const focused = (page: Page): Promise<string> =>
  page.evaluate(() => {
    const element = document.activeElement;
    if (element === null || element === document.body) return "body";
    const id =
      element.getAttribute("data-testid") ??
      element.closest("[data-testid]")?.getAttribute("data-testid");
    const name = element.closest("li")?.getAttribute("data-name");
    const role = element.getAttribute("role");
    if (name) return `${id} of ${name}`;
    return role ? `${id} (${role})` : `${id}`;
  });

/** Wait until focus is on `expected` (see `focused`). */
export const expectFocus = async (
  page: Page,
  expected: string,
): Promise<void> => {
  await expect.poll(() => focused(page)).toBe(expected);
};

/** Make the draft dirty from the keyboard: Move up on the second row. */
export const editFromKeyboard = async (page: Page): Promise<void> => {
  await control(page, MACBOOK, "move-up").focus();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("microphone-settings-draft")).toBeVisible();
};

/** Hold the next write, then press Enter on a focused Save: Save is busy and keeps focus. */
export const startHeldSave = async (page: Page): Promise<Locator> => {
  const save = page.getByTestId("microphone-settings-save");
  await page.evaluate(() => window.harness.holdSave());
  await save.focus();
  await page.keyboard.press("Enter");
  await expect(save).toHaveAttribute("data-state", "loading");
  await expect(save).toHaveAttribute("aria-disabled", "true");
  await expectFocus(page, "microphone-settings-save");
  return save;
};
