/**
 * Prompt editor (feature 005) in headless Chromium, against the fake Tauri IPC of
 * `tests/harness/prompt-editor-fake.ts` with Apple Intelligence as the default provider: the
 * "Provider" dropdown and the "Model" combo share one row with the same height, bottom, width,
 * border radius and chevron size, and a long label is truncated inside its box (native session
 * finding of 2026-10-29). Not a native window proof.
 */
import { expect, test, type Locator } from "@playwright/test";

const HARNESS = "/tests/harness/prompt-editor.html?appleProvider";
const EDITOR = "prompt-settings-editor";

const box = async (locator: Locator) => {
  const found = await locator.boundingBox();
  expect(found).not.toBeNull();
  return found ?? { x: 0, y: 0, width: 0, height: 0 };
};
const style = (locator: Locator, property: string) =>
  locator.evaluate(
    (element, name) => getComputedStyle(element).getPropertyValue(name),
    property,
  );

for (const width of [1000, 560]) {
  test(`provider and model controls are aligned at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(HARNESS);
    const provider = page.getByTestId(`${EDITOR}-form-provider`);
    const model = page
      .getByTestId(`${EDITOR}-form-model`)
      .locator(".app-select__control");
    await expect(provider).toContainText("Apple Intelligence");
    await expect(model).toContainText("Apple Intelligence");
    const [p, m] = [await box(provider), await box(model)];
    expect(Math.abs(p.height - m.height)).toBeLessThanOrEqual(0.5);
    expect(Math.abs(p.y + p.height - (m.y + m.height))).toBeLessThanOrEqual(
      0.5,
    );
    expect(Math.abs(p.width - m.width)).toBeLessThanOrEqual(1);
    for (const property of [
      "border-top-width",
      "border-top-left-radius",
      "font-size",
    ]) {
      expect(await style(model, property)).toBe(
        await style(provider, property),
      );
    }
    const chevrons = [
      await box(provider.locator("svg")),
      await box(model.locator(".app-select__dropdown-indicator svg")),
    ];
    expect(chevrons[0].width).toBe(chevrons[1].width);
    await expect(page.locator(".app-select__indicator-separator")).toHaveCount(
      0,
    );
    const texts = [
      await box(provider.locator("span.truncate")),
      await box(model.locator(".app-select__single-value")),
    ];
    expect(Math.abs(texts[0].x - p.x - (texts[1].x - m.x))).toBeLessThanOrEqual(
      1,
    );
    const label = provider.locator("span.truncate");
    const clipped = await label.evaluate(
      (element) =>
        element.scrollWidth <= element.clientWidth + 1 ||
        getComputedStyle(element).textOverflow === "ellipsis",
    );
    expect(clipped).toBe(true);
  });
}
