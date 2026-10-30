/**
 * Automatic Enter row of the output settings (009 Behavioral AC, AC-004) in headless Chromium:
 * the real component and settings store against the fake Tauri IPC of
 * `tests/harness/automatic-enter-fake.ts`. The selector lists Never, Always and Only in PIN mode
 * with the current value marked, arrow keys and Enter save Only in PIN mode and announce it, and
 * Tab reaches the selector then its key selector with visible focus. Not a native window proof
 * (the badge menu stays pointer-operated per 006 and is covered by the indicator markup tests).
 * Run with `bun run test:pin-enter-ui`.
 */
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/automatic-enter.html";
// Closed, the row is on the output-settings screen; its open list is the
// output-settings-enter-options screen, whose prefix every id of the row then carries (D1).
const MODE = "output-settings-auto-enter-mode";
const KEY = "output-settings-auto-enter-key";
const OPEN = "output-settings-enter-options-auto-enter";

const open = async (page: Page): Promise<void> => {
  await page.goto(HARNESS);
  await expect(page.getByTestId(MODE)).toBeVisible();
};
const writes = (page: Page) =>
  page.evaluate(
    () =>
      (
        window as unknown as {
          harness: {
            backend: { writes: Array<{ command: string; value: unknown }> };
          };
        }
      ).harness.backend.writes,
  );

// @spec AC-004: Automatic Enter selector, keyboard and announcement — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
test("the selector lists the three values with the current one marked", async ({
  page,
}) => {
  await open(page);
  await expect(page.getByTestId(MODE)).toHaveText("Never");
  await page.getByTestId(MODE).click();
  const list = page.getByTestId(`${OPEN}-options`);
  await expect(page.getByTestId(`${OPEN}-mode`)).toBeVisible();
  await expect(page.getByTestId(MODE)).toHaveCount(0);
  await expect(list.getByRole("option")).toHaveText([
    "Never",
    /^Always/,
    /^Only in PIN mode/,
  ]);
  await expect(page.getByTestId(`${OPEN}-never`)).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.getByTestId(`${OPEN}-key`)).toHaveText("Enter");
});

test("arrow keys and Enter save Only in PIN mode and announce it", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("harness-start").focus();
  await page.keyboard.press("Tab");
  await expect(page.getByTestId(MODE)).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(page.getByTestId(MODE)).toHaveText("Only in PIN mode");
  expect(await writes(page)).toEqual([
    { command: "change_automatic_enter_setting", value: "pin_only" },
  ]);
  await expect(page.getByRole("status")).toHaveText(
    "Automatic Enter: Only in PIN mode",
  );
});

test("Tab gives the selector then its key selector visible focus in reading order", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("harness-start").focus();
  for (const id of [MODE, KEY]) {
    await page.keyboard.press("Tab");
    await expect(page.getByTestId(id)).toBeFocused();
    // The visible focus ring (`focus-visible:ring-2`) is a box shadow.
    const ring = await page
      .getByTestId(id)
      .evaluate((element) => getComputedStyle(element).boxShadow);
    expect(ring).not.toBe("none");
  }
});
