/**
 * Dictation shortcut list (feature 004) in headless Chromium with the HandyKeys recorder,
 * against the fake Tauri IPC of `tests/harness/shortcut-list-fake.ts`: when "Add shortcut" opens
 * a draft whose recording macOS refuses (Secure Input), Add gives way to Discard and Save and the
 * keyboard focus goes to Discard, never to the page (native session finding N8, review).
 */
import { expect, test } from "@playwright/test";

/** Test id `suffix` under whichever shortcut screen is shown (a start refusal shows `-conflict`). */
const anyScreen = (suffix: string): RegExp =>
  new RegExp(`^shortcut-settings(-conflict|-limit)?-dictation-${suffix}$`);

test("a draft blocked by Secure Input gives the focus to Discard", async ({
  page,
}) => {
  await page.goto("/tests/harness/shortcut-list.html");
  const add = page.getByTestId(anyScreen("add"));
  await expect(add).toHaveText("Add shortcut");
  await add.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator('[data-phase="blocked"]')).toBeVisible();
  const discard = page.getByTestId(anyScreen("recording-discard"));
  await expect(discard).toBeFocused();
  // Discard closes the draft and gives the focus back to "Add shortcut".
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-phase]")).toHaveCount(0);
  await expect(page.getByTestId(anyScreen("add"))).toBeFocused();
});
