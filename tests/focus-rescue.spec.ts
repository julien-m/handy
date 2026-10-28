/**
 * `useFocusRescue` (native session finding N5) in headless Chromium: a focused control that
 * leaves the section gives the focus to the section title, even when the section's root ref is
 * attached again on the same render (an inline ref callback).
 */
import { expect, test } from "@playwright/test";

for (const query of ["", "?webkitFocus"]) {
  test(`a focused control removed on a re-render gives the focus to the section title ${query}`, async ({
    page,
  }) => {
    await page.goto(`/tests/harness/focus-rescue.html${query}`);
    await page.getByTestId("leaving-button").focus();
    await page.keyboard.press("Enter");
    await expect(page.getByText("Saved")).toBeVisible();
    await expect
      .poll(() =>
        page.evaluate(() => ({
          tag: document.activeElement?.tagName,
          text: document.activeElement?.textContent,
        })),
      )
      .toEqual({ tag: "H2", text: "Section" });
  });
}
