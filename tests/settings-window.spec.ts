// The settings window scrolls only its content area (native session finding N4): the window
// root never scrolls, so the sidebar and the footer stay in view whatever the content holds.
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/settings-window.html";

const documentOverflow = (page: Page) =>
  page.evaluate(() => {
    const scroller = document.scrollingElement as HTMLElement;
    return scroller.scrollHeight - scroller.clientHeight;
  });

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 600 });
  await page.goto(HARNESS);
  await expect(page.getByTestId("settings-window")).toBeVisible();
});

test("the window root has nothing to scroll", async ({ page }) => {
  expect(await documentOverflow(page)).toBeLessThanOrEqual(0);
  await page.mouse.move(80, 300);
  await page.mouse.wheel(0, 4000);
  const scrolled = await page.evaluate(() => ({
    window: window.scrollY,
    root: document.querySelector<HTMLElement>("[data-testid=settings-window]")!
      .scrollTop,
  }));
  expect(scrolled).toEqual({ window: 0, root: 0 });
});

test("focusing the last control scrolls the content only", async ({ page }) => {
  await page.getByTestId("last-control").focus();
  await expect(page.getByTestId("last-control")).toBeInViewport();
  const state = await page.evaluate(() => ({
    window: window.scrollY,
    root: document.querySelector<HTMLElement>("[data-testid=settings-window]")!
      .scrollTop,
    content: document.querySelector<HTMLElement>(
      "[data-testid=settings-window-content]",
    )!.scrollTop,
  }));
  expect(state.window).toBe(0);
  expect(state.root).toBe(0);
  expect(state.content).toBeGreaterThan(0);
  await expect(page.getByTestId("settings-window-sidebar")).toBeInViewport();
  await expect(page.getByTestId("settings-window-footer")).toBeInViewport();
});

test("positioned content is contained by the scrolling area", async ({
  page,
}) => {
  // WebKit (the macOS window) lets an absolutely positioned element whose containing block is
  // the page stretch the document; inside the content area, none may resolve past it.
  const escaped = await page.evaluate(() => {
    const content = document.querySelector<HTMLElement>(
      "[data-testid=settings-window-content]",
    )!;
    return Array.from(content.querySelectorAll<HTMLElement>("*"))
      .filter((node) => getComputedStyle(node).position === "absolute")
      .filter((node) => {
        const parent = node.offsetParent;
        return parent === null || !content.contains(parent);
      }).length;
  });
  const hints = await page.locator("#last-hint").count();
  expect(hints).toBe(1);
  expect(escaped).toBe(0);
});
