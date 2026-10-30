/** Focus rescue preserves deliberate focus changes while the prompt is saved. */
import { expect, test } from "@playwright/test";

for (const destination of ["outside", "body"]) {
  test(`Save preserves deliberate focus on ${destination}`, async ({
    page,
  }) => {
    await page.goto("/tests/harness/prompt-editor.html");
    const name = page.getByTestId("prompt-settings-editor-form-name");
    await expect(name).toHaveValue("Meeting notes");
    await name.fill("Meeting notes (short)");
    await page.evaluate((target) => {
      if (target === "outside") {
        const input = document.createElement("input");
        input.dataset.testid = "outside-editor";
        document.body.append(input);
        input.focus();
      } else {
        (document.activeElement as HTMLElement).blur();
      }
      return new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      );
    }, destination);
    // Programmatic activation does not move the focus chosen above.
    await page
      .getByTestId("prompt-settings-editor-save")
      .evaluate((button: HTMLButtonElement) => button.click());
    await expect(
      page.getByTestId("prompt-settings-editor-saved"),
    ).toBeVisible();
    if (destination === "outside") {
      await expect(page.getByTestId("outside-editor")).toBeFocused();
    } else {
      await expect
        .poll(() => page.evaluate(() => document.activeElement?.tagName))
        .toBe("BODY");
    }
  });
}
