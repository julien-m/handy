/**
 * Prompt editor (feature 005) in headless Chromium, against the fake Tauri IPC of
 * `tests/harness/prompt-editor-fake.ts`: after Save, which turns disabled once the prompt is
 * saved, the keyboard focus goes to the editor's title instead of the page (native session
 * finding N5). Not a native window proof.
 */
import { expect, test } from "@playwright/test";

const HARNESS = "/tests/harness/prompt-editor.html";
const EDITOR = "prompt-settings-editor";

for (const query of ["", "?webkitFocus"]) {
  test(`Save gives the focus to the editor title once the prompt is saved ${query}`, async ({
    page,
  }) => {
    await page.goto(`${HARNESS}${query}`);
    const name = page.getByTestId(`${EDITOR}-form-name`);
    await expect(name).toHaveValue("Meeting notes");
    await name.fill("Meeting notes (short)");
    const save = page.getByTestId(`${EDITOR}-save`);
    await expect(save).toHaveAttribute("data-state", "enabled");
    await save.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByTestId(`${EDITOR}-saved`)).toBeVisible();
    await expect
      .poll(() =>
        page.evaluate(() => ({
          tag: document.activeElement?.tagName,
          text: document.activeElement?.textContent,
        })),
      )
      .toEqual({ tag: "H2", text: "Edit prompt" });
    expect(
      await page.evaluate(
        () =>
          (window as unknown as { harness: { backend: { updates: number } } })
            .harness.backend.updates,
      ),
    ).toBe(1);
  });
}
