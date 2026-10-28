/**
 * Replacement rules editor (feature 002) in headless Chromium, against the fake Tauri IPC of
 * `tests/harness/word-replacements-fake.ts`: the refused-import alert inside the section (native
 * session finding N6). Not a native window proof.
 */
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/word-replacements.html";
const RULE = "replacement-settings-rule";

const open = async (page: Page): Promise<void> => {
  await page.goto(HARNESS);
  await expect(page.getByTestId(RULE)).toHaveCount(2);
};

test("a refused import file is announced under the Replacements title, inside its section", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("replacement-settings-import").click();
  const alert = page.getByTestId("replacement-settings-import-unreadable");
  await expect(alert).toBeVisible();
  const section = page.getByTestId("replacement-settings-replacements");
  await expect(
    section.getByTestId("replacement-settings-import-unreadable"),
  ).toBeVisible();
  const title = section.getByRole("heading", { name: /replacements/i });
  const titleBox = await title.boundingBox();
  const alertBox = await alert.boundingBox();
  expect(titleBox).not.toBeNull();
  expect(alertBox!.y).toBeGreaterThan(titleBox!.y + titleBox!.height);
});
