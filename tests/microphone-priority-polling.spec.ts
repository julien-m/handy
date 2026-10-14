/**
 * Status polling of the microphone priority editor (feature 001, AC-002) in headless Chromium, on
 * Playwright's simulated clock: the real status hook against the fake Tauri IPC of
 * `tests/harness/`. The page visibility and the Tauri window visibility are switched apart, so the
 * window cases reproduce a webview that keeps reporting "visible" after `window.hide()` or a
 * minimize (expected on Windows with WebView2), and a window shown without a focus event. Not a native window proof. Run with
 * `bun run test:microphone-priority-ui`.
 */
import { expect, test, type Page } from "@playwright/test";
import { open } from "./harness/microphone-priority-page";

const PERIOD = 5000;

const reads = (page: Page): Promise<number> =>
  page.evaluate(() => window.harness.backend.statusCalls);

/** Open the editor on a paused clock once the read on open is done; returns the read count. */
const openOnPausedClock = async (page: Page): Promise<number> => {
  await page.clock.install();
  await open(page);
  await expect.poll(() => reads(page)).toBe(1);
  // From here the page clock only moves with runFor.
  await page.clock.pauseAt(new Date(Date.now() + 2000));
  return reads(page);
};

test("page visibility: on open, every 5 s while visible, paused while hidden, cleared on unmount", async ({
  page,
}) => {
  await openOnPausedClock(page);
  await page.evaluate(() => window.harness.setVisible(false));
  const base = await reads(page);
  await page.clock.runFor(60_000);
  expect(await reads(page)).toBe(base);
  // Shown again: a read at once, then one every 5 s.
  await page.evaluate(() => window.harness.setVisible(true));
  await expect.poll(() => reads(page)).toBe(base + 1);
  await page.clock.runFor(PERIOD - 1);
  expect(await reads(page)).toBe(base + 1);
  await page.clock.runFor(1);
  await expect.poll(() => reads(page)).toBe(base + 2);
  await page.clock.runFor(PERIOD);
  await expect.poll(() => reads(page)).toBe(base + 3);
  // Unmounted: no more reads.
  await page.evaluate(() => window.harness.unmount());
  await page.clock.runFor(30_000);
  expect(await reads(page)).toBe(base + 3);
});

test("window hidden while the page stays visible: no read; shown and focused from the tray: a read at once", async ({
  page,
}) => {
  const base = await openOnPausedClock(page);
  await page.evaluate(() => window.harness.setWindowShown(false));
  // The next period asks the window, finds it hidden and stops the timer.
  await page.clock.runFor(PERIOD);
  const checks = () =>
    page.evaluate(() => window.harness.backend.windowVisibilityChecks);
  await expect.poll(checks).toBeGreaterThan(0);
  const asked = await checks();
  await page.clock.runFor(60_000);
  expect(await reads(page)).toBe(base);
  expect(await checks()).toBe(asked);
  // Reopened from the tray: shown and focused, so a read at once, then one every 5 s.
  await page.evaluate(() => window.harness.setWindowShown(true));
  await expect.poll(() => reads(page)).toBe(base + 1);
  await page.clock.runFor(PERIOD);
  await expect.poll(() => reads(page)).toBe(base + 2);
});

test("window shown by the backend without a focus event: a read at once (main-window-shown)", async ({
  page,
}) => {
  const base = await openOnPausedClock(page);
  await page.evaluate(() => window.harness.setWindowShown(false));
  await page.clock.runFor(PERIOD);
  await page.clock.runFor(30_000);
  expect(await reads(page)).toBe(base);
  await page.evaluate(() => window.harness.showWindowWithoutFocus());
  await expect.poll(() => reads(page)).toBe(base + 1);
  await page.clock.runFor(PERIOD);
  await expect.poll(() => reads(page)).toBe(base + 2);
});

test("window minimized while the page stays visible: no read until shown again", async ({
  page,
}) => {
  const base = await openOnPausedClock(page);
  await page.evaluate(() => window.harness.setWindowMinimized(true));
  // The next period asks the window, finds it minimized and stops the timer.
  await page.clock.runFor(PERIOD);
  await page.clock.runFor(60_000);
  expect(await reads(page)).toBe(base);
  await page.evaluate(() => window.harness.showWindowWithoutFocus());
  await expect.poll(() => reads(page)).toBe(base + 1);
});

test("the window cannot tell its visibility: the page visibility decides", async ({
  page,
}) => {
  const base = await openOnPausedClock(page);
  await page.evaluate(() => {
    window.harness.backend.windowVisibilityFails = true;
  });
  await page.clock.runFor(PERIOD);
  await expect.poll(() => reads(page)).toBe(base + 1);
  await page.clock.runFor(PERIOD);
  await expect.poll(() => reads(page)).toBe(base + 2);
  await page.evaluate(() => window.harness.setVisible(false));
  await page.clock.runFor(60_000);
  expect(await reads(page)).toBe(base + 2);
});
