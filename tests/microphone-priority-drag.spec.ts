/**
 * Pointer reorder of the microphone priority editor (feature 001) in headless Chromium: the real
 * list against the fake Tauri IPC of `tests/harness/`. An interrupted drag (window blur, page
 * hidden, lost pointer capture, pointercancel), a drag whose row is removed by a change saved
 * elsewhere and a drag overtaken by a starting write are cancelled without reordering; an
 * uninterrupted drag drops. Not a native window proof. Run with `bun run test:microphone-priority-ui`.
 */
import { expect, test, type Page } from "@playwright/test";
import {
  MACBOOK,
  ORIGINAL,
  STUDIO,
  USB,
  editFromKeyboard,
  open,
  order,
  row,
} from "./harness/microphone-priority-page";

const handleOf = (page: Page, name: string) =>
  row(page, name).locator('span[aria-hidden="true"]').first();

/** Center of the handle of `name`. */
const handleCenter = async (page: Page, name: string) => {
  const box = await handleOf(page, name).boundingBox();
  if (box === null) throw new Error(`${name} not laid out`);
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
};

/** Press on the USB headset handle and move over the Studio row: a drag is in progress. */
const startDrag = async (page: Page): Promise<void> => {
  await page.evaluate(() =>
    document.addEventListener(
      "pointerdown",
      (event) => {
        document.body.dataset.pointerId = String(event.pointerId);
      },
      true,
    ),
  );
  const from = await handleCenter(page, USB);
  const to = await row(page, STUDIO).boundingBox();
  if (to === null) throw new Error("rows not laid out");
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(from.x, to.y + to.height / 2, { steps: 6 });
  await expect(page.getByTestId("microphone-settings-choices")).toHaveAttribute(
    "data-state",
    "drop-target",
  );
  await expect(row(page, USB)).toHaveAttribute("data-state", "dragging");
};

/** No row is dragged and the list is no drop target. */
const expectNoDrag = async (page: Page): Promise<void> => {
  const list = page.getByTestId("microphone-settings-choices");
  await expect.poll(() => list.getAttribute("data-state")).toBeNull();
  await expect(page.locator("li[data-state]")).toHaveCount(0);
};

const expectCancelled = async (page: Page): Promise<void> => {
  await expectNoDrag(page);
  await page.mouse.up();
  expect(await order(page)).toEqual(ORIGINAL);
  await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
};

const onUsbHandle = (script: string) => (page: Page) =>
  page.evaluate((source) => {
    const handle = document.querySelector<HTMLElement>(
      'li[data-name="USB headset"] span[aria-hidden="true"]',
    );
    const pointerId = Number(document.body.dataset.pointerId);
    if (source === "release") handle?.releasePointerCapture(pointerId);
    else
      handle?.dispatchEvent(
        new PointerEvent("pointercancel", { bubbles: true, pointerId }),
      );
  }, script);

const interruptions: Record<string, (page: Page) => Promise<unknown>> = {
  "window blur": (page) =>
    page.evaluate(() => window.dispatchEvent(new Event("blur"))),
  "page hidden": (page) =>
    page.evaluate(() => window.harness.setVisible(false)),
  "lost pointer capture": onUsbHandle("release"),
  "pointer cancel": onUsbHandle("cancel"),
};

for (const [name, interrupt] of Object.entries(interruptions)) {
  test(`${name} cancels the drag without reordering`, async ({ page }) => {
    await open(page);
    await startDrag(page);
    await interrupt(page);
    await expectCancelled(page);
  });
}

test("an uninterrupted drag still drops", async ({ page }) => {
  await open(page);
  await startDrag(page);
  await page.mouse.up();
  expect(await order(page)).toEqual([MACBOOK, STUDIO, USB]);
  await expect(page.getByTestId("microphone-settings-draft")).toBeVisible();
});

test("the dragged row removed by a change saved elsewhere: the drag is cancelled and moves nothing", async ({
  page,
}) => {
  await open(page);
  await page.evaluate(
    (names) => window.harness.commitElsewhere(names),
    [...ORIGINAL, "Dock"],
  );
  await expect(row(page, "Dock")).toBeVisible();
  await startDrag(page);
  const remaining = [MACBOOK, STUDIO, "Dock"];
  await page.evaluate(
    (names) => window.harness.commitElsewhere(names),
    remaining,
  );
  await expect(row(page, USB)).toHaveCount(0);
  await expectNoDrag(page);
  // Release over another handle: the same mouse pointer must not drop a row it never grabbed.
  const over = await handleCenter(page, STUDIO);
  await page.mouse.move(over.x, over.y, { steps: 3 });
  await page.mouse.up();
  expect(await order(page)).toEqual(remaining);
  await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
});

test("a write starting during a drag cancels it", async ({ page }) => {
  await open(page);
  await editFromKeyboard(page);
  const draft = [MACBOOK, USB, STUDIO];
  await page.evaluate(() => window.harness.holdSave());
  await startDrag(page);
  await page.getByTestId("microphone-settings-save").focus();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("microphone-settings-save")).toHaveAttribute(
    "data-state",
    "loading",
  );
  await expectNoDrag(page);
  await page.mouse.up();
  expect(await order(page)).toEqual(draft);
  await page.evaluate(() => window.harness.releaseSave(null));
  await expect(page.getByTestId("microphone-settings-screen")).toHaveAttribute(
    "data-state",
    "success",
  );
  expect(await order(page)).toEqual(draft);
});
